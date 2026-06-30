use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::codec::{put_u16, WireDecode, WireEncode};
use crate::lingo::audio;
use crate::lingo::display_remote as disp;
use crate::lingo::ext_remote as ext;
use crate::lingo::general;
use crate::lingo::simple_remote as simple;
use crate::{
    Error, LingoCmdId, Result, LINGO_DIGITAL_AUDIO_ID, LINGO_DISPLAY_REMOTE_ID,
    LINGO_EXT_REMOTE_ID, LINGO_GENERAL_ID, LINGO_SIMPLE_REMOTE_ID,
};

pub type UnknownPayload = Vec<u8>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Transaction(pub u16);

impl Transaction {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn delta(self, delta: i32) -> Self {
        Self((self.0 as i32 + delta) as u16)
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#04x}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub id: LingoCmdId,
    pub transaction: Option<Transaction>,
    pub payload: CommandPayload,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPayload {
    Unknown(UnknownPayload),

    GeneralRequestIdentify(general::RequestIdentify),
    GeneralAck(general::Ack),
    GeneralAckPending(general::AckPending),
    GeneralAckDataDropped(general::AckDataDropped),
    GeneralRequestRemoteUiMode(general::RequestRemoteUiMode),
    GeneralReturnRemoteUiMode(general::ReturnRemoteUiMode),
    GeneralEnterRemoteUiMode(general::EnterRemoteUiMode),
    GeneralExitRemoteUiMode(general::ExitRemoteUiMode),
    GeneralRequestIPodName(general::RequestIPodName),
    GeneralReturnIPodName(general::ReturnIPodName),
    GeneralRequestIPodSoftwareVersion(general::RequestIPodSoftwareVersion),
    GeneralReturnIPodSoftwareVersion(general::ReturnIPodSoftwareVersion),
    GeneralRequestIPodSerialNum(general::RequestIPodSerialNum),
    GeneralReturnIPodSerialNum(general::ReturnIPodSerialNum),
    GeneralRequestIPodModelNum(general::RequestIPodModelNum),
    GeneralReturnIPodModelNum(general::ReturnIPodModelNum),
    GeneralRequestLingoProtocolVersion(general::RequestLingoProtocolVersion),
    GeneralReturnLingoProtocolVersion(general::ReturnLingoProtocolVersion),
    GeneralRequestTransportMaxPayloadSize(general::RequestTransportMaxPayloadSize),
    GeneralReturnTransportMaxPayloadSize(general::ReturnTransportMaxPayloadSize),
    GeneralIdentifyDeviceLingoes(general::IdentifyDeviceLingoes),
    GeneralGetDevAuthenticationInfo(general::GetDevAuthenticationInfo),
    GeneralRetDevAuthenticationInfo(general::RetDevAuthenticationInfo),
    GeneralAckDevAuthenticationInfo(general::AckDevAuthenticationInfo),
    GeneralGetDevAuthenticationSignatureV1(general::GetDevAuthenticationSignatureV1),
    GeneralGetDevAuthenticationSignatureV2(general::GetDevAuthenticationSignatureV2),
    GeneralRetDevAuthenticationSignature(general::RetDevAuthenticationSignature),
    GeneralAckDevAuthenticationStatus(general::AckDevAuthenticationStatus),
    GeneralGetIPodAuthenticationInfo(general::GetIPodAuthenticationInfo),
    GeneralRetiPodAuthenticationInfo(general::RetiPodAuthenticationInfo),
    GeneralAckiPodAuthenticationInfo(general::AckiPodAuthenticationInfo),
    GeneralGetiPodAuthenticationSignature(general::GetiPodAuthenticationSignature),
    GeneralRetiPodAuthenticationSignature(general::RetiPodAuthenticationSignature),
    GeneralAckiPodAuthenticationStatus(general::AckiPodAuthenticationStatus),
    GeneralNotifyiPodStateChange(general::NotifyiPodStateChange),
    GeneralGetIPodOptions(general::GetIPodOptions),
    GeneralRetiPodOptions(general::RetiPodOptions),
    GeneralGetAccessoryInfo(general::GetAccessoryInfo),
    GeneralGetAccessoryInfo2(general::GetAccessoryInfo2),
    GeneralGetAccessoryInfo3(general::GetAccessoryInfo3),
    GeneralRetAccessoryInfo(general::RetAccessoryInfo),
    GeneralGetiPodPreferences(general::GetiPodPreferences),
    GeneralRetiPodPreferences(general::RetiPodPreferences),
    GeneralSetiPodPreferences(general::SetiPodPreferences),
    GeneralGetUiMode(general::GetUiMode),
    GeneralRetUiMode(general::RetUiMode),
    GeneralSetUiMode(general::SetUiMode),
    GeneralStartIdps(general::StartIdps),
    GeneralSetFidTokenValues(general::SetFidTokenValues),
    GeneralRetFidTokenValueAcks(general::RetFidTokenValueAcks),
    GeneralEndIdps(general::EndIdps),
    GeneralIdpsStatus(general::IdpsStatus),
    GeneralOpenDataSessionForProtocol(general::OpenDataSessionForProtocol),
    GeneralCloseDataSession(general::CloseDataSession),
    GeneralDevAck(general::DevAck),
    GeneralDevDataTransfer(general::DevDataTransfer),
    GeneralIPodDataTransfer(general::IPodDataTransfer),
    GeneralSetAccStatusNotification(general::SetAccStatusNotification),
    GeneralRetAccStatusNotification(general::RetAccStatusNotification),
    GeneralAccessoryStatusNotification(general::AccessoryStatusNotification),
    GeneralSetEventNotification(general::SetEventNotification),
    GeneralIPodNotification(general::IPodNotification),
    GeneralGetiPodOptionsForLingo(general::GetiPodOptionsForLingo),
    GeneralRetiPodOptionsForLingo(general::RetiPodOptionsForLingo),
    GeneralGetEventNotification(general::GetEventNotification),
    GeneralRetEventNotification(general::RetEventNotification),
    GeneralGetSupportedEventNotification(general::GetSupportedEventNotification),
    GeneralCancelCommand(general::CancelCommand),
    GeneralRetSupportedEventNotification(general::RetSupportedEventNotification),
    GeneralSetAvailableCurrent(general::SetAvailableCurrent),
    GeneralRequestApplicationLaunch(general::RequestApplicationLaunch),
    GeneralGetNowPlayingFocusApp(general::GetNowPlayingFocusApp),
    GeneralRetNowPlayingFocusApp(general::RetNowPlayingFocusApp),

    AudioAccAck(audio::AccAck),
    AudioIPodAck(audio::IPodAck),
    AudioGetAccSampleRateCaps(audio::GetAccSampleRateCaps),
    AudioRetAccSampleRateCaps(audio::RetAccSampleRateCaps),
    AudioTrackNewAudioAttributes(audio::TrackNewAudioAttributes),
    AudioSetVideoDelay(audio::SetVideoDelay),

    SimpleContextButtonStatus(simple::ContextButtonStatus),
    SimpleAck(simple::Ack),
    SimpleVideoButtonStatus(simple::VideoButtonStatus),
    SimpleAudioButtonStatus(simple::AudioButtonStatus),
    SimpleIPodOutButtonStatus(simple::IPodOutButtonStatus),
    SimpleRotationInputStatus(simple::RotationInputStatus),
    SimpleRadioButtonStatus(simple::RadioButtonStatus),
    SimpleCameraButtonStatus(simple::CameraButtonStatus),
    SimpleRegisterDescriptor(simple::RegisterDescriptor),
    SimpleSendHidReportToIPod(simple::SendHidReportToIPod),
    SimpleSendHidReportToAcc(simple::SendHidReportToAcc),
    SimpleUnregisterDescriptor(simple::UnregisterDescriptor),
    SimpleAccessibilityEvent(simple::AccessibilityEvent),
    SimpleGetAccessibilityParameter(simple::GetAccessibilityParameter),
    SimpleRetAccessibilityParameter(simple::RetAccessibilityParameter),
    SimpleSetAccessibilityParameter(simple::SetAccessibilityParameter),
    SimpleGetCurrentItemProperty(simple::GetCurrentItemProperty),
    SimpleRetCurrentItemProperty(simple::RetCurrentItemProperty),
    SimpleSetContext(simple::SetContext),
    SimpleAccParameterChanged(simple::AccParameterChanged),
    SimpleDevAck(simple::DevAck),

    DisplayAck(disp::Ack),
    DisplayGetCurrentEqProfileIndex(disp::GetCurrentEqProfileIndex),
    DisplayRetCurrentEqProfileIndex(disp::RetCurrentEqProfileIndex),
    DisplaySetCurrentEqProfileIndex(disp::SetCurrentEqProfileIndex),
    DisplayGetNumEqProfiles(disp::GetNumEqProfiles),
    DisplayRetNumEqProfiles(disp::RetNumEqProfiles),
    DisplayGetIndexedEqProfileName(disp::GetIndexedEqProfileName),
    DisplayRetIndexedEqProfileName(disp::RetIndexedEqProfileName),
    DisplaySetRemoteEventNotification(disp::SetRemoteEventNotification),
    DisplayRemoteEventNotification(disp::RemoteEventNotification),
    DisplayGetRemoteEventStatus(disp::GetRemoteEventStatus),
    DisplayRetRemoteEventStatus(disp::RetRemoteEventStatus),
    DisplayGetiPodStateInfo(disp::GetiPodStateInfo),
    DisplayRetiPodStateInfo(disp::RetiPodStateInfo),
    DisplaySetiPodStateInfo(disp::SetiPodStateInfo),
    DisplayGetPlayStatus(disp::GetPlayStatus),
    DisplayRetPlayStatus(disp::RetPlayStatus),
    DisplaySetCurrentPlayingTrack(disp::SetCurrentPlayingTrack),
    DisplayGetIndexedPlayingTrackInfo(disp::GetIndexedPlayingTrackInfo),
    DisplayRetIndexedPlayingTrackInfo(disp::RetIndexedPlayingTrackInfo),
    DisplayGetNumPlayingTracks(disp::GetNumPlayingTracks),
    DisplayRetNumPlayingTracks(disp::RetNumPlayingTracks),
    DisplayGetArtworkFormats(disp::GetArtworkFormats),
    DisplayRetArtworkFormats(disp::RetArtworkFormats),
    DisplayGetTrackArtworkData(disp::GetTrackArtworkData),
    DisplayRetTrackArtworkData(disp::RetTrackArtworkData),
    DisplayGetPowerBatteryState(disp::GetPowerBatteryState),
    DisplayRetPowerBatteryState(disp::RetPowerBatteryState),
    DisplayGetSoundCheckState(disp::GetSoundCheckState),
    DisplayRetSoundCheckState(disp::RetSoundCheckState),
    DisplaySetSoundCheckState(disp::SetSoundCheckState),
    DisplayGetTrackArtworkTimes(disp::GetTrackArtworkTimes),
    DisplayRetTrackArtworkTimes(disp::RetTrackArtworkTimes),

    ExtAck(ext::Ack),
    ExtGetCurrentPlayingTrackChapterInfo(ext::GetCurrentPlayingTrackChapterInfo),
    ExtReturnCurrentPlayingTrackChapterInfo(ext::ReturnCurrentPlayingTrackChapterInfo),
    ExtSetCurrentPlayingTrackChapter(ext::SetCurrentPlayingTrackChapter),
    ExtGetCurrentPlayingTrackChapterPlayStatus(ext::GetCurrentPlayingTrackChapterPlayStatus),
    ExtReturnCurrentPlayingTrackChapterPlayStatus(ext::ReturnCurrentPlayingTrackChapterPlayStatus),
    ExtGetCurrentPlayingTrackChapterName(ext::GetCurrentPlayingTrackChapterName),
    ExtReturnCurrentPlayingTrackChapterName(ext::ReturnCurrentPlayingTrackChapterName),
    ExtGetAudiobookSpeed(ext::GetAudiobookSpeed),
    ExtReturnAudiobookSpeed(ext::ReturnAudiobookSpeed),
    ExtSetAudiobookSpeed(ext::SetAudiobookSpeed),
    ExtGetIndexedPlayingTrackInfo(ext::GetIndexedPlayingTrackInfo),
    ExtReturnIndexedPlayingTrackInfo(ext::ReturnIndexedPlayingTrackInfo),
    ExtGetArtworkFormats(ext::GetArtworkFormats),
    ExtRetArtworkFormats(ext::RetArtworkFormats),
    ExtGetTrackArtworkData(ext::GetTrackArtworkData),
    ExtRetTrackArtworkData(ext::RetTrackArtworkData),
    ExtResetDbSelection(ext::ResetDbSelection),
    ExtSelectDbRecord(ext::SelectDbRecord),
    ExtGetNumberCategorizedDbRecords(ext::GetNumberCategorizedDbRecords),
    ExtReturnNumberCategorizedDbRecords(ext::ReturnNumberCategorizedDbRecords),
    ExtRetrieveCategorizedDatabaseRecords(ext::RetrieveCategorizedDatabaseRecords),
    ExtReturnCategorizedDatabaseRecord(ext::ReturnCategorizedDatabaseRecord),
    ExtGetPlayStatus(ext::GetPlayStatus),
    ExtReturnPlayStatus(ext::ReturnPlayStatus),
    ExtGetCurrentPlayingTrackIndex(ext::GetCurrentPlayingTrackIndex),
    ExtReturnCurrentPlayingTrackIndex(ext::ReturnCurrentPlayingTrackIndex),
    ExtGetIndexedPlayingTrackTitle(ext::GetIndexedPlayingTrackTitle),
    ExtReturnIndexedPlayingTrackTitle(ext::ReturnIndexedPlayingTrackTitle),
    ExtGetIndexedPlayingTrackArtistName(ext::GetIndexedPlayingTrackArtistName),
    ExtReturnIndexedPlayingTrackArtistName(ext::ReturnIndexedPlayingTrackArtistName),
    ExtGetIndexedPlayingTrackAlbumName(ext::GetIndexedPlayingTrackAlbumName),
    ExtReturnIndexedPlayingTrackAlbumName(ext::ReturnIndexedPlayingTrackAlbumName),
    ExtSetPlayStatusChangeNotification(ext::SetPlayStatusChangeNotification),
    ExtSetPlayStatusChangeNotificationShort(ext::SetPlayStatusChangeNotificationShort),
    ExtPlayStatusChangeNotification(ext::PlayStatusChangeNotification),
    ExtPlayCurrentSelection(ext::PlayCurrentSelection),
    ExtPlayControl(ext::PlayControl),
    ExtGetTrackArtworkTimes(ext::GetTrackArtworkTimes),
    ExtRetTrackArtworkTimes(ext::RetTrackArtworkTimes),
    ExtGetShuffle(ext::GetShuffle),
    ExtReturnShuffle(ext::ReturnShuffle),
    ExtSetShuffle(ext::SetShuffle),
    ExtGetRepeat(ext::GetRepeat),
    ExtReturnRepeat(ext::ReturnRepeat),
    ExtSetRepeat(ext::SetRepeat),
    ExtSetDisplayImage(ext::SetDisplayImage),
    ExtGetMonoDisplayImageLimits(ext::GetMonoDisplayImageLimits),
    ExtReturnMonoDisplayImageLimits(ext::ReturnMonoDisplayImageLimits),
    ExtGetNumPlayingTracks(ext::GetNumPlayingTracks),
    ExtReturnNumPlayingTracks(ext::ReturnNumPlayingTracks),
    ExtSetCurrentPlayingTrack(ext::SetCurrentPlayingTrack),
    ExtSelectSortDbRecord(ext::SelectSortDbRecord),
    ExtGetColorDisplayImageLimits(ext::GetColorDisplayImageLimits),
    ExtReturnColorDisplayImageLimits(ext::ReturnColorDisplayImageLimits),
    ExtResetDbSelectionHierarchy(ext::ResetDbSelectionHierarchy),
    ExtGetDbITunesInfo(ext::GetDbITunesInfo),
    ExtRetDbITunesInfo(ext::RetDbITunesInfo),
    ExtGetUidTrackInfo(ext::GetUidTrackInfo),
    ExtRetUidTrackInfo(ext::RetUidTrackInfo),
    ExtGetDbTrackInfo(ext::GetDbTrackInfo),
    ExtRetDbTrackInfo(ext::RetDbTrackInfo),
    ExtGetPbTrackInfo(ext::GetPbTrackInfo),
    ExtRetPbTrackInfo(ext::RetPbTrackInfo),
}

macro_rules! impl_payload_methods {
    ($( $variant:ident => ($lingo:expr, $cmd:expr), )+ ) => {
        impl CommandPayload {
            pub fn id(&self) -> Option<LingoCmdId> {
                match self {
                    CommandPayload::Unknown(_) => None,
                    $(CommandPayload::$variant(_) => Some(LingoCmdId::new($lingo, $cmd)),)+
                }
            }

            pub fn encode(&self, out: &mut Vec<u8>) -> Result<()> {
                match self {
                    CommandPayload::Unknown(value) => {
                        out.extend_from_slice(value);
                        Ok(())
                    }
                    $(CommandPayload::$variant(value) => value.encode(out),)+
                }
            }
        }
    };
}

impl_payload_methods! {
    GeneralRequestIdentify => (LINGO_GENERAL_ID, 0x00),
    GeneralAck => (LINGO_GENERAL_ID, 0x02),
    GeneralAckPending => (LINGO_GENERAL_ID, 0x02),
    GeneralAckDataDropped => (LINGO_GENERAL_ID, 0x02),
    GeneralRequestRemoteUiMode => (LINGO_GENERAL_ID, 0x03),
    GeneralReturnRemoteUiMode => (LINGO_GENERAL_ID, 0x04),
    GeneralEnterRemoteUiMode => (LINGO_GENERAL_ID, 0x05),
    GeneralExitRemoteUiMode => (LINGO_GENERAL_ID, 0x06),
    GeneralRequestIPodName => (LINGO_GENERAL_ID, 0x07),
    GeneralReturnIPodName => (LINGO_GENERAL_ID, 0x08),
    GeneralRequestIPodSoftwareVersion => (LINGO_GENERAL_ID, 0x09),
    GeneralReturnIPodSoftwareVersion => (LINGO_GENERAL_ID, 0x0a),
    GeneralRequestIPodSerialNum => (LINGO_GENERAL_ID, 0x0b),
    GeneralReturnIPodSerialNum => (LINGO_GENERAL_ID, 0x0c),
    GeneralRequestIPodModelNum => (LINGO_GENERAL_ID, 0x0d),
    GeneralReturnIPodModelNum => (LINGO_GENERAL_ID, 0x0e),
    GeneralRequestLingoProtocolVersion => (LINGO_GENERAL_ID, 0x0f),
    GeneralReturnLingoProtocolVersion => (LINGO_GENERAL_ID, 0x10),
    GeneralRequestTransportMaxPayloadSize => (LINGO_GENERAL_ID, 0x11),
    GeneralReturnTransportMaxPayloadSize => (LINGO_GENERAL_ID, 0x12),
    GeneralIdentifyDeviceLingoes => (LINGO_GENERAL_ID, 0x13),
    GeneralGetDevAuthenticationInfo => (LINGO_GENERAL_ID, 0x14),
    GeneralRetDevAuthenticationInfo => (LINGO_GENERAL_ID, 0x15),
    GeneralAckDevAuthenticationInfo => (LINGO_GENERAL_ID, 0x16),
    GeneralGetDevAuthenticationSignatureV1 => (LINGO_GENERAL_ID, 0x17),
    GeneralGetDevAuthenticationSignatureV2 => (LINGO_GENERAL_ID, 0x17),
    GeneralRetDevAuthenticationSignature => (LINGO_GENERAL_ID, 0x18),
    GeneralAckDevAuthenticationStatus => (LINGO_GENERAL_ID, 0x19),
    GeneralGetIPodAuthenticationInfo => (LINGO_GENERAL_ID, 0x1a),
    GeneralRetiPodAuthenticationInfo => (LINGO_GENERAL_ID, 0x1b),
    GeneralAckiPodAuthenticationInfo => (LINGO_GENERAL_ID, 0x1c),
    GeneralGetiPodAuthenticationSignature => (LINGO_GENERAL_ID, 0x1d),
    GeneralRetiPodAuthenticationSignature => (LINGO_GENERAL_ID, 0x1e),
    GeneralAckiPodAuthenticationStatus => (LINGO_GENERAL_ID, 0x1f),
    GeneralNotifyiPodStateChange => (LINGO_GENERAL_ID, 0x23),
    GeneralGetIPodOptions => (LINGO_GENERAL_ID, 0x24),
    GeneralRetiPodOptions => (LINGO_GENERAL_ID, 0x25),
    GeneralGetAccessoryInfo => (LINGO_GENERAL_ID, 0x27),
    GeneralGetAccessoryInfo2 => (LINGO_GENERAL_ID, 0x27),
    GeneralGetAccessoryInfo3 => (LINGO_GENERAL_ID, 0x27),
    GeneralRetAccessoryInfo => (LINGO_GENERAL_ID, 0x28),
    GeneralGetiPodPreferences => (LINGO_GENERAL_ID, 0x29),
    GeneralRetiPodPreferences => (LINGO_GENERAL_ID, 0x2a),
    GeneralSetiPodPreferences => (LINGO_GENERAL_ID, 0x2b),
    GeneralGetUiMode => (LINGO_GENERAL_ID, 0x35),
    GeneralRetUiMode => (LINGO_GENERAL_ID, 0x36),
    GeneralSetUiMode => (LINGO_GENERAL_ID, 0x37),
    GeneralStartIdps => (LINGO_GENERAL_ID, 0x38),
    GeneralSetFidTokenValues => (LINGO_GENERAL_ID, 0x39),
    GeneralRetFidTokenValueAcks => (LINGO_GENERAL_ID, 0x3a),
    GeneralEndIdps => (LINGO_GENERAL_ID, 0x3b),
    GeneralIdpsStatus => (LINGO_GENERAL_ID, 0x3c),
    GeneralOpenDataSessionForProtocol => (LINGO_GENERAL_ID, 0x3f),
    GeneralCloseDataSession => (LINGO_GENERAL_ID, 0x40),
    GeneralDevAck => (LINGO_GENERAL_ID, 0x41),
    GeneralDevDataTransfer => (LINGO_GENERAL_ID, 0x42),
    GeneralIPodDataTransfer => (LINGO_GENERAL_ID, 0x43),
    GeneralSetAccStatusNotification => (LINGO_GENERAL_ID, 0x46),
    GeneralRetAccStatusNotification => (LINGO_GENERAL_ID, 0x47),
    GeneralAccessoryStatusNotification => (LINGO_GENERAL_ID, 0x48),
    GeneralSetEventNotification => (LINGO_GENERAL_ID, 0x49),
    GeneralIPodNotification => (LINGO_GENERAL_ID, 0x4a),
    GeneralGetiPodOptionsForLingo => (LINGO_GENERAL_ID, 0x4b),
    GeneralRetiPodOptionsForLingo => (LINGO_GENERAL_ID, 0x4c),
    GeneralGetEventNotification => (LINGO_GENERAL_ID, 0x4d),
    GeneralRetEventNotification => (LINGO_GENERAL_ID, 0x4e),
    GeneralGetSupportedEventNotification => (LINGO_GENERAL_ID, 0x4f),
    GeneralCancelCommand => (LINGO_GENERAL_ID, 0x50),
    GeneralRetSupportedEventNotification => (LINGO_GENERAL_ID, 0x51),
    GeneralSetAvailableCurrent => (LINGO_GENERAL_ID, 0x54),
    GeneralRequestApplicationLaunch => (LINGO_GENERAL_ID, 0x64),
    GeneralGetNowPlayingFocusApp => (LINGO_GENERAL_ID, 0x65),
    GeneralRetNowPlayingFocusApp => (LINGO_GENERAL_ID, 0x66),

    AudioAccAck => (LINGO_DIGITAL_AUDIO_ID, 0x00),
    AudioIPodAck => (LINGO_DIGITAL_AUDIO_ID, 0x01),
    AudioGetAccSampleRateCaps => (LINGO_DIGITAL_AUDIO_ID, 0x02),
    AudioRetAccSampleRateCaps => (LINGO_DIGITAL_AUDIO_ID, 0x03),
    AudioTrackNewAudioAttributes => (LINGO_DIGITAL_AUDIO_ID, 0x04),
    AudioSetVideoDelay => (LINGO_DIGITAL_AUDIO_ID, 0x05),

    SimpleContextButtonStatus => (LINGO_SIMPLE_REMOTE_ID, 0x00),
    SimpleAck => (LINGO_SIMPLE_REMOTE_ID, 0x01),
    SimpleVideoButtonStatus => (LINGO_SIMPLE_REMOTE_ID, 0x03),
    SimpleAudioButtonStatus => (LINGO_SIMPLE_REMOTE_ID, 0x04),
    SimpleIPodOutButtonStatus => (LINGO_SIMPLE_REMOTE_ID, 0x0b),
    SimpleRotationInputStatus => (LINGO_SIMPLE_REMOTE_ID, 0x0c),
    SimpleRadioButtonStatus => (LINGO_SIMPLE_REMOTE_ID, 0x0d),
    SimpleCameraButtonStatus => (LINGO_SIMPLE_REMOTE_ID, 0x0e),
    SimpleRegisterDescriptor => (LINGO_SIMPLE_REMOTE_ID, 0x0f),
    SimpleSendHidReportToIPod => (LINGO_SIMPLE_REMOTE_ID, 0x10),
    SimpleSendHidReportToAcc => (LINGO_SIMPLE_REMOTE_ID, 0x11),
    SimpleUnregisterDescriptor => (LINGO_SIMPLE_REMOTE_ID, 0x12),
    SimpleAccessibilityEvent => (LINGO_SIMPLE_REMOTE_ID, 0x13),
    SimpleGetAccessibilityParameter => (LINGO_SIMPLE_REMOTE_ID, 0x14),
    SimpleRetAccessibilityParameter => (LINGO_SIMPLE_REMOTE_ID, 0x15),
    SimpleSetAccessibilityParameter => (LINGO_SIMPLE_REMOTE_ID, 0x16),
    SimpleGetCurrentItemProperty => (LINGO_SIMPLE_REMOTE_ID, 0x17),
    SimpleRetCurrentItemProperty => (LINGO_SIMPLE_REMOTE_ID, 0x18),
    SimpleSetContext => (LINGO_SIMPLE_REMOTE_ID, 0x19),
    SimpleAccParameterChanged => (LINGO_SIMPLE_REMOTE_ID, 0x1a),
    SimpleDevAck => (LINGO_SIMPLE_REMOTE_ID, 0x81),

    DisplayAck => (LINGO_DISPLAY_REMOTE_ID, 0x00),
    DisplayGetCurrentEqProfileIndex => (LINGO_DISPLAY_REMOTE_ID, 0x01),
    DisplayRetCurrentEqProfileIndex => (LINGO_DISPLAY_REMOTE_ID, 0x02),
    DisplaySetCurrentEqProfileIndex => (LINGO_DISPLAY_REMOTE_ID, 0x03),
    DisplayGetNumEqProfiles => (LINGO_DISPLAY_REMOTE_ID, 0x04),
    DisplayRetNumEqProfiles => (LINGO_DISPLAY_REMOTE_ID, 0x05),
    DisplayGetIndexedEqProfileName => (LINGO_DISPLAY_REMOTE_ID, 0x06),
    DisplayRetIndexedEqProfileName => (LINGO_DISPLAY_REMOTE_ID, 0x07),
    DisplaySetRemoteEventNotification => (LINGO_DISPLAY_REMOTE_ID, 0x08),
    DisplayRemoteEventNotification => (LINGO_DISPLAY_REMOTE_ID, 0x09),
    DisplayGetRemoteEventStatus => (LINGO_DISPLAY_REMOTE_ID, 0x0a),
    DisplayRetRemoteEventStatus => (LINGO_DISPLAY_REMOTE_ID, 0x0b),
    DisplayGetiPodStateInfo => (LINGO_DISPLAY_REMOTE_ID, 0x0c),
    DisplayRetiPodStateInfo => (LINGO_DISPLAY_REMOTE_ID, 0x0d),
    DisplaySetiPodStateInfo => (LINGO_DISPLAY_REMOTE_ID, 0x0e),
    DisplayGetPlayStatus => (LINGO_DISPLAY_REMOTE_ID, 0x0f),
    DisplayRetPlayStatus => (LINGO_DISPLAY_REMOTE_ID, 0x10),
    DisplaySetCurrentPlayingTrack => (LINGO_DISPLAY_REMOTE_ID, 0x11),
    DisplayGetIndexedPlayingTrackInfo => (LINGO_DISPLAY_REMOTE_ID, 0x12),
    DisplayRetIndexedPlayingTrackInfo => (LINGO_DISPLAY_REMOTE_ID, 0x13),
    DisplayGetNumPlayingTracks => (LINGO_DISPLAY_REMOTE_ID, 0x14),
    DisplayRetNumPlayingTracks => (LINGO_DISPLAY_REMOTE_ID, 0x15),
    DisplayGetArtworkFormats => (LINGO_DISPLAY_REMOTE_ID, 0x16),
    DisplayRetArtworkFormats => (LINGO_DISPLAY_REMOTE_ID, 0x17),
    DisplayGetTrackArtworkData => (LINGO_DISPLAY_REMOTE_ID, 0x18),
    DisplayRetTrackArtworkData => (LINGO_DISPLAY_REMOTE_ID, 0x19),
    DisplayGetPowerBatteryState => (LINGO_DISPLAY_REMOTE_ID, 0x1a),
    DisplayRetPowerBatteryState => (LINGO_DISPLAY_REMOTE_ID, 0x1b),
    DisplayGetSoundCheckState => (LINGO_DISPLAY_REMOTE_ID, 0x1c),
    DisplayRetSoundCheckState => (LINGO_DISPLAY_REMOTE_ID, 0x1d),
    DisplaySetSoundCheckState => (LINGO_DISPLAY_REMOTE_ID, 0x1e),
    DisplayGetTrackArtworkTimes => (LINGO_DISPLAY_REMOTE_ID, 0x1f),
    DisplayRetTrackArtworkTimes => (LINGO_DISPLAY_REMOTE_ID, 0x20),

    ExtAck => (LINGO_EXT_REMOTE_ID, 0x0001),
    ExtGetCurrentPlayingTrackChapterInfo => (LINGO_EXT_REMOTE_ID, 0x0002),
    ExtReturnCurrentPlayingTrackChapterInfo => (LINGO_EXT_REMOTE_ID, 0x0003),
    ExtSetCurrentPlayingTrackChapter => (LINGO_EXT_REMOTE_ID, 0x0004),
    ExtGetCurrentPlayingTrackChapterPlayStatus => (LINGO_EXT_REMOTE_ID, 0x0005),
    ExtReturnCurrentPlayingTrackChapterPlayStatus => (LINGO_EXT_REMOTE_ID, 0x0006),
    ExtGetCurrentPlayingTrackChapterName => (LINGO_EXT_REMOTE_ID, 0x0007),
    ExtReturnCurrentPlayingTrackChapterName => (LINGO_EXT_REMOTE_ID, 0x0008),
    ExtGetAudiobookSpeed => (LINGO_EXT_REMOTE_ID, 0x0009),
    ExtReturnAudiobookSpeed => (LINGO_EXT_REMOTE_ID, 0x000a),
    ExtSetAudiobookSpeed => (LINGO_EXT_REMOTE_ID, 0x000b),
    ExtGetIndexedPlayingTrackInfo => (LINGO_EXT_REMOTE_ID, 0x000c),
    ExtReturnIndexedPlayingTrackInfo => (LINGO_EXT_REMOTE_ID, 0x000d),
    ExtGetArtworkFormats => (LINGO_EXT_REMOTE_ID, 0x000e),
    ExtRetArtworkFormats => (LINGO_EXT_REMOTE_ID, 0x000f),
    ExtGetTrackArtworkData => (LINGO_EXT_REMOTE_ID, 0x0010),
    ExtRetTrackArtworkData => (LINGO_EXT_REMOTE_ID, 0x0011),
    ExtResetDbSelection => (LINGO_EXT_REMOTE_ID, 0x0016),
    ExtSelectDbRecord => (LINGO_EXT_REMOTE_ID, 0x0017),
    ExtGetNumberCategorizedDbRecords => (LINGO_EXT_REMOTE_ID, 0x0018),
    ExtReturnNumberCategorizedDbRecords => (LINGO_EXT_REMOTE_ID, 0x0019),
    ExtRetrieveCategorizedDatabaseRecords => (LINGO_EXT_REMOTE_ID, 0x001a),
    ExtReturnCategorizedDatabaseRecord => (LINGO_EXT_REMOTE_ID, 0x001b),
    ExtGetPlayStatus => (LINGO_EXT_REMOTE_ID, 0x001c),
    ExtReturnPlayStatus => (LINGO_EXT_REMOTE_ID, 0x001d),
    ExtGetCurrentPlayingTrackIndex => (LINGO_EXT_REMOTE_ID, 0x001e),
    ExtReturnCurrentPlayingTrackIndex => (LINGO_EXT_REMOTE_ID, 0x001f),
    ExtGetIndexedPlayingTrackTitle => (LINGO_EXT_REMOTE_ID, 0x0020),
    ExtReturnIndexedPlayingTrackTitle => (LINGO_EXT_REMOTE_ID, 0x0021),
    ExtGetIndexedPlayingTrackArtistName => (LINGO_EXT_REMOTE_ID, 0x0022),
    ExtReturnIndexedPlayingTrackArtistName => (LINGO_EXT_REMOTE_ID, 0x0023),
    ExtGetIndexedPlayingTrackAlbumName => (LINGO_EXT_REMOTE_ID, 0x0024),
    ExtReturnIndexedPlayingTrackAlbumName => (LINGO_EXT_REMOTE_ID, 0x0025),
    ExtSetPlayStatusChangeNotification => (LINGO_EXT_REMOTE_ID, 0x0026),
    ExtSetPlayStatusChangeNotificationShort => (LINGO_EXT_REMOTE_ID, 0x0026),
    ExtPlayStatusChangeNotification => (LINGO_EXT_REMOTE_ID, 0x0027),
    ExtPlayCurrentSelection => (LINGO_EXT_REMOTE_ID, 0x0028),
    ExtPlayControl => (LINGO_EXT_REMOTE_ID, 0x0029),
    ExtGetTrackArtworkTimes => (LINGO_EXT_REMOTE_ID, 0x002a),
    ExtRetTrackArtworkTimes => (LINGO_EXT_REMOTE_ID, 0x002b),
    ExtGetShuffle => (LINGO_EXT_REMOTE_ID, 0x002c),
    ExtReturnShuffle => (LINGO_EXT_REMOTE_ID, 0x002d),
    ExtSetShuffle => (LINGO_EXT_REMOTE_ID, 0x002e),
    ExtGetRepeat => (LINGO_EXT_REMOTE_ID, 0x002f),
    ExtReturnRepeat => (LINGO_EXT_REMOTE_ID, 0x0030),
    ExtSetRepeat => (LINGO_EXT_REMOTE_ID, 0x0031),
    ExtSetDisplayImage => (LINGO_EXT_REMOTE_ID, 0x0032),
    ExtGetMonoDisplayImageLimits => (LINGO_EXT_REMOTE_ID, 0x0033),
    ExtReturnMonoDisplayImageLimits => (LINGO_EXT_REMOTE_ID, 0x0034),
    ExtGetNumPlayingTracks => (LINGO_EXT_REMOTE_ID, 0x0035),
    ExtReturnNumPlayingTracks => (LINGO_EXT_REMOTE_ID, 0x0036),
    ExtSetCurrentPlayingTrack => (LINGO_EXT_REMOTE_ID, 0x0037),
    ExtSelectSortDbRecord => (LINGO_EXT_REMOTE_ID, 0x0038),
    ExtGetColorDisplayImageLimits => (LINGO_EXT_REMOTE_ID, 0x0039),
    ExtReturnColorDisplayImageLimits => (LINGO_EXT_REMOTE_ID, 0x003a),
    ExtResetDbSelectionHierarchy => (LINGO_EXT_REMOTE_ID, 0x003b),
    ExtGetDbITunesInfo => (LINGO_EXT_REMOTE_ID, 0x003c),
    ExtRetDbITunesInfo => (LINGO_EXT_REMOTE_ID, 0x003d),
    ExtGetUidTrackInfo => (LINGO_EXT_REMOTE_ID, 0x003e),
    ExtRetUidTrackInfo => (LINGO_EXT_REMOTE_ID, 0x003f),
    ExtGetDbTrackInfo => (LINGO_EXT_REMOTE_ID, 0x0040),
    ExtRetDbTrackInfo => (LINGO_EXT_REMOTE_ID, 0x0041),
    ExtGetPbTrackInfo => (LINGO_EXT_REMOTE_ID, 0x0042),
    ExtRetPbTrackInfo => (LINGO_EXT_REMOTE_ID, 0x0043),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegistryEntry {
    pub id: LingoCmdId,
    pub name: &'static str,
    pub fixed_len: Option<usize>,
    kind: PayloadKind,
}

macro_rules! define_registry {
    ($( $kind:ident => ($variant:ident, $ty:ty, $name:expr, ($lingo:expr, $cmd:expr), $fixed:expr), )+ ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        enum PayloadKind {
            $($kind,)+
        }

        impl PayloadKind {
            fn decode(self, data: &[u8]) -> Result<CommandPayload> {
                Ok(match self {
                    $(PayloadKind::$kind => CommandPayload::$variant(<$ty>::decode(data)?),)+
                })
            }
        }

        static REGISTRY: &[RegistryEntry] = &[
            $(RegistryEntry {
                id: LingoCmdId::new($lingo, $cmd),
                name: $name,
                fixed_len: $fixed,
                kind: PayloadKind::$kind,
            },)+
        ];

        pub fn registry() -> &'static [RegistryEntry] {
            REGISTRY
        }
    };
}

define_registry! {
    GeneralRequestIdentify => (GeneralRequestIdentify, general::RequestIdentify, "general::RequestIdentify", (LINGO_GENERAL_ID, 0x00), Some(0)),
    GeneralAck => (GeneralAck, general::Ack, "general::ACK", (LINGO_GENERAL_ID, 0x02), Some(2)),
    GeneralAckPending => (GeneralAckPending, general::AckPending, "general::ACKPending", (LINGO_GENERAL_ID, 0x02), Some(6)),
    GeneralAckDataDropped => (GeneralAckDataDropped, general::AckDataDropped, "general::ACKDataDropped", (LINGO_GENERAL_ID, 0x02), Some(8)),
    GeneralRequestRemoteUiMode => (GeneralRequestRemoteUiMode, general::RequestRemoteUiMode, "general::RequestRemoteUIMode", (LINGO_GENERAL_ID, 0x03), Some(0)),
    GeneralReturnRemoteUiMode => (GeneralReturnRemoteUiMode, general::ReturnRemoteUiMode, "general::ReturnRemoteUIMode", (LINGO_GENERAL_ID, 0x04), Some(1)),
    GeneralEnterRemoteUiMode => (GeneralEnterRemoteUiMode, general::EnterRemoteUiMode, "general::EnterRemoteUIMode", (LINGO_GENERAL_ID, 0x05), Some(0)),
    GeneralExitRemoteUiMode => (GeneralExitRemoteUiMode, general::ExitRemoteUiMode, "general::ExitRemoteUIMode", (LINGO_GENERAL_ID, 0x06), Some(0)),
    GeneralRequestIPodName => (GeneralRequestIPodName, general::RequestIPodName, "general::RequestiPodName", (LINGO_GENERAL_ID, 0x07), Some(0)),
    GeneralReturnIPodName => (GeneralReturnIPodName, general::ReturnIPodName, "general::ReturniPodName", (LINGO_GENERAL_ID, 0x08), None),
    GeneralRequestIPodSoftwareVersion => (GeneralRequestIPodSoftwareVersion, general::RequestIPodSoftwareVersion, "general::RequestiPodSoftwareVersion", (LINGO_GENERAL_ID, 0x09), Some(0)),
    GeneralReturnIPodSoftwareVersion => (GeneralReturnIPodSoftwareVersion, general::ReturnIPodSoftwareVersion, "general::ReturniPodSoftwareVersion", (LINGO_GENERAL_ID, 0x0a), Some(3)),
    GeneralRequestIPodSerialNum => (GeneralRequestIPodSerialNum, general::RequestIPodSerialNum, "general::RequestiPodSerialNum", (LINGO_GENERAL_ID, 0x0b), Some(0)),
    GeneralReturnIPodSerialNum => (GeneralReturnIPodSerialNum, general::ReturnIPodSerialNum, "general::ReturniPodSerialNum", (LINGO_GENERAL_ID, 0x0c), None),
    GeneralRequestIPodModelNum => (GeneralRequestIPodModelNum, general::RequestIPodModelNum, "general::RequestiPodModelNum", (LINGO_GENERAL_ID, 0x0d), Some(0)),
    GeneralReturnIPodModelNum => (GeneralReturnIPodModelNum, general::ReturnIPodModelNum, "general::ReturniPodModelNum", (LINGO_GENERAL_ID, 0x0e), None),
    GeneralRequestLingoProtocolVersion => (GeneralRequestLingoProtocolVersion, general::RequestLingoProtocolVersion, "general::RequestLingoProtocolVersion", (LINGO_GENERAL_ID, 0x0f), Some(1)),
    GeneralReturnLingoProtocolVersion => (GeneralReturnLingoProtocolVersion, general::ReturnLingoProtocolVersion, "general::ReturnLingoProtocolVersion", (LINGO_GENERAL_ID, 0x10), Some(3)),
    GeneralRequestTransportMaxPayloadSize => (GeneralRequestTransportMaxPayloadSize, general::RequestTransportMaxPayloadSize, "general::RequestTransportMaxPayloadSize", (LINGO_GENERAL_ID, 0x11), Some(0)),
    GeneralReturnTransportMaxPayloadSize => (GeneralReturnTransportMaxPayloadSize, general::ReturnTransportMaxPayloadSize, "general::ReturnTransportMaxPayloadSize", (LINGO_GENERAL_ID, 0x12), Some(2)),
    GeneralIdentifyDeviceLingoes => (GeneralIdentifyDeviceLingoes, general::IdentifyDeviceLingoes, "general::IdentifyDeviceLingoes", (LINGO_GENERAL_ID, 0x13), Some(12)),
    GeneralGetDevAuthenticationInfo => (GeneralGetDevAuthenticationInfo, general::GetDevAuthenticationInfo, "general::GetDevAuthenticationInfo", (LINGO_GENERAL_ID, 0x14), Some(0)),
    GeneralRetDevAuthenticationInfo => (GeneralRetDevAuthenticationInfo, general::RetDevAuthenticationInfo, "general::RetDevAuthenticationInfo", (LINGO_GENERAL_ID, 0x15), None),
    GeneralAckDevAuthenticationInfo => (GeneralAckDevAuthenticationInfo, general::AckDevAuthenticationInfo, "general::AckDevAuthenticationInfo", (LINGO_GENERAL_ID, 0x16), Some(1)),
    GeneralGetDevAuthenticationSignatureV1 => (GeneralGetDevAuthenticationSignatureV1, general::GetDevAuthenticationSignatureV1, "general::GetDevAuthenticationSignatureV1", (LINGO_GENERAL_ID, 0x17), Some(17)),
    GeneralGetDevAuthenticationSignatureV2 => (GeneralGetDevAuthenticationSignatureV2, general::GetDevAuthenticationSignatureV2, "general::GetDevAuthenticationSignatureV2", (LINGO_GENERAL_ID, 0x17), Some(21)),
    GeneralRetDevAuthenticationSignature => (GeneralRetDevAuthenticationSignature, general::RetDevAuthenticationSignature, "general::RetDevAuthenticationSignature", (LINGO_GENERAL_ID, 0x18), None),
    GeneralAckDevAuthenticationStatus => (GeneralAckDevAuthenticationStatus, general::AckDevAuthenticationStatus, "general::AckDevAuthenticationStatus", (LINGO_GENERAL_ID, 0x19), Some(1)),
    GeneralGetIPodAuthenticationInfo => (GeneralGetIPodAuthenticationInfo, general::GetIPodAuthenticationInfo, "general::GetiPodAuthenticationInfo", (LINGO_GENERAL_ID, 0x1a), Some(0)),
    GeneralRetiPodAuthenticationInfo => (GeneralRetiPodAuthenticationInfo, general::RetiPodAuthenticationInfo, "general::RetiPodAuthenticationInfo", (LINGO_GENERAL_ID, 0x1b), None),
    GeneralAckiPodAuthenticationInfo => (GeneralAckiPodAuthenticationInfo, general::AckiPodAuthenticationInfo, "general::AckiPodAuthenticationInfo", (LINGO_GENERAL_ID, 0x1c), Some(1)),
    GeneralGetiPodAuthenticationSignature => (GeneralGetiPodAuthenticationSignature, general::GetiPodAuthenticationSignature, "general::GetiPodAuthenticationSignature", (LINGO_GENERAL_ID, 0x1d), Some(21)),
    GeneralRetiPodAuthenticationSignature => (GeneralRetiPodAuthenticationSignature, general::RetiPodAuthenticationSignature, "general::RetiPodAuthenticationSignature", (LINGO_GENERAL_ID, 0x1e), Some(20)),
    GeneralAckiPodAuthenticationStatus => (GeneralAckiPodAuthenticationStatus, general::AckiPodAuthenticationStatus, "general::AckiPodAuthenticationStatus", (LINGO_GENERAL_ID, 0x1f), Some(1)),
    GeneralNotifyiPodStateChange => (GeneralNotifyiPodStateChange, general::NotifyiPodStateChange, "general::NotifyiPodStateChange", (LINGO_GENERAL_ID, 0x23), Some(1)),
    GeneralGetIPodOptions => (GeneralGetIPodOptions, general::GetIPodOptions, "general::GetiPodOptions", (LINGO_GENERAL_ID, 0x24), Some(0)),
    GeneralRetiPodOptions => (GeneralRetiPodOptions, general::RetiPodOptions, "general::RetiPodOptions", (LINGO_GENERAL_ID, 0x25), Some(8)),
    GeneralGetAccessoryInfo => (GeneralGetAccessoryInfo, general::GetAccessoryInfo, "general::GetAccessoryInfo", (LINGO_GENERAL_ID, 0x27), Some(1)),
    GeneralGetAccessoryInfo2 => (GeneralGetAccessoryInfo2, general::GetAccessoryInfo2, "general::GetAccessoryInfo2", (LINGO_GENERAL_ID, 0x27), Some(8)),
    GeneralGetAccessoryInfo3 => (GeneralGetAccessoryInfo3, general::GetAccessoryInfo3, "general::GetAccessoryInfo3", (LINGO_GENERAL_ID, 0x27), Some(2)),
    GeneralRetAccessoryInfo => (GeneralRetAccessoryInfo, general::RetAccessoryInfo, "general::RetAccessoryInfo", (LINGO_GENERAL_ID, 0x28), None),
    GeneralGetiPodPreferences => (GeneralGetiPodPreferences, general::GetiPodPreferences, "general::GetiPodPreferences", (LINGO_GENERAL_ID, 0x29), Some(1)),
    GeneralRetiPodPreferences => (GeneralRetiPodPreferences, general::RetiPodPreferences, "general::RetiPodPreferences", (LINGO_GENERAL_ID, 0x2a), Some(2)),
    GeneralSetiPodPreferences => (GeneralSetiPodPreferences, general::SetiPodPreferences, "general::SetiPodPreferences", (LINGO_GENERAL_ID, 0x2b), Some(3)),
    GeneralGetUiMode => (GeneralGetUiMode, general::GetUiMode, "general::GetUIMode", (LINGO_GENERAL_ID, 0x35), Some(0)),
    GeneralRetUiMode => (GeneralRetUiMode, general::RetUiMode, "general::RetUIMode", (LINGO_GENERAL_ID, 0x36), Some(1)),
    GeneralSetUiMode => (GeneralSetUiMode, general::SetUiMode, "general::SetUIMode", (LINGO_GENERAL_ID, 0x37), Some(1)),
    GeneralStartIdps => (GeneralStartIdps, general::StartIdps, "general::StartIDPS", (LINGO_GENERAL_ID, 0x38), Some(0)),
    GeneralSetFidTokenValues => (GeneralSetFidTokenValues, general::SetFidTokenValues, "general::SetFIDTokenValues", (LINGO_GENERAL_ID, 0x39), None),
    GeneralRetFidTokenValueAcks => (GeneralRetFidTokenValueAcks, general::RetFidTokenValueAcks, "general::RetFIDTokenValueACKs", (LINGO_GENERAL_ID, 0x3a), None),
    GeneralEndIdps => (GeneralEndIdps, general::EndIdps, "general::EndIDPS", (LINGO_GENERAL_ID, 0x3b), Some(1)),
    GeneralIdpsStatus => (GeneralIdpsStatus, general::IdpsStatus, "general::IDPSStatus", (LINGO_GENERAL_ID, 0x3c), Some(1)),
    GeneralOpenDataSessionForProtocol => (GeneralOpenDataSessionForProtocol, general::OpenDataSessionForProtocol, "general::OpenDataSessionForProtocol", (LINGO_GENERAL_ID, 0x3f), Some(3)),
    GeneralCloseDataSession => (GeneralCloseDataSession, general::CloseDataSession, "general::CloseDataSession", (LINGO_GENERAL_ID, 0x40), Some(2)),
    GeneralDevAck => (GeneralDevAck, general::DevAck, "general::DevACK", (LINGO_GENERAL_ID, 0x41), Some(2)),
    GeneralDevDataTransfer => (GeneralDevDataTransfer, general::DevDataTransfer, "general::DevDataTransfer", (LINGO_GENERAL_ID, 0x42), None),
    GeneralIPodDataTransfer => (GeneralIPodDataTransfer, general::IPodDataTransfer, "general::IPodDataTransfer", (LINGO_GENERAL_ID, 0x43), None),
    GeneralSetAccStatusNotification => (GeneralSetAccStatusNotification, general::SetAccStatusNotification, "general::SetAccStatusNotification", (LINGO_GENERAL_ID, 0x46), Some(4)),
    GeneralRetAccStatusNotification => (GeneralRetAccStatusNotification, general::RetAccStatusNotification, "general::RetAccStatusNotification", (LINGO_GENERAL_ID, 0x47), Some(4)),
    GeneralAccessoryStatusNotification => (GeneralAccessoryStatusNotification, general::AccessoryStatusNotification, "general::AccessoryStatusNotification", (LINGO_GENERAL_ID, 0x48), None),
    GeneralSetEventNotification => (GeneralSetEventNotification, general::SetEventNotification, "general::SetEventNotification", (LINGO_GENERAL_ID, 0x49), Some(8)),
    GeneralIPodNotification => (GeneralIPodNotification, general::IPodNotification, "general::IPodNotification", (LINGO_GENERAL_ID, 0x4a), None),
    GeneralGetiPodOptionsForLingo => (GeneralGetiPodOptionsForLingo, general::GetiPodOptionsForLingo, "general::GetiPodOptionsForLingo", (LINGO_GENERAL_ID, 0x4b), Some(1)),
    GeneralRetiPodOptionsForLingo => (GeneralRetiPodOptionsForLingo, general::RetiPodOptionsForLingo, "general::RetiPodOptionsForLingo", (LINGO_GENERAL_ID, 0x4c), Some(9)),
    GeneralGetEventNotification => (GeneralGetEventNotification, general::GetEventNotification, "general::GetEventNotification", (LINGO_GENERAL_ID, 0x4d), Some(0)),
    GeneralRetEventNotification => (GeneralRetEventNotification, general::RetEventNotification, "general::RetEventNotification", (LINGO_GENERAL_ID, 0x4e), Some(8)),
    GeneralGetSupportedEventNotification => (GeneralGetSupportedEventNotification, general::GetSupportedEventNotification, "general::GetSupportedEventNotification", (LINGO_GENERAL_ID, 0x4f), Some(0)),
    GeneralCancelCommand => (GeneralCancelCommand, general::CancelCommand, "general::CancelCommand", (LINGO_GENERAL_ID, 0x50), Some(5)),
    GeneralRetSupportedEventNotification => (GeneralRetSupportedEventNotification, general::RetSupportedEventNotification, "general::RetSupportedEventNotification", (LINGO_GENERAL_ID, 0x51), Some(8)),
    GeneralSetAvailableCurrent => (GeneralSetAvailableCurrent, general::SetAvailableCurrent, "general::SetAvailableCurrent", (LINGO_GENERAL_ID, 0x54), Some(2)),
    GeneralRequestApplicationLaunch => (GeneralRequestApplicationLaunch, general::RequestApplicationLaunch, "general::RequestApplicationLaunch", (LINGO_GENERAL_ID, 0x64), None),
    GeneralGetNowPlayingFocusApp => (GeneralGetNowPlayingFocusApp, general::GetNowPlayingFocusApp, "general::GetNowPlayingFocusApp", (LINGO_GENERAL_ID, 0x65), Some(0)),
    GeneralRetNowPlayingFocusApp => (GeneralRetNowPlayingFocusApp, general::RetNowPlayingFocusApp, "general::RetNowPlayingFocusApp", (LINGO_GENERAL_ID, 0x66), None),

    AudioAccAck => (AudioAccAck, audio::AccAck, "audio::AccAck", (LINGO_DIGITAL_AUDIO_ID, 0x00), Some(2)),
    AudioIPodAck => (AudioIPodAck, audio::IPodAck, "audio::iPodAck", (LINGO_DIGITAL_AUDIO_ID, 0x01), Some(2)),
    AudioGetAccSampleRateCaps => (AudioGetAccSampleRateCaps, audio::GetAccSampleRateCaps, "audio::GetAccSampleRateCaps", (LINGO_DIGITAL_AUDIO_ID, 0x02), Some(0)),
    AudioRetAccSampleRateCaps => (AudioRetAccSampleRateCaps, audio::RetAccSampleRateCaps, "audio::RetAccSampleRateCaps", (LINGO_DIGITAL_AUDIO_ID, 0x03), None),
    AudioTrackNewAudioAttributes => (AudioTrackNewAudioAttributes, audio::TrackNewAudioAttributes, "audio::TrackNewAudioAttributes", (LINGO_DIGITAL_AUDIO_ID, 0x04), Some(12)),
    AudioSetVideoDelay => (AudioSetVideoDelay, audio::SetVideoDelay, "audio::SetVideoDelay", (LINGO_DIGITAL_AUDIO_ID, 0x05), Some(4)),

    SimpleContextButtonStatus => (SimpleContextButtonStatus, simple::ContextButtonStatus, "simpleremote::ContextButtonStatus", (LINGO_SIMPLE_REMOTE_ID, 0x00), None),
    SimpleAck => (SimpleAck, simple::Ack, "simpleremote::ACK", (LINGO_SIMPLE_REMOTE_ID, 0x01), Some(0)),
    SimpleVideoButtonStatus => (SimpleVideoButtonStatus, simple::VideoButtonStatus, "simpleremote::VideoButtonStatus", (LINGO_SIMPLE_REMOTE_ID, 0x03), None),
    SimpleAudioButtonStatus => (SimpleAudioButtonStatus, simple::AudioButtonStatus, "simpleremote::AudioButtonStatus", (LINGO_SIMPLE_REMOTE_ID, 0x04), None),
    SimpleIPodOutButtonStatus => (SimpleIPodOutButtonStatus, simple::IPodOutButtonStatus, "simpleremote::iPodOutButtonStatus", (LINGO_SIMPLE_REMOTE_ID, 0x0b), Some(1)),
    SimpleRotationInputStatus => (SimpleRotationInputStatus, simple::RotationInputStatus, "simpleremote::RotationInputStatus", (LINGO_SIMPLE_REMOTE_ID, 0x0c), Some(0)),
    SimpleRadioButtonStatus => (SimpleRadioButtonStatus, simple::RadioButtonStatus, "simpleremote::RadioButtonStatus", (LINGO_SIMPLE_REMOTE_ID, 0x0d), Some(0)),
    SimpleCameraButtonStatus => (SimpleCameraButtonStatus, simple::CameraButtonStatus, "simpleremote::CameraButtonStatus", (LINGO_SIMPLE_REMOTE_ID, 0x0e), Some(0)),
    SimpleRegisterDescriptor => (SimpleRegisterDescriptor, simple::RegisterDescriptor, "simpleremote::RegisterDescriptor", (LINGO_SIMPLE_REMOTE_ID, 0x0f), Some(0)),
    SimpleSendHidReportToIPod => (SimpleSendHidReportToIPod, simple::SendHidReportToIPod, "simpleremote::SendHIDReportToiPod", (LINGO_SIMPLE_REMOTE_ID, 0x10), Some(0)),
    SimpleSendHidReportToAcc => (SimpleSendHidReportToAcc, simple::SendHidReportToAcc, "simpleremote::SendHIDReportToAcc", (LINGO_SIMPLE_REMOTE_ID, 0x11), Some(0)),
    SimpleUnregisterDescriptor => (SimpleUnregisterDescriptor, simple::UnregisterDescriptor, "simpleremote::UnregisterDescriptor", (LINGO_SIMPLE_REMOTE_ID, 0x12), Some(0)),
    SimpleAccessibilityEvent => (SimpleAccessibilityEvent, simple::AccessibilityEvent, "simpleremote::AccessibilityEvent", (LINGO_SIMPLE_REMOTE_ID, 0x13), Some(0)),
    SimpleGetAccessibilityParameter => (SimpleGetAccessibilityParameter, simple::GetAccessibilityParameter, "simpleremote::GetAccessibilityParameter", (LINGO_SIMPLE_REMOTE_ID, 0x14), Some(0)),
    SimpleRetAccessibilityParameter => (SimpleRetAccessibilityParameter, simple::RetAccessibilityParameter, "simpleremote::RetAccessibilityParameter", (LINGO_SIMPLE_REMOTE_ID, 0x15), Some(0)),
    SimpleSetAccessibilityParameter => (SimpleSetAccessibilityParameter, simple::SetAccessibilityParameter, "simpleremote::SetAccessibilityParameter", (LINGO_SIMPLE_REMOTE_ID, 0x16), Some(0)),
    SimpleGetCurrentItemProperty => (SimpleGetCurrentItemProperty, simple::GetCurrentItemProperty, "simpleremote::GetCurrentItemProperty", (LINGO_SIMPLE_REMOTE_ID, 0x17), Some(0)),
    SimpleRetCurrentItemProperty => (SimpleRetCurrentItemProperty, simple::RetCurrentItemProperty, "simpleremote::RetCurrentItemProperty", (LINGO_SIMPLE_REMOTE_ID, 0x18), Some(0)),
    SimpleSetContext => (SimpleSetContext, simple::SetContext, "simpleremote::SetContext", (LINGO_SIMPLE_REMOTE_ID, 0x19), Some(0)),
    SimpleAccParameterChanged => (SimpleAccParameterChanged, simple::AccParameterChanged, "simpleremote::AccParameterChanged", (LINGO_SIMPLE_REMOTE_ID, 0x1a), Some(0)),
    SimpleDevAck => (SimpleDevAck, simple::DevAck, "simpleremote::DevACK", (LINGO_SIMPLE_REMOTE_ID, 0x81), Some(0)),

    DisplayAck => (DisplayAck, disp::Ack, "dispremote::ACK", (LINGO_DISPLAY_REMOTE_ID, 0x00), Some(2)),
    DisplayGetCurrentEqProfileIndex => (DisplayGetCurrentEqProfileIndex, disp::GetCurrentEqProfileIndex, "dispremote::GetCurrentEQProfileIndex", (LINGO_DISPLAY_REMOTE_ID, 0x01), Some(0)),
    DisplayRetCurrentEqProfileIndex => (DisplayRetCurrentEqProfileIndex, disp::RetCurrentEqProfileIndex, "dispremote::RetCurrentEQProfileIndex", (LINGO_DISPLAY_REMOTE_ID, 0x02), Some(4)),
    DisplaySetCurrentEqProfileIndex => (DisplaySetCurrentEqProfileIndex, disp::SetCurrentEqProfileIndex, "dispremote::SetCurrentEQProfileIndex", (LINGO_DISPLAY_REMOTE_ID, 0x03), Some(5)),
    DisplayGetNumEqProfiles => (DisplayGetNumEqProfiles, disp::GetNumEqProfiles, "dispremote::GetNumEQProfiles", (LINGO_DISPLAY_REMOTE_ID, 0x04), Some(0)),
    DisplayRetNumEqProfiles => (DisplayRetNumEqProfiles, disp::RetNumEqProfiles, "dispremote::RetNumEQProfiles", (LINGO_DISPLAY_REMOTE_ID, 0x05), Some(4)),
    DisplayGetIndexedEqProfileName => (DisplayGetIndexedEqProfileName, disp::GetIndexedEqProfileName, "dispremote::GetIndexedEQProfileName", (LINGO_DISPLAY_REMOTE_ID, 0x06), Some(4)),
    DisplayRetIndexedEqProfileName => (DisplayRetIndexedEqProfileName, disp::RetIndexedEqProfileName, "dispremote::RetIndexedEQProfileName", (LINGO_DISPLAY_REMOTE_ID, 0x07), None),
    DisplaySetRemoteEventNotification => (DisplaySetRemoteEventNotification, disp::SetRemoteEventNotification, "dispremote::SetRemoteEventNotification", (LINGO_DISPLAY_REMOTE_ID, 0x08), Some(4)),
    DisplayRemoteEventNotification => (DisplayRemoteEventNotification, disp::RemoteEventNotification, "dispremote::RemoteEventNotification", (LINGO_DISPLAY_REMOTE_ID, 0x09), None),
    DisplayGetRemoteEventStatus => (DisplayGetRemoteEventStatus, disp::GetRemoteEventStatus, "dispremote::GetRemoteEventStatus", (LINGO_DISPLAY_REMOTE_ID, 0x0a), Some(0)),
    DisplayRetRemoteEventStatus => (DisplayRetRemoteEventStatus, disp::RetRemoteEventStatus, "dispremote::RetRemoteEventStatus", (LINGO_DISPLAY_REMOTE_ID, 0x0b), Some(4)),
    DisplayGetiPodStateInfo => (DisplayGetiPodStateInfo, disp::GetiPodStateInfo, "dispremote::GetiPodStateInfo", (LINGO_DISPLAY_REMOTE_ID, 0x0c), Some(1)),
    DisplayRetiPodStateInfo => (DisplayRetiPodStateInfo, disp::RetiPodStateInfo, "dispremote::RetiPodStateInfo", (LINGO_DISPLAY_REMOTE_ID, 0x0d), None),
    DisplaySetiPodStateInfo => (DisplaySetiPodStateInfo, disp::SetiPodStateInfo, "dispremote::SetiPodStateInfo", (LINGO_DISPLAY_REMOTE_ID, 0x0e), Some(2)),
    DisplayGetPlayStatus => (DisplayGetPlayStatus, disp::GetPlayStatus, "dispremote::GetPlayStatus", (LINGO_DISPLAY_REMOTE_ID, 0x0f), Some(0)),
    DisplayRetPlayStatus => (DisplayRetPlayStatus, disp::RetPlayStatus, "dispremote::RetPlayStatus", (LINGO_DISPLAY_REMOTE_ID, 0x10), Some(13)),
    DisplaySetCurrentPlayingTrack => (DisplaySetCurrentPlayingTrack, disp::SetCurrentPlayingTrack, "dispremote::SetCurrentPlayingTrack", (LINGO_DISPLAY_REMOTE_ID, 0x11), Some(4)),
    DisplayGetIndexedPlayingTrackInfo => (DisplayGetIndexedPlayingTrackInfo, disp::GetIndexedPlayingTrackInfo, "dispremote::GetIndexedPlayingTrackInfo", (LINGO_DISPLAY_REMOTE_ID, 0x12), Some(7)),
    DisplayRetIndexedPlayingTrackInfo => (DisplayRetIndexedPlayingTrackInfo, disp::RetIndexedPlayingTrackInfo, "dispremote::RetIndexedPlayingTrackInfo", (LINGO_DISPLAY_REMOTE_ID, 0x13), None),
    DisplayGetNumPlayingTracks => (DisplayGetNumPlayingTracks, disp::GetNumPlayingTracks, "dispremote::GetNumPlayingTracks", (LINGO_DISPLAY_REMOTE_ID, 0x14), Some(0)),
    DisplayRetNumPlayingTracks => (DisplayRetNumPlayingTracks, disp::RetNumPlayingTracks, "dispremote::RetNumPlayingTracks", (LINGO_DISPLAY_REMOTE_ID, 0x15), Some(4)),
    DisplayGetArtworkFormats => (DisplayGetArtworkFormats, disp::GetArtworkFormats, "dispremote::GetArtworkFormats", (LINGO_DISPLAY_REMOTE_ID, 0x16), Some(0)),
    DisplayRetArtworkFormats => (DisplayRetArtworkFormats, disp::RetArtworkFormats, "dispremote::RetArtworkFormats", (LINGO_DISPLAY_REMOTE_ID, 0x17), None),
    DisplayGetTrackArtworkData => (DisplayGetTrackArtworkData, disp::GetTrackArtworkData, "dispremote::GetTrackArtworkData", (LINGO_DISPLAY_REMOTE_ID, 0x18), Some(10)),
    DisplayRetTrackArtworkData => (DisplayRetTrackArtworkData, disp::RetTrackArtworkData, "dispremote::RetTrackArtworkData", (LINGO_DISPLAY_REMOTE_ID, 0x19), Some(0)),
    DisplayGetPowerBatteryState => (DisplayGetPowerBatteryState, disp::GetPowerBatteryState, "dispremote::GetPowerBatteryState", (LINGO_DISPLAY_REMOTE_ID, 0x1a), Some(0)),
    DisplayRetPowerBatteryState => (DisplayRetPowerBatteryState, disp::RetPowerBatteryState, "dispremote::RetPowerBatteryState", (LINGO_DISPLAY_REMOTE_ID, 0x1b), Some(2)),
    DisplayGetSoundCheckState => (DisplayGetSoundCheckState, disp::GetSoundCheckState, "dispremote::GetSoundCheckState", (LINGO_DISPLAY_REMOTE_ID, 0x1c), Some(0)),
    DisplayRetSoundCheckState => (DisplayRetSoundCheckState, disp::RetSoundCheckState, "dispremote::RetSoundCheckState", (LINGO_DISPLAY_REMOTE_ID, 0x1d), Some(1)),
    DisplaySetSoundCheckState => (DisplaySetSoundCheckState, disp::SetSoundCheckState, "dispremote::SetSoundCheckState", (LINGO_DISPLAY_REMOTE_ID, 0x1e), Some(2)),
    DisplayGetTrackArtworkTimes => (DisplayGetTrackArtworkTimes, disp::GetTrackArtworkTimes, "dispremote::GetTrackArtworkTimes", (LINGO_DISPLAY_REMOTE_ID, 0x1f), Some(10)),
    DisplayRetTrackArtworkTimes => (DisplayRetTrackArtworkTimes, disp::RetTrackArtworkTimes, "dispremote::RetTrackArtworkTimes", (LINGO_DISPLAY_REMOTE_ID, 0x20), None),

    ExtAck => (ExtAck, ext::Ack, "extremote::ACK", (LINGO_EXT_REMOTE_ID, 0x0001), Some(3)),
    ExtGetCurrentPlayingTrackChapterInfo => (ExtGetCurrentPlayingTrackChapterInfo, ext::GetCurrentPlayingTrackChapterInfo, "extremote::GetCurrentPlayingTrackChapterInfo", (LINGO_EXT_REMOTE_ID, 0x0002), Some(0)),
    ExtReturnCurrentPlayingTrackChapterInfo => (ExtReturnCurrentPlayingTrackChapterInfo, ext::ReturnCurrentPlayingTrackChapterInfo, "extremote::ReturnCurrentPlayingTrackChapterInfo", (LINGO_EXT_REMOTE_ID, 0x0003), Some(8)),
    ExtSetCurrentPlayingTrackChapter => (ExtSetCurrentPlayingTrackChapter, ext::SetCurrentPlayingTrackChapter, "extremote::SetCurrentPlayingTrackChapter", (LINGO_EXT_REMOTE_ID, 0x0004), Some(4)),
    ExtGetCurrentPlayingTrackChapterPlayStatus => (ExtGetCurrentPlayingTrackChapterPlayStatus, ext::GetCurrentPlayingTrackChapterPlayStatus, "extremote::GetCurrentPlayingTrackChapterPlayStatus", (LINGO_EXT_REMOTE_ID, 0x0005), Some(4)),
    ExtReturnCurrentPlayingTrackChapterPlayStatus => (ExtReturnCurrentPlayingTrackChapterPlayStatus, ext::ReturnCurrentPlayingTrackChapterPlayStatus, "extremote::ReturnCurrentPlayingTrackChapterPlayStatus", (LINGO_EXT_REMOTE_ID, 0x0006), Some(8)),
    ExtGetCurrentPlayingTrackChapterName => (ExtGetCurrentPlayingTrackChapterName, ext::GetCurrentPlayingTrackChapterName, "extremote::GetCurrentPlayingTrackChapterName", (LINGO_EXT_REMOTE_ID, 0x0007), Some(4)),
    ExtReturnCurrentPlayingTrackChapterName => (ExtReturnCurrentPlayingTrackChapterName, ext::ReturnCurrentPlayingTrackChapterName, "extremote::ReturnCurrentPlayingTrackChapterName", (LINGO_EXT_REMOTE_ID, 0x0008), None),
    ExtGetAudiobookSpeed => (ExtGetAudiobookSpeed, ext::GetAudiobookSpeed, "extremote::GetAudiobookSpeed", (LINGO_EXT_REMOTE_ID, 0x0009), Some(0)),
    ExtReturnAudiobookSpeed => (ExtReturnAudiobookSpeed, ext::ReturnAudiobookSpeed, "extremote::ReturnAudiobookSpeed", (LINGO_EXT_REMOTE_ID, 0x000a), Some(1)),
    ExtSetAudiobookSpeed => (ExtSetAudiobookSpeed, ext::SetAudiobookSpeed, "extremote::SetAudiobookSpeed", (LINGO_EXT_REMOTE_ID, 0x000b), Some(1)),
    ExtGetIndexedPlayingTrackInfo => (ExtGetIndexedPlayingTrackInfo, ext::GetIndexedPlayingTrackInfo, "extremote::GetIndexedPlayingTrackInfo", (LINGO_EXT_REMOTE_ID, 0x000c), Some(7)),
    ExtReturnIndexedPlayingTrackInfo => (ExtReturnIndexedPlayingTrackInfo, ext::ReturnIndexedPlayingTrackInfo, "extremote::ReturnIndexedPlayingTrackInfo", (LINGO_EXT_REMOTE_ID, 0x000d), None),
    ExtGetArtworkFormats => (ExtGetArtworkFormats, ext::GetArtworkFormats, "extremote::GetArtworkFormats", (LINGO_EXT_REMOTE_ID, 0x000e), Some(0)),
    ExtRetArtworkFormats => (ExtRetArtworkFormats, ext::RetArtworkFormats, "extremote::RetArtworkFormats", (LINGO_EXT_REMOTE_ID, 0x000f), None),
    ExtGetTrackArtworkData => (ExtGetTrackArtworkData, ext::GetTrackArtworkData, "extremote::GetTrackArtworkData", (LINGO_EXT_REMOTE_ID, 0x0010), Some(10)),
    ExtRetTrackArtworkData => (ExtRetTrackArtworkData, ext::RetTrackArtworkData, "extremote::RetTrackArtworkData", (LINGO_EXT_REMOTE_ID, 0x0011), None),
    ExtResetDbSelection => (ExtResetDbSelection, ext::ResetDbSelection, "extremote::ResetDBSelection", (LINGO_EXT_REMOTE_ID, 0x0016), Some(0)),
    ExtSelectDbRecord => (ExtSelectDbRecord, ext::SelectDbRecord, "extremote::SelectDBRecord", (LINGO_EXT_REMOTE_ID, 0x0017), Some(5)),
    ExtGetNumberCategorizedDbRecords => (ExtGetNumberCategorizedDbRecords, ext::GetNumberCategorizedDbRecords, "extremote::GetNumberCategorizedDBRecords", (LINGO_EXT_REMOTE_ID, 0x0018), Some(1)),
    ExtReturnNumberCategorizedDbRecords => (ExtReturnNumberCategorizedDbRecords, ext::ReturnNumberCategorizedDbRecords, "extremote::ReturnNumberCategorizedDBRecords", (LINGO_EXT_REMOTE_ID, 0x0019), Some(4)),
    ExtRetrieveCategorizedDatabaseRecords => (ExtRetrieveCategorizedDatabaseRecords, ext::RetrieveCategorizedDatabaseRecords, "extremote::RetrieveCategorizedDatabaseRecords", (LINGO_EXT_REMOTE_ID, 0x001a), Some(9)),
    ExtReturnCategorizedDatabaseRecord => (ExtReturnCategorizedDatabaseRecord, ext::ReturnCategorizedDatabaseRecord, "extremote::ReturnCategorizedDatabaseRecord", (LINGO_EXT_REMOTE_ID, 0x001b), Some(20)),
    ExtGetPlayStatus => (ExtGetPlayStatus, ext::GetPlayStatus, "extremote::GetPlayStatus", (LINGO_EXT_REMOTE_ID, 0x001c), Some(0)),
    ExtReturnPlayStatus => (ExtReturnPlayStatus, ext::ReturnPlayStatus, "extremote::ReturnPlayStatus", (LINGO_EXT_REMOTE_ID, 0x001d), Some(9)),
    ExtGetCurrentPlayingTrackIndex => (ExtGetCurrentPlayingTrackIndex, ext::GetCurrentPlayingTrackIndex, "extremote::GetCurrentPlayingTrackIndex", (LINGO_EXT_REMOTE_ID, 0x001e), Some(0)),
    ExtReturnCurrentPlayingTrackIndex => (ExtReturnCurrentPlayingTrackIndex, ext::ReturnCurrentPlayingTrackIndex, "extremote::ReturnCurrentPlayingTrackIndex", (LINGO_EXT_REMOTE_ID, 0x001f), Some(4)),
    ExtGetIndexedPlayingTrackTitle => (ExtGetIndexedPlayingTrackTitle, ext::GetIndexedPlayingTrackTitle, "extremote::GetIndexedPlayingTrackTitle", (LINGO_EXT_REMOTE_ID, 0x0020), Some(4)),
    ExtReturnIndexedPlayingTrackTitle => (ExtReturnIndexedPlayingTrackTitle, ext::ReturnIndexedPlayingTrackTitle, "extremote::ReturnIndexedPlayingTrackTitle", (LINGO_EXT_REMOTE_ID, 0x0021), None),
    ExtGetIndexedPlayingTrackArtistName => (ExtGetIndexedPlayingTrackArtistName, ext::GetIndexedPlayingTrackArtistName, "extremote::GetIndexedPlayingTrackArtistName", (LINGO_EXT_REMOTE_ID, 0x0022), Some(4)),
    ExtReturnIndexedPlayingTrackArtistName => (ExtReturnIndexedPlayingTrackArtistName, ext::ReturnIndexedPlayingTrackArtistName, "extremote::ReturnIndexedPlayingTrackArtistName", (LINGO_EXT_REMOTE_ID, 0x0023), None),
    ExtGetIndexedPlayingTrackAlbumName => (ExtGetIndexedPlayingTrackAlbumName, ext::GetIndexedPlayingTrackAlbumName, "extremote::GetIndexedPlayingTrackAlbumName", (LINGO_EXT_REMOTE_ID, 0x0024), Some(4)),
    ExtReturnIndexedPlayingTrackAlbumName => (ExtReturnIndexedPlayingTrackAlbumName, ext::ReturnIndexedPlayingTrackAlbumName, "extremote::ReturnIndexedPlayingTrackAlbumName", (LINGO_EXT_REMOTE_ID, 0x0025), None),
    ExtSetPlayStatusChangeNotification => (ExtSetPlayStatusChangeNotification, ext::SetPlayStatusChangeNotification, "extremote::SetPlayStatusChangeNotification", (LINGO_EXT_REMOTE_ID, 0x0026), Some(4)),
    ExtSetPlayStatusChangeNotificationShort => (ExtSetPlayStatusChangeNotificationShort, ext::SetPlayStatusChangeNotificationShort, "extremote::SetPlayStatusChangeNotificationShort", (LINGO_EXT_REMOTE_ID, 0x0026), Some(1)),
    ExtPlayStatusChangeNotification => (ExtPlayStatusChangeNotification, ext::PlayStatusChangeNotification, "extremote::PlayStatusChangeNotification", (LINGO_EXT_REMOTE_ID, 0x0027), Some(1)),
    ExtPlayCurrentSelection => (ExtPlayCurrentSelection, ext::PlayCurrentSelection, "extremote::PlayCurrentSelection", (LINGO_EXT_REMOTE_ID, 0x0028), Some(4)),
    ExtPlayControl => (ExtPlayControl, ext::PlayControl, "extremote::PlayControl", (LINGO_EXT_REMOTE_ID, 0x0029), Some(1)),
    ExtGetTrackArtworkTimes => (ExtGetTrackArtworkTimes, ext::GetTrackArtworkTimes, "extremote::GetTrackArtworkTimes", (LINGO_EXT_REMOTE_ID, 0x002a), Some(10)),
    ExtRetTrackArtworkTimes => (ExtRetTrackArtworkTimes, ext::RetTrackArtworkTimes, "extremote::RetTrackArtworkTimes", (LINGO_EXT_REMOTE_ID, 0x002b), Some(0)),
    ExtGetShuffle => (ExtGetShuffle, ext::GetShuffle, "extremote::GetShuffle", (LINGO_EXT_REMOTE_ID, 0x002c), Some(0)),
    ExtReturnShuffle => (ExtReturnShuffle, ext::ReturnShuffle, "extremote::ReturnShuffle", (LINGO_EXT_REMOTE_ID, 0x002d), Some(1)),
    ExtSetShuffle => (ExtSetShuffle, ext::SetShuffle, "extremote::SetShuffle", (LINGO_EXT_REMOTE_ID, 0x002e), Some(1)),
    ExtGetRepeat => (ExtGetRepeat, ext::GetRepeat, "extremote::GetRepeat", (LINGO_EXT_REMOTE_ID, 0x002f), Some(0)),
    ExtReturnRepeat => (ExtReturnRepeat, ext::ReturnRepeat, "extremote::ReturnRepeat", (LINGO_EXT_REMOTE_ID, 0x0030), Some(1)),
    ExtSetRepeat => (ExtSetRepeat, ext::SetRepeat, "extremote::SetRepeat", (LINGO_EXT_REMOTE_ID, 0x0031), Some(1)),
    ExtSetDisplayImage => (ExtSetDisplayImage, ext::SetDisplayImage, "extremote::SetDisplayImage", (LINGO_EXT_REMOTE_ID, 0x0032), Some(0)),
    ExtGetMonoDisplayImageLimits => (ExtGetMonoDisplayImageLimits, ext::GetMonoDisplayImageLimits, "extremote::GetMonoDisplayImageLimits", (LINGO_EXT_REMOTE_ID, 0x0033), Some(0)),
    ExtReturnMonoDisplayImageLimits => (ExtReturnMonoDisplayImageLimits, ext::ReturnMonoDisplayImageLimits, "extremote::ReturnMonoDisplayImageLimits", (LINGO_EXT_REMOTE_ID, 0x0034), Some(5)),
    ExtGetNumPlayingTracks => (ExtGetNumPlayingTracks, ext::GetNumPlayingTracks, "extremote::GetNumPlayingTracks", (LINGO_EXT_REMOTE_ID, 0x0035), Some(0)),
    ExtReturnNumPlayingTracks => (ExtReturnNumPlayingTracks, ext::ReturnNumPlayingTracks, "extremote::ReturnNumPlayingTracks", (LINGO_EXT_REMOTE_ID, 0x0036), Some(4)),
    ExtSetCurrentPlayingTrack => (ExtSetCurrentPlayingTrack, ext::SetCurrentPlayingTrack, "extremote::SetCurrentPlayingTrack", (LINGO_EXT_REMOTE_ID, 0x0037), Some(4)),
    ExtSelectSortDbRecord => (ExtSelectSortDbRecord, ext::SelectSortDbRecord, "extremote::SelectSortDBRecord", (LINGO_EXT_REMOTE_ID, 0x0038), Some(6)),
    ExtGetColorDisplayImageLimits => (ExtGetColorDisplayImageLimits, ext::GetColorDisplayImageLimits, "extremote::GetColorDisplayImageLimits", (LINGO_EXT_REMOTE_ID, 0x0039), Some(0)),
    ExtReturnColorDisplayImageLimits => (ExtReturnColorDisplayImageLimits, ext::ReturnColorDisplayImageLimits, "extremote::ReturnColorDisplayImageLimits", (LINGO_EXT_REMOTE_ID, 0x003a), Some(5)),
    ExtResetDbSelectionHierarchy => (ExtResetDbSelectionHierarchy, ext::ResetDbSelectionHierarchy, "extremote::ResetDBSelectionHierarchy", (LINGO_EXT_REMOTE_ID, 0x003b), Some(1)),
    ExtGetDbITunesInfo => (ExtGetDbITunesInfo, ext::GetDbITunesInfo, "extremote::GetDBiTunesInfo", (LINGO_EXT_REMOTE_ID, 0x003c), Some(0)),
    ExtRetDbITunesInfo => (ExtRetDbITunesInfo, ext::RetDbITunesInfo, "extremote::RetDBiTunesInfo", (LINGO_EXT_REMOTE_ID, 0x003d), Some(0)),
    ExtGetUidTrackInfo => (ExtGetUidTrackInfo, ext::GetUidTrackInfo, "extremote::GetUIDTrackInfo", (LINGO_EXT_REMOTE_ID, 0x003e), Some(0)),
    ExtRetUidTrackInfo => (ExtRetUidTrackInfo, ext::RetUidTrackInfo, "extremote::RetUIDTrackInfo", (LINGO_EXT_REMOTE_ID, 0x003f), Some(0)),
    ExtGetDbTrackInfo => (ExtGetDbTrackInfo, ext::GetDbTrackInfo, "extremote::GetDBTrackInfo", (LINGO_EXT_REMOTE_ID, 0x0040), Some(0)),
    ExtRetDbTrackInfo => (ExtRetDbTrackInfo, ext::RetDbTrackInfo, "extremote::RetDBTrackInfo", (LINGO_EXT_REMOTE_ID, 0x0041), Some(0)),
    ExtGetPbTrackInfo => (ExtGetPbTrackInfo, ext::GetPbTrackInfo, "extremote::GetPBTrackInfo", (LINGO_EXT_REMOTE_ID, 0x0042), Some(0)),
    ExtRetPbTrackInfo => (ExtRetPbTrackInfo, ext::RetPbTrackInfo, "extremote::RetPBTrackInfo", (LINGO_EXT_REMOTE_ID, 0x0043), Some(0)),
}

#[derive(Debug, Clone, Default)]
pub struct CommandSerde {
    pub trx_enabled: bool,
}

impl CommandSerde {
    fn handle_cmd_id(&mut self, cmd_id: LingoCmdId) {
        match cmd_id {
            id if id == LingoCmdId::new(LINGO_GENERAL_ID, 0x00) => self.trx_enabled = false,
            id if id == LingoCmdId::new(LINGO_GENERAL_ID, 0x13) => self.trx_enabled = false,
            id if id == LingoCmdId::new(LINGO_GENERAL_ID, 0x38) => self.trx_enabled = true,
            _ => {}
        }
    }

