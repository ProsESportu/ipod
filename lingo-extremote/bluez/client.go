// Package bluez provides a client for the BlueZ org.bluez.MediaPlayer1 D-Bus
// interface, adapted to the lingo-extremote DeviceExtRemote interface.
//
// The client auto-discovers the first available MediaPlayer1 object on the
// system bus using org.freedesktop.DBus.ObjectManager, and keeps a live
// snapshot of its properties by subscribing to PropertiesChanged,
// InterfacesAdded and InterfacesRemoved signals.
package bluez

import (
	"fmt"
	"log"
	"math"
	"sort"
	"sync"
	"time"

	"github.com/godbus/dbus/v5"

	extremote "github.com/oandrew/ipod/lingo-extremote"
)

const (
	bluezService        = "org.bluez"
	mediaPlayerIface    = "org.bluez.MediaPlayer1"
	objectManagerIface  = "org.freedesktop.DBus.ObjectManager"
	propertiesIface     = "org.freedesktop.DBus.Properties"
	getManagedObjectsFn = "org.freedesktop.DBus.ObjectManager.GetManagedObjects"
	propertiesChangedFn = "org.freedesktop.DBus.Properties.PropertiesChanged"
	interfacesAddedFn   = "org.freedesktop.DBus.ObjectManager.InterfacesAdded"
	interfacesRemovedFn = "org.freedesktop.DBus.ObjectManager.InterfacesRemoved"
)

// Client is a live view of the currently selected MediaPlayer1 on org.bluez.
//
// It implements extremote.DeviceExtRemote.
type Client struct {
	conn *dbus.Conn

	mu            sync.RWMutex
	path          dbus.ObjectPath // "" when no player is available
	status        string
	position      uint32 // milliseconds
	positionSetAt time.Time
	track         extremote.TrackMetadata
	shuffle       string
	repeat        string
}

var _ extremote.DeviceExtRemote = (*Client)(nil)

// NewClient connects to the system bus, discovers an existing MediaPlayer1 (if
// any), and starts watching for player lifecycle and property changes in a
// background goroutine.
func NewClient() (*Client, error) {
	conn, err := dbus.SystemBus()
	if err != nil {
		return nil, fmt.Errorf("bluez: system bus: %w", err)
	}
	c := &Client{conn: conn}
	if err := c.watch(); err != nil {
		return nil, err
	}
	if err := c.discover(); err != nil {
		return nil, err
	}
	return c, nil
}

// watch sets up signal matches and starts the signal-reading goroutine.
func (c *Client) watch() error {
	matches := []dbus.MatchOption{
		dbus.WithMatchSender(bluezService),
		dbus.WithMatchInterface(propertiesIface),
		dbus.WithMatchMember("PropertiesChanged"),
	}
	if err := c.conn.AddMatchSignal(matches...); err != nil {
		return fmt.Errorf("bluez: add match PropertiesChanged: %w", err)
	}
	if err := c.conn.AddMatchSignal(
		dbus.WithMatchSender(bluezService),
		dbus.WithMatchInterface(objectManagerIface),
		dbus.WithMatchMember("InterfacesAdded"),
	); err != nil {
		return fmt.Errorf("bluez: add match InterfacesAdded: %w", err)
	}
	if err := c.conn.AddMatchSignal(
		dbus.WithMatchSender(bluezService),
		dbus.WithMatchInterface(objectManagerIface),
		dbus.WithMatchMember("InterfacesRemoved"),
	); err != nil {
		return fmt.Errorf("bluez: add match InterfacesRemoved: %w", err)
	}

	ch := make(chan *dbus.Signal, 32)
	c.conn.Signal(ch)
	go c.loop(ch)
	return nil
}

func (c *Client) loop(ch <-chan *dbus.Signal) {
	for sig := range ch {
		switch sig.Name {
		case propertiesChangedFn:
			if len(sig.Body) < 2 {
				continue
			}
			iface, _ := sig.Body[0].(string)
			if iface != mediaPlayerIface {
				continue
			}
			c.mu.RLock()
			current := c.path
			c.mu.RUnlock()
			if current == "" || sig.Path != current {
				continue
			}
			changed, _ := sig.Body[1].(map[string]dbus.Variant)
			c.applyProps(changed)

		case interfacesAddedFn:
			if len(sig.Body) < 2 {
				continue
			}
			path, _ := sig.Body[0].(dbus.ObjectPath)
			ifaces, _ := sig.Body[1].(map[string]map[string]dbus.Variant)
			props, ok := ifaces[mediaPlayerIface]
			if !ok {
				continue
			}
			c.mu.RLock()
			have := c.path != ""
			c.mu.RUnlock()
			if have {
				continue
			}
			c.adopt(path, props)

		case interfacesRemovedFn:
			if len(sig.Body) < 2 {
				continue
			}
			path, _ := sig.Body[0].(dbus.ObjectPath)
			ifaces, _ := sig.Body[1].([]string)
			for _, i := range ifaces {
				if i != mediaPlayerIface {
					continue
				}
				c.mu.Lock()
				if c.path == path {
					log.Printf("bluez: MediaPlayer1 removed: %s", path)
					c.path = ""
					c.status = ""
					c.position = 0
					c.positionSetAt = time.Time{}
					c.track = extremote.TrackMetadata{}
					c.shuffle = ""
					c.repeat = ""
				}
				c.mu.Unlock()
			}
		}
	}
}

