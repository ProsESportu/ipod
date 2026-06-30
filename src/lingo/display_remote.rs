use std::time::{SystemTime, UNIX_EPOCH};

use crate::codec::{put_u16, put_u32, Cursor, WireDecode, WireEncode};
use crate::command::{respond, Command, CommandPayload, CommandWriter};
use crate::util::{bool_from_wire, string_to_bytes};
use crate::{Error, Result};

pub type AckStatus = u8;
pub const ACK_STATUS_SUCCESS: AckStatus = 0x00;
pub const ACK_STATUS_PENDING: AckStatus = 0x06;

pub type InfoType = u8;
pub const INFO_TYPE_TRACK_POSITION_MS: InfoType = 0;
pub const INFO_TYPE_TRACK_INDEX: InfoType = 1;
pub const INFO_TYPE_CHAPTER_INDEX: InfoType = 2;
pub const INFO_TYPE_PLAY_STATUS: InfoType = 3;
pub const INFO_TYPE_VOLUME: InfoType = 4;
pub const INFO_TYPE_POWER: InfoType = 5;
pub const INFO_TYPE_EQUALIZER: InfoType = 6;
pub const INFO_TYPE_SHUFFLE: InfoType = 7;
pub const INFO_TYPE_REPEAT: InfoType = 8;
pub const INFO_TYPE_DATE_TIME: InfoType = 9;
pub const INFO_TYPE_BACKLIGHT: InfoType = 11;
pub const INFO_TYPE_HOLD_SWITCH: InfoType = 12;
pub const INFO_TYPE_SOUND_CHECK: InfoType = 13;
pub const INFO_TYPE_AUDIOBOOK_SPEED: InfoType = 14;
pub const INFO_TYPE_TRACK_POSITION_SEC: InfoType = 15;
pub const INFO_TYPE_VOLUME2: InfoType = 16;

pub type PlayStatusType = u8;
pub const PLAY_STATUS_STOPPED: PlayStatusType = 0;
pub const PLAY_STATUS_PLAYING: PlayStatusType = 1;
pub const PLAY_STATUS_PAUSED: PlayStatusType = 2;
pub const PLAY_STATUS_FF: PlayStatusType = 3;
pub const PLAY_STATUS_REW: PlayStatusType = 4;
pub const PLAY_STATUS_END_FF_REW: PlayStatusType = 5;

pub type TrackInfoType = u8;
pub const TRACK_INFO_TYPE_CAPS: TrackInfoType = 0;
pub const TRACK_INFO_TYPE_CHAPTER_TIME_NAME: TrackInfoType = 1;
pub const TRACK_INFO_TYPE_ARTIST: TrackInfoType = 2;
pub const TRACK_INFO_TYPE_ALBUM: TrackInfoType = 3;
pub const TRACK_INFO_TYPE_GENRE: TrackInfoType = 4;
pub const TRACK_INFO_TYPE_TRACK: TrackInfoType = 5;
pub const TRACK_INFO_TYPE_COMPOSER: TrackInfoType = 6;
pub const TRACK_INFO_TYPE_LYRICS: TrackInfoType = 7;
pub const TRACK_INFO_TYPE_ARTWORK_COUNT: TrackInfoType = 8;

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
    pub cmd_id: u8,
}

impl WireEncode for Ack {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.status);
        out.push(self.cmd_id);
        Ok(())
    }
}