    pub fn marshal_cmd(&mut self, cmd: &Command) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        cmd.id.encode(&mut out)?;
        self.handle_cmd_id(cmd.id);

        if self.trx_enabled {
            if let Some(transaction) = cmd.transaction {
                put_u16(&mut out, transaction.0);
            }
        }

        cmd.payload.encode(&mut out)?;
        Ok(out)
    }

    pub fn unmarshal_cmd(&mut self, packet: &[u8]) -> Result<Command> {
        let (cmd, err) = self.unmarshal_cmd_lossy(packet);
        match err {
            Some(err) => Err(err),
            None => Ok(cmd),
        }
    }

    pub fn unmarshal_cmd_lossy(&mut self, packet: &[u8]) -> (Command, Option<Error>) {
        let (id, offset) = match LingoCmdId::decode(packet) {
            Ok(value) => value,
            Err(err) => {
                return (
                    Command {
                        id: LingoCmdId::new(0, 0),
                        transaction: None,
                        payload: CommandPayload::Unknown(packet.to_vec()),
                    },
                    Some(err),
                );
            }
        };

        self.handle_cmd_id(id);
        let payload_with_optional_trx = &packet[offset..];
        let Some((entry, has_transaction)) =
            lookup(id, payload_with_optional_trx.len(), self.trx_enabled)
        else {
            return (
                Command {
                    id,
                    transaction: None,
                    payload: CommandPayload::Unknown(payload_with_optional_trx.to_vec()),
                },
                Some(Error::UnknownCommand(id)),
            );
        };

        let mut payload_data = payload_with_optional_trx;
        let transaction = if has_transaction {
            if payload_data.len() < 2 {
                return (
                    Command {
                        id,
                        transaction: None,
                        payload: CommandPayload::Unknown(payload_with_optional_trx.to_vec()),
                    },
                    Some(Error::UnexpectedEof),
                );
            }
            let transaction = Transaction(u16::from_be_bytes([payload_data[0], payload_data[1]]));
            payload_data = &payload_data[2..];
            Some(transaction)
        } else {
            None
        };

        match entry.kind.decode(payload_data) {
            Ok(payload) => (
                Command {
                    id,
                    transaction,
                    payload,
                },
                None,
            ),
            Err(err) => (
                Command {
                    id,
                    transaction,
                    payload: CommandPayload::Unknown(payload_data.to_vec()),
                },
                Some(err),
            ),
        }
    }
}

fn lookup(
    id: LingoCmdId,
    payload_size: usize,
    default_trx_enabled: bool,
) -> Option<(&'static RegistryEntry, bool)> {
    let mut candidates = registry().iter().filter(|entry| entry.id == id);
    let first = candidates.next()?;
    let mut all = vec![first];
    all.extend(candidates);

    for entry in &all {
        if let Some(cmd_size) = entry.fixed_len {
            if cmd_size == payload_size {
                return Some((entry, false));
            }
            if payload_size >= 2 && cmd_size == payload_size - 2 {
                return Some((entry, true));
            }
        }
    }

    if all.len() == 1 {
        Some((all[0], default_trx_enabled))
    } else {
        None
    }
}

pub trait CommandReader {
    fn read_command(&mut self) -> Result<Option<Command>>;
}

pub trait CommandWriter {
    fn write_command(&mut self, cmd: Command) -> Result<()>;
}

pub fn build_command(payload: CommandPayload) -> Result<Command> {
    let id = payload.id().ok_or(Error::UnknownPayload)?;
    Ok(Command {
        id,
        transaction: None,
        payload,
    })
}

pub fn respond(req: &Command, writer: &mut impl CommandWriter, payload: CommandPayload) {
    if let Ok(mut cmd) = build_command(payload) {
        cmd.transaction = req.transaction;
        let _ = writer.write_command(cmd);
    }
}

static TRX_COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn trx_reset() {
    TRX_COUNTER.store(0, Ordering::SeqCst);
}

pub fn trx_next() -> Transaction {
    let value = TRX_COUNTER.fetch_add(1, Ordering::SeqCst).wrapping_add(1);
    Transaction(value as u16)
}

pub fn send(writer: &mut impl CommandWriter, payload: CommandPayload) {
    if let Ok(mut cmd) = build_command(payload) {
        cmd.transaction = Some(trx_next());
        let _ = writer.write_command(cmd);
    }
}

#[derive(Debug, Clone, Default)]
pub struct CmdBuffer {
    pub commands: Vec<Command>,
}

impl CommandWriter for CmdBuffer {
    fn write_command(&mut self, cmd: Command) -> Result<()> {
        self.commands.push(cmd);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, CommandPayload, CommandSerde, Transaction};
    use crate::lingo::audio;
    use crate::lingo::general;
    use crate::LingoCmdId;

