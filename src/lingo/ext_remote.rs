use crate::codec::{put_bool, put_i16, put_i32, put_u16, put_u32, Cursor, WireDecode, WireEncode};
use crate::command::{respond, Command, CommandPayload, CommandWriter};
use crate::util::{bool_from_wire, string_to_bytes};
use crate::{Error, Result};

pub type AckStatus = u8;
pub const ACK_STATUS_SUCCESS: AckStatus = 0x00;
pub const ACK_STATUS_FAILED: AckStatus = 0x02;

pub type TrackInfoType = u8;
pub const TRACK_INFO_CAPS: TrackInfoType = 0x00;
pub const TRACK_INFO_PODCAST_NAME: TrackInfoType = 0x01;
pub const TRACK_INFO_RELEASE_DATE: TrackInfoType = 0x02;
pub const TRACK_INFO_DESCRIPTION: TrackInfoType = 0x03;
pub const TRACK_INFO_LYRICS: TrackInfoType = 0x04;
pub const TRACK_INFO_GENRE: TrackInfoType = 0x05;
pub const TRACK_INFO_COMPOSER: TrackInfoType = 0x06;
pub const TRACK_INFO_ARTWORK_COUNT: TrackInfoType = 0x07;

pub type DbCategoryType = u8;
pub const DB_CATEGORY_PLAYLIST: DbCategoryType = 0x01;
pub const DB_CATEGORY_ARTIST: DbCategoryType = 0x02;
pub const DB_CATEGORY_ALBUM: DbCategoryType = 0x03;
pub const DB_CATEGORY_GENRE: DbCategoryType = 0x04;
pub const DB_CATEGORY_TRACK: DbCategoryType = 0x05;
pub const DB_CATEGORY_COMPOSER: DbCategoryType = 0x06;
pub const DB_CATEGORY_AUDIOBOOK: DbCategoryType = 0x07;
pub const DB_CATEGORY_PODCAST: DbCategoryType = 0x08;
pub const DB_CATEGORY_NESTED_PLAYLIST: DbCategoryType = 0x09;

pub type PlayerState = u8;
pub const PLAYER_STATE_STOPPED: PlayerState = 0x00;
pub const PLAYER_STATE_PLAYING: PlayerState = 0x01;
pub const PLAYER_STATE_PAUSED: PlayerState = 0x02;
pub const PLAYER_STATE_ERROR: PlayerState = 0xff;

pub type PlayControlCmd = u8;
pub const PLAY_CONTROL_TOGGLE: PlayControlCmd = 0x01;
pub const PLAY_CONTROL_STOP: PlayControlCmd = 0x02;
pub const PLAY_CONTROL_NEXT_TRACK: PlayControlCmd = 0x03;
pub const PLAY_CONTROL_PREV_TRACK: PlayControlCmd = 0x04;
pub const PLAY_CONTROL_START_FF: PlayControlCmd = 0x05;
pub const PLAY_CONTROL_START_REW: PlayControlCmd = 0x06;
pub const PLAY_CONTROL_END_FF_REW: PlayControlCmd = 0x07;
pub const PLAY_CONTROL_NEXT: PlayControlCmd = 0x08;
pub const PLAY_CONTROL_PREV: PlayControlCmd = 0x09;
pub const PLAY_CONTROL_PLAY: PlayControlCmd = 0x0a;
pub const PLAY_CONTROL_PAUSE: PlayControlCmd = 0x0b;
pub const PLAY_CONTROL_NEXT_CHAPTER: PlayControlCmd = 0x0c;
pub const PLAY_CONTROL_PREV_CHAPTER: PlayControlCmd = 0x0d;

pub type ShuffleMode = u8;
pub const SHUFFLE_OFF: ShuffleMode = 0x00;
pub const SHUFFLE_TRACKS: ShuffleMode = 0x01;
pub const SHUFFLE_ALBUMS: ShuffleMode = 0x02;