impl WireDecode for Ack {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            status: cursor.read_u8()?,
            cmd_id: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

empty_payload!(GetCurrentEqProfileIndex);
empty_payload!(GetNumEqProfiles);
empty_payload!(GetRemoteEventStatus);
empty_payload!(GetPlayStatus);
empty_payload!(GetNumPlayingTracks);
empty_payload!(GetArtworkFormats);
empty_payload!(GetPowerBatteryState);
empty_payload!(GetSoundCheckState);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetCurrentEqProfileIndex {
    pub current_eq_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCurrentEqProfileIndex {
    pub current_eq_index: u32,
    pub restore_on_exit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetNumEqProfiles {
    pub num_eq_profiles: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetIndexedEqProfileName {
    pub eq_profile_index: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetIndexedEqProfileName {
    pub eq_profile_name: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetRemoteEventNotification {
    pub event_mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteEventNotification {
    pub event_num: u8,
    pub event_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetRemoteEventStatus {
    pub event_status: u32,
}

macro_rules! one_u32 {
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

one_u32!(RetCurrentEqProfileIndex, current_eq_index);
one_u32!(RetNumEqProfiles, num_eq_profiles);
one_u32!(GetIndexedEqProfileName, eq_profile_index);
one_u32!(SetRemoteEventNotification, event_mask);
one_u32!(RetRemoteEventStatus, event_status);

impl WireEncode for SetCurrentEqProfileIndex {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.current_eq_index);
        crate::codec::put_bool(out, self.restore_on_exit);
        Ok(())
    }
}

impl WireDecode for SetCurrentEqProfileIndex {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            current_eq_index: cursor.read_u32()?,
            restore_on_exit: bool_from_wire(cursor.read_u8()?),
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for RetIndexedEqProfileName {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.eq_profile_name);
        Ok(())
    }
}

impl WireDecode for RetIndexedEqProfileName {
    fn decode(data: &[u8]) -> Result<Self> {
        Ok(Self {
            eq_profile_name: data.to_vec(),
        })
    }
}

impl WireEncode for RemoteEventNotification {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.event_num);
        out.extend_from_slice(&self.event_data);
        Ok(())
    }
}

impl WireDecode for RemoteEventNotification {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            event_num: cursor.read_u8()?,
            event_data: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateInfo {
    TrackPositionMs(u32),
    TrackIndex(u32),
    ChapterIndex {
        track_index: u32,
        chapter_count: u16,
        chapter_index: u16,
    },
    PlayStatus(PlayStatusType),
    Volume {
        mute_state: u8,
        ui_volume_level: u8,
    },
    Power {
        power_state: u8,
        battery_level: u8,
    },
    Equalizer(u32),
    Shuffle(u8),
    Repeat(u8),
    DateTime {
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
    },
    Backlight(u8),
    HoldSwitch(u8),
    SoundCheck(u8),
    AudiobookSpeed(u8),
    TrackPositionSec(u16),
    Volume2 {
        mute_state: u8,
        ui_volume_level: u8,
        absolute_volume_level: u8,
    },
}

impl StateInfo {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::TrackPositionMs(value) | Self::TrackIndex(value) | Self::Equalizer(value) => {
                put_u32(out, *value)
            }
            Self::ChapterIndex {
                track_index,
                chapter_count,
                chapter_index,
            } => {
                put_u32(out, *track_index);
                put_u16(out, *chapter_count);
                put_u16(out, *chapter_index);
            }
            Self::PlayStatus(value)
            | Self::Shuffle(value)
            | Self::Repeat(value)
            | Self::Backlight(value)
            | Self::HoldSwitch(value)
            | Self::SoundCheck(value)
            | Self::AudiobookSpeed(value) => out.push(*value),
            Self::Volume {
                mute_state,
                ui_volume_level,
            }
            | Self::Power {
                power_state: mute_state,
                battery_level: ui_volume_level,
            } => {
                out.push(*mute_state);
                out.push(*ui_volume_level);
            }
            Self::DateTime {
                year,
                month,
                day,
                hour,
                minute,
            } => {
                put_u16(out, *year);
                out.push(*month);
                out.push(*day);
                out.push(*hour);
                out.push(*minute);
            }
            Self::TrackPositionSec(value) => put_u16(out, *value),
            Self::Volume2 {
                mute_state,
                ui_volume_level,
                absolute_volume_level,
            } => {
                out.push(*mute_state);
                out.push(*ui_volume_level);
                out.push(*absolute_volume_level);
            }
        }
    }

    fn decode(info_type: InfoType, data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = match info_type {
            INFO_TYPE_TRACK_POSITION_MS => Self::TrackPositionMs(cursor.read_u32()?),
            INFO_TYPE_TRACK_INDEX => Self::TrackIndex(cursor.read_u32()?),
            INFO_TYPE_CHAPTER_INDEX => Self::ChapterIndex {
                track_index: cursor.read_u32()?,
                chapter_count: cursor.read_u16()?,
                chapter_index: cursor.read_u16()?,
            },
            INFO_TYPE_PLAY_STATUS => Self::PlayStatus(cursor.read_u8()?),
            INFO_TYPE_VOLUME => Self::Volume {
                mute_state: cursor.read_u8()?,
                ui_volume_level: cursor.read_u8()?,
            },
            INFO_TYPE_POWER => Self::Power {
                power_state: cursor.read_u8()?,
                battery_level: cursor.read_u8()?,
            },
            INFO_TYPE_EQUALIZER => Self::Equalizer(cursor.read_u32()?),
            INFO_TYPE_SHUFFLE => Self::Shuffle(cursor.read_u8()?),
            INFO_TYPE_REPEAT => Self::Repeat(cursor.read_u8()?),
            INFO_TYPE_DATE_TIME => Self::DateTime {
                year: cursor.read_u16()?,
                month: cursor.read_u8()?,
                day: cursor.read_u8()?,
                hour: cursor.read_u8()?,
                minute: cursor.read_u8()?,
            },
            INFO_TYPE_BACKLIGHT => Self::Backlight(cursor.read_u8()?),
            INFO_TYPE_HOLD_SWITCH => Self::HoldSwitch(cursor.read_u8()?),
            INFO_TYPE_SOUND_CHECK => Self::SoundCheck(cursor.read_u8()?),
            INFO_TYPE_AUDIOBOOK_SPEED => Self::AudiobookSpeed(cursor.read_u8()?),
            INFO_TYPE_TRACK_POSITION_SEC => Self::TrackPositionSec(cursor.read_u16()?),
            INFO_TYPE_VOLUME2 => Self::Volume2 {
                mute_state: cursor.read_u8()?,
                ui_volume_level: cursor.read_u8()?,
                absolute_volume_level: cursor.read_u8()?,
            },
            _ => return Error::invalid("unknown display-remote info type"),
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetiPodStateInfo {
    pub info_type: InfoType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiPodStateInfo {
    pub info_type: InfoType,
    pub info_data: StateInfo,
}

impl WireEncode for GetiPodStateInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        Ok(())
    }
}

impl WireDecode for GetiPodStateInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            info_type: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for RetiPodStateInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        self.info_data.encode(out);
        Ok(())
    }
}

impl WireDecode for RetiPodStateInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let info_type = cursor.read_u8()?;
        let info_data = StateInfo::decode(info_type, cursor.read_rest())?;
        Ok(Self {
            info_type,
            info_data,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetiPodStateInfo {
    pub info_type: u8,
    pub info_data: u8,
}

impl WireEncode for SetiPodStateInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        out.push(self.info_data);
        Ok(())
    }
}

impl WireDecode for SetiPodStateInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            info_type: cursor.read_u8()?,
            info_data: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetPlayStatus {
    pub play_state: u8,
    pub track_index: u32,
    pub track_length: u32,
    pub track_pos: u32,
}

impl WireEncode for RetPlayStatus {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.play_state);
        put_u32(out, self.track_index);
        put_u32(out, self.track_length);
        put_u32(out, self.track_pos);
        Ok(())
    }
}

impl WireDecode for RetPlayStatus {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            play_state: cursor.read_u8()?,
            track_index: cursor.read_u32()?,
            track_length: cursor.read_u32()?,
            track_pos: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetCurrentPlayingTrack {
    pub track_index: u32,
}

one_u32!(SetCurrentPlayingTrack, track_index);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetIndexedPlayingTrackInfo {
    pub info_type: TrackInfoType,
    pub track_index: u32,
    pub chapter_index: u16,
}

impl WireEncode for GetIndexedPlayingTrackInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        put_u32(out, self.track_index);
        put_u16(out, self.chapter_index);
        Ok(())
    }
}

impl WireDecode for GetIndexedPlayingTrackInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            info_type: cursor.read_u8()?,
            track_index: cursor.read_u32()?,
            chapter_index: cursor.read_u16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackInfo {
    Caps {
        caps: u32,
        track_total_ms: u32,
        chapter_count: u16,
    },
    ChapterTimeName {
        chapter_time: u32,
        chapter_name: Vec<u8>,
    },
    Artist(Vec<u8>),
    Album(Vec<u8>),
    Genre(Vec<u8>),
    Track(Vec<u8>),
    Composer(Vec<u8>),
    Lyrics {
        flags: u8,
        packet_index: u16,
        lyrics: Vec<u8>,
    },
    ArtworkCount(u8),
}

impl TrackInfo {
    fn encode(&self, out: &mut Vec<u8>) {
        match self {
            Self::Caps {
                caps,
                track_total_ms,
                chapter_count,
            } => {
                put_u32(out, *caps);
                put_u32(out, *track_total_ms);
                put_u16(out, *chapter_count);
            }
            Self::ChapterTimeName {
                chapter_time,
                chapter_name,
            } => {
                put_u32(out, *chapter_time);
                out.extend_from_slice(chapter_name);
            }
            Self::Artist(value)
            | Self::Album(value)
            | Self::Genre(value)
            | Self::Track(value)
            | Self::Composer(value) => out.extend_from_slice(value),
            Self::Lyrics {
                flags,
                packet_index,
                lyrics,
            } => {
                out.push(*flags);
                put_u16(out, *packet_index);
                out.extend_from_slice(lyrics);
            }
            Self::ArtworkCount(value) => out.push(*value),
        }
    }

