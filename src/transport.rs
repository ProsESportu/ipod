use crate::Result;

pub trait FrameReader {
    fn read_frame(&mut self) -> Result<Option<Vec<u8>>>;
}

pub trait FrameWriter {
    fn write_frame(&mut self, data: &[u8]) -> Result<()>;
}

pub trait FrameReadWriter: FrameReader + FrameWriter {}

impl<T> FrameReadWriter for T where T: FrameReader + FrameWriter {}
