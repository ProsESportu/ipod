package extremote

import (
	"github.com/oandrew/ipod"
)

// TrackMetadata mirrors the subset of org.bluez.MediaPlayer1.Track that is
// exposed to the ext-remote lingo.
type TrackMetadata struct {
	Title          string
	Artist         string
	Album          string
	Genre          string
	NumberOfTracks uint32
	TrackNumber    uint32
	Duration       uint32 // milliseconds
}

// DeviceExtRemote is the integration point used by HandleExtRemote to service
// incoming lingo-extremote commands with live data (track metadata, play
// status, shuffle/repeat, and playback transport).
//
// All methods must be safe to call concurrently.
type DeviceExtRemote interface {
	// PlaybackStatus returns the current track length and position (in
	// milliseconds) and the player state.
	PlaybackStatus() (trackLength, trackPos uint32, state PlayerState)

	// Track returns the current track metadata.
	Track() TrackMetadata

	// Shuffle/Repeat getters and setters.
	Shuffle() ShuffleMode
	SetShuffle(ShuffleMode) error
	Repeat() RepeatMode
	SetRepeat(RepeatMode) error

	// PlayControl forwards an ipod PlayControl command to the backend.
	PlayControl(PlayControlCmd) error
}

func ackSuccess(req *ipod.Command) *ACK {
	return &ACK{Status: ACKStatusSuccess, CmdID: req.ID.CmdID()}
}

func ackStatus(req *ipod.Command, err error) *ACK {
	if err != nil {
		return &ACK{Status: ACKStatusFailed, CmdID: req.ID.CmdID()}
	}
	return &ACK{Status: ACKStatusSuccess, CmdID: req.ID.CmdID()}
}

