package main

import (
	"errors"
	"io"
	"testing"
	"time"

	"github.com/oandrew/ipod"
	audio "github.com/oandrew/ipod/lingo-audio"
	extremote "github.com/oandrew/ipod/lingo-extremote"
)

type testFrameTransport struct {
	frames  [][]byte
	writes  [][]byte
	readErr error
}

func (t *testFrameTransport) ReadFrame() ([]byte, error) {
	if t.readErr != nil {
		return nil, t.readErr
	}
	if len(t.frames) == 0 {
		return nil, io.EOF
	}
	frame := t.frames[0]
	t.frames = t.frames[1:]
	return frame, nil
}

func (t *testFrameTransport) WriteFrame(frame []byte) error {
	t.writes = append(t.writes, append([]byte(nil), frame...))
	return nil
}

type readFrameResult struct {
	frame []byte
	err   error
}

type blockingFrameTransport struct {
	reads  chan readFrameResult
	writes chan []byte
}

func (t *blockingFrameTransport) ReadFrame() ([]byte, error) {
	r := <-t.reads
	return r.frame, r.err
}

func (t *blockingFrameTransport) WriteFrame(frame []byte) error {
	t.writes <- append([]byte(nil), frame...)
	return nil
}

type fakeTrackChangeDevice struct {
	changes chan extremote.TrackChange
}

func (d *fakeTrackChangeDevice) PlaybackStatus() (uint32, uint32, extremote.PlayerState) {
	return 0, 0, extremote.PlayerStateStopped
}

func (d *fakeTrackChangeDevice) Track() extremote.TrackMetadata { return extremote.TrackMetadata{} }

func (d *fakeTrackChangeDevice) Shuffle() extremote.ShuffleMode { return extremote.ShuffleOff }

func (d *fakeTrackChangeDevice) SetShuffle(extremote.ShuffleMode) error { return nil }

func (d *fakeTrackChangeDevice) Repeat() extremote.RepeatMode { return extremote.RepeatOff }

func (d *fakeTrackChangeDevice) SetRepeat(extremote.RepeatMode) error { return nil }

func (d *fakeTrackChangeDevice) PlayControl(extremote.PlayControlCmd) error { return nil }

func (d *fakeTrackChangeDevice) TrackChanges() <-chan extremote.TrackChange { return d.changes }

func audioFrame(tb testing.TB, payload interface{}) []byte {
	tb.Helper()
	cmd, err := ipod.BuildCommand(payload)
	if err != nil {
		tb.Fatalf("BuildCommand: %v", err)
	}
	var serde ipod.CommandSerde
	packet, err := serde.MarshalCmd(cmd)
	if err != nil {
		tb.Fatalf("MarshalCmd: %v", err)
	}
	writer := ipod.NewPacketWriter()
	if err := writer.WritePacket(packet); err != nil {
		tb.Fatalf("WritePacket: %v", err)
	}
	return writer.Bytes()
}

func TestProcessFramesStartsPlaybackHookOnceForSampleRateCaps(t *testing.T) {
	tr := &testFrameTransport{
		frames: [][]byte{
			audioFrame(t, &audio.RetAccSampleRateCaps{SampleRates: []uint32{32000, 44100, 48000}}),
			audioFrame(t, &audio.RetAccSampleRateCaps{SampleRates: []uint32{32000, 44100, 48000}}),
		},
	}
	calls := 0

	if err := processFramesWithOptions(tr, frameLoopOptions{
		onPlaybackInitialized: func() error {
			calls++
			return nil
		},
	}); err != nil {
		t.Fatalf("processFramesWithOptions: %v", err)
	}

	if calls != 1 {
		t.Fatalf("playback hook calls = %d, want 1", calls)
	}
	if len(tr.writes) != 2 {
		t.Fatalf("writes = %d, want 2", len(tr.writes))
	}
}

