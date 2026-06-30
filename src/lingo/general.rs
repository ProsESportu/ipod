use std::fmt;

use crate::codec::{put_u16, put_u32, put_u64, Cursor, WireDecode, WireEncode};
use crate::command::{respond, send, Command, CommandPayload, CommandWriter};
use crate::util::{bool_to_byte, byte_to_bool, string_to_bytes};
use crate::{Error, Result};

pub type AckStatus = u8;
pub const ACK_STATUS_SUCCESS: AckStatus = 0x00;
pub const ACK_STATUS_FAILED: AckStatus = 0x02;
pub const ACK_STATUS_UNKNOWN_ID: AckStatus = 0x05;
pub const ACK_STATUS_PENDING: AckStatus = 0x06;

pub type DevAuthInfoStatus = u8;
pub const DEV_AUTH_INFO_STATUS_SUPPORTED: DevAuthInfoStatus = 0x00;

pub type DevAuthStatus = u8;
pub const DEV_AUTH_STATUS_PASSED: DevAuthStatus = 0x00;
pub const DEV_AUTH_STATUS_FAILED: DevAuthStatus = 0x01;

pub type AccEndIdpsStatus = u8;
pub const ACC_END_IDPS_STATUS_CONTINUE: AccEndIdpsStatus = 0x00;
pub const ACC_END_IDPS_STATUS_RESET: AccEndIdpsStatus = 0x01;
pub const ACC_END_IDPS_STATUS_ABANDON: AccEndIdpsStatus = 0x02;
pub const ACC_END_IDPS_STATUS_NEW_LINK: AccEndIdpsStatus = 0x03;

pub type IdpsStatusCode = u8;
pub const IDPS_STATUS_OK: IdpsStatusCode = 0x00;
pub const IDPS_STATUS_TIME_LIMIT_NOT_EXCEEDED: IdpsStatusCode = 0x04;
pub const IDPS_STATUS_WILL_NOT_ACCEPT: IdpsStatusCode = 0x06;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiMode {
    #[default]
    Standard,
    Extended,
    IPodOut,
    Other(u8),
}

impl From<u8> for UiMode {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Standard,
            0x01 => Self::Extended,
            0x02 => Self::IPodOut,
            other => Self::Other(other),
        }
    }
}

impl From<UiMode> for u8 {
    fn from(value: UiMode) -> Self {
        match value {
            UiMode::Standard => 0x00,
            UiMode::Extended => 0x01,
            UiMode::IPodOut => 0x02,
            UiMode::Other(other) => other,
        }
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

empty_payload!(RequestIdentify);
empty_payload!(RequestRemoteUiMode);
empty_payload!(EnterRemoteUiMode);
empty_payload!(ExitRemoteUiMode);
empty_payload!(RequestIPodName);
empty_payload!(RequestIPodSoftwareVersion);
empty_payload!(RequestIPodSerialNum);
empty_payload!(RequestIPodModelNum);
empty_payload!(RequestTransportMaxPayloadSize);
empty_payload!(GetDevAuthenticationInfo);
empty_payload!(GetIPodAuthenticationInfo);
empty_payload!(GetIPodOptions);
empty_payload!(GetUiMode);
empty_payload!(StartIdps);
empty_payload!(GetEventNotification);
empty_payload!(GetSupportedEventNotification);
empty_payload!(GetNowPlayingFocusApp);

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckPending {
    pub status: AckStatus,
    pub cmd_id: u8,
    pub max_wait: u32,
}

impl WireEncode for AckPending {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.status);
        out.push(self.cmd_id);
        put_u32(out, self.max_wait);
        Ok(())
    }
}

impl WireDecode for AckPending {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            status: cursor.read_u8()?,
            cmd_id: cursor.read_u8()?,
            max_wait: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckDataDropped {
    pub status: AckStatus,
    pub cmd_id: u8,
    pub session_id: u16,
    pub num_bytes_dropped: u32,
}

impl WireEncode for AckDataDropped {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.status);
        out.push(self.cmd_id);
        put_u16(out, self.session_id);
        put_u32(out, self.num_bytes_dropped);
        Ok(())
    }
}

impl WireDecode for AckDataDropped {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            status: cursor.read_u8()?,
            cmd_id: cursor.read_u8()?,
            session_id: cursor.read_u16()?,
            num_bytes_dropped: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnRemoteUiMode {
    pub mode: u8,
}

impl WireEncode for ReturnRemoteUiMode {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.mode);
        Ok(())
    }
}

impl WireDecode for ReturnRemoteUiMode {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            mode: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReturnIPodName {
    pub name: Vec<u8>,
}

impl WireEncode for ReturnIPodName {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.name);
        Ok(())
    }
}

