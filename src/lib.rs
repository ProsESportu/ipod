//! Classic iPod Accessory Protocol helpers.

mod codec;
pub mod command;
pub mod crc;
mod error;
pub mod hid;
pub mod lingo;
pub mod packet;
pub mod trace;
pub mod transport;
pub mod util;

pub use command::{
    build_command, respond, send, trx_next, trx_reset, CmdBuffer, Command, CommandPayload,
    CommandReader, CommandSerde, CommandWriter, Transaction, UnknownPayload,
};
pub use crc::{checksum, Crc8};
pub use error::{Error, Result};
pub use lingo::{
    cmd_id_len, dump_lingos, LingoCmdId, LINGO_DIGITAL_AUDIO_ID, LINGO_DISPLAY_REMOTE_ID,
    LINGO_EQ_ID, LINGO_EXT_REMOTE_ID, LINGO_GENERAL_ID, LINGO_RF_TUNER_ID, LINGO_SIMPLE_REMOTE_ID,
    LINGO_SPORTS_ID, LINGO_STORAGE_ID, LINGO_USB_HOST_ID,
};
pub use packet::{PacketReader, PacketWriter, PACKET_START_BYTE};
pub use transport::{FrameReadWriter, FrameReader, FrameWriter};
