use std::collections::BTreeMap;
use std::fmt;

use crate::command::registry;
use crate::{Error, Result};

pub const LINGO_GENERAL_ID: u8 = 0x00;
pub const LINGO_SIMPLE_REMOTE_ID: u8 = 0x02;
pub const LINGO_DISPLAY_REMOTE_ID: u8 = 0x03;
pub const LINGO_EXT_REMOTE_ID: u8 = 0x04;
pub const LINGO_USB_HOST_ID: u8 = 0x06;
pub const LINGO_RF_TUNER_ID: u8 = 0x07;
pub const LINGO_EQ_ID: u8 = 0x08;
pub const LINGO_SPORTS_ID: u8 = 0x09;
pub const LINGO_DIGITAL_AUDIO_ID: u8 = 0x0a;
pub const LINGO_STORAGE_ID: u8 = 0x0c;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LingoCmdId {
    lingo: u8,
    cmd: u16,
}

impl LingoCmdId {
    pub const fn new(lingo: u8, cmd: u16) -> Self {
        Self { lingo, cmd }
    }

    pub const fn lingo_id(self) -> u8 {
        self.lingo
    }

    pub const fn cmd_id(self) -> u16 {
        self.cmd
    }

    pub fn encode(self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.lingo);
        match cmd_id_len(self.lingo) {
            2 => out.extend_from_slice(&self.cmd.to_be_bytes()),
            _ => {
                if self.cmd > u8::MAX as u16 {
                    return Error::invalid(format!(
                        "command id {:#06x} does not fit lingo {:#04x}",
                        self.cmd, self.lingo
                    ));
                }
                out.push(self.cmd as u8);
            }
        }
        Ok(())
    }

    pub fn decode(data: &[u8]) -> Result<(Self, usize)> {
        let Some(&lingo) = data.first() else {
            return Err(Error::UnexpectedEof);
        };

        match cmd_id_len(lingo) {
            2 => {
                if data.len() < 3 {
                    return Err(Error::UnexpectedEof);
                }
                Ok((Self::new(lingo, u16::from_be_bytes([data[1], data[2]])), 3))
            }
            _ => {
                if data.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                Ok((Self::new(lingo, data[1] as u16), 2))
            }
        }
    }
}

impl fmt::Display for LingoCmdId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match cmd_id_len(self.lingo) {
            2 => write!(f, "{:#04x},{:#06x}", self.lingo, self.cmd),
            _ => write!(f, "{:#04x},{:#04x}", self.lingo, self.cmd),
        }
    }
}

pub fn cmd_id_len(lingo_id: u8) -> usize {
    match lingo_id {
        LINGO_EXT_REMOTE_ID => 2,
        _ => 1,
    }
}

pub fn dump_lingos() -> String {
    let mut first_by_id: BTreeMap<LingoCmdId, &'static str> = BTreeMap::new();
    for entry in registry() {
        first_by_id.entry(entry.id).or_insert(entry.name);
    }

    let mut out = String::new();
    for (id, name) in first_by_id {
        let formatted = match cmd_id_len(id.lingo_id()) {
            2 => format!("({:#04x}|{:#06x})", id.lingo_id(), id.cmd_id()),
            _ => format!("({:#04x}|{:#04x})", id.lingo_id(), id.cmd_id()),
        };
        out.push_str(&formatted);
        out.push('\t');
        out.push_str(name);
        out.push('\n');
    }
    out
}

pub mod audio;
pub mod display_remote;
pub mod ext_remote;
pub mod general;
pub mod simple_remote;