impl WireDecode for ReturnIPodName {
    fn decode(data: &[u8]) -> Result<Self> {
        Ok(Self {
            name: data.to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnIPodSoftwareVersion {
    pub major: u8,
    pub minor: u8,
    pub rev: u8,
}

impl WireEncode for ReturnIPodSoftwareVersion {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.major);
        out.push(self.minor);
        out.push(self.rev);
        Ok(())
    }
}

impl WireDecode for ReturnIPodSoftwareVersion {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            major: cursor.read_u8()?,
            minor: cursor.read_u8()?,
            rev: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReturnIPodSerialNum {
    pub serial: Vec<u8>,
}

impl WireEncode for ReturnIPodSerialNum {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.serial);
        Ok(())
    }
}

impl WireDecode for ReturnIPodSerialNum {
    fn decode(data: &[u8]) -> Result<Self> {
        Ok(Self {
            serial: data.to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReturnIPodModelNum {
    pub model_id: u32,
    pub model: Vec<u8>,
}

impl WireEncode for ReturnIPodModelNum {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.model_id);
        out.extend_from_slice(&self.model);
        Ok(())
    }
}

impl WireDecode for ReturnIPodModelNum {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            model_id: cursor.read_u32()?,
            model: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestLingoProtocolVersion {
    pub lingo: u8,
}

impl WireEncode for RequestLingoProtocolVersion {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.lingo);
        Ok(())
    }
}

impl WireDecode for RequestLingoProtocolVersion {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            lingo: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnLingoProtocolVersion {
    pub lingo: u8,
    pub major: u8,
    pub minor: u8,
}

impl WireEncode for ReturnLingoProtocolVersion {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.lingo);
        out.push(self.major);
        out.push(self.minor);
        Ok(())
    }
}

impl WireDecode for ReturnLingoProtocolVersion {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            lingo: cursor.read_u8()?,
            major: cursor.read_u8()?,
            minor: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnTransportMaxPayloadSize {
    pub max_payload: u16,
}

impl WireEncode for ReturnTransportMaxPayloadSize {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u16(out, self.max_payload);
        Ok(())
    }
}

impl WireDecode for ReturnTransportMaxPayloadSize {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            max_payload: cursor.read_u16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

pub type LingoMask = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LingoBit {
    General = 1 << crate::LINGO_GENERAL_ID,
    SimpleRemote = 1 << crate::LINGO_SIMPLE_REMOTE_ID,
    DisplayRemote = 1 << crate::LINGO_DISPLAY_REMOTE_ID,
    ExtRemote = 1 << crate::LINGO_EXT_REMOTE_ID,
    UsbHost = 1 << crate::LINGO_USB_HOST_ID,
    RfTuner = 1 << crate::LINGO_RF_TUNER_ID,
    Eq = 1 << crate::LINGO_EQ_ID,
    Sports = 1 << crate::LINGO_SPORTS_ID,
    DigitalAudio = 1 << crate::LINGO_DIGITAL_AUDIO_ID,
    Storage = 1 << crate::LINGO_STORAGE_ID,
}

impl fmt::Display for LingoBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::General => "LingoGeneralBit",
            Self::SimpleRemote => "LingoSimpleRemoteBit",
            Self::DisplayRemote => "LingoDisplayRemoteBit",
            Self::ExtRemote => "LingoExtRemoteBit",
            Self::UsbHost => "LingoUSBHostBit",
            Self::RfTuner => "LingoRFTunerBit",
            Self::Eq => "LingoEqBit",
            Self::Sports => "LingoSportsBit",
            Self::DigitalAudio => "LingoDigitalAudioBit",
            Self::Storage => "LingoStorageBit",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifyDeviceLingoes {
    pub lingos: LingoMask,
    pub options: u32,
    pub device_id: u32,
}

impl WireEncode for IdentifyDeviceLingoes {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.lingos);
        put_u32(out, self.options);
        put_u32(out, self.device_id);
        Ok(())
    }
}

impl WireDecode for IdentifyDeviceLingoes {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            lingos: cursor.read_u32()?,
            options: cursor.read_u32()?,
            device_id: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetDevAuthenticationInfo {
    pub major: u8,
    pub minor: u8,
    pub cert_current_section: u8,
    pub cert_max_section: u8,
    pub cert_data: Vec<u8>,
}

impl WireEncode for RetDevAuthenticationInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.major);
        out.push(self.minor);
        if self.major >= 0x02 {
            out.push(self.cert_current_section);
            out.push(self.cert_max_section);
            out.extend_from_slice(&self.cert_data);
        }
        Ok(())
    }
}

impl WireDecode for RetDevAuthenticationInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let major = cursor.read_u8()?;
        let minor = cursor.read_u8()?;
        let mut value = Self {
            major,
            minor,
            ..Self::default()
        };
        if major >= 0x02 {
            value.cert_current_section = cursor.read_u8()?;
            value.cert_max_section = cursor.read_u8()?;
            value.cert_data = cursor.read_rest().to_vec();
        } else {
            cursor.finish()?;
        }
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckDevAuthenticationInfo {
    pub status: DevAuthInfoStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetDevAuthenticationSignatureV1 {
    pub challenge: [u8; 16],
    pub counter: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetDevAuthenticationSignatureV2 {
    pub challenge: [u8; 20],
    pub counter: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetDevAuthenticationSignature {
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckDevAuthenticationStatus {
    pub status: DevAuthStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetiPodAuthenticationInfo {
    pub major: u8,
    pub minor: u8,
    pub cert_current_section: u8,
    pub cert_max_section: u8,
    pub cert_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckiPodAuthenticationInfo {
    pub status: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetiPodAuthenticationSignature {
    pub challenge: [u8; 20],
    pub counter: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiPodAuthenticationSignature {
    pub signature: [u8; 20],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AckiPodAuthenticationStatus {
    pub status: u8,
}

macro_rules! one_u8_payload {
    ($name:ident, $field:ident, $ty:ty) => {
        impl WireEncode for $name {
            fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                out.push(self.$field as u8);
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                let mut cursor = Cursor::new(data);
                let value = Self {
                    $field: cursor.read_u8()? as $ty,
                };
                cursor.finish()?;
                Ok(value)
            }
        }
    };
}

one_u8_payload!(AckDevAuthenticationInfo, status, DevAuthInfoStatus);
one_u8_payload!(AckDevAuthenticationStatus, status, DevAuthStatus);
one_u8_payload!(AckiPodAuthenticationInfo, status, u8);
one_u8_payload!(AckiPodAuthenticationStatus, status, u8);

impl WireEncode for GetDevAuthenticationSignatureV1 {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.challenge);
        out.push(self.counter);
        Ok(())
    }
}

impl WireDecode for GetDevAuthenticationSignatureV1 {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            challenge: cursor.read_array()?,
            counter: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for GetDevAuthenticationSignatureV2 {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.challenge);
        out.push(self.counter);
        Ok(())
    }
}

impl WireDecode for GetDevAuthenticationSignatureV2 {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            challenge: cursor.read_array()?,
            counter: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for RetDevAuthenticationSignature {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.signature);
        Ok(())
    }
}

impl WireDecode for RetDevAuthenticationSignature {
    fn decode(data: &[u8]) -> Result<Self> {
        Ok(Self {
            signature: data.to_vec(),
        })
    }
}

impl WireEncode for RetiPodAuthenticationInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.major);
        out.push(self.minor);
        out.push(self.cert_current_section);
        out.push(self.cert_max_section);
        out.extend_from_slice(&self.cert_data);
        Ok(())
    }
}

impl WireDecode for RetiPodAuthenticationInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            major: cursor.read_u8()?,
            minor: cursor.read_u8()?,
            cert_current_section: cursor.read_u8()?,
            cert_max_section: cursor.read_u8()?,
            cert_data: cursor.read_rest().to_vec(),
        })
    }
}

