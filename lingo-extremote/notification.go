package extremote

import "sync"

const (
	PlayStatusNotificationBasicPlayState uint32 = 1 << iota
	PlayStatusNotificationExtendedPlayState
	PlayStatusNotificationTrackIndex
	PlayStatusNotificationTrackTimeOffsetMs
	PlayStatusNotificationTrackTimeOffsetSec
	PlayStatusNotificationChapterIndex
)

const playStatusNotificationShortMask = PlayStatusNotificationBasicPlayState |
	PlayStatusNotificationExtendedPlayState |
	PlayStatusNotificationTrackIndex |
	PlayStatusNotificationTrackTimeOffsetMs |
	PlayStatusNotificationTrackTimeOffsetSec |
	PlayStatusNotificationChapterIndex

type PlayStatusNotificationState struct {
	mu   sync.RWMutex
	mask uint32
}

func NewPlayStatusNotificationState() *PlayStatusNotificationState {
	return &PlayStatusNotificationState{}
}

func (s *PlayStatusNotificationState) SetMask(mask uint32) {
	if s == nil {
		return
	}
	s.mu.Lock()
	s.mask = mask
	s.mu.Unlock()
}

func (s *PlayStatusNotificationState) SetShort(enabled bool) {
	if enabled {
		s.SetMask(playStatusNotificationShortMask)
		return
	}
	s.SetMask(0)
}

func (s *PlayStatusNotificationState) Mask() uint32 {
	if s == nil {
		return 0
	}
	s.mu.RLock()
	defer s.mu.RUnlock()
	return s.mask
}

func (s *PlayStatusNotificationState) TrackIndexEnabled() bool {
	return s.Mask()&PlayStatusNotificationTrackIndex != 0
}

type TrackChange struct {
	Track      TrackMetadata
	TrackIndex int32
}

type TrackChangeSource interface {
	TrackChanges() <-chan TrackChange
}

func TrackIndexFromMetadata(t TrackMetadata) int32 {
	if t.TrackNumber == 0 {
		return 0
	}
	const maxInt32 = uint32(1<<31 - 1)
	idx := t.TrackNumber - 1
	if idx > maxInt32 {
		return int32(maxInt32)
	}
	return int32(idx)
}
