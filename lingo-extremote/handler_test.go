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
