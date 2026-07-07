package bluez

import (
	"testing"

	"github.com/godbus/dbus/v5"

	extremote "github.com/oandrew/ipod/lingo-extremote"
)

func TestApplyPropsTrackChanges(t *testing.T) {
	c := &Client{trackChanges: make(chan extremote.TrackChange, 4)}

	c.applyProps(trackProps("first", 1, 1000, ""))
	assertNoTrackChange(t, c)

	c.applyProps(trackProps("first", 1, 1000, ""))
	assertNoTrackChange(t, c)

	c.applyProps(trackProps("second", 2, 2000, ""))
	got := assertTrackChange(t, c)
	if got.TrackIndex != 1 {
		t.Fatalf("TrackIndex = %d, want 1", got.TrackIndex)
	}
	if got.Track.Title != "second" {
		t.Fatalf("Title = %q, want %q", got.Track.Title, "second")
	}
}

func TestApplyPropsIgnoresNonTrackChanges(t *testing.T) {
	c := &Client{trackChanges: make(chan extremote.TrackChange, 4)}

	c.applyProps(trackProps("first", 1, 1000, ""))
	assertNoTrackChange(t, c)

	c.applyProps(map[string]dbus.Variant{
		"Status":   dbus.MakeVariant("playing"),
		"Position": dbus.MakeVariant(uint32(500)),
	})
	assertNoTrackChange(t, c)
}

func TestApplyPropsTrackFingerprintIncludesUnparsedKeys(t *testing.T) {
	c := &Client{trackChanges: make(chan extremote.TrackChange, 4)}

	c.applyProps(trackProps("first", 1, 1000, ""))
	assertNoTrackChange(t, c)

	c.applyProps(trackProps("first", 1, 1000, "img-1"))
	assertTrackChange(t, c)
}

func TestApplyPropsTrackChangeWithoutTrackNumberUsesZeroIndex(t *testing.T) {
	c := &Client{trackChanges: make(chan extremote.TrackChange, 4)}

	c.applyProps(map[string]dbus.Variant{
		"Track": dbus.MakeVariant(map[string]dbus.Variant{
			"Title": dbus.MakeVariant("first"),
		}),
	})
	assertNoTrackChange(t, c)

	c.applyProps(map[string]dbus.Variant{
		"Track": dbus.MakeVariant(map[string]dbus.Variant{
			"Title": dbus.MakeVariant("second"),
		}),
	})
	got := assertTrackChange(t, c)
	if got.TrackIndex != 0 {
		t.Fatalf("TrackIndex = %d, want 0", got.TrackIndex)
	}
}

func trackProps(title string, trackNumber, duration uint32, imgHandle string) map[string]dbus.Variant {
	track := map[string]dbus.Variant{
		"Title":       dbus.MakeVariant(title),
		"Artist":      dbus.MakeVariant("artist"),
		"Album":       dbus.MakeVariant("album"),
		"TrackNumber": dbus.MakeVariant(trackNumber),
		"Duration":    dbus.MakeVariant(duration),
	}
	if imgHandle != "" {
		track["ImgHandle"] = dbus.MakeVariant(imgHandle)
	}
	return map[string]dbus.Variant{
		"Track": dbus.MakeVariant(track),
	}
}

func assertTrackChange(t *testing.T, c *Client) extremote.TrackChange {
	t.Helper()
	select {
	case got := <-c.TrackChanges():
		return got
	default:
		t.Fatal("expected track change")
		return extremote.TrackChange{}
	}
}

func assertNoTrackChange(t *testing.T, c *Client) {
	t.Helper()
	select {
	case got := <-c.TrackChanges():
		t.Fatalf("unexpected track change: %#v", got)
	default:
	}
}