    fn decode(info_type: TrackInfoType, data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(match info_type {
            TRACK_INFO_TYPE_CAPS => {
                let value = Self::Caps {
                    caps: cursor.read_u32()?,
                    track_total_ms: cursor.read_u32()?,
                    chapter_count: cursor.read_u16()?,
                };
                cursor.finish()?;
                value
            }
            TRACK_INFO_TYPE_CHAPTER_TIME_NAME => Self::ChapterTimeName {
                chapter_time: cursor.read_u32()?,
                chapter_name: cursor.read_rest().to_vec(),
            },
            TRACK_INFO_TYPE_ARTIST => Self::Artist(data.to_vec()),
            TRACK_INFO_TYPE_ALBUM => Self::Album(data.to_vec()),
            TRACK_INFO_TYPE_GENRE => Self::Genre(data.to_vec()),
            TRACK_INFO_TYPE_TRACK => Self::Track(data.to_vec()),
            TRACK_INFO_TYPE_COMPOSER => Self::Composer(data.to_vec()),
            TRACK_INFO_TYPE_LYRICS => Self::Lyrics {
                flags: cursor.read_u8()?,
                packet_index: cursor.read_u16()?,
                lyrics: cursor.read_rest().to_vec(),
            },
            TRACK_INFO_TYPE_ARTWORK_COUNT => {
                let value = Self::ArtworkCount(cursor.read_u8()?);
                cursor.finish()?;
                value
            }
            _ => return Error::invalid("unknown display-remote track info type"),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetIndexedPlayingTrackInfo {
    pub info_type: TrackInfoType,
    pub info_data: TrackInfo,
}

impl WireEncode for RetIndexedPlayingTrackInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        self.info_data.encode(out);
        Ok(())
    }
}

impl WireDecode for RetIndexedPlayingTrackInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let info_type = cursor.read_u8()?;
        let info_data = TrackInfo::decode(info_type, cursor.read_rest())?;
        Ok(Self {
            info_type,
            info_data,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetNumPlayingTracks {
    pub num_play_tracks: u32,
}

one_u32!(RetNumPlayingTracks, num_play_tracks);

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
    pub track_index: u32,
    pub format_id: u16,
    pub time_offset: u32,
}

impl WireEncode for GetTrackArtworkData {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.track_index);
        put_u16(out, self.format_id);
        put_u32(out, self.time_offset);
        Ok(())
    }
}

impl WireDecode for GetTrackArtworkData {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            track_index: cursor.read_u32()?,
            format_id: cursor.read_u16()?,
            time_offset: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

empty_payload!(RetTrackArtworkData);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetPowerBatteryState {
    pub power_state: u8,
    pub battery_level: u8,
}

impl WireEncode for RetPowerBatteryState {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.power_state);
        out.push(self.battery_level);
        Ok(())
    }
}

