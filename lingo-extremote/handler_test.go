package extremote

import (
	"testing"

	"github.com/oandrew/ipod"
)

type testDevice struct {
	track TrackMetadata
	calls []PlayControlCmd
}

func (d *testDevice) PlaybackStatus() (uint32, uint32, PlayerState) {
	return 0, 0, PlayerStateStopped
}

func (d *testDevice) Track() TrackMetadata { return d.track }

func (d *testDevice) Shuffle() ShuffleMode { return ShuffleOff }

func (d *testDevice) SetShuffle(ShuffleMode) error { return nil }

func (d *testDevice) Repeat() RepeatMode { return RepeatOff }

func (d *testDevice) SetRepeat(RepeatMode) error { return nil }

func (d *testDevice) PlayControl(cmd PlayControlCmd) error {
	d.calls = append(d.calls, cmd)
	return nil
}

func TestHandleExtRemoteSetCurrentPlayingTrackSkipsOnce(t *testing.T) {
	tests := []struct {
		name      string
		requested int32
		wantCalls []PlayControlCmd
	}{
		{name: "forward", requested: 5, wantCalls: []PlayControlCmd{PlayControlNextTrack}},
		{name: "backward", requested: 1, wantCalls: []PlayControlCmd{PlayControlPrevTrack}},
		{name: "same", requested: 2, wantCalls: nil},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			dev := &testDevice{track: TrackMetadata{TrackNumber: 3}}
			req, err := ipod.BuildCommand(&SetCurrentPlayingTrack{TrackIndex: tt.requested})
			if err != nil {
				t.Fatalf("BuildCommand: %v", err)
			}
			tr := &ipod.CmdBuffer{}

			if err := HandleExtRemote(req, tr, dev); err != nil {
				t.Fatalf("HandleExtRemote: %v", err)
			}

			if len(dev.calls) != len(tt.wantCalls) {
				t.Fatalf("PlayControl calls = %v, want %v", dev.calls, tt.wantCalls)
			}
			for i := range tt.wantCalls {
				if dev.calls[i] != tt.wantCalls[i] {
					t.Fatalf("PlayControl calls = %v, want %v", dev.calls, tt.wantCalls)
				}
			}

			if len(tr.Commands) != 1 {
				t.Fatalf("responses = %d, want 1", len(tr.Commands))
			}
			ack, ok := tr.Commands[0].Payload.(*ACK)
			if !ok {
				t.Fatalf("response payload = %T, want *ACK", tr.Commands[0].Payload)
			}
			if ack.Status != ACKStatusSuccess {
				t.Fatalf("ACK status = %v, want %v", ack.Status, ACKStatusSuccess)
			}
		})
	}
}

func TestHandleExtRemotePlayStatusNotificationMask(t *testing.T) {
	tests := []struct {
		name    string
		payload interface{}
		want    bool
	}{
		{
			name:    "four byte track index mask",
			payload: &SetPlayStatusChangeNotification{EventMask: PlayStatusNotificationTrackIndex},
			want:    true,
		},
		{
			name:    "short enable",
			payload: &SetPlayStatusChangeNotificationShort{Enabled: true},
			want:    true,
		},
		{
			name:    "four byte disable",
			payload: &SetPlayStatusChangeNotification{EventMask: 0},
			want:    false,
		},
		{
			name:    "short disable",
			payload: &SetPlayStatusChangeNotificationShort{Enabled: false},
			want:    false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			req, err := ipod.BuildCommand(tt.payload)
			if err != nil {
				t.Fatalf("BuildCommand: %v", err)
			}
			tr := &ipod.CmdBuffer{}
			notifications := NewPlayStatusNotificationState()

			if err := HandleExtRemoteWithNotifications(req, tr, nil, notifications); err != nil {
				t.Fatalf("HandleExtRemoteWithNotifications: %v", err)
			}

			if got := notifications.TrackIndexEnabled(); got != tt.want {
				t.Fatalf("TrackIndexEnabled = %v, want %v", got, tt.want)
			}
			if len(tr.Commands) != 1 {
				t.Fatalf("responses = %d, want 1", len(tr.Commands))
			}
			if _, ok := tr.Commands[0].Payload.(*ACK); !ok {
				t.Fatalf("response payload = %T, want *ACK", tr.Commands[0].Payload)
			}
		})
	}
}