// discover scans the object tree for any existing MediaPlayer1 object and
// adopts the first one found (sorted by path for determinism).
func (c *Client) discover() error {
	obj := c.conn.Object(bluezService, "/")
	var managed map[dbus.ObjectPath]map[string]map[string]dbus.Variant
	if err := obj.Call(getManagedObjectsFn, 0).Store(&managed); err != nil {
		return fmt.Errorf("bluez: GetManagedObjects: %w", err)
	}
	paths := make([]string, 0, len(managed))
	for p, ifaces := range managed {
		if _, ok := ifaces[mediaPlayerIface]; ok {
			paths = append(paths, string(p))
		}
	}
	if len(paths) == 0 {
		log.Printf("bluez: no MediaPlayer1 object found on org.bluez (waiting for one to appear)")
		return nil
	}
	sort.Strings(paths)
	path := dbus.ObjectPath(paths[0])
	c.adopt(path, managed[path][mediaPlayerIface])
	return nil
}

// adopt switches to the given player path and seeds the cache from props.
func (c *Client) adopt(path dbus.ObjectPath, props map[string]dbus.Variant) {
	c.mu.Lock()
	c.path = path
	c.mu.Unlock()
	log.Printf("bluez: using MediaPlayer1 at %s", path)
	c.applyProps(props)
}

// applyProps merges a PropertiesChanged-style dict into the cached snapshot.
func (c *Client) applyProps(props map[string]dbus.Variant) {
	c.mu.Lock()
	defer c.mu.Unlock()
	for k, v := range props {
		switch k {
		case "Status":
			if s, ok := v.Value().(string); ok {
				c.status = s
			}
		case "Position":
			if p, ok := v.Value().(uint32); ok {
				c.position = p
				c.positionSetAt = time.Now()
			}
		case "Shuffle":
			if s, ok := v.Value().(string); ok {
				c.shuffle = s
			}
		case "Repeat":
			if s, ok := v.Value().(string); ok {
				c.repeat = s
			}
		case "Track":
			if td, ok := v.Value().(map[string]dbus.Variant); ok {
				c.track = trackFromVariants(td)
			}
		}
	}
}

func trackFromVariants(m map[string]dbus.Variant) extremote.TrackMetadata {
	var t extremote.TrackMetadata
	get := func(key string) interface{} {
		if v, ok := m[key]; ok {
			return v.Value()
		}
		return nil
	}
	if s, ok := get("Title").(string); ok {
		t.Title = s
	}
	if s, ok := get("Artist").(string); ok {
		t.Artist = s
	}
	if s, ok := get("Album").(string); ok {
		t.Album = s
	}
	if s, ok := get("Genre").(string); ok {
		t.Genre = s
	}
	if n, ok := get("NumberOfTracks").(uint32); ok {
		t.NumberOfTracks = n
	}
	if n, ok := get("TrackNumber").(uint32); ok {
		t.TrackNumber = n
	}
	if n, ok := get("Duration").(uint32); ok {
		t.Duration = n
	}
	return t
}

// Available reports whether a MediaPlayer1 object is currently adopted.
func (c *Client) Available() bool {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.path != ""
}

// Track returns the latest cached track metadata (zero-value when none).
func (c *Client) Track() extremote.TrackMetadata {
	c.mu.RLock()
	defer c.mu.RUnlock()
	return c.track
}

// PlaybackStatus implements extremote.DeviceExtRemote.
//
// Note: MediaPlayer1.Position is signalled only when it changes discontinuously
// (seek, track change). To provide a reasonable live position we linearly
// extrapolate from the last observed value using wall-clock time while the
// player is in the "playing" state.
func (c *Client) PlaybackStatus() (trackLength, trackPos uint32, state extremote.PlayerState) {
	c.mu.RLock()
	defer c.mu.RUnlock()

	trackLength = c.track.Duration
	trackPos = c.position
	state = mapStatus(c.status)

	if state == extremote.PlayerStatePlaying && !c.positionSetAt.IsZero() {
		elapsed := time.Since(c.positionSetAt).Milliseconds()
		if elapsed > 0 {
			extrapolated := int64(c.position) + elapsed
			if extrapolated > int64(math.MaxUint32) {
				extrapolated = int64(math.MaxUint32)
			}
			if trackLength > 0 && uint32(extrapolated) > trackLength {
				trackPos = trackLength
			} else {
				trackPos = uint32(extrapolated)
			}
		}
	}
	return
}

