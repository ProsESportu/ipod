use std::io::{self, Read, Write};

use crate::transport::{FrameReader, FrameWriter};
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub id: u8,
    pub link_control: LinkControl,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkControl(pub u8);

impl LinkControl {
    pub const DONE: Self = Self(0x00);
    pub const CONTINUE: Self = Self(0x01);
    pub const MORE_TO_FOLLOW: Self = Self(0x02);
}

impl std::ops::BitOr for LinkControl {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportDir {
    AccIn,
    AccOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReportDef {
    pub id: u8,
    pub len: usize,
    pub dir: ReportDir,
}

impl ReportDef {
    pub const fn max_payload(self) -> usize {
        self.len - 1
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReportDefs(pub Vec<ReportDef>);

impl ReportDefs {
    pub fn pick(&self, payload_size: usize, dir: ReportDir) -> Result<ReportDef> {
        let mut selected = None;
        for def in &self.0 {
            if def.dir == dir {
                selected = Some(*def);
                if def.max_payload() >= payload_size {
                    break;
                }
            }
        }
        selected.ok_or_else(|| Error::InvalidData("no matching report found".to_string()))
    }

    pub fn find(&self, id: u8) -> Result<ReportDef> {
        self.0
            .iter()
            .copied()
            .find(|def| def.id == id)
            .ok_or_else(|| Error::InvalidData(format!("report id not found: {id:#04x}")))
    }
}

pub fn default_report_defs() -> ReportDefs {
    ReportDefs(vec![
        ReportDef {
            id: 0x01,
            len: 12,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x02,
            len: 14,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x03,
            len: 20,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x04,
            len: 63,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x05,
            len: 8,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x06,
            len: 10,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x07,
            len: 14,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x08,
            len: 20,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x09,
            len: 63,
            dir: ReportDir::AccIn,
        },
    ])
}

pub fn legacy_report_defs() -> ReportDefs {
    ReportDefs(vec![
        ReportDef {
            id: 0x01,
            len: 5,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x02,
            len: 9,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x03,
            len: 13,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x04,
            len: 17,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x05,
            len: 25,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x06,
            len: 49,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x07,
            len: 95,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x08,
            len: 193,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x09,
            len: 257,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x0a,
            len: 385,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x0b,
            len: 513,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x0c,
            len: 767,
            dir: ReportDir::AccIn,
        },
        ReportDef {
            id: 0x0d,
            len: 5,
            dir: ReportDir::AccOut,
        },
        ReportDef {
            id: 0x0e,
            len: 9,
            dir: ReportDir::AccOut,
        },
        ReportDef {
            id: 0x0f,
            len: 13,
            dir: ReportDir::AccOut,
        },
        ReportDef {
            id: 0x10,
            len: 17,
            dir: ReportDir::AccOut,
        },
        ReportDef {
            id: 0x11,
            len: 25,
            dir: ReportDir::AccOut,
        },
        ReportDef {
            id: 0x12,
            len: 49,
            dir: ReportDir::AccOut,
        },
        ReportDef {
            id: 0x13,
            len: 95,
            dir: ReportDir::AccOut,
        },
        ReportDef {
            id: 0x14,
            len: 193,
            dir: ReportDir::AccOut,
        },
        ReportDef {
            id: 0x15,
            len: 255,
            dir: ReportDir::AccOut,
        },
    ])
}

pub trait ReportReader {
    fn read_report(&mut self) -> Result<Option<Report>>;
}

pub trait ReportWriter {
    fn write_report(&mut self, report: &Report) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct SingleReport {
    data: Vec<u8>,
    read: bool,
}

impl SingleReport {
    pub fn new(data: impl Into<Vec<u8>>) -> Self {
        Self {
            data: data.into(),
            read: false,
        }
    }
}

impl ReportReader for SingleReport {
    fn read_report(&mut self) -> Result<Option<Report>> {
        if self.read {
            return Ok(None);
        }
        self.read = true;
        if self.data.len() < 2 {
            return Err(Error::UnexpectedEof);
        }
        Ok(Some(Report {
            id: self.data[0],
            link_control: LinkControl(self.data[1]),
            data: self.data[2..].to_vec(),
        }))
    }
}

pub struct RawReportReader<R> {
    reader: R,
    buf: Vec<u8>,
}

impl<R: Read> RawReportReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: vec![0; 1024],
        }
    }
}

impl<R: Read> ReportReader for RawReportReader<R> {
    fn read_report(&mut self) -> Result<Option<Report>> {
        let n = self.reader.read(&mut self.buf)?;
        if n == 0 {
            return Ok(None);
        }
        if n < 3 {
            return Err(Error::Io(io::Error::new(
                io::ErrorKind::WouldBlock,
                "short HID report",
            )));
        }
        Ok(Some(Report {
            id: self.buf[0],
            link_control: LinkControl(self.buf[1]),
            data: self.buf[2..n].to_vec(),
        }))
    }
}

pub struct RawReportWriter<W> {
    writer: W,
}

impl<W: Write> RawReportWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> ReportWriter for RawReportWriter<W> {
    fn write_report(&mut self, report: &Report) -> Result<()> {
        self.writer.write_all(&[report.id, report.link_control.0])?;
        self.writer.write_all(&report.data)?;
        Ok(())
    }
}

pub struct Encoder<W> {
    report_defs: ReportDefs,
    writer: W,
}

impl<W: ReportWriter> Encoder<W> {
    pub fn new(writer: W, defs: ReportDefs) -> Self {
        Self {
            report_defs: defs,
            writer,
        }
    }
}

impl<W: ReportWriter> FrameWriter for Encoder<W> {
    fn write_frame(&mut self, data: &[u8]) -> Result<()> {
        let mut offset = 0usize;
        let mut bytes_left = data.len();
        while bytes_left > 0 {
            let report_def = self.report_defs.pick(bytes_left, ReportDir::AccIn)?;
            let mut payload_len = bytes_left;
            let mut link_control = LinkControl::DONE;

            if bytes_left > report_def.max_payload() {
                payload_len = report_def.max_payload();
                link_control = if offset == 0 {
                    LinkControl::MORE_TO_FOLLOW
                } else {
                    LinkControl::CONTINUE | LinkControl::MORE_TO_FOLLOW
                };
            } else if offset > 0 {
                link_control = LinkControl::CONTINUE;
            }

            let mut report_data = vec![0; report_def.max_payload()];
            report_data[..payload_len].copy_from_slice(&data[offset..offset + payload_len]);
            self.writer.write_report(&Report {
                id: report_def.id,
                link_control,
                data: report_data,
            })?;

            bytes_left -= payload_len;
            offset += payload_len;
        }
        Ok(())
    }
}

pub struct Decoder<R> {
    report_defs: ReportDefs,
    reader: R,
    buf: Vec<u8>,
}

impl<R: ReportReader> Decoder<R> {
    pub fn new(reader: R, defs: ReportDefs) -> Self {
        Self {
            report_defs: defs,
            reader,
            buf: Vec::new(),
        }
    }
}

impl<R: ReportReader> FrameReader for Decoder<R> {
    fn read_frame(&mut self) -> Result<Option<Vec<u8>>> {
        self.buf.clear();
        loop {
            let Some(report) = self.reader.read_report()? else {
                return Ok(None);
            };
            let report_def = self.report_defs.find(report.id)?;
            let n = report.data.len().min(report_def.max_payload());
            let report_data = &report.data[..n];
            match report.link_control {
                LinkControl::DONE => {
                    self.buf.clear();
                    self.buf.extend_from_slice(report_data);
                    return Ok(Some(self.buf.clone()));
                }
                LinkControl::MORE_TO_FOLLOW => {
                    self.buf.clear();
                    self.buf.extend_from_slice(report_data);
                }
                link if link == (LinkControl::CONTINUE | LinkControl::MORE_TO_FOLLOW) => {
                    self.buf.extend_from_slice(report_data);
                }
                LinkControl::CONTINUE => {
                    self.buf.extend_from_slice(report_data);
                    return Ok(Some(self.buf.clone()));
                }
                other => return Error::invalid(format!("unknown link control: {:#04x}", other.0)),
            }
        }
    }
}

pub struct Transport<R, W> {
    decoder: Decoder<R>,
    encoder: Encoder<W>,
}

impl<R: ReportReader, W: ReportWriter> Transport<R, W> {
    pub fn new(reader: R, writer: W, defs: ReportDefs) -> Self {
        Self {
            decoder: Decoder::new(reader, defs.clone()),
            encoder: Encoder::new(writer, defs),
        }
    }
}

impl<R: ReportReader, W: ReportWriter> FrameReader for Transport<R, W> {
    fn read_frame(&mut self) -> Result<Option<Vec<u8>>> {
        self.decoder.read_frame()
    }
}

impl<R: ReportReader, W: ReportWriter> FrameWriter for Transport<R, W> {
    fn write_frame(&mut self, data: &[u8]) -> Result<()> {
        self.encoder.write_frame(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestReportWriter {
        reports: Vec<Report>,
    }

    impl ReportWriter for TestReportWriter {
        fn write_report(&mut self, report: &Report) -> Result<()> {
            self.reports.push(report.clone());
            Ok(())
        }
    }

    struct TestReportReader {
        reports: Vec<Report>,
        offset: usize,
    }

    impl ReportReader for TestReportReader {
        fn read_report(&mut self) -> Result<Option<Report>> {
            if self.offset >= self.reports.len() {
                return Ok(None);
            }
            let report = self.reports[self.offset].clone();
            self.offset += 1;
            Ok(Some(report))
        }
    }

    fn defs1() -> ReportDefs {
        ReportDefs(vec![ReportDef {
            id: 0x01,
            len: 2,
            dir: ReportDir::AccIn,
        }])
    }

    fn defs2() -> ReportDefs {
        ReportDefs(vec![
            ReportDef {
                id: 0x01,
                len: 2,
                dir: ReportDir::AccIn,
            },
            ReportDef {
                id: 0x02,
                len: 3,
                dir: ReportDir::AccIn,
            },
        ])
    }

    fn defs3() -> ReportDefs {
        ReportDefs(vec![ReportDef {
            id: 0x01,
            len: 4,
            dir: ReportDir::AccIn,
        }])
    }

    #[test]
    fn encoder_matches_go_vectors() {
        let cases = vec![
            (
                defs1(),
                vec![0x01],
                vec![Report {
                    id: 0x01,
                    link_control: LinkControl::DONE,
                    data: vec![0x01],
                }],
            ),
            (
                defs1(),
                vec![0x01, 0x02],
                vec![
                    Report {
                        id: 0x01,
                        link_control: LinkControl::MORE_TO_FOLLOW,
                        data: vec![0x01],
                    },
                    Report {
                        id: 0x01,
                        link_control: LinkControl::CONTINUE,
                        data: vec![0x02],
                    },
                ],
            ),
            (
                defs2(),
                vec![0x01, 0x02],
                vec![Report {
                    id: 0x02,
                    link_control: LinkControl::DONE,
                    data: vec![0x01, 0x02],
                }],
            ),
            (
                defs3(),
                vec![0x01, 0x02],
                vec![Report {
                    id: 0x01,
                    link_control: LinkControl::DONE,
                    data: vec![0x01, 0x02, 0x00],
                }],
            ),
        ];

        for (defs, data, want) in cases {
            let writer = TestReportWriter::default();
            let mut encoder = Encoder::new(writer, defs);
            encoder.write_frame(&data).unwrap();
            assert_eq!(encoder.writer.reports, want);
        }
    }

    #[test]
    fn decoder_matches_go_vectors() {
        let reports = vec![
            Report {
                id: 0x01,
                link_control: LinkControl::MORE_TO_FOLLOW,
                data: vec![0x01],
            },
            Report {
                id: 0x01,
                link_control: LinkControl::CONTINUE | LinkControl::MORE_TO_FOLLOW,
                data: vec![0x02],
            },
            Report {
                id: 0x01,
                link_control: LinkControl::CONTINUE,
                data: vec![0x03],
            },
        ];
        let reader = TestReportReader { reports, offset: 0 };
        let mut decoder = Decoder::new(reader, defs1());
        assert_eq!(
            decoder.read_frame().unwrap().unwrap(),
            vec![0x01, 0x02, 0x03]
        );
    }
}