pub type RepeatMode = u8;
pub const REPEAT_OFF: RepeatMode = 0x00;
pub const REPEAT_ONE: RepeatMode = 0x01;
pub const REPEAT_ALL: RepeatMode = 0x02;

macro_rules! empty_payload {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Default)]
        pub struct $name;

        impl WireEncode for $name {
            fn encode(&self, _out: &mut Vec<u8>) -> Result<()> {
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                Cursor::new(data).finish()?;
                Ok(Self)
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ack {
    pub status: AckStatus,
    pub cmd_id: u16,
}

impl WireEncode for Ack {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.status);
        put_u16(out, self.cmd_id);
        Ok(())
    }
}

impl WireDecode for Ack {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            status: cursor.read_u8()?,
            cmd_id: cursor.read_u16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

empty_payload!(GetCurrentPlayingTrackChapterInfo);
empty_payload!(GetAudiobookSpeed);
empty_payload!(GetArtworkFormats);
empty_payload!(ResetDbSelection);
empty_payload!(GetPlayStatus);
empty_payload!(GetCurrentPlayingTrackIndex);
empty_payload!(GetShuffle);
empty_payload!(GetRepeat);
empty_payload!(SetDisplayImage);
empty_payload!(GetMonoDisplayImageLimits);
empty_payload!(GetNumPlayingTracks);
empty_payload!(GetColorDisplayImageLimits);
empty_payload!(GetDbITunesInfo);
empty_payload!(RetDbITunesInfo);
empty_payload!(GetUidTrackInfo);
empty_payload!(RetUidTrackInfo);
empty_payload!(GetDbTrackInfo);
empty_payload!(RetDbTrackInfo);
empty_payload!(GetPbTrackInfo);
empty_payload!(RetPbTrackInfo);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnCurrentPlayingTrackChapterInfo {
    pub current_chapter_index: i32,
    pub chapter_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCurrentPlayingTrackChapter {
    pub chapter_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetCurrentPlayingTrackChapterPlayStatus {
    pub current_chapter_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnCurrentPlayingTrackChapterPlayStatus {
    pub chapter_length: u32,
    pub chapter_position: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetCurrentPlayingTrackChapterName {
    pub chapter_index: i32,
}

macro_rules! one_i32 {
    ($name:ident, $field:ident) => {
        impl WireEncode for $name {
            fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                put_i32(out, self.$field);
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                let mut cursor = Cursor::new(data);
                let value = Self {
                    $field: cursor.read_i32()?,
                };
                cursor.finish()?;
                Ok(value)
            }
        }
    };
}

one_i32!(SetCurrentPlayingTrackChapter, chapter_index);
one_i32!(
    GetCurrentPlayingTrackChapterPlayStatus,
    current_chapter_index
);
one_i32!(GetCurrentPlayingTrackChapterName, chapter_index);

impl WireEncode for ReturnCurrentPlayingTrackChapterInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_i32(out, self.current_chapter_index);
        put_i32(out, self.chapter_count);
        Ok(())
    }
}

impl WireDecode for ReturnCurrentPlayingTrackChapterInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            current_chapter_index: cursor.read_i32()?,
            chapter_count: cursor.read_i32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for ReturnCurrentPlayingTrackChapterPlayStatus {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.chapter_length);
        put_u32(out, self.chapter_position);
        Ok(())
    }
}

impl WireDecode for ReturnCurrentPlayingTrackChapterPlayStatus {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            chapter_length: cursor.read_u32()?,
            chapter_position: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReturnCurrentPlayingTrackChapterName {
    pub chapter_name: Vec<u8>,
}

macro_rules! bytes_payload {
    ($name:ident, $field:ident) => {
        impl WireEncode for $name {
            fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                out.extend_from_slice(&self.$field);
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                Ok(Self {
                    $field: data.to_vec(),
                })
            }
        }
    };
}