func mapStatus(s string) extremote.PlayerState {
	switch s {
	case "playing", "forward-seek", "reverse-seek":
		return extremote.PlayerStatePlaying
	case "paused":
		return extremote.PlayerStatePaused
	case "stopped":
		return extremote.PlayerStateStopped
	case "error":
		return extremote.PlayerStateError
	default:
		return extremote.PlayerStateStopped
	}
}

// Shuffle returns the ipod-lingo ShuffleMode corresponding to the current
// org.bluez.MediaPlayer1.Shuffle property.
func (c *Client) Shuffle() extremote.ShuffleMode {
	c.mu.RLock()
	defer c.mu.RUnlock()
	switch c.shuffle {
	case "alltracks":
		return extremote.ShuffleTracks
	case "group":
		return extremote.ShuffleAlbums
	default:
		return extremote.ShuffleOff
	}
}

// SetShuffle writes the corresponding org.bluez.MediaPlayer1.Shuffle property.
func (c *Client) SetShuffle(m extremote.ShuffleMode) error {
	var v string
	switch m {
	case extremote.ShuffleTracks:
		v = "alltracks"
	case extremote.ShuffleAlbums:
		v = "group"
	default:
		v = "off"
	}
	return c.setProperty("Shuffle", v)
}

// Repeat returns the ipod-lingo RepeatMode corresponding to the current
// org.bluez.MediaPlayer1.Repeat property.
func (c *Client) Repeat() extremote.RepeatMode {
	c.mu.RLock()
	defer c.mu.RUnlock()
	switch c.repeat {
	case "singletrack":
		return extremote.RepeatOne
	case "alltracks", "group":
		return extremote.RepeatAll
	default:
		return extremote.RepeatOff
	}
}

// SetRepeat writes the corresponding org.bluez.MediaPlayer1.Repeat property.
func (c *Client) SetRepeat(m extremote.RepeatMode) error {
	var v string
	switch m {
	case extremote.RepeatOne:
		v = "singletrack"
	case extremote.RepeatAll:
		v = "alltracks"
	default:
		v = "off"
	}
	return c.setProperty("Repeat", v)
}

func (c *Client) setProperty(name string, value string) error {
	path, ok := c.currentPath()
	if !ok {
		return fmt.Errorf("bluez: no MediaPlayer1 available")
	}
	obj := c.conn.Object(bluezService, path)
	return obj.Call(propertiesIface+".Set", 0, mediaPlayerIface, name, dbus.MakeVariant(value)).Err
}

func (c *Client) currentPath() (dbus.ObjectPath, bool) {
	c.mu.RLock()
	defer c.mu.RUnlock()
	if c.path == "" {
		return "", false
	}
	return c.path, true
}

// call invokes a MediaPlayer1 method with no arguments.
func (c *Client) call(method string) error {
	path, ok := c.currentPath()
	if !ok {
		return fmt.Errorf("bluez: no MediaPlayer1 available")
	}
	obj := c.conn.Object(bluezService, path)
	return obj.Call(mediaPlayerIface+"."+method, 0).Err
}

// Play resumes playback.
func (c *Client) Play() error { return c.call("Play") }

// Pause pauses playback.
func (c *Client) Pause() error { return c.call("Pause") }

// Stop stops playback.
func (c *Client) Stop() error { return c.call("Stop") }

// Next skips to the next track.
func (c *Client) Next() error { return c.call("Next") }

// Previous skips to the previous track.
func (c *Client) Previous() error { return c.call("Previous") }

// FastForward begins fast-forward seeking.
func (c *Client) FastForward() error { return c.call("FastForward") }

// Rewind begins rewind seeking.
func (c *Client) Rewind() error { return c.call("Rewind") }

// TogglePlayPause toggles between playing and paused states.
func (c *Client) TogglePlayPause() error {
	c.mu.RLock()
	s := c.status
	c.mu.RUnlock()
	if s == "playing" {
		return c.Pause()
	}
	return c.Play()
}

// PlayControl maps an ipod PlayControl command to the corresponding
// MediaPlayer1 method and invokes it.
func (c *Client) PlayControl(cmd extremote.PlayControlCmd) error {
	switch cmd {
	case extremote.PlayControlToggle:
		return c.TogglePlayPause()
	case extremote.PlayControlStop:
		return c.Stop()
	case extremote.PlayControlNextTrack, extremote.PlayControlNext:
		return c.Next()
	case extremote.PlayControlPrevTrack, extremote.PlayControlPrev:
		return c.Previous()
	case extremote.PlayControlStartFF:
		return c.FastForward()
	case extremote.PlayControlStartRew:
		return c.Rewind()
	case extremote.PlayControlEndFFRew:
		// FastForward/Rewind stop on any other method call; resume playback.
		return c.Play()
	case extremote.PlayControlPlay:
		return c.Play()
	case extremote.PlayControlPause:
		return c.Pause()
	case extremote.PlayControlNextChapter, extremote.PlayControlPrevChapter:
		// Chapters are not modelled by MediaPlayer1.
		return nil
	default:
		return fmt.Errorf("bluez: unsupported PlayControl command 0x%02x", byte(cmd))
	}
}