    #[test]
    fn command_marshal_matches_go_vectors() {
        let mut serde = CommandSerde { trx_enabled: false };
        let cmd = Command {
            id: LingoCmdId::new(0x01, 0x02),
            transaction: None,
            payload: CommandPayload::Unknown(vec![0x00, 0x00, 0x00, 0x03]),
        };
        assert_eq!(
            serde.marshal_cmd(&cmd).unwrap(),
            vec![0x01, 0x02, 0x00, 0x00, 0x00, 0x03]
        );

        let mut serde = CommandSerde { trx_enabled: true };
        let cmd = Command {
            id: LingoCmdId::new(0x01, 0x02),
            transaction: Some(Transaction(0x01)),
            payload: CommandPayload::Unknown(vec![0x00, 0x00, 0x00, 0x03]),
        };
        assert_eq!(
            serde.marshal_cmd(&cmd).unwrap(),
            vec![0x01, 0x02, 0x00, 0x01, 0x00, 0x00, 0x00, 0x03]
        );
    }

    #[test]
    fn known_payload_unmarshal_detects_transaction_by_size() {
        let mut serde = CommandSerde { trx_enabled: true };
        let cmd = serde
            .unmarshal_cmd(&[0x0a, 0x03, 0x03, 0xe7, 0x00, 0x00, 0x1f, 0x40])
            .unwrap();
        assert_eq!(cmd.transaction, Some(Transaction(0x03e7)));
        assert_eq!(
            cmd.payload,
            CommandPayload::AudioRetAccSampleRateCaps(audio::RetAccSampleRateCaps {
                sample_rates: vec![8000],
            })
        );
    }