// HandleExtRemote dispatches an incoming ext-remote command. The dev argument
// provides the live backing data (track info, play status, controls).
func HandleExtRemote(req *ipod.Command, tr ipod.CommandWriter, dev DeviceExtRemote) error {
	switch msg := req.Payload.(type) {

	case *GetCurrentPlayingTrackChapterInfo:
		ipod.Respond(req, tr, &ReturnCurrentPlayingTrackChapterInfo{
			CurrentChapterIndex: 0,
			ChapterCount:        1,
		})
	case *SetCurrentPlayingTrackChapter:
		ipod.Respond(req, tr, ackSuccess(req))
	case *GetCurrentPlayingTrackChapterPlayStatus:
		length, pos, _ := playbackStatus(dev)
		ipod.Respond(req, tr, &ReturnCurrentPlayingTrackChapterPlayStatus{
			ChapterPosition: pos,
			ChapterLength:   length,
		})
	case *GetCurrentPlayingTrackChapterName:
		ipod.Respond(req, tr, &ReturnCurrentPlayingTrackChapterName{
			ChapterName: ipod.StringToBytes("chapter"),
		})
	case *GetAudiobookSpeed:
		ipod.Respond(req, tr, &ReturnAudiobookSpeed{
			Speed: 0,
		})
	case *SetAudiobookSpeed:
		ipod.Respond(req, tr, ackSuccess(req))
	case *GetIndexedPlayingTrackInfo:
		var info interface{}
		switch msg.InfoType {
		case TrackInfoCaps:
			length, _, _ := playbackStatus(dev)
			info = &TrackCaps{
				Caps:         0x0,
				TrackLength:  length,
				ChapterCount: 1,
			}
		case TrackInfoDescription, TrackInfoLyrics:
			info = &TrackLongText{
				Flags:       0x0,
				PacketIndex: 0,
				Text:        0x00,
			}
		case TrackInfoArtworkCount:
			info = struct{}{}
		default:
			info = []byte{0x00}

		}
		ipod.Respond(req, tr, &ReturnIndexedPlayingTrackInfo{
			InfoType: msg.InfoType,
			Info:     info,
		})
	case *GetArtworkFormats:
		ipod.Respond(req, tr, &RetArtworkFormats{})
	case *GetTrackArtworkData:
		ipod.Respond(req, tr, &ACK{
			Status: ACKStatusFailed,
			CmdID:  req.ID.CmdID(),
		})
	case *ResetDBSelection:
		ipod.Respond(req, tr, ackSuccess(req))
	case *SelectDBRecord:
		ipod.Respond(req, tr, ackSuccess(req))
	case *GetNumberCategorizedDBRecords:
		ipod.Respond(req, tr, &ReturnNumberCategorizedDBRecords{
			RecordCount: 1,
		})
	case *RetrieveCategorizedDatabaseRecords:
		ipod.Respond(req, tr, &ReturnCategorizedDatabaseRecord{})
	case *GetPlayStatus:
		length, pos, state := playbackStatus(dev)
		ipod.Respond(req, tr, &ReturnPlayStatus{
			TrackLength:   length,
			TrackPosition: pos,
			State:         state,
		})
	case *GetCurrentPlayingTrackIndex:
		var idx int32
		if dev != nil {
			if n := dev.Track().TrackNumber; n > 0 {
				idx = int32(n - 1)
			}
		}
		ipod.Respond(req, tr, &ReturnCurrentPlayingTrackIndex{
			TrackIndex: idx,
		})
	case *GetIndexedPlayingTrackTitle:
		ipod.Respond(req, tr, &ReturnIndexedPlayingTrackTitle{
			Title: ipod.StringToBytes(trackMeta(dev).Title),
		})
	case *GetIndexedPlayingTrackArtistName:
		ipod.Respond(req, tr, &ReturnIndexedPlayingTrackArtistName{
			ArtistName: ipod.StringToBytes(trackMeta(dev).Artist),
		})
	case *GetIndexedPlayingTrackAlbumName:
		ipod.Respond(req, tr, &ReturnIndexedPlayingTrackAlbumName{
			AlbumName: ipod.StringToBytes(trackMeta(dev).Album),
		})
	case *SetPlayStatusChangeNotification:
		ipod.Respond(req, tr, ackSuccess(req))
	case *SetPlayStatusChangeNotificationShort:
		ipod.Respond(req, tr, ackSuccess(req))
	case *PlayCurrentSelection:
		ipod.Respond(req, tr, ackSuccess(req))
	case *PlayControl:
		var err error
		if dev != nil {
			err = dev.PlayControl(msg.Cmd)
		}
		ipod.Respond(req, tr, ackStatus(req, err))
	case *GetTrackArtworkTimes:
		ipod.Respond(req, tr, &RetTrackArtworkTimes{})
	case *GetShuffle:
		mode := ShuffleOff
		if dev != nil {
			mode = dev.Shuffle()
		}
		ipod.Respond(req, tr, &ReturnShuffle{Mode: mode})
	case *SetShuffle:
		var err error
		if dev != nil {
			err = dev.SetShuffle(msg.Mode)
		}
		ipod.Respond(req, tr, ackStatus(req, err))

	case *GetRepeat:
		mode := RepeatOff
		if dev != nil {
			mode = dev.Repeat()
		}
		ipod.Respond(req, tr, &ReturnRepeat{Mode: mode})
	case *SetRepeat:
		var err error
		if dev != nil {
			err = dev.SetRepeat(msg.Mode)
		}
		ipod.Respond(req, tr, ackStatus(req, err))

	case *SetDisplayImage:
		ipod.Respond(req, tr, ackSuccess(req))
	case *GetMonoDisplayImageLimits:
		ipod.Respond(req, tr, &ReturnMonoDisplayImageLimits{
			MaxWidth:    640,
			MaxHeight:   960,
			PixelFormat: 0x01,
		})
	case *GetNumPlayingTracks:
		var n uint32 = 1
		if dev != nil {
			if t := dev.Track().NumberOfTracks; t > 0 {
				n = t
			}
		}
		ipod.Respond(req, tr, &ReturnNumPlayingTracks{
			NumTracks: n,
		})
	case *SetCurrentPlayingTrack:
		var err error
		if dev != nil {
			current := int32(0)
			if n := dev.Track().TrackNumber; n > 0 {
				current = int32(n - 1)
			}
			switch {
			case msg.TrackIndex > current:
				err = dev.PlayControl(PlayControlNextTrack)
			case msg.TrackIndex < current:
				err = dev.PlayControl(PlayControlPrevTrack)
			}
		}
		ipod.Respond(req, tr, ackStatus(req, err))
	case *SelectSortDBRecord:
		ipod.Respond(req, tr, ackSuccess(req))
	case *GetColorDisplayImageLimits:
		ipod.Respond(req, tr, &ReturnColorDisplayImageLimits{
			MaxWidth:    640,
			MaxHeight:   960,
			PixelFormat: 0x01,
		})
	case *ResetDBSelectionHierarchy:
		ipod.Respond(req, tr, &ACK{Status: ACKStatusFailed, CmdID: req.ID.CmdID()})

	case *GetDBiTunesInfo:
	// RetDBiTunesInfo:
	case *GetUIDTrackInfo:
	// RetUIDTrackInfo:
	case *GetDBTrackInfo:
	// RetDBTrackInfo:
	case *GetPBTrackInfo:
	// RetPBTrackInfo:

	default:
		_ = msg
	}
	return nil
}

func playbackStatus(dev DeviceExtRemote) (trackLength, trackPos uint32, state PlayerState) {
	if dev == nil {
		return 0, 0, PlayerStateStopped
	}
	return dev.PlaybackStatus()
}

func trackMeta(dev DeviceExtRemote) TrackMetadata {
	if dev == nil {
		return TrackMetadata{}
	}
	return dev.Track()
}