func TestProcessFramesReturnsPlaybackHookErrorAfterResponse(t *testing.T) {
	tr := &testFrameTransport{
		frames: [][]byte{
			audioFrame(t, &audio.RetAccSampleRateCaps{SampleRates: []uint32{32000, 44100, 48000}}),
		},
	}
	hookErr := errors.New("bluealsa failed")

	err := processFramesWithOptions(tr, frameLoopOptions{
		onPlaybackInitialized: func() error {
			return hookErr
		},
	})
	if !errors.Is(err, hookErr) {
		t.Fatalf("processFramesWithOptions error = %v, want %v", err, hookErr)
	}
	if len(tr.writes) != 1 {
		t.Fatalf("writes = %d, want response written before hook failure", len(tr.writes))
	}
}

func TestProcessFramesReturnsReadErrorWhenStopped(t *testing.T) {
	readErr := errors.New("device closed")
	stop := make(chan struct{})
	close(stop)
	tr := &testFrameTransport{readErr: readErr}

	err := processFramesWithOptions(tr, frameLoopOptions{stop: stop})
	if !errors.Is(err, readErr) {
		t.Fatalf("processFramesWithOptions error = %v, want %v", err, readErr)
	}
}

func TestProcessFramesWritesAsyncCommandWhileReadBlocked(t *testing.T) {
	tr := &blockingFrameTransport{
		reads:  make(chan readFrameResult),
		writes: make(chan []byte, 1),
	}
	asyncCommands := make(chan *ipod.Command, 1)
	done := make(chan error, 1)
	go func() {
		done <- processFramesWithOptions(tr, frameLoopOptions{asyncCommands: asyncCommands})
	}()

	cmd, err := ipod.BuildCommand(extremote.NewTrackIndexPlayStatusChangeNotification(4))
	if err != nil {
		t.Fatalf("BuildCommand: %v", err)
	}
	asyncCommands <- cmd

	var frame []byte
	select {
	case frame = <-tr.writes:
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for async frame write")
	}

	packet, err := ipod.NewPacketReader(frame).ReadPacket()
	if err != nil {
		t.Fatalf("ReadPacket: %v", err)
	}
	var serde ipod.CommandSerde
	got, err := serde.UnmarshalCmd(packet)
	if err != nil {
		t.Fatalf("UnmarshalCmd: %v", err)
	}
	notification, ok := got.Payload.(*extremote.PlayStatusChangeNotification)
	if !ok {
		t.Fatalf("payload = %T, want *PlayStatusChangeNotification", got.Payload)
	}
	if notification.Status != extremote.PlayStatusChangeTrackIndex || notification.TrackIndex != 4 {
		t.Fatalf("notification = %#v, want track index 4", notification)
	}

	tr.reads <- readFrameResult{err: io.EOF}
	select {
	case err := <-done:
		if err != nil {
			t.Fatalf("processFramesWithOptions: %v", err)
		}
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for processFramesWithOptions")
	}
}

func TestStartExtRemoteNotificationsBuildsTrackIndexCommand(t *testing.T) {
	dev := &fakeTrackChangeDevice{changes: make(chan extremote.TrackChange, 1)}
	notifications := extremote.NewPlayStatusNotificationState()
	notifications.SetMask(extremote.PlayStatusNotificationTrackIndex)
	out := make(chan *ipod.Command, 1)
	stop := make(chan struct{})
	defer close(stop)

	startExtRemoteNotifications(dev, notifications, out, stop)
	dev.changes <- extremote.TrackChange{TrackIndex: 7}

	var cmd *ipod.Command
	select {
	case cmd = <-out:
	case <-time.After(time.Second):
		t.Fatal("timed out waiting for notification command")
	}

	notification, ok := cmd.Payload.(*extremote.PlayStatusChangeNotification)
	if !ok {
		t.Fatalf("payload = %T, want *PlayStatusChangeNotification", cmd.Payload)
	}
	if notification.Status != extremote.PlayStatusChangeTrackIndex || notification.TrackIndex != 7 {
		t.Fatalf("notification = %#v, want track index 7", notification)
	}
	if cmd.Transaction == nil {
		t.Fatal("notification command transaction is nil")
	}
}
