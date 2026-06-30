use crate::codec::{put_u32, Cursor, WireDecode, WireEncode};
use crate::command::{send, CommandWriter};
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AckStatus {
    Success = 0x00,
    Other(u8),
}

impl From<u8> for AckStatus {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Success,
            other => Self::Other(other),
        }
    }
}

impl From<AckStatus> for u8 {
    fn from(value: AckStatus) -> Self {
        match value {
            AckStatus::Success => 0x00,
            AckStatus::Other(other) => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccAck {
    pub status: AckStatus,
    pub cmd_id: u8,
}

impl WireEncode for AccAck {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.status.into());
        out.push(self.cmd_id);
        Ok(())
    }
}

impl WireDecode for AccAck {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            status: cursor.read_u8()?.into(),
            cmd_id: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IPodAck {
    pub status: AckStatus,
    pub cmd_id: u8,
}

impl WireEncode for IPodAck {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.status.into());
        out.push(self.cmd_id);
        Ok(())
    }
}

impl WireDecode for IPodAck {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            status: cursor.read_u8()?.into(),
            cmd_id: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GetAccSampleRateCaps;

impl WireDecode for GetAccSampleRateCaps {
    fn decode(data: &[u8]) -> Result<Self> {
        Cursor::new(data).finish()?;
        Ok(Self)
    }
}

impl WireEncode for GetAccSampleRateCaps {
    fn encode(&self, _out: &mut Vec<u8>) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetAccSampleRateCaps {
    pub sample_rates: Vec<u32>,
}

impl WireEncode for RetAccSampleRateCaps {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        for rate in &self.sample_rates {
            put_u32(out, *rate);
        }
        Ok(())
    }
}

impl WireDecode for RetAccSampleRateCaps {
    fn decode(data: &[u8]) -> Result<Self> {
        if data.len() % 4 != 0 {
            return Error::invalid("sample-rate payload length is not a multiple of 4");
        }
        let mut cursor = Cursor::new(data);
        let mut sample_rates = Vec::with_capacity(data.len() / 4);
        while !cursor.is_empty() {
            sample_rates.push(cursor.read_u32()?);
        }
        Ok(Self { sample_rates })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrackNewAudioAttributes {
    pub sample_rate: u32,
    pub sound_check_value: u32,
    pub volume_adjustment: u32,
}

impl WireEncode for TrackNewAudioAttributes {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.sample_rate);
        put_u32(out, self.sound_check_value);
        put_u32(out, self.volume_adjustment);
        Ok(())
    }
}

impl WireDecode for TrackNewAudioAttributes {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            sample_rate: cursor.read_u32()?,
            sound_check_value: cursor.read_u32()?,
            volume_adjustment: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SetVideoDelay {
    pub delay: u32,
}

impl WireEncode for SetVideoDelay {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.delay);
        Ok(())
    }
}

impl WireDecode for SetVideoDelay {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            delay: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

pub trait DeviceAudio {}

pub fn start(writer: &mut impl CommandWriter) {
    send(
        writer,
        crate::command::CommandPayload::AudioGetAccSampleRateCaps(GetAccSampleRateCaps),
    );
}

pub fn handle_audio(
    req: &crate::command::Command,
    writer: &mut impl CommandWriter,
    _dev: &mut impl DeviceAudio,
) -> Result<()> {
    match &req.payload {
        crate::command::CommandPayload::AudioAccAck(_ack) => {}
        crate::command::CommandPayload::AudioRetAccSampleRateCaps(_caps) => {
            crate::command::respond(
                req,
                writer,
                crate::command::CommandPayload::AudioTrackNewAudioAttributes(
                    TrackNewAudioAttributes {
                        sample_rate: 44_100,
                        ..TrackNewAudioAttributes::default()
                    },
                ),
            );
        }
        _ => {}
    }
    Ok(())
}