impl WireEncode for GetiPodAuthenticationSignature {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.challenge);
        out.push(self.counter);
        Ok(())
    }
}

impl WireDecode for GetiPodAuthenticationSignature {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            challenge: cursor.read_array()?,
            counter: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for RetiPodAuthenticationSignature {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.signature);
        Ok(())
    }
}

impl WireDecode for RetiPodAuthenticationSignature {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            signature: cursor.read_array()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyiPodStateChange {
    pub state_change: u8,
}

one_u8_payload!(NotifyiPodStateChange, state_change, u8);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiPodOptions {
    pub options: u64,
}

impl WireEncode for RetiPodOptions {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u64(out, self.options);
        Ok(())
    }
}

impl WireDecode for RetiPodOptions {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            options: cursor.read_u64()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAccessoryInfo {
    pub info_type: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAccessoryInfo2 {
    pub info_type: u8,
    pub model_id: u32,
    pub major: u8,
    pub minor: u8,
    pub rev: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetAccessoryInfo3 {
    pub info_type: u8,
    pub lingo_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetAccessoryInfo {
    pub info_type: u8,
    pub data: Vec<u8>,
}

impl WireEncode for GetAccessoryInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        Ok(())
    }
}

impl WireDecode for GetAccessoryInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            info_type: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for GetAccessoryInfo2 {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        put_u32(out, self.model_id);
        out.push(self.major);
        out.push(self.minor);
        out.push(self.rev);
        Ok(())
    }
}

impl WireDecode for GetAccessoryInfo2 {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            info_type: cursor.read_u8()?,
            model_id: cursor.read_u32()?,
            major: cursor.read_u8()?,
            minor: cursor.read_u8()?,
            rev: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for GetAccessoryInfo3 {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        out.push(self.lingo_id);
        Ok(())
    }
}

impl WireDecode for GetAccessoryInfo3 {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            info_type: cursor.read_u8()?,
            lingo_id: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for RetAccessoryInfo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.info_type);
        out.extend_from_slice(&self.data);
        Ok(())
    }
}

impl WireDecode for RetAccessoryInfo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            info_type: cursor.read_u8()?,
            data: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetiPodPreferences {
    pub pref_class_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiPodPreferences {
    pub pref_class_id: u8,
    pub pref_class_setting_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetiPodPreferences {
    pub pref_class_id: u8,
    pub pref_class_setting_id: u8,
    pub restore_on_exit: u8,
}

impl WireEncode for GetiPodPreferences {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.pref_class_id);
        Ok(())
    }
}

impl WireDecode for GetiPodPreferences {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            pref_class_id: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for RetiPodPreferences {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.pref_class_id);
        out.push(self.pref_class_setting_id);
        Ok(())
    }
}

impl WireDecode for RetiPodPreferences {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            pref_class_id: cursor.read_u8()?,
            pref_class_setting_id: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for SetiPodPreferences {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.pref_class_id);
        out.push(self.pref_class_setting_id);
        out.push(self.restore_on_exit);
        Ok(())
    }
}

impl WireDecode for SetiPodPreferences {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            pref_class_id: cursor.read_u8()?,
            pref_class_setting_id: cursor.read_u8()?,
            restore_on_exit: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetUiMode {
    pub ui_mode: UiMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetUiMode {
    pub ui_mode: UiMode,
}

impl WireEncode for RetUiMode {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.ui_mode.into());
        Ok(())
    }
}

impl WireDecode for RetUiMode {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            ui_mode: cursor.read_u8()?.into(),
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for SetUiMode {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.ui_mode.into());
        Ok(())
    }
}

impl WireDecode for SetUiMode {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            ui_mode: cursor.read_u8()?.into(),
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FidIdentifyToken {
    pub acc_lingoes: Vec<u8>,
    pub device_options: u32,
    pub device_id: u32,
}

impl WireEncode for FidIdentifyToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        if self.acc_lingoes.len() > u8::MAX as usize {
            return Error::invalid("too many accessory lingoes");
        }
        out.push(self.acc_lingoes.len() as u8);
        out.extend_from_slice(&self.acc_lingoes);
        put_u32(out, self.device_options);
        put_u32(out, self.device_id);
        Ok(())
    }
}

impl WireDecode for FidIdentifyToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let lingo_count = cursor.read_u8()? as usize;
        let acc_lingoes = cursor.read_bytes(lingo_count)?.to_vec();
        let value = Self {
            acc_lingoes,
            device_options: cursor.read_u32()?,
            device_id: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccCapBit {
    AnalogLineOut = 1 << 0,
    AnalogLineIn = 1 << 1,
    AnalogVideoOut = 1 << 2,
    UsbAudio = 1 << 4,
    AppComm = 1 << 9,
    CheckVolume = 1 << 11,
}

impl fmt::Display for AccCapBit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AnalogLineOut => "AccCapAnalogLineOut",
            Self::AnalogLineIn => "AccCapAnalogLineIn",
            Self::AnalogVideoOut => "AccCapAnalogVideoOut",
            Self::UsbAudio => "AccCapUSBAudio",
            Self::AppComm => "AccCapAppComm",
            Self::CheckVolume => "AccCapCheckVolume",
        })
    }
}

pub const ACC_CAPS: &[AccCapBit] = &[
    AccCapBit::AnalogLineOut,
    AccCapBit::AnalogLineIn,
    AccCapBit::AnalogVideoOut,
    AccCapBit::UsbAudio,
    AccCapBit::AppComm,
    AccCapBit::CheckVolume,
];

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FidAccCapsToken {
    pub acc_caps_bitmask: u64,
}

