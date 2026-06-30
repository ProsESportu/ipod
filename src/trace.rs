use std::collections::VecDeque;
use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};

use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    In,
    Out,
}

impl Dir {
    pub fn as_symbol(self) -> char {
        match self {
            Self::In => '<',
            Self::Out => '>',
        }
    }

    pub fn from_symbol(value: u8) -> Result<Self> {
        match value {
            b'<' => Ok(Self::In),
            b'>' => Ok(Self::Out),
            _ => Error::invalid(format!(
                "trace dir unmarshal: unknown symbol '{}'",
                value as char
            )),
        }
    }
}

impl fmt::Display for Dir {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::In => "<",
            Self::Out => ">",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msg {
    pub dir: Dir,
    pub ts: u64,
    pub data: Vec<u8>,
}

impl Msg {
    pub fn marshal_text(&self) -> Result<Vec<u8>> {
        if self.data.is_empty() {
            return Error::invalid("trace marshal: no data");
        }
        let mut out = Vec::with_capacity(2 + self.data.len() * 3);
        out.push(self.dir.as_symbol() as u8);
        out.push(b' ');
        for (index, byte) in self.data.iter().enumerate() {
            if index > 0 {
                out.push(b' ');
            }
            write!(&mut out, "{byte:02X}").expect("writing to Vec cannot fail");
        }
        Ok(out)
    }

    pub fn unmarshal_text(text: &[u8]) -> Result<Self> {
        if text.len() < 4 {
            return Error::invalid("trace unmarshal: short msg");
        }
        let dir = Dir::from_symbol(text[0])?;
        if text[1] != b' ' {
            return Error::invalid("trace unmarshal: missing separator");
        }
        let mut compact = Vec::new();
        for byte in &text[2..] {
            if !byte.is_ascii_whitespace() {
                compact.push(*byte);
            }
        }
        if compact.is_empty() || compact.len() % 2 != 0 {
            return Error::invalid("trace unmarshal: bad data");
        }
        let mut data = Vec::with_capacity(compact.len() / 2);
        for chunk in compact.chunks_exact(2) {
            let hex = std::str::from_utf8(chunk)
                .map_err(|_| Error::InvalidData("trace unmarshal: bad data".to_string()))?;
            let value = u8::from_str_radix(hex, 16)
                .map_err(|_| Error::InvalidData("trace unmarshal: bad data".to_string()))?;
            data.push(value);
        }
        Ok(Self { dir, ts: 0, data })
    }
}

pub struct Reader<R> {
    reader: BufReader<R>,
    line: String,
    ts: u64,
}

impl<R: Read> Reader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line: String::new(),
            ts: 0,
        }
    }

    pub fn read_msg(&mut self) -> Result<Option<Msg>> {
        loop {
            self.line.clear();
            let n = self.reader.read_line(&mut self.line)?;
            if n == 0 {
                return Ok(None);
            }
            let text = self.line.trim_end_matches(['\r', '\n']);
            if text.is_empty() {
                continue;
            }
            let mut msg = Msg::unmarshal_text(text.as_bytes())?;
            msg.ts = self.ts;
            self.ts += 1;
            return Ok(Some(msg));
        }
    }
}

pub struct Writer<W> {
    writer: W,
}

impl<W: Write> Writer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn write_msg(&mut self, msg: &Msg) -> Result<()> {
        let mut text = msg.marshal_text()?;
        text.push(b'\n');
        self.writer.write_all(&text)?;
        Ok(())
    }
}

pub struct TraceDirReader<R> {
    reader: Reader<R>,
    dir: Dir,
}

impl<R: Read> TraceDirReader<R> {
    pub fn new(reader: Reader<R>, dir: Dir) -> Self {
        Self { reader, dir }
    }
}