bytes_payload!(ReturnCurrentPlayingTrackChapterName, chapter_name);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnAudiobookSpeed {
    pub speed: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetAudiobookSpeed {
    pub speed: u8,
}

macro_rules! one_u8 {
    ($name:ident, $field:ident) => {
        impl WireEncode for $name {
            fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                out.push(self.$field);
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                let mut cursor = Cursor::new(data);
                let value = Self {
                    $field: cursor.read_u8()?,
                };
                cursor.finish()?;
                Ok(value)
            }
        }
    };
}

one_u8!(ReturnAudiobookSpeed, speed);
one_u8!(SetAudiobookSpeed, speed);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackCaps {
    pub caps: u32,
    pub track_length: u32,
    pub chapter_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackLongText {
    pub flags: u8,
    pub packet_index: u16,
    pub text: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexedTrackInfo {
    Caps(TrackCaps),
    LongText(TrackLongText),
    Empty,
    Raw(Vec<u8>),
}

impl IndexedTrackInfo {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Caps(value) => {
                put_u32(out, value.caps);
                put_u32(out, value.track_length);
                put_u16(out, value.chapter_count);
            }
            Self::LongText(value) => {
                out.push(value.flags);
                put_u16(out, value.packet_index);
                out.extend_from_slice(&value.text);
            }
            Self::Empty => {}
            Self::Raw(value) => out.extend_from_slice(value),
        }
    }

    fn decode(info_type: TrackInfoType, data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(match info_type {
            TRACK_INFO_CAPS => {
                let value = Self::Caps(TrackCaps {
                    caps: cursor.read_u32()?,
                    track_length: cursor.read_u32()?,
                    chapter_count: cursor.read_u16()?,
                });
                cursor.finish()?;
                value
            }
            TRACK_INFO_DESCRIPTION | TRACK_INFO_LYRICS => Self::LongText(TrackLongText {
                flags: cursor.read_u8()?,
                packet_index: cursor.read_u16()?,
                text: cursor.read_rest().to_vec(),
            }),
            TRACK_INFO_ARTWORK_COUNT => {
                cursor.finish()?;
                Self::Empty
            }
            _ => Self::Raw(data.to_vec()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetIndexedPlayingTrackInfo {
    pub info_type: TrackInfoType,
    pub track_index: i32,
    pub chapter_index: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnIndexedPlayingTrackInfo {
    pub info_type: TrackInfoType,
    pub info: IndexedTrackInfo,
}

impl WireEncode for GetIndexedPlayingTrackInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        put_i32(out, self.track_index);
        put_i16(out, self.chapter_index);
        Ok(())
    }
}

impl WireDecode for GetIndexedPlayingTrackInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            info_type: cursor.read_u8()?,
            track_index: cursor.read_i32()?,
            chapter_index: cursor.read_i16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for ReturnIndexedPlayingTrackInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        self.info.encode(out);
        Ok(())
    }
}

impl WireDecode for ReturnIndexedPlayingTrackInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let info_type = cursor.read_u8()?;
        let info = IndexedTrackInfo::decode(info_type, cursor.read_rest())?;
        Ok(Self { info_type, info })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtworkFormat {
    pub format_id: u16,
    pub pixel_format: u8,
    pub image_width: u16,
    pub image_height: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetArtworkFormats {
    pub formats: Vec<ArtworkFormat>,
}

impl WireEncode for RetArtworkFormats {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        for format in &self.formats {
            put_u16(out, format.format_id);
            out.push(format.pixel_format);
            put_u16(out, format.image_width);
            put_u16(out, format.image_height);
        }
        Ok(())
    }
}

impl WireDecode for RetArtworkFormats {
    fn decode(data: &[u8]) -> Result<Self> {
        if data.len() % 7 != 0 {
            return Error::invalid("artwork format payload length is not a multiple of 7");
        }
        let mut cursor = Cursor::new(data);
        let mut formats = Vec::new();
        while !cursor.is_empty() {
            formats.push(ArtworkFormat {
                format_id: cursor.read_u16()?,
                pixel_format: cursor.read_u8()?,
                image_width: cursor.read_u16()?,
                image_height: cursor.read_u16()?,
            });
        }
        Ok(Self { formats })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTrackArtworkData {
    pub track_index: i32,
    pub format_id: u16,
    pub offset: u32,
}

impl WireEncode for GetTrackArtworkData {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_i32(out, self.track_index);
        put_u16(out, self.format_id);
        put_u32(out, self.offset);
        Ok(())
    }
}

impl WireDecode for GetTrackArtworkData {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            track_index: cursor.read_i32()?,
            format_id: cursor.read_u16()?,
            offset: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetTrackArtworkData {
    pub packet_index: u16,
    pub pixel_format: u8,
    pub image_width: u16,
    pub image_height: u16,
    pub top_left_x: u16,
    pub top_left_y: u16,
    pub bottom_right_x: u16,
    pub bottom_right_y: u16,
    pub row_size: u32,
    pub data: Vec<u8>,
}

impl WireEncode for RetTrackArtworkData {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u16(out, self.packet_index);
        out.push(self.pixel_format);
        put_u16(out, self.image_width);
        put_u16(out, self.image_height);
        put_u16(out, self.top_left_x);
        put_u16(out, self.top_left_y);
        put_u16(out, self.bottom_right_x);
        put_u16(out, self.bottom_right_y);
        put_u32(out, self.row_size);
        out.extend_from_slice(&self.data);
        Ok(())
    }
}

impl WireDecode for RetTrackArtworkData {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            packet_index: cursor.read_u16()?,
            pixel_format: cursor.read_u8()?,
            image_width: cursor.read_u16()?,
            image_height: cursor.read_u16()?,
            top_left_x: cursor.read_u16()?,
            top_left_y: cursor.read_u16()?,
            bottom_right_x: cursor.read_u16()?,
            bottom_right_y: cursor.read_u16()?,
            row_size: cursor.read_u32()?,
            data: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectDbRecord {
    pub category_type: DbCategoryType,
    pub record_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetNumberCategorizedDbRecords {
    pub category_type: DbCategoryType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnNumberCategorizedDbRecords {
    pub record_count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetrieveCategorizedDatabaseRecords {
    pub category_type: DbCategoryType,
    pub offset: u32,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReturnCategorizedDatabaseRecord {
    pub record_category_index: u32,
    pub string: [u8; 16],
}

impl WireEncode for SelectDbRecord {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.category_type);
        put_i32(out, self.record_index);
        Ok(())
    }
}

impl WireDecode for SelectDbRecord {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            category_type: cursor.read_u8()?,
            record_index: cursor.read_i32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

one_u8!(GetNumberCategorizedDbRecords, category_type);
one_i32!(ReturnNumberCategorizedDbRecords, record_count);

impl WireEncode for RetrieveCategorizedDatabaseRecords {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.category_type);
        put_u32(out, self.offset);
        put_i32(out, self.count);
        Ok(())
    }
}

impl WireDecode for RetrieveCategorizedDatabaseRecords {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            category_type: cursor.read_u8()?,
            offset: cursor.read_u32()?,
            count: cursor.read_i32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for ReturnCategorizedDatabaseRecord {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.record_category_index);
        out.extend_from_slice(&self.string);
        Ok(())
    }
}

impl WireDecode for ReturnCategorizedDatabaseRecord {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            record_category_index: cursor.read_u32()?,
            string: cursor.read_array()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnPlayStatus {
    pub track_length: u32,
    pub track_position: u32,
    pub state: PlayerState,
}

impl WireEncode for ReturnPlayStatus {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.track_length);
        put_u32(out, self.track_position);
        out.push(self.state);
        Ok(())
    }
}

impl WireDecode for ReturnPlayStatus {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            track_length: cursor.read_u32()?,
            track_position: cursor.read_u32()?,
            state: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnCurrentPlayingTrackIndex {
    pub track_index: i32,
}

one_i32!(ReturnCurrentPlayingTrackIndex, track_index);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetIndexedPlayingTrackTitle {
    pub track_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReturnIndexedPlayingTrackTitle {
    pub title: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetIndexedPlayingTrackArtistName {
    pub track_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReturnIndexedPlayingTrackArtistName {
    pub artist_name: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetIndexedPlayingTrackAlbumName {
    pub track_index: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReturnIndexedPlayingTrackAlbumName {
    pub album_name: Vec<u8>,
}

one_i32!(GetIndexedPlayingTrackTitle, track_index);
bytes_payload!(ReturnIndexedPlayingTrackTitle, title);
one_i32!(GetIndexedPlayingTrackArtistName, track_index);
bytes_payload!(ReturnIndexedPlayingTrackArtistName, artist_name);
one_i32!(GetIndexedPlayingTrackAlbumName, track_index);
bytes_payload!(ReturnIndexedPlayingTrackAlbumName, album_name);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPlayStatusChangeNotification {
    pub event_mask: u32,
}

macro_rules! one_u32_payload {
    ($name:ident, $field:ident) => {
        impl WireEncode for $name {
            fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                put_u32(out, self.$field);
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                let mut cursor = Cursor::new(data);
                let value = Self {
                    $field: cursor.read_u32()?,
                };
                cursor.finish()?;
                Ok(value)
            }
        }
    };
}

one_u32_payload!(SetPlayStatusChangeNotification, event_mask);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetPlayStatusChangeNotificationShort {
    pub enabled: bool,
}

impl WireEncode for SetPlayStatusChangeNotificationShort {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_bool(out, self.enabled);
        Ok(())
    }
}

impl WireDecode for SetPlayStatusChangeNotificationShort {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            enabled: bool_from_wire(cursor.read_u8()?),
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayStatusChangeNotification {
    pub status: u8,
}

one_u8!(PlayStatusChangeNotification, status);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayCurrentSelection {
    pub selected_track_index: i32,
}

one_i32!(PlayCurrentSelection, selected_track_index);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayControl {
    pub cmd: PlayControlCmd,
}

one_u8!(PlayControl, cmd);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTrackArtworkTimes {
    pub track_index: i32,
    pub format_id: u16,
    pub artwork_index: u16,
    pub artwork_count: i16,
}

empty_payload!(RetTrackArtworkTimes);

impl WireEncode for GetTrackArtworkTimes {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_i32(out, self.track_index);
        put_u16(out, self.format_id);
        put_u16(out, self.artwork_index);
        put_i16(out, self.artwork_count);
        Ok(())
    }
}

impl WireDecode for GetTrackArtworkTimes {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            track_index: cursor.read_i32()?,
            format_id: cursor.read_u16()?,
            artwork_index: cursor.read_u16()?,
            artwork_count: cursor.read_i16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnShuffle {
    pub mode: ShuffleMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetShuffle {
    pub mode: ShuffleMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnRepeat {
    pub mode: RepeatMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetRepeat {
    pub mode: RepeatMode,
}

one_u8!(ReturnShuffle, mode);
one_u8!(SetShuffle, mode);
one_u8!(ReturnRepeat, mode);
one_u8!(SetRepeat, mode);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnMonoDisplayImageLimits {
    pub max_width: u16,
    pub max_height: u16,
    pub pixel_format: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnColorDisplayImageLimits {
    pub max_width: u16,
    pub max_height: u16,
    pub pixel_format: u8,
}

macro_rules! display_limits {
    ($name:ident) => {
        impl WireEncode for $name {
            fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                put_u16(out, self.max_width);
                put_u16(out, self.max_height);
                out.push(self.pixel_format);
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                let mut cursor = Cursor::new(data);
                let value = Self {
                    max_width: cursor.read_u16()?,
                    max_height: cursor.read_u16()?,
                    pixel_format: cursor.read_u8()?,
                };
                cursor.finish()?;
                Ok(value)
            }
        }
    };
}

display_limits!(ReturnMonoDisplayImageLimits);
display_limits!(ReturnColorDisplayImageLimits);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnNumPlayingTracks {
    pub num_tracks: u32,
}

one_u32_payload!(ReturnNumPlayingTracks, num_tracks);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCurrentPlayingTrack {
    pub track_index: i32,
}

one_i32!(SetCurrentPlayingTrack, track_index);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectSortDbRecord {
    pub category_type: DbCategoryType,
    pub record_index: i32,
    pub sort_type: u8,
}

impl WireEncode for SelectSortDbRecord {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.category_type);
        put_i32(out, self.record_index);
        out.push(self.sort_type);
        Ok(())
    }
}

impl WireDecode for SelectSortDbRecord {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            category_type: cursor.read_u8()?,
            record_index: cursor.read_i32()?,
            sort_type: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetDbSelectionHierarchy {
    pub selection: u8,
}

one_u8!(ResetDbSelectionHierarchy, selection);

pub trait DeviceExtRemote {
    fn playback_status(&self) -> (u32, u32, PlayerState) {
        (300_000, 20_000, PLAYER_STATE_PAUSED)
    }
}

fn ack_success(req: &Command) -> CommandPayload {
    CommandPayload::ExtAck(Ack {
        status: ACK_STATUS_SUCCESS,
        cmd_id: req.id.cmd_id(),
    })
}

pub fn handle_ext_remote(
    req: &Command,
    writer: &mut impl CommandWriter,
    dev: &mut impl DeviceExtRemote,
) -> Result<()> {
    match &req.payload {
        CommandPayload::ExtGetCurrentPlayingTrackChapterInfo(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnCurrentPlayingTrackChapterInfo(
                ReturnCurrentPlayingTrackChapterInfo {
                    current_chapter_index: 0,
                    chapter_count: 1,
                },
            ),
        ),
        CommandPayload::ExtSetCurrentPlayingTrackChapter(_) => {
            respond(req, writer, ack_success(req))
        }
        CommandPayload::ExtGetCurrentPlayingTrackChapterPlayStatus(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnCurrentPlayingTrackChapterPlayStatus(
                ReturnCurrentPlayingTrackChapterPlayStatus {
                    chapter_position: 0,
                    chapter_length: 0,
                },
            ),
        ),
        CommandPayload::ExtGetCurrentPlayingTrackChapterName(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnCurrentPlayingTrackChapterName(
                ReturnCurrentPlayingTrackChapterName {
                    chapter_name: string_to_bytes("chapter"),
                },
            ),
        ),
        CommandPayload::ExtGetAudiobookSpeed(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnAudiobookSpeed(ReturnAudiobookSpeed { speed: 0 }),
        ),
        CommandPayload::ExtSetAudiobookSpeed(_) => respond(req, writer, ack_success(req)),
        CommandPayload::ExtGetIndexedPlayingTrackInfo(msg) => {
            let info = match msg.info_type {
                TRACK_INFO_CAPS => IndexedTrackInfo::Caps(TrackCaps {
                    caps: 0,
                    track_length: 300_000,
                    chapter_count: 1,
                }),
                TRACK_INFO_DESCRIPTION | TRACK_INFO_LYRICS => {
                    IndexedTrackInfo::LongText(TrackLongText {
                        flags: 0,
                        packet_index: 0,
                        text: vec![0],
                    })
                }
                TRACK_INFO_ARTWORK_COUNT => IndexedTrackInfo::Empty,
                _ => IndexedTrackInfo::Raw(vec![0]),
            };
            respond(
                req,
                writer,
                CommandPayload::ExtReturnIndexedPlayingTrackInfo(ReturnIndexedPlayingTrackInfo {
                    info_type: msg.info_type,
                    info,
                }),
            );
        }
        CommandPayload::ExtGetArtworkFormats(_) => respond(
            req,
            writer,
            CommandPayload::ExtRetArtworkFormats(RetArtworkFormats::default()),
        ),
        CommandPayload::ExtGetTrackArtworkData(_) => respond(
            req,
            writer,
            CommandPayload::ExtAck(Ack {
                status: ACK_STATUS_FAILED,
                cmd_id: req.id.cmd_id(),
            }),
        ),
        CommandPayload::ExtResetDbSelection(_)
        | CommandPayload::ExtSelectDbRecord(_)
        | CommandPayload::ExtSetPlayStatusChangeNotification(_)
        | CommandPayload::ExtSetPlayStatusChangeNotificationShort(_)
        | CommandPayload::ExtPlayCurrentSelection(_)
        | CommandPayload::ExtPlayControl(_)
        | CommandPayload::ExtSetShuffle(_)
        | CommandPayload::ExtSetRepeat(_)
        | CommandPayload::ExtSetDisplayImage(_)
        | CommandPayload::ExtSetCurrentPlayingTrack(_)
        | CommandPayload::ExtSelectSortDbRecord(_) => respond(req, writer, ack_success(req)),
        CommandPayload::ExtGetNumberCategorizedDbRecords(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnNumberCategorizedDbRecords(ReturnNumberCategorizedDbRecords {
                record_count: 1,
            }),
        ),
        CommandPayload::ExtRetrieveCategorizedDatabaseRecords(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnCategorizedDatabaseRecord(
                ReturnCategorizedDatabaseRecord::default(),
            ),
        ),
        CommandPayload::ExtGetPlayStatus(_) => {
            let (track_length, track_position, state) = dev.playback_status();
            respond(
                req,
                writer,
                CommandPayload::ExtReturnPlayStatus(ReturnPlayStatus {
                    track_length,
                    track_position,
                    state,
                }),
            );
        }
        CommandPayload::ExtGetCurrentPlayingTrackIndex(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnCurrentPlayingTrackIndex(ReturnCurrentPlayingTrackIndex {
                track_index: 0,
            }),
        ),
        CommandPayload::ExtGetIndexedPlayingTrackTitle(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnIndexedPlayingTrackTitle(ReturnIndexedPlayingTrackTitle {
                title: string_to_bytes("title"),
            }),
        ),
        CommandPayload::ExtGetIndexedPlayingTrackArtistName(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnIndexedPlayingTrackArtistName(
                ReturnIndexedPlayingTrackArtistName {
                    artist_name: string_to_bytes("artist"),
                },
            ),
        ),
        CommandPayload::ExtGetIndexedPlayingTrackAlbumName(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnIndexedPlayingTrackAlbumName(
                ReturnIndexedPlayingTrackAlbumName {
                    album_name: string_to_bytes("album"),
                },
            ),
        ),
        CommandPayload::ExtGetTrackArtworkTimes(_) => respond(
            req,
            writer,
            CommandPayload::ExtRetTrackArtworkTimes(RetTrackArtworkTimes),
        ),
        CommandPayload::ExtGetShuffle(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnShuffle(ReturnShuffle { mode: SHUFFLE_OFF }),
        ),
        CommandPayload::ExtGetRepeat(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnRepeat(ReturnRepeat { mode: REPEAT_OFF }),
        ),
        CommandPayload::ExtGetMonoDisplayImageLimits(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnMonoDisplayImageLimits(ReturnMonoDisplayImageLimits {
                max_width: 640,
                max_height: 960,
                pixel_format: 0x01,
            }),
        ),
        CommandPayload::ExtGetNumPlayingTracks(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnNumPlayingTracks(ReturnNumPlayingTracks { num_tracks: 1 }),
        ),
        CommandPayload::ExtGetColorDisplayImageLimits(_) => respond(
            req,
            writer,
            CommandPayload::ExtReturnColorDisplayImageLimits(ReturnColorDisplayImageLimits {
                max_width: 640,
                max_height: 960,
                pixel_format: 0x01,
            }),
        ),
        CommandPayload::ExtResetDbSelectionHierarchy(_) => respond(
            req,
            writer,
            CommandPayload::ExtAck(Ack {
                status: ACK_STATUS_FAILED,
                cmd_id: req.id.cmd_id(),
            }),
        ),
        _ => {}
    }
    Ok(())
}