impl WireEncode for FidAccCapsToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u64(out, self.acc_caps_bitmask);
        Ok(())
    }
}

impl WireDecode for FidAccCapsToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            acc_caps_bitmask: cursor.read_u64()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccInfoType {
    Name = 0x01,
    Firmware = 0x04,
    Hardware = 0x05,
    Manufacturer = 0x06,
    Model = 0x07,
    Serial = 0x08,
    MaxPayload = 0x09,
}

impl fmt::Display for AccInfoType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Name => "AccInfoName",
            Self::Firmware => "AccInfoFirmware",
            Self::Hardware => "AccInfoHardware",
            Self::Manufacturer => "AccInfoMfr",
            Self::Model => "AccInfoModel",
            Self::Serial => "AccInfoSerial",
            Self::MaxPayload => "AccInfoMaxPayload",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidAccInfoToken {
    pub acc_info_type: u8,
    pub value: Vec<u8>,
}

impl WireEncode for FidAccInfoToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.acc_info_type);
        out.extend_from_slice(&self.value);
        Ok(())
    }
}

impl WireDecode for FidAccInfoToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let acc_info_type = cursor.read_u8()?;
        let expected_len = match acc_info_type {
            0x01 | 0x06 | 0x07 | 0x08 => None,
            0x04 | 0x05 => Some(3),
            0x09 => Some(2),
            0x0b | 0x0c => Some(4),
            _ => return Error::invalid("unknown accessory-info token type"),
        };
        let value = match expected_len {
            Some(len) => cursor.read_bytes(len)?.to_vec(),
            None => cursor.read_rest().to_vec(),
        };
        cursor.finish()?;
        Ok(Self {
            acc_info_type,
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidiPodPreferenceToken {
    pub pref_class: u8,
    pub pref_class_setting: u8,
    pub restore_on_exit: u8,
}

impl WireEncode for FidiPodPreferenceToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.pref_class);
        out.push(self.pref_class_setting);
        out.push(self.restore_on_exit);
        Ok(())
    }
}

impl WireDecode for FidiPodPreferenceToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            pref_class: cursor.read_u8()?,
            pref_class_setting: cursor.read_u8()?,
            restore_on_exit: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FidEaProtocolToken {
    pub protocol_index: u8,
    pub protocol_string: Vec<u8>,
}

impl WireEncode for FidEaProtocolToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.protocol_index);
        out.extend_from_slice(&self.protocol_string);
        Ok(())
    }
}

impl WireDecode for FidEaProtocolToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            protocol_index: cursor.read_u8()?,
            protocol_string: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidBundleSeedIdPrefToken {
    pub bundle_seed_id_string: [u8; 11],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidScreenInfoToken {
    pub screen_width_inches: u16,
    pub screen_height_inches: u16,
    pub screen_width_pixels: u16,
    pub screen_height_pixels: u16,
    pub ipod_screen_width_pixels: u16,
    pub ipod_screen_height_pixels: u16,
    pub screen_features_mask: u8,
    pub screen_gamma_value: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidEaProtocolMetadataToken {
    pub protocol_index: u8,
    pub metadata_type: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidMicrophoneCapsToken {
    pub mic_caps_bitmask: u32,
}

impl WireEncode for FidBundleSeedIdPrefToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.bundle_seed_id_string);
        Ok(())
    }
}

impl WireDecode for FidBundleSeedIdPrefToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            bundle_seed_id_string: cursor.read_array()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for FidScreenInfoToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u16(out, self.screen_width_inches);
        put_u16(out, self.screen_height_inches);
        put_u16(out, self.screen_width_pixels);
        put_u16(out, self.screen_height_pixels);
        put_u16(out, self.ipod_screen_width_pixels);
        put_u16(out, self.ipod_screen_height_pixels);
        out.push(self.screen_features_mask);
        out.push(self.screen_gamma_value);
        Ok(())
    }
}

impl WireDecode for FidScreenInfoToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            screen_width_inches: cursor.read_u16()?,
            screen_height_inches: cursor.read_u16()?,
            screen_width_pixels: cursor.read_u16()?,
            screen_height_pixels: cursor.read_u16()?,
            ipod_screen_width_pixels: cursor.read_u16()?,
            ipod_screen_height_pixels: cursor.read_u16()?,
            screen_features_mask: cursor.read_u8()?,
            screen_gamma_value: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for FidEaProtocolMetadataToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.protocol_index);
        out.push(self.metadata_type);
        Ok(())
    }
}

impl WireDecode for FidEaProtocolMetadataToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            protocol_index: cursor.read_u8()?,
            metadata_type: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for FidMicrophoneCapsToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u32(out, self.mic_caps_bitmask);
        Ok(())
    }
}

impl WireDecode for FidMicrophoneCapsToken {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            mic_caps_bitmask: cursor.read_u32()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenId {
    pub fid_type: u8,
    pub fid_subtype: u8,
}

impl WireEncode for TokenId {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.fid_type);
        out.push(self.fid_subtype);
        Ok(())
    }
}

