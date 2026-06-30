use crate::{Error, Result};

#[derive(Debug, Clone)]
pub(crate) struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub(crate) fn remaining(&self) -> &'a [u8] {
        &self.data[self.pos..]
    }

    pub(crate) fn remaining_len(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.remaining_len() == 0
    }

    pub(crate) fn read_u8(&mut self) -> Result<u8> {
        if self.pos >= self.data.len() {
            return Err(Error::UnexpectedEof);
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }

    pub(crate) fn read_i16(&mut self) -> Result<i16> {
        Ok(i16::from_be_bytes(self.read_array()?))
    }

    pub(crate) fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_be_bytes(self.read_array()?))
    }

    pub(crate) fn read_i32(&mut self) -> Result<i32> {
        Ok(i32::from_be_bytes(self.read_array()?))
    }

    pub(crate) fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_be_bytes(self.read_array()?))
    }

    pub(crate) fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.read_array()?))
    }

    pub(crate) fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.remaining_len() < N {
            return Err(Error::UnexpectedEof);
        }
        let mut value = [0; N];
        value.copy_from_slice(&self.data[self.pos..self.pos + N]);
        self.pos += N;
        Ok(value)
    }

    pub(crate) fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.remaining_len() < len {
            return Err(Error::UnexpectedEof);
        }
        let value = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(value)
    }

    pub(crate) fn read_rest(&mut self) -> &'a [u8] {
        let value = self.remaining();
        self.pos = self.data.len();
        value
    }

    pub(crate) fn finish(&self) -> Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Error::invalid(format!("{} trailing bytes", self.remaining_len()))
        }
    }
}

pub(crate) trait WireEncode {
    fn encode(&self, out: &mut Vec<u8>) -> Result<()>;
}

pub(crate) trait WireDecode: Sized {
    fn decode(data: &[u8]) -> Result<Self>;
}

pub(crate) fn put_bool(out: &mut Vec<u8>, value: bool) {
    out.push(if value { 1 } else { 0 });
}

pub(crate) fn put_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn put_i16(out: &mut Vec<u8>, value: i16) {
    out.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn put_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_be_bytes());
}

pub(crate) fn put_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

impl WireEncode for () {
    fn encode(&self, _out: &mut Vec<u8>) -> Result<()> {
        Ok(())
    }
}