    #[test]
    fn strict_unknown_returns_error_but_lossy_keeps_payload() {
        let mut serde = CommandSerde::default();
        assert!(serde.unmarshal_cmd(&[0xee, 0x01, 0x00, 0x03]).is_err());
        let (cmd, err) = serde.unmarshal_cmd_lossy(&[0xee, 0x01, 0x00, 0x03]);
        assert!(err.is_some());
        assert_eq!(cmd.id, LingoCmdId::new(0xee, 0x01));
        assert_eq!(cmd.payload, CommandPayload::Unknown(vec![0x00, 0x03]));
    }

    #[test]
    fn idps_start_enables_transactions_for_following_variable_payloads() {
        let mut serde = CommandSerde::default();
        let _ = serde
            .unmarshal_cmd(&[0x00, 0x38])
            .expect("StartIDPS decodes");
        assert!(serde.trx_enabled);

        let cmd = serde
            .unmarshal_cmd(&[0x00, 0x39, 0x00, 0x02, 0x00])
            .unwrap();
        assert_eq!(cmd.transaction, Some(Transaction(2)));
        assert_eq!(
            cmd.payload,
            CommandPayload::GeneralSetFidTokenValues(general::SetFidTokenValues {
                fid_token_values: vec![],
            })
        );
    }
}