impl<R: Read> Read for TraceDirReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            let msg = self
                .reader
                .read_msg()
                .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
            let Some(msg) = msg else {
                return Ok(0);
            };
            if msg.dir == self.dir {
                let n = buf.len().min(msg.data.len());
                buf[..n].copy_from_slice(&msg.data[..n]);
                return Ok(n);
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Queue {
    items: VecDeque<Msg>,
}

impl Queue {
    pub fn enqueue(&mut self, msg: Msg) {
        self.items.push_back(msg);
    }

    pub fn head(&self) -> Option<&Msg> {
        self.items.front()
    }

    pub fn dequeue(&mut self) -> Option<Msg> {
        self.items.pop_front()
    }

    pub fn dequeue_dir(&mut self, dir: Dir) -> Option<Msg> {
        let index = self.items.iter().position(|msg| msg.dir == dir)?;
        self.items.remove(index)
    }
}

pub struct QueueDirReader<'a> {
    queue: &'a mut Queue,
    dir: Dir,
}

impl<'a> QueueDirReader<'a> {
    pub fn new(queue: &'a mut Queue, dir: Dir) -> Self {
        Self { queue, dir }
    }
}

impl Read for QueueDirReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let Some(msg) = self.queue.dequeue_dir(self.dir) else {
            return Ok(0);
        };
        let n = buf.len().min(msg.data.len());
        buf[..n].copy_from_slice(&msg.data[..n]);
        Ok(n)
    }
}

pub struct TracingReader<R, W> {
    reader: R,
    writer: Writer<W>,
}

impl<R: Read, W: Write> TracingReader<R, W> {
    pub fn new(reader: R, trace_writer: W) -> Self {
        Self {
            reader,
            writer: Writer::new(trace_writer),
        }
    }
}

impl<R: Read, W: Write> Read for TracingReader<R, W> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.reader.read(buf)?;
        if n > 0 {
            self.writer
                .write_msg(&Msg {
                    dir: Dir::In,
                    ts: 0,
                    data: buf[..n].to_vec(),
                })
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        }
        Ok(n)
    }
}

pub struct TracingWriter<W, TW> {
    writer: W,
    trace_writer: Writer<TW>,
}

impl<W: Write, TW: Write> TracingWriter<W, TW> {
    pub fn new(writer: W, trace_writer: TW) -> Self {
        Self {
            writer,
            trace_writer: Writer::new(trace_writer),
        }
    }
}

impl<W: Write, TW: Write> Write for TracingWriter<W, TW> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let n = self.writer.write(buf)?;
        if n > 0 {
            self.trace_writer
                .write_msg(&Msg {
                    dir: Dir::Out,
                    ts: 0,
                    data: buf[..n].to_vec(),
                })
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        }
        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::*;

    #[test]
    fn write_read_roundtrip() {
        let msg = Msg {
            dir: Dir::Out,
            ts: 0,
            data: vec![0x01, 0x02, 0x03],
        };
        let mut buf = Vec::new();
        Writer::new(&mut buf).write_msg(&msg).unwrap();
        assert_eq!(buf, b"> 01 02 03\n");

        let mut reader = Reader::new(buf.as_slice());
        assert_eq!(reader.read_msg().unwrap().unwrap(), msg);
    }

    #[test]
    fn tracer_records_writes_and_reads() {
        let mut trace = Vec::new();
        let mut out = Vec::new();
        {
            let mut writer = TracingWriter::new(&mut out, &mut trace);
            writer.write_all(b"ab").unwrap();
        }
        assert_eq!(out, b"ab");
        assert_eq!(trace, b"> 61 62\n");

        let mut trace = Vec::new();
        let mut reader = TracingReader::new(&b"ab"[..], &mut trace);
        let mut data = Vec::new();
        reader.read_to_end(&mut data).unwrap();
        assert_eq!(data, b"ab");
        assert_eq!(trace, b"< 61 62\n");
    }

    #[test]
    fn queue_dequeue_by_dir() {
        let mut queue = Queue::default();
        queue.enqueue(Msg {
            dir: Dir::In,
            ts: 0,
            data: vec![1],
        });
        queue.enqueue(Msg {
            dir: Dir::Out,
            ts: 1,
            data: vec![2],
        });
        queue.enqueue(Msg {
            dir: Dir::In,
            ts: 2,
            data: vec![3],
        });
        assert_eq!(queue.dequeue_dir(Dir::Out).unwrap().data, vec![2]);
        assert_eq!(queue.dequeue().unwrap().data, vec![1]);
        assert_eq!(queue.dequeue().unwrap().data, vec![3]);
    }
}