impl WireDecode for TokenId {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            fid_type: cursor.read_u8()?,
            fid_subtype: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FidToken {
    Identify(FidIdentifyToken),
    AccCaps(FidAccCapsToken),
    AccInfo(FidAccInfoToken),
    IPodPreference(FidiPodPreferenceToken),
    EaProtocol(FidEaProtocolToken),
    BundleSeedIdPref(FidBundleSeedIdPrefToken),
    ScreenInfo(FidScreenInfoToken),
    EaProtocolMetadata(FidEaProtocolMetadataToken),
    MicrophoneCaps(FidMicrophoneCapsToken),
    Raw(Vec<u8>),
}

impl FidToken {
    fn decode(id: TokenId, data: &[u8]) -> Result<Self> {
        Ok(match (id.fid_type, id.fid_subtype) {
            (0x00, 0x00) => Self::Identify(FidIdentifyToken::decode(data)?),
            (0x00, 0x01) => Self::AccCaps(FidAccCapsToken::decode(data)?),
            (0x00, 0x02) => Self::AccInfo(FidAccInfoToken::decode(data)?),
            (0x00, 0x03) => Self::IPodPreference(FidiPodPreferenceToken::decode(data)?),
            (0x00, 0x04) => Self::EaProtocol(FidEaProtocolToken::decode(data)?),
            (0x00, 0x05) => Self::BundleSeedIdPref(FidBundleSeedIdPrefToken::decode(data)?),
            (0x00, 0x07) => Self::ScreenInfo(FidScreenInfoToken::decode(data)?),
            (0x00, 0x08) => Self::EaProtocolMetadata(FidEaProtocolMetadataToken::decode(data)?),
            (0x01, _) => Self::MicrophoneCaps(FidMicrophoneCapsToken::decode(data)?),
            _ => Self::Raw(data.to_vec()),
        })
    }
}

impl WireEncode for FidToken {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        match self {
            Self::Identify(value) => value.encode(out),
            Self::AccCaps(value) => value.encode(out),
            Self::AccInfo(value) => value.encode(out),
            Self::IPodPreference(value) => value.encode(out),
            Self::EaProtocol(value) => value.encode(out),
            Self::BundleSeedIdPref(value) => value.encode(out),
            Self::ScreenInfo(value) => value.encode(out),
            Self::EaProtocolMetadata(value) => value.encode(out),
            Self::MicrophoneCaps(value) => value.encode(out),
            Self::Raw(value) => {
                out.extend_from_slice(value);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidTokenValue {
    pub id: TokenId,
    pub token: FidToken,
}

impl WireEncode for FidTokenValue {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        self.id.encode(out)?;
        self.token.encode(out)
    }
}

impl WireDecode for FidTokenValue {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let id = TokenId {
            fid_type: cursor.read_u8()?,
            fid_subtype: cursor.read_u8()?,
        };
        let token = FidToken::decode(id, cursor.read_rest())?;
        Ok(Self { id, token })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SetFidTokenValues {
    pub fid_token_values: Vec<FidTokenValue>,
}

impl WireEncode for SetFidTokenValues {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        if self.fid_token_values.len() > u8::MAX as usize {
            return Error::invalid("too many FID token values");
        }
        out.push(self.fid_token_values.len() as u8);
        for token in &self.fid_token_values {
            let mut token_bytes = Vec::new();
            token.encode(&mut token_bytes)?;
            if token_bytes.len() > u8::MAX as usize {
                return Error::invalid("FID token value too large");
            }
            out.push(token_bytes.len() as u8);
            out.extend_from_slice(&token_bytes);
        }
        Ok(())
    }
}

impl WireDecode for SetFidTokenValues {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let count = cursor.read_u8()? as usize;
        let mut fid_token_values = Vec::with_capacity(count);
        for _ in 0..count {
            let token_len = cursor.read_u8()? as usize;
            fid_token_values.push(FidTokenValue::decode(cursor.read_bytes(token_len)?)?);
        }
        cursor.finish()?;
        Ok(Self { fid_token_values })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FidTokenValueAck {
    pub id: TokenId,
    pub ack: Vec<u8>,
}

impl WireEncode for FidTokenValueAck {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        self.id.encode(out)?;
        out.extend_from_slice(&self.ack);
        Ok(())
    }
}

impl WireDecode for FidTokenValueAck {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            id: TokenId {
                fid_type: cursor.read_u8()?,
                fid_subtype: cursor.read_u8()?,
            },
            ack: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetFidTokenValueAcks {
    pub fid_token_value_acks: Vec<FidTokenValueAck>,
}

impl WireEncode for RetFidTokenValueAcks {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        if self.fid_token_value_acks.len() > u8::MAX as usize {
            return Error::invalid("too many FID token ACKs");
        }
        out.push(self.fid_token_value_acks.len() as u8);
        for ack in &self.fid_token_value_acks {
            let mut ack_bytes = Vec::new();
            ack.encode(&mut ack_bytes)?;
            if ack_bytes.len() > u8::MAX as usize {
                return Error::invalid("FID token ACK too large");
            }
            out.push(ack_bytes.len() as u8);
            out.extend_from_slice(&ack_bytes);
        }
        Ok(())
    }
}

impl WireDecode for RetFidTokenValueAcks {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let count = cursor.read_u8()? as usize;
        let mut fid_token_value_acks = Vec::with_capacity(count);
        for _ in 0..count {
            let ack_len = cursor.read_u8()? as usize;
            fid_token_value_acks.push(FidTokenValueAck::decode(cursor.read_bytes(ack_len)?)?);
        }
        cursor.finish()?;
        Ok(Self {
            fid_token_value_acks,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndIdps {
    pub acc_end_idps_status: AccEndIdpsStatus,
}

one_u8_payload!(EndIdps, acc_end_idps_status, AccEndIdpsStatus);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdpsStatus {
    pub status: IdpsStatusCode,
}

one_u8_payload!(IdpsStatus, status, IdpsStatusCode);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenDataSessionForProtocol {
    pub session_id: u16,
    pub protocol_index: u8,
}

impl WireEncode for OpenDataSessionForProtocol {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u16(out, self.session_id);
        out.push(self.protocol_index);
        Ok(())
    }
}

impl WireDecode for OpenDataSessionForProtocol {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            session_id: cursor.read_u16()?,
            protocol_index: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloseDataSession {
    pub session_id: u16,
}

impl WireEncode for CloseDataSession {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u16(out, self.session_id);
        Ok(())
    }
}

impl WireDecode for CloseDataSession {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            session_id: cursor.read_u16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevAck {
    pub ack_status: u8,
    pub cmd_id: u8,
}

impl WireEncode for DevAck {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.ack_status);
        out.push(self.cmd_id);
        Ok(())
    }
}

impl WireDecode for DevAck {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            ack_status: cursor.read_u8()?,
            cmd_id: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DevDataTransfer {
    pub session_id: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IPodDataTransfer {
    pub session_id: u16,
    pub data: Vec<u8>,
}

macro_rules! session_data_payload {
    ($name:ident) => {
        impl WireEncode for $name {
            fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                put_u16(out, self.session_id);
                out.extend_from_slice(&self.data);
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                let mut cursor = Cursor::new(data);
                Ok(Self {
                    session_id: cursor.read_u16()?,
                    data: cursor.read_rest().to_vec(),
                })
            }
        }
    };
}

session_data_payload!(DevDataTransfer);
session_data_payload!(IPodDataTransfer);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetAccStatusNotification {
    pub status_mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetAccStatusNotification {
    pub status_mask: u32,
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

one_u32_payload!(SetAccStatusNotification, status_mask);
one_u32_payload!(RetAccStatusNotification, status_mask);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AccessoryStatusNotification {
    pub status_type: u8,
    pub status_params: Vec<u8>,
}

impl WireEncode for AccessoryStatusNotification {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.status_type);
        out.extend_from_slice(&self.status_params);
        Ok(())
    }
}

impl WireDecode for AccessoryStatusNotification {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            status_type: cursor.read_u8()?,
            status_params: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetEventNotification {
    pub event_mask: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetEventNotification {
    pub event_mask: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetSupportedEventNotification {
    pub event_mask: u64,
}

macro_rules! one_u64_payload {
    ($name:ident, $field:ident) => {
        impl WireEncode for $name {
            fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                put_u64(out, self.$field);
                Ok(())
            }
        }

        impl WireDecode for $name {
            fn decode(data: &[u8]) -> Result<Self> {
                let mut cursor = Cursor::new(data);
                let value = Self {
                    $field: cursor.read_u64()?,
                };
                cursor.finish()?;
                Ok(value)
            }
        }
    };
}

one_u64_payload!(SetEventNotification, event_mask);
one_u64_payload!(RetEventNotification, event_mask);
one_u64_payload!(RetSupportedEventNotification, event_mask);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IPodNotification {
    pub notification_type: u8,
    pub data: Vec<u8>,
}

impl WireEncode for IPodNotification {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.notification_type);
        out.extend_from_slice(&self.data);
        Ok(())
    }
}

impl WireDecode for IPodNotification {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            notification_type: cursor.read_u8()?,
            data: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetiPodOptionsForLingo {
    pub lingo_id: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiPodOptionsForLingo {
    pub lingo_id: u8,
    pub options: u64,
}

impl WireEncode for GetiPodOptionsForLingo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.lingo_id);
        Ok(())
    }
}

impl WireDecode for GetiPodOptionsForLingo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            lingo_id: cursor.read_u8()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

impl WireEncode for RetiPodOptionsForLingo {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.lingo_id);
        put_u64(out, self.options);
        Ok(())
    }
}

impl WireDecode for RetiPodOptionsForLingo {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            lingo_id: cursor.read_u8()?,
            options: cursor.read_u64()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelCommand {
    pub lingo_id: u8,
    pub cmd_id: u16,
    pub transaction_id: u16,
}

impl WireEncode for CancelCommand {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.lingo_id);
        put_u16(out, self.cmd_id);
        put_u16(out, self.transaction_id);
        Ok(())
    }
}

impl WireDecode for CancelCommand {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            lingo_id: cursor.read_u8()?,
            cmd_id: cursor.read_u16()?,
            transaction_id: cursor.read_u16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetAvailableCurrent {
    pub current_limit: u16,
}

impl WireEncode for SetAvailableCurrent {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        put_u16(out, self.current_limit);
        Ok(())
    }
}

impl WireDecode for SetAvailableCurrent {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let value = Self {
            current_limit: cursor.read_u16()?,
        };
        cursor.finish()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RequestApplicationLaunch {
    pub reserved0: u8,
    pub reserved1: u8,
    pub reserved2: u8,
    pub app_id: Vec<u8>,
}

impl WireEncode for RequestApplicationLaunch {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.push(self.reserved0);
        out.push(self.reserved1);
        out.push(self.reserved2);
        out.extend_from_slice(&self.app_id);
        Ok(())
    }
}

impl WireDecode for RequestApplicationLaunch {
    fn decode(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        Ok(Self {
            reserved0: cursor.read_u8()?,
            reserved1: cursor.read_u8()?,
            reserved2: cursor.read_u8()?,
            app_id: cursor.read_rest().to_vec(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetNowPlayingFocusApp {
    pub app_id: Vec<u8>,
}

impl WireEncode for RetNowPlayingFocusApp {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
        out.extend_from_slice(&self.app_id);
        Ok(())
    }
}

impl WireDecode for RetNowPlayingFocusApp {
    fn decode(data: &[u8]) -> Result<Self> {
        Ok(Self {
            app_id: data.to_vec(),
        })
    }
}

pub trait DeviceGeneral {
    fn ui_mode(&self) -> UiMode;
    fn set_ui_mode(&mut self, mode: UiMode);
    fn name(&self) -> String;
    fn software_version(&self) -> (u8, u8, u8);
    fn serial_num(&self) -> String;
    fn lingo_protocol_version(&self, lingo: u8) -> (u8, u8);
    fn lingo_options(&self, lingo: u8) -> u64;
    fn pref_setting_id(&self, class_id: u8) -> u8;
    fn set_pref_setting_id(&mut self, class_id: u8, setting_id: u8, restore_on_exit: bool);
    fn start_idps(&mut self);
    fn end_idps(&mut self, status: AccEndIdpsStatus);
    fn set_token(&mut self, token: FidTokenValue) -> Result<()>;
    fn acc_auth_cert(&mut self, cert: &[u8]);
    fn set_event_notification_mask(&mut self, mask: u64);
    fn event_notification_mask(&self) -> u64;
    fn supported_event_notification_mask(&self) -> u64;
    fn cancel_command(&mut self, lingo: u8, cmd: u16, transaction: u16);
    fn max_payload(&self) -> u16;
}

fn ack_success(req: &Command) -> CommandPayload {
    CommandPayload::GeneralAck(Ack {
        status: ACK_STATUS_SUCCESS,
        cmd_id: req.id.cmd_id() as u8,
    })
}

fn ack_pending(req: &Command, max_wait: u32) -> CommandPayload {
    CommandPayload::GeneralAckPending(AckPending {
        status: ACK_STATUS_PENDING,
        cmd_id: req.id.cmd_id() as u8,
        max_wait,
    })
}

fn ack(req: &Command, status: AckStatus) -> CommandPayload {
    CommandPayload::GeneralAck(Ack {
        status,
        cmd_id: req.id.cmd_id() as u8,
    })
}

fn ack_fid_token_value(token_value: &FidTokenValue) -> FidTokenValueAck {
    let ack = match &token_value.token {
        FidToken::Identify(_) => vec![0x00],
        FidToken::AccCaps(_) => vec![0x00],
        FidToken::AccInfo(token) => vec![0x00, token.acc_info_type],
        FidToken::IPodPreference(token) => vec![0x00, token.pref_class],
        FidToken::EaProtocol(token) => vec![0x00, token.protocol_index],
        FidToken::BundleSeedIdPref(_) => vec![0x00],
        FidToken::ScreenInfo(_) => vec![0x00],
        FidToken::EaProtocolMetadata(_) => vec![0x00],
        FidToken::MicrophoneCaps(_) => vec![0x00],
        FidToken::Raw(_) => Vec::new(),
    };
    FidTokenValueAck {
        id: token_value.id,
        ack,
    }
}

fn ack_fid_token_values(tokens: &SetFidTokenValues) -> CommandPayload {
    CommandPayload::GeneralRetFidTokenValueAcks(RetFidTokenValueAcks {
        fid_token_value_acks: tokens
            .fid_token_values
            .iter()
            .map(ack_fid_token_value)
            .collect(),
    })
}

#[derive(Debug, Clone, Default)]
pub struct GeneralHandlerState {
    acc_cert_buf: Vec<u8>,
}

pub fn handle_general(
    req: &Command,
    writer: &mut impl CommandWriter,
    dev: &mut impl DeviceGeneral,
    state: &mut GeneralHandlerState,
) -> Result<()> {
    match &req.payload {
        CommandPayload::GeneralRequestRemoteUiMode(_) => {
            respond(
                req,
                writer,
                CommandPayload::GeneralReturnRemoteUiMode(ReturnRemoteUiMode {
                    mode: bool_to_byte(dev.ui_mode() == UiMode::Extended),
                }),
            );
        }
        CommandPayload::GeneralEnterRemoteUiMode(_) => {
            if dev.ui_mode() == UiMode::Extended {
                respond(req, writer, ack_success(req));
            } else {
                respond(req, writer, ack_pending(req, 300));
                dev.set_ui_mode(UiMode::Extended);
                respond(req, writer, ack_success(req));
            }
        }
        CommandPayload::GeneralExitRemoteUiMode(_) => {
            if dev.ui_mode() != UiMode::Extended {
                respond(req, writer, ack_success(req));
            } else {
                respond(req, writer, ack_pending(req, 300));
                dev.set_ui_mode(UiMode::Standard);
                respond(req, writer, ack_success(req));
            }
        }
        CommandPayload::GeneralRequestIPodName(_) => respond(
            req,
            writer,
            CommandPayload::GeneralReturnIPodName(ReturnIPodName {
                name: string_to_bytes(&dev.name()),
            }),
        ),
        CommandPayload::GeneralRequestIPodSoftwareVersion(_) => {
            let (major, minor, rev) = dev.software_version();
            respond(
                req,
                writer,
                CommandPayload::GeneralReturnIPodSoftwareVersion(ReturnIPodSoftwareVersion {
                    major,
                    minor,
                    rev,
                }),
            );
        }
        CommandPayload::GeneralRequestIPodSerialNum(_) => respond(
            req,
            writer,
            CommandPayload::GeneralReturnIPodSerialNum(ReturnIPodSerialNum {
                serial: string_to_bytes(&dev.serial_num()),
            }),
        ),
        CommandPayload::GeneralRequestIPodModelNum(_) => respond(
            req,
            writer,
            CommandPayload::GeneralReturnIPodModelNum(ReturnIPodModelNum {
                model_id: 0x0011_1349,
                model: string_to_bytes("MC676"),
            }),
        ),
        CommandPayload::GeneralRequestLingoProtocolVersion(msg) => {
            let (major, minor) = dev.lingo_protocol_version(msg.lingo);
            respond(
                req,
                writer,
                CommandPayload::GeneralReturnLingoProtocolVersion(ReturnLingoProtocolVersion {
                    lingo: msg.lingo,
                    major,
                    minor,
                }),
            );
        }
        CommandPayload::GeneralRequestTransportMaxPayloadSize(_) => respond(
            req,
            writer,
            CommandPayload::GeneralReturnTransportMaxPayloadSize(ReturnTransportMaxPayloadSize {
                max_payload: dev.max_payload(),
            }),
        ),
        CommandPayload::GeneralIdentifyDeviceLingoes(msg) => {
            respond(req, writer, ack_success(req));
            if msg.device_id != 0x00 {
                respond(
                    req,
                    writer,
                    CommandPayload::GeneralGetDevAuthenticationInfo(GetDevAuthenticationInfo),
                );
            }
        }
        CommandPayload::GeneralRetDevAuthenticationInfo(msg) => {
            if msg.major >= 2 {
                if msg.cert_current_section == 0 {
                    state.acc_cert_buf.clear();
                }
                state.acc_cert_buf.extend_from_slice(&msg.cert_data);
                if msg.cert_current_section < msg.cert_max_section {
                    respond(req, writer, ack_success(req));
                } else {
                    respond(
                        req,
                        writer,
                        CommandPayload::GeneralAckDevAuthenticationInfo(AckDevAuthenticationInfo {
                            status: DEV_AUTH_INFO_STATUS_SUPPORTED,
                        }),
                    );
                    dev.acc_auth_cert(&state.acc_cert_buf);
                    respond(
                        req,
                        writer,
                        CommandPayload::GeneralGetDevAuthenticationSignatureV2(
                            GetDevAuthenticationSignatureV2 {
                                challenge: [0; 20],
                                counter: 0,
                            },
                        ),
                    );
                }
            } else {
                respond(
                    req,
                    writer,
                    CommandPayload::GeneralAckDevAuthenticationInfo(AckDevAuthenticationInfo {
                        status: DEV_AUTH_INFO_STATUS_SUPPORTED,
                    }),
                );
            }
        }
        CommandPayload::GeneralRetDevAuthenticationSignature(_) => respond(
            req,
            writer,
            CommandPayload::GeneralAckDevAuthenticationStatus(AckDevAuthenticationStatus {
                status: DEV_AUTH_STATUS_PASSED,
            }),
        ),
        CommandPayload::GeneralGetIPodAuthenticationInfo(_) => respond(
            req,
            writer,
            CommandPayload::GeneralRetiPodAuthenticationInfo(RetiPodAuthenticationInfo {
                major: 1,
                minor: 1,
                cert_current_section: 0,
                cert_max_section: 0,
                cert_data: Vec::new(),
            }),
        ),
        CommandPayload::GeneralGetiPodAuthenticationSignature(msg) => respond(
            req,
            writer,
            CommandPayload::GeneralRetiPodAuthenticationSignature(RetiPodAuthenticationSignature {
                signature: msg.challenge,
            }),
        ),
        CommandPayload::GeneralGetIPodOptions(_) => respond(
            req,
            writer,
            CommandPayload::GeneralRetiPodOptions(RetiPodOptions { options: 0 }),
        ),
        CommandPayload::GeneralGetiPodPreferences(msg) => respond(
            req,
            writer,
            CommandPayload::GeneralRetiPodPreferences(RetiPodPreferences {
                pref_class_id: msg.pref_class_id,
                pref_class_setting_id: dev.pref_setting_id(msg.pref_class_id),
            }),
        ),
        CommandPayload::GeneralSetiPodPreferences(msg) => {
            dev.set_pref_setting_id(
                msg.pref_class_id,
                msg.pref_class_setting_id,
                byte_to_bool(msg.restore_on_exit),
            );
            respond(req, writer, ack_success(req));
        }
        CommandPayload::GeneralGetUiMode(_) => respond(
            req,
            writer,
            CommandPayload::GeneralRetUiMode(RetUiMode {
                ui_mode: dev.ui_mode(),
            }),
        ),
        CommandPayload::GeneralSetUiMode(_) => respond(req, writer, ack_success(req)),
        CommandPayload::GeneralStartIdps(_) => {
            crate::trx_reset();
            dev.start_idps();
            respond(req, writer, ack_success(req));
        }
        CommandPayload::GeneralSetFidTokenValues(msg) => {
            for token in &msg.fid_token_values {
                dev.set_token(token.clone())?;
            }
            respond(req, writer, ack_fid_token_values(msg));
        }
        CommandPayload::GeneralEndIdps(msg) => {
            dev.end_idps(msg.acc_end_idps_status);
            match msg.acc_end_idps_status {
                ACC_END_IDPS_STATUS_CONTINUE => {
                    respond(
                        req,
                        writer,
                        CommandPayload::GeneralIdpsStatus(IdpsStatus {
                            status: IDPS_STATUS_OK,
                        }),
                    );
                    send(
                        writer,
                        CommandPayload::GeneralGetDevAuthenticationInfo(GetDevAuthenticationInfo),
                    );
                }
                ACC_END_IDPS_STATUS_RESET => respond(
                    req,
                    writer,
                    CommandPayload::GeneralIdpsStatus(IdpsStatus {
                        status: IDPS_STATUS_TIME_LIMIT_NOT_EXCEEDED,
                    }),
                ),
                ACC_END_IDPS_STATUS_ABANDON => respond(
                    req,
                    writer,
                    CommandPayload::GeneralIdpsStatus(IdpsStatus {
                        status: IDPS_STATUS_WILL_NOT_ACCEPT,
                    }),
                ),
                ACC_END_IDPS_STATUS_NEW_LINK => {}
                _ => {}
            }
        }
        CommandPayload::GeneralSetEventNotification(msg) => {
            dev.set_event_notification_mask(msg.event_mask);
            respond(req, writer, ack_success(req));
        }
        CommandPayload::GeneralGetiPodOptionsForLingo(msg) => respond(
            req,
            writer,
            CommandPayload::GeneralRetiPodOptionsForLingo(RetiPodOptionsForLingo {
                lingo_id: msg.lingo_id,
                options: dev.lingo_options(msg.lingo_id),
            }),
        ),
        CommandPayload::GeneralGetEventNotification(_) => respond(
            req,
            writer,
            CommandPayload::GeneralRetEventNotification(RetEventNotification {
                event_mask: dev.event_notification_mask(),
            }),
        ),
        CommandPayload::GeneralGetSupportedEventNotification(_) => respond(
            req,
            writer,
            CommandPayload::GeneralRetSupportedEventNotification(RetSupportedEventNotification {
                event_mask: dev.supported_event_notification_mask(),
            }),
        ),
        CommandPayload::GeneralCancelCommand(msg) => {
            dev.cancel_command(msg.lingo_id, msg.cmd_id, msg.transaction_id);
            respond(req, writer, ack_success(req));
        }
        CommandPayload::GeneralRequestApplicationLaunch(_) => {
            respond(req, writer, ack(req, ACK_STATUS_FAILED));
        }
        CommandPayload::GeneralGetNowPlayingFocusApp(_) => respond(
            req,
            writer,
            CommandPayload::GeneralRetNowPlayingFocusApp(RetNowPlayingFocusApp {
                app_id: string_to_bytes(""),
            }),
        ),
        CommandPayload::Unknown(_) => respond(req, writer, ack(req, ACK_STATUS_UNKNOWN_ID)),
        _ => {}
    }

    Ok(())
}
