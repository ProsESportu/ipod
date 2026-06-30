use crate::crc::checksum;
use crate::{Error, Result};

pub const PACKET_START_BYTE: u8 = 0x55;
const LARGE_PACKET_MIN_LEN: usize = 256;

#[derive(Debug, Clone)]
pub struct PacketReader {
    frame: Vec<u8>,
    offset: usize,
}

impl PacketReader {
    pub fn new(frame: impl Into<Vec<u8>>) -> Self {
        Self {
            frame: frame.into(),
            offset: 0,
        }
    }

    pub fn read_packet(&mut self) -> Result<Option<Vec<u8>>> {
        let Some(relative_start) = self.frame[self.offset..]
            .iter()
            .position(|value| *value == PACKET_START_BYTE)
        else {
            self.offset = self.frame.len();
            return Ok(None);
        };

        self.offset += relative_start + 1;
        let (packet_len, payload) = parse_packet(&self.frame[self.offset..])?;
        self.offset += packet_len;
        Ok(Some(payload.to_vec()))
    }
}

fn parse_header(data: &[u8]) -> Result<(usize, usize)> {
    if data.len() < 3 {
        return Err(Error::UnexpectedEof);
    }

    if data[0] == 0x00 {
        Ok((3, u16::from_be_bytes([data[1], data[2]]) as usize))
    } else {
        Ok((1, data[0] as usize))
    }
}

fn parse_packet(data: &[u8]) -> Result<(usize, &[u8])> {
    let (payload_offset, payload_len) = parse_header(data)?;
    let packet_len = payload_offset + payload_len + 1;
    if data.len() < packet_len {
        return Err(Error::UnexpectedEof);
    }
    let packet = &data[..packet_len];
    if checksum(packet) != 0x00 {
        return Err(Error::InvalidChecksum);
    }
    Ok((
        packet_len,
        &packet[payload_offset..payload_offset + payload_len],
    ))
}

#[derive(Debug, Clone)]
pub struct PacketWriter {
    frame: Vec<u8>,
}

impl PacketWriter {
    pub fn new() -> Self {
        Self {
            frame: Vec::with_capacity(512),
        }
    }

    pub fn write_packet(&mut self, payload: &[u8]) -> Result<()> {
        if payload.is_empty() {
            return Error::invalid("packet encode: empty packet");
        }

        self.frame.push(PACKET_START_BYTE);
        let packet_start = self.frame.len();

        if payload.len() >= LARGE_PACKET_MIN_LEN {
            if payload.len() > u16::MAX as usize {
                return Error::invalid("packet encode: payload too large");
            }
            self.frame.push(0x00);
            self.frame
                .extend_from_slice(&(payload.len() as u16).to_be_bytes());
        } else {
            self.frame.push(payload.len() as u8);
        }

        self.frame.extend_from_slice(payload);
        let crc = checksum(&self.frame[packet_start..]);
        self.frame.push(crc);
        Ok(())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.frame
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.frame
    }
}

impl Default for PacketWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{PacketReader, PacketWriter};

    #[test]
    fn packet_writer_matches_go_vectors() {
        let large_data = vec![0xee; 255];

        let tests: &[(&str, &[u8], Option<Vec<u8>>)] = &[
            ("no-data", &[], None),
            (
                "with-data",
                &[0x01, 0x02, 0xfd],
                Some(vec![0x55, 0x03, 0x01, 0x02, 0xfd, 0xfd]),
            ),
        ];

        for (_name, data, want) in tests {
            let mut writer = PacketWriter::new();
            let got = writer.write_packet(data);
            match want {
                Some(want) => {
                    got.unwrap();
                    assert_eq!(writer.bytes(), want.as_slice());
                }
                None => assert!(got.is_err()),
            }
        }

        let mut payload = vec![0x01, 0x02];
        payload.extend_from_slice(&large_data);
        let mut want = vec![0x55, 0x00, 0x01, 0x01, 0x01, 0x02];
        want.extend_from_slice(&large_data);
        want.push(0xe9);

        let mut writer = PacketWriter::new();
        writer.write_packet(&payload).unwrap();
        assert_eq!(writer.bytes(), want.as_slice());
    }

    #[test]
    fn packet_reader_matches_go_vectors() {
        let large_data = vec![0xee; 255];
        let mut large_frame = vec![0x55, 0x00, 0x01, 0x01, 0x01, 0x02];
        large_frame.extend_from_slice(&large_data);
        large_frame.push(0xe9);

        let tests: Vec<(&str, Vec<u8>, Option<Vec<u8>>)> = vec![
            (
                "no-data",
                vec![0x55, 0x02, 0x01, 0x02, 256u16.wrapping_sub(0x05) as u8],
                Some(vec![0x01, 0x02]),
            ),
            (
                "with-data",
                vec![0x55, 0x03, 0x01, 0x02, 0xfd, 0xfd],
                Some(vec![0x01, 0x02, 0xfd]),
            ),
            ("bad-crc", vec![0x55, 0x03, 0x01, 0x02, 0xfd, 0x22], None),
            (
                "wrong-start-byte",
                vec![0xff, 0x03, 0x01, 0x02, 0xfd, 0xfd],
                None,
            ),
            ("large-with-data", large_frame, {
                let mut payload = vec![0x01, 0x02];
                payload.extend_from_slice(&large_data);
                Some(payload)
            }),
            ("short-packet", vec![0x55], None),
        ];

        for (_name, frame, want) in tests {
            let mut reader = PacketReader::new(frame);
            let got = reader.read_packet();
            match want {
                Some(want) => assert_eq!(got.unwrap().unwrap(), want),
                None => assert!(got.is_err() || got.unwrap().is_none()),
            }
        }
    }

    #[test]
    fn packet_roundtrip() {
        let packets: &[&[u8]] = &[b"packet1", b"_packet2"];
        let mut writer = PacketWriter::new();
        for packet in packets {
            writer.write_packet(packet).unwrap();
        }

        let mut reader = PacketReader::new(writer.into_bytes());
        for packet in packets {
            assert_eq!(reader.read_packet().unwrap().unwrap(), *packet);
        }
        assert!(reader.read_packet().unwrap().is_none());
    }

    #[test]
    fn payload_len_256_uses_large_packet_header() {
        let payload = vec![0xaa; 256];
        let mut writer = PacketWriter::new();
        writer.write_packet(&payload).unwrap();
        assert_eq!(&writer.bytes()[..4], &[0x55, 0x00, 0x01, 0x00]);
    }
}