impl WireDecode for RetPowerBatteryState {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            power_state: cursor.read_u8()?,
            battery_level: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetSoundCheckState {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetSoundCheckState {
    pub enabled: bool,
    pub restore_on_exit: bool,
}

impl WireEncode for RetSoundCheckState {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        crate::codec::put_bool(out, self.enabled);
        Ok(())
    }
}

impl WireDecode for RetSoundCheckState {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            enabled: bool_from_wire(cursor.read_u8()?),
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for SetSoundCheckState {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        crate::codec::put_bool(out, self.enabled);
        crate::codec::put_bool(out, self.restore_on_exit);
        Ok(())
    }
}

impl WireDecode for SetSoundCheckState {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            enabled: bool_from_wire(cursor.read_u8()?),
            restore_on_exit: bool_from_wire(cursor.read_u8()?),
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetTrackArtworkTimes {
    pub track_index: u32,
    pub format_id: u16,
    pub artwork_index: u16,
    pub artwork_count: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetTrackArtworkTimes {
    pub time_offset: Vec<u32>,
}

impl WireEncode for GetTrackArtworkTimes {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.track_index);
        put_u16(out, self.format_id);
        put_u16(out, self.artwork_index);
        put_u16(out, self.artwork_count);
        Ok(())
    }
}

impl WireDecode for GetTrackArtworkTimes {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            track_index: cursor.read_u32()?,
            format_id: cursor.read_u16()?,
            artwork_index: cursor.read_u16()?,
            artwork_count: cursor.read_u16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for RetTrackArtworkTimes {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        for offset in &self.time_offset {
            put_u32(out, *offset);
        }
        Ok(())
    }
}

impl WireDecode for RetTrackArtworkTimes {
    fn decode(data: &[u8]) -> Result<Self> {
        if data.len() % 4 != 0 {
            return Error::invalid("artwork time payload length is not a multiple of 4");
        }
        let mut cursor = Cursor::new(data);
        let mut time_offset = Vec::new();
        while !cursor.is_empty() {
            time_offset.push(cursor.read_u32()?);
        }
        Ok(Self { time_offset })
    }
}

pub trait DeviceDisplayRemote {}

fn ack_success(req: &Command) -> CommandPayload {
    CommandPayload::DisplayAck(Ack {
        status: ACK_STATUS_SUCCESS,
        cmd_id: req.id.cmd_id() as u8,
    })
}

fn coarse_datetime() -> (u16, u8, u8, u8, u8) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let year = 1970 + (days / 365) as u16;
    let day_of_year = days % 365;
    let month = (day_of_year / 31 + 1).min(12) as u8;
    let day = (day_of_year % 31 + 1) as u8;
    let hour = ((secs % 86_400) / 3600) as u8;
    let minute = ((secs % 3600) / 60) as u8;
    (year, month, day, hour, minute)
}

