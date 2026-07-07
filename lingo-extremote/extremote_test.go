package extremote

import (
	"reflect"
	"testing"

	"github.com/oandrew/ipod"
)

func TestPlayStatusChangeNotificationMarshalTrackIndex(t *testing.T) {
	cmd, err := ipod.BuildCommand(NewTrackIndexPlayStatusChangeNotification(2))
	if err != nil {
		t.Fatalf("BuildCommand: %v", err)
	}

	var serde ipod.CommandSerde
	got, err := serde.MarshalCmd(cmd)
	if err != nil {
		t.Fatalf("MarshalCmd: %v", err)
	}
	want := []byte{0x04, 0x00, 0x27, 0x01, 0x00, 0x00, 0x00, 0x02}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("MarshalCmd = %#v, want %#v", got, want)
	}
}

func TestPlayStatusChangeNotificationMarshalTrackIndexWithTransaction(t *testing.T) {
	cmd, err := ipod.BuildCommand(NewTrackIndexPlayStatusChangeNotification(3))
	if err != nil {
		t.Fatalf("BuildCommand: %v", err)
	}
	cmd.Transaction = ipod.NewTransaction(0x1234)

	serde := ipod.CommandSerde{TrxEnabled: true}
	got, err := serde.MarshalCmd(cmd)
	if err != nil {
		t.Fatalf("MarshalCmd: %v", err)
	}
	want := []byte{0x04, 0x00, 0x27, 0x12, 0x34, 0x01, 0x00, 0x00, 0x00, 0x03}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("MarshalCmd = %#v, want %#v", got, want)
	}
}
