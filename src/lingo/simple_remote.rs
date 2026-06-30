use std::fmt;

use crate::codec::{Cursor, WireDecode, WireEncode};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonStates {
    pub button_states: u32,
}

impl ButtonStates {
    pub fn encode_mask(&self) -> Vec<u8> {
        let mask = self.button_states.to_le_bytes();
        let mut byte_len = ((32 - self.button_states.leading_zeros()) as usize + 7) / 8;
        if byte_len == 0 {
            byte_len = 1;
        }
        mask[..byte_len].to_vec()
    }

    pub fn decode_mask(data: &[u8]) -> Result<Self> {
        if !(1..=4).contains(&data.len()) {
            return Error::invalid("invalid button-state data length");
        }
        let mut mask = [0u8; 4];
        mask[..data.len()].copy_from_slice(data);
        Ok(Self {
            button_states: u32::from_le_bytes(mask),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextButtonBit {
    PlayPause,
    VolumeUp,
    VolumeDown,
    NextTrack,
    PreviousTrack,
    NextAlbum,
    PreviousAlbum,
    Stop,
    PlayResume,
    Pause,
    MuteToggle,
    NextChapter,
    PreviousChapter,
    NextPlaylist,
    PreviousPlaylist,
    ShuffleSettingAdvance,
    RepeatSettingAdvance,
    PowerOn,
    PowerOff,
    BacklightFor30Seconds,
    BeginFastForward,
    BeginRewind,
    Menu,
    Select,
    UpArrow,
    DownArrow,
    BacklightOff,
}

impl ContextButtonBit {
    pub fn mask(self) -> u32 {
        1u32 << (self as u32)
    }

    fn from_index(index: usize) -> Option<Self> {
        Some(match index {
            0 => Self::PlayPause,
            1 => Self::VolumeUp,
            2 => Self::VolumeDown,
            3 => Self::NextTrack,
            4 => Self::PreviousTrack,
            5 => Self::NextAlbum,
            6 => Self::PreviousAlbum,
            7 => Self::Stop,
            8 => Self::PlayResume,
            9 => Self::Pause,
            10 => Self::MuteToggle,
            11 => Self::NextChapter,
            12 => Self::PreviousChapter,
            13 => Self::NextPlaylist,
            14 => Self::PreviousPlaylist,
            15 => Self::ShuffleSettingAdvance,
            16 => Self::RepeatSettingAdvance,
            17 => Self::PowerOn,
            18 => Self::PowerOff,
            19 => Self::BacklightFor30Seconds,
            20 => Self::BeginFastForward,
            21 => Self::BeginRewind,
            22 => Self::Menu,
            23 => Self::Select,
            24 => Self::UpArrow,
            25 => Self::DownArrow,
            26 => Self::BacklightOff,
            _ => return None,
        })
    }
}

impl fmt::Display for ContextButtonBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PlayPause => "ContextButtonPlayPause",
            Self::VolumeUp => "ContextButtonVolumeUp",
            Self::VolumeDown => "ContextButtonVolumeDown",
            Self::NextTrack => "ContextButtonNextTrack",
            Self::PreviousTrack => "ContextButtonPreviousTrack",
            Self::NextAlbum => "ContextButtonNextAlbum",
            Self::PreviousAlbum => "ContextButtonPreviousAlbum",
            Self::Stop => "ContextButtonStop",
            Self::PlayResume => "ContextButtonPlayResume",
            Self::Pause => "ContextButtonPause",
            Self::MuteToggle => "ContextButtonMuteToggle",
            Self::NextChapter => "ContextButtonNextChapter",
            Self::PreviousChapter => "ContextButtonPreviousChapter",
            Self::NextPlaylist => "ContextButtonNextPlaylist",
            Self::PreviousPlaylist => "ContextButtonPreviousPlaylist",
            Self::ShuffleSettingAdvance => "ContextButtonShuffleSettingAdvance",
            Self::RepeatSettingAdvance => "ContextButtonRepeatSettingAdvance",
            Self::PowerOn => "ContextButtonPowerOn",
            Self::PowerOff => "ContextButtonPowerOff",
            Self::BacklightFor30Seconds => "ContextButtonBacklightfor30Seconds",
            Self::BeginFastForward => "ContextButtonBeginFastForward",
            Self::BeginRewind => "ContextButtonBeginRewind",
            Self::Menu => "ContextButtonMenu",
            Self::Select => "ContextButtonSelect",
            Self::UpArrow => "ContextButtonUpArrow",
            Self::DownArrow => "ContextButtonDownArrow",
            Self::BacklightOff => "ContextButtonBacklightOff",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ContextButtonMask(pub u32);

impl fmt::Display for ContextButtonMask {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for index in 0..32 {
            let bit = 1u32 << index;
            if self.0 & bit == 0 {
                continue;
            }
            let label = ContextButtonBit::from_index(index)
                .map(|value| value.to_string())
                .unwrap_or_else(|| format!("ContextButtonBit({bit})"));
            if !first {
                f.write_str(" | ")?;
            }
            first = false;
            f.write_str(&label)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextButtonStatus {
    pub state: ContextButtonMask,
}

impl WireEncode for ContextButtonStatus {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(
            &ButtonStates {
                button_states: self.state.0,
            }
            .encode_mask(),
        );
        Ok(())
    }
}

impl WireDecode for ContextButtonStatus {
    fn decode(data: &[u8]) -> Result<Self> {
        Ok(Self {
            state: ContextButtonMask(ButtonStates::decode_mask(data)?.button_states),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ack;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoButtonStatus {
    pub states: ButtonStates,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioButtonStatus {
    pub states: ButtonStates,
}

impl WireEncode for VideoButtonStatus {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.states.encode_mask());
        Ok(())
    }
}

impl WireDecode for VideoButtonStatus {
    fn decode(data: &[u8]) -> Result<Self> {
        Ok(Self {
            states: ButtonStates::decode_mask(data)?,
        })
    }
}

impl WireEncode for AudioButtonStatus {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.states.encode_mask());
        Ok(())
    }
}

impl WireDecode for AudioButtonStatus {
    fn decode(data: &[u8]) -> Result<Self> {
        Ok(Self {
            states: ButtonStates::decode_mask(data)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IPodOutButtonStatus {
    pub button_source: u8,
}

impl WireEncode for IPodOutButtonStatus {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.button_source);
        Ok(())
    }
}

impl WireDecode for IPodOutButtonStatus {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            button_source: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

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

impl WireEncode for Ack {
    fn encode(&self, _out: &mut Vec<u8>) -> Result<()> {
        Ok(())
    }
}

impl WireDecode for Ack {
    fn decode(data: &[u8]) -> Result<Self> {
        Cursor::new(data).finish()?;
        Ok(Self)
    }
}

empty_payload!(RotationInputStatus);
empty_payload!(RadioButtonStatus);
empty_payload!(CameraButtonStatus);
empty_payload!(RegisterDescriptor);
empty_payload!(SendHidReportToIPod);
empty_payload!(SendHidReportToAcc);
empty_payload!(UnregisterDescriptor);
empty_payload!(AccessibilityEvent);
empty_payload!(GetAccessibilityParameter);
empty_payload!(RetAccessibilityParameter);
empty_payload!(SetAccessibilityParameter);
empty_payload!(GetCurrentItemProperty);
empty_payload!(RetCurrentItemProperty);
empty_payload!(SetContext);
empty_payload!(AccParameterChanged);
empty_payload!(DevAck);

#[cfg(test)]
mod tests {
    use super::{ButtonStates, ContextButtonMask, ContextButtonStatus};
    use crate::codec::{WireDecode, WireEncode};

    #[test]
    fn button_states_use_little_endian_shortest_mask() {
        assert_eq!(
            ButtonStates {
                button_states: 0x0102_0304
            }
            .encode_mask(),
            vec![0x04, 0x03, 0x02, 0x01]
        );
        assert_eq!(
            ButtonStates {
                button_states: 0x0000_0000
            }
            .encode_mask(),
            vec![0x00]
        );
        assert_eq!(
            ButtonStates {
                button_states: 0x0000_0101
            }
            .encode_mask(),
            vec![0x01, 0x01]
        );
    }

    #[test]
    fn context_button_roundtrip() {
        let status = ContextButtonStatus {
            state: ContextButtonMask(0x0101),
        };
        let mut data = Vec::new();
        status.encode(&mut data).unwrap();
        assert_eq!(ContextButtonStatus::decode(&data).unwrap(), status);
    }
}