pub fn handle_display_remote(
    req: &Command,
    writer: &mut impl CommandWriter,
    _dev: &mut impl DeviceDisplayRemote,
) -> Result<()> {
    match &req.payload {
        CommandPayload::DisplayGetCurrentEqProfileIndex(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetCurrentEqProfileIndex(RetCurrentEqProfileIndex {
                current_eq_index: 0,
            }),
        ),
        CommandPayload::DisplaySetCurrentEqProfileIndex(_) => {
            respond(req, writer, ack_success(req))
        }
        CommandPayload::DisplayGetNumEqProfiles(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetNumEqProfiles(RetNumEqProfiles { num_eq_profiles: 1 }),
        ),
        CommandPayload::DisplayGetIndexedEqProfileName(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetIndexedEqProfileName(RetIndexedEqProfileName {
                eq_profile_name: string_to_bytes("Default"),
            }),
        ),
        CommandPayload::DisplaySetRemoteEventNotification(_) => {
            respond(req, writer, ack_success(req))
        }
        CommandPayload::DisplayGetRemoteEventStatus(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetRemoteEventStatus(RetRemoteEventStatus { event_status: 0 }),
        ),
        CommandPayload::DisplayGetiPodStateInfo(msg) => {
            let info_data = match msg.info_type {
                INFO_TYPE_TRACK_POSITION_MS => StateInfo::TrackPositionMs(0),
                INFO_TYPE_TRACK_INDEX => StateInfo::TrackIndex(1),
                INFO_TYPE_CHAPTER_INDEX => StateInfo::ChapterIndex {
                    track_index: 0,
                    chapter_count: 0,
                    chapter_index: 0,
                },
                INFO_TYPE_PLAY_STATUS => StateInfo::PlayStatus(PLAY_STATUS_PLAYING),
                INFO_TYPE_VOLUME => StateInfo::Volume {
                    mute_state: 0,
                    ui_volume_level: 255,
                },
                INFO_TYPE_POWER => StateInfo::Power {
                    power_state: 0x05,
                    battery_level: 255,
                },
                INFO_TYPE_EQUALIZER => StateInfo::Equalizer(0),
                INFO_TYPE_SHUFFLE => StateInfo::Shuffle(0),
                INFO_TYPE_REPEAT => StateInfo::Repeat(0),
                INFO_TYPE_DATE_TIME => {
                    let (year, month, day, hour, minute) = coarse_datetime();
                    StateInfo::DateTime {
                        year,
                        month,
                        day,
                        hour,
                        minute,
                    }
                }
                INFO_TYPE_BACKLIGHT => StateInfo::Backlight(255),
                INFO_TYPE_HOLD_SWITCH => StateInfo::HoldSwitch(0),
                INFO_TYPE_SOUND_CHECK => StateInfo::SoundCheck(0),
                INFO_TYPE_AUDIOBOOK_SPEED => StateInfo::AudiobookSpeed(0),
                INFO_TYPE_TRACK_POSITION_SEC => StateInfo::TrackPositionSec(0),
                INFO_TYPE_VOLUME2 => StateInfo::Volume2 {
                    mute_state: 0,
                    ui_volume_level: 255,
                    absolute_volume_level: 255,
                },
                _ => return Error::invalid("unknown info type"),
            };
            respond(
                req,
                writer,
                CommandPayload::DisplayRetiPodStateInfo(RetiPodStateInfo {
                    info_type: msg.info_type,
                    info_data,
                }),
            );
        }
        CommandPayload::DisplaySetiPodStateInfo(_) => respond(req, writer, ack_success(req)),
        CommandPayload::DisplayGetPlayStatus(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetPlayStatus(RetPlayStatus {
                play_state: 0,
                ..RetPlayStatus::default()
            }),
        ),
        CommandPayload::DisplaySetCurrentPlayingTrack(_) => respond(req, writer, ack_success(req)),
        CommandPayload::DisplayGetIndexedPlayingTrackInfo(msg) => {
            let info_data = match msg.info_type {
                TRACK_INFO_TYPE_CAPS => TrackInfo::Caps {
                    caps: 0,
                    track_total_ms: 300_000,
                    chapter_count: 0,
                },
                TRACK_INFO_TYPE_CHAPTER_TIME_NAME => TrackInfo::ChapterTimeName {
                    chapter_time: 0,
                    chapter_name: string_to_bytes(""),
                },
                TRACK_INFO_TYPE_ARTIST => TrackInfo::Artist(string_to_bytes("")),
                TRACK_INFO_TYPE_ALBUM => TrackInfo::Album(string_to_bytes("")),
                TRACK_INFO_TYPE_GENRE => TrackInfo::Genre(string_to_bytes("")),
                TRACK_INFO_TYPE_TRACK => TrackInfo::Track(string_to_bytes("track")),
                TRACK_INFO_TYPE_COMPOSER => TrackInfo::Composer(string_to_bytes("")),
                TRACK_INFO_TYPE_LYRICS => TrackInfo::Lyrics {
                    flags: 0,
                    packet_index: 0,
                    lyrics: string_to_bytes(""),
                },
                TRACK_INFO_TYPE_ARTWORK_COUNT => TrackInfo::ArtworkCount(0x08),
                _ => return Error::invalid("unknown track info type"),
            };
            respond(
                req,
                writer,
                CommandPayload::DisplayRetIndexedPlayingTrackInfo(RetIndexedPlayingTrackInfo {
                    info_type: msg.info_type,
                    info_data,
                }),
            );
        }
        CommandPayload::DisplayGetNumPlayingTracks(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetNumPlayingTracks(RetNumPlayingTracks { num_play_tracks: 0 }),
        ),
        CommandPayload::DisplayGetArtworkFormats(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetArtworkFormats(RetArtworkFormats::default()),
        ),
        CommandPayload::DisplayGetPowerBatteryState(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetPowerBatteryState(RetPowerBatteryState {
                battery_level: 255,
                power_state: 0x01,
            }),
        ),
        CommandPayload::DisplayGetSoundCheckState(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetSoundCheckState(RetSoundCheckState { enabled: false }),
        ),
        CommandPayload::DisplaySetSoundCheckState(_) => respond(req, writer, ack_success(req)),
        CommandPayload::DisplayGetTrackArtworkTimes(_) => respond(
            req,
            writer,
            CommandPayload::DisplayRetTrackArtworkTimes(RetTrackArtworkTimes::default()),
        ),
        _ => {}
    }
    Ok(())
}
