package main

import (
	"errors"
	"io"
	"testing"

	"github.com/oandrew/ipod"
	audio "github.com/oandrew/ipod/lingo-audio"
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
