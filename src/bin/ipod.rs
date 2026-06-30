use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::unix::fs::FileTypeExt;
use std::path::Path;
use std::thread;
use std::time::Duration;

use ipod::hid::{self, RawReportReader, RawReportWriter, ReportReader, ReportWriter};
use ipod::lingo::audio::{self, DeviceAudio};
use ipod::lingo::display_remote::{self, DeviceDisplayRemote};
use ipod::lingo::ext_remote::{self, DeviceExtRemote};
use ipod::lingo::general::{self, DeviceGeneral, GeneralHandlerState, UiMode};
use ipod::trace::{self, Dir, Queue, QueueDirReader, TraceDirReader};
use ipod::{
    dump_lingos, CmdBuffer, Command, CommandPayload, CommandSerde, FrameReadWriter, FrameReader,
    LINGO_DIGITAL_AUDIO_ID, LINGO_DISPLAY_REMOTE_ID, LINGO_EXT_REMOTE_ID, LINGO_GENERAL_ID,
    LINGO_SIMPLE_REMOTE_ID,
};

#[derive(Debug, Default)]
struct DevGeneral {
    ui_mode: UiMode,
    tokens: Vec<general::FidTokenValue>,
}

impl DeviceGeneral for DevGeneral {
    fn ui_mode(&self) -> UiMode {
        self.ui_mode
    }

    fn set_ui_mode(&mut self, mode: UiMode) {
        self.ui_mode = mode;
    }

    fn name(&self) -> String {
        "ipod-gadget".to_string()
    }

    fn software_version(&self) -> (u8, u8, u8) {
        (7, 1, 2)
    }

    fn serial_num(&self) -> String {
        "abcd1234".to_string()
    }

    fn lingo_protocol_version(&self, lingo: u8) -> (u8, u8) {
        match lingo {
            LINGO_GENERAL_ID => (1, 9),
            LINGO_DISPLAY_REMOTE_ID => (1, 5),
            LINGO_EXT_REMOTE_ID => (1, 12),
            LINGO_DIGITAL_AUDIO_ID => (1, 2),
            _ => (1, 1),
        }
    }

    fn lingo_options(&self, lingo: u8) -> u64 {
        match lingo {
            LINGO_GENERAL_ID => 0x0000_0006_3def_73ff,
            _ => 0,
        }
    }

    fn pref_setting_id(&self, _class_id: u8) -> u8 {
        0
    }

    fn set_pref_setting_id(&mut self, _class_id: u8, _setting_id: u8, _restore_on_exit: bool) {}

    fn start_idps(&mut self) {
        self.tokens.clear();
    }

    fn end_idps(&mut self, _status: general::AccEndIdpsStatus) {
        eprintln!("Tokens:");
        for token in &self.tokens {
            eprintln!("* Token: {:?}", token.token);
        }
    }

    fn set_token(&mut self, token: general::FidTokenValue) -> ipod::Result<()> {
        self.tokens.push(token);
        Ok(())
    }

    fn acc_auth_cert(&mut self, cert: &[u8]) {
        eprintln!("cert: {} bytes", cert.len());
    }

    fn set_event_notification_mask(&mut self, _mask: u64) {}

    fn event_notification_mask(&self) -> u64 {
        0
    }

    fn supported_event_notification_mask(&self) -> u64 {
        0
    }

    fn cancel_command(&mut self, _lingo: u8, _cmd: u16, _transaction: u16) {}

    fn max_payload(&self) -> u16 {
        u16::MAX
    }
}

#[derive(Debug, Default)]
struct DevDisplayRemote;
impl DeviceDisplayRemote for DevDisplayRemote {}

#[derive(Debug, Default)]
struct DevExtRemote;
impl DeviceExtRemote for DevExtRemote {}

#[derive(Debug, Default)]
struct DevAudio;
impl DeviceAudio for DevAudio {}

#[derive(Debug)]
struct AppState {
    general: DevGeneral,
    general_handler: GeneralHandlerState,
    display: DevDisplayRemote,
    ext: DevExtRemote,
    audio: DevAudio,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            general: DevGeneral::default(),
            general_handler: GeneralHandlerState::default(),
            display: DevDisplayRemote,
            ext: DevExtRemote,
            audio: DevAudio,
        }
    }
}

fn open_device(path: &Path) -> io::Result<File> {
    let file = OpenOptions::new().read(true).write(true).open(path)?;
    let metadata = file.metadata()?;
    if !metadata.file_type().is_char_device() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a char device",
        ));
    }
    Ok(file)
}

fn process_frames<T: FrameReadWriter>(transport: &mut T) -> ipod::Result<()> {
    let mut serde = CommandSerde::default();
    let mut app = AppState::default();

    while let Some(in_frame) = transport.read_frame()? {
        log_bytes("<< FRAME", &in_frame);
        let mut packet_reader = ipod::PacketReader::new(in_frame);
        let mut in_cmd_buf = CmdBuffer::default();
        while let Some(in_packet) = packet_reader.read_packet()? {
            log_bytes("<< PACKET", &in_packet);
            let (cmd, err) = serde.unmarshal_cmd_lossy(&in_packet);
            if let Some(err) = err {
                eprintln!("<< CMD error: {err}");
            }
            log_command("<< CMD", &cmd);
            in_cmd_buf.commands.push(cmd);
        }

        let mut out_cmd_buf = CmdBuffer::default();
        for command in &in_cmd_buf.commands {
            handle_packet(&mut out_cmd_buf, command, &mut app)?;
        }

        for out_cmd in &out_cmd_buf.commands {
            log_command(">> CMD", out_cmd);
            let out_packet = serde.marshal_cmd(out_cmd)?;
            log_bytes(">> PACKET", &out_packet);
            let mut packet_writer = ipod::PacketWriter::new();
            packet_writer.write_packet(&out_packet)?;
            log_bytes(">> FRAME", packet_writer.bytes());
            transport.write_frame(packet_writer.bytes())?;
        }
    }

    eprintln!("EOF");
    Ok(())
}

fn handle_packet(writer: &mut CmdBuffer, cmd: &Command, app: &mut AppState) -> ipod::Result<()> {
    match cmd.id.lingo_id() {
        LINGO_GENERAL_ID => {
            if let CommandPayload::GeneralRetDevAuthenticationInfo(auth) = &cmd.payload {
                if (auth.major >= 2 && auth.cert_current_section >= auth.cert_max_section)
                    || auth.major < 2
                {
                    audio::start(writer);
                }
            }
            general::handle_general(cmd, writer, &mut app.general, &mut app.general_handler)?;
        }
        LINGO_SIMPLE_REMOTE_ID => eprintln!("Lingo SimpleRemote is not supported yet"),
        LINGO_DISPLAY_REMOTE_ID => {
            display_remote::handle_display_remote(cmd, writer, &mut app.display)?;
        }
        LINGO_EXT_REMOTE_ID => {
            ext_remote::handle_ext_remote(cmd, writer, &mut app.ext)?;
        }
        LINGO_DIGITAL_AUDIO_ID => {
            audio::handle_audio(cmd, writer, &mut app.audio)?;
        }
        _ => {}
    }
    Ok(())
}

fn dump_trace<R: Read>(reader: trace::Reader<R>, defs: hid::ReportDefs) -> ipod::Result<()> {
    let mut reader = reader;
    let mut queue = Queue::default();
    while let Some(msg) = reader.read_msg()? {
        queue.enqueue(msg);
    }

    let mut serde = CommandSerde::default();
    while let Some(head) = queue.head().cloned() {
        let dir = head.dir;
        let qdr = QueueDirReader::new(&mut queue, dir);
        let report_reader = RawReportReader::new(qdr);
        let mut decoder = hid::Decoder::new(report_reader, defs.clone());
        let Some(frame) = decoder.read_frame()? else {
            break;
        };
        log_bytes(&dir_prefix(dir, "FRAME"), &frame);

        let mut packet_reader = ipod::PacketReader::new(frame);
        while let Some(packet) = packet_reader.read_packet()? {
            log_bytes(&dir_prefix(dir, "PACKET"), &packet);
            let (cmd, err) = serde.unmarshal_cmd_lossy(&packet);
            if let Some(err) = err {
                eprintln!("{} CMD error: {err}", dir_prefix(dir, ""));
            }
            log_command(&dir_prefix(dir, "CMD"), &cmd);
        }
    }
    eprintln!("EOF");
    Ok(())
}

fn dir_prefix(dir: Dir, text: &str) -> String {
    match dir {
        Dir::In => format!("<< {text}"),
        Dir::Out => format!(">> {text}"),
    }
}

fn log_bytes(label: &str, data: &[u8]) {
    eprintln!("{label} len={}", data.len());
}

fn log_command(label: &str, cmd: &Command) {
    eprintln!(
        "{label} id={} trx={:?} type={}",
        cmd.id,
        cmd.transaction,
        payload_name(&cmd.payload)
    );
}

fn payload_name(payload: &CommandPayload) -> &'static str {
    match payload {
        CommandPayload::Unknown(_) => "Unknown",
        other => other
            .id()
            .and_then(|id| {
                ipod::command::registry()
                    .iter()
                    .find(|entry| entry.id == id)
                    .map(|entry| entry.name)
            })
            .unwrap_or("Unknown"),
    }
}

fn usage() {
    eprintln!(
        "usage:
  ipod [-d] [-l] lingos
  ipod [-d] [-l] serve [-w trace] <dev>
  ipod [-d] [-l] replay <trace>
  ipod [-d] [-l] view <trace>
  ipod [-d] [-l] send [-w trace] <dev> <trace>"
    );
}

fn main() -> ipod::Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let mut legacy = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--debug" => {
                args.remove(i);
            }
            "-l" | "--legacy" => {
                legacy = true;
                args.remove(i);
            }
            _ => i += 1,
        }
    }

    let defs = if legacy {
        hid::legacy_report_defs()
    } else {
        hid::default_report_defs()
    };

    let Some(command) = args.first().map(String::as_str) else {
        usage();
        return Ok(());
    };

    match command {
        "lingos" => {
            println!("Registered lingos:");
            println!("{}", dump_lingos());
        }
        "serve" | "s" => {
            let mut write_trace = None;
            let mut rest = args[1..].to_vec();
            if rest.first().map(String::as_str) == Some("-w")
                || rest.first().map(String::as_str) == Some("--write-trace")
            {
                if rest.len() < 2 {
                    return Err(ipod::Error::InvalidData(
                        "trace file path is missing".to_string(),
                    ));
                }
                write_trace = Some(rest[1].clone());
                rest.drain(0..2);
            }
            let Some(path) = rest.first() else {
                return Err(ipod::Error::InvalidData(
                    "device path is missing".to_string(),
                ));
            };
            let file = open_device(Path::new(path))?;
            let reader_file = file.try_clone()?;
            let writer_file = file;
            if let Some(trace_path) = write_trace {
                let trace_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(trace_path)?;
                let trace_file_writer = trace_file.try_clone()?;
                let reader = trace::TracingReader::new(reader_file, trace_file);
                let writer = trace::TracingWriter::new(writer_file, trace_file_writer);
                let report_reader = RawReportReader::new(reader);
                let report_writer = RawReportWriter::new(writer);
                let mut transport = hid::Transport::new(report_reader, report_writer, defs);
                process_frames(&mut transport)?;
            } else {
                let report_reader = RawReportReader::new(reader_file);
                let report_writer = RawReportWriter::new(writer_file);
                let mut transport = hid::Transport::new(report_reader, report_writer, defs);
                process_frames(&mut transport)?;
            }
        }
        "replay" | "r" => {
            let Some(path) = args.get(1) else {
                return Err(ipod::Error::InvalidData(
                    "trace file path is missing".to_string(),
                ));
            };
            let file = File::open(path)?;
            let trace_reader = trace::Reader::new(file);
            let dir_reader = TraceDirReader::new(trace_reader, Dir::In);
            let report_reader = RawReportReader::new(dir_reader);
            let report_writer = RawReportWriter::new(io::sink());
            let mut transport = hid::Transport::new(report_reader, report_writer, defs);
            process_frames(&mut transport)?;
        }
        "view" | "v" => {
            let Some(path) = args.get(1) else {
                return Err(ipod::Error::InvalidData(
                    "trace file path is missing".to_string(),
                ));
            };
            let file = File::open(path)?;
            dump_trace(trace::Reader::new(file), defs)?;
        }
        "send" => {
            let mut write_trace = None;
            let mut rest = args[1..].to_vec();
            if rest.first().map(String::as_str) == Some("-w")
                || rest.first().map(String::as_str) == Some("--write-trace")
            {
                if rest.len() < 2 {
                    return Err(ipod::Error::InvalidData(
                        "trace file path is missing".to_string(),
                    ));
                }
                write_trace = Some(rest[1].clone());
                rest.drain(0..2);
            }
            let Some(device_path) = rest.first() else {
                return Err(ipod::Error::InvalidData(
                    "device path is missing".to_string(),
                ));
            };
            let Some(trace_path) = rest.get(1) else {
                return Err(ipod::Error::InvalidData(
                    "trace file path is missing".to_string(),
                ));
            };

            let file = open_device(Path::new(device_path))?;
            let reader_file = file.try_clone()?;
            let writer_file = file;
            let trace_input = File::open(trace_path)?;

            if let Some(trace_path) = write_trace {
                let trace_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(trace_path)?;
                let trace_file_writer = trace_file.try_clone()?;
                let reader = trace::TracingReader::new(reader_file, trace_file);
                let writer = trace::TracingWriter::new(writer_file, trace_file_writer);
                send_trace_reports(reader, writer, trace_input, defs)?;
            } else {
                send_trace_reports(reader_file, writer_file, trace_input, defs)?;
            }
        }
        _ => {
            usage();
        }
    }

    Ok(())
}

fn send_trace_reports<R, W>(
    reader: R,
    writer: W,
    trace_input: File,
    defs: hid::ReportDefs,
) -> ipod::Result<()>
where
    R: Read + Send + 'static,
    W: io::Write,
{
    let process_defs = defs.clone();
    thread::spawn(move || {
        let report_reader = RawReportReader::new(reader);
        let report_writer = RawReportWriter::new(io::sink());
        let mut transport = hid::Transport::new(report_reader, report_writer, process_defs);
        if let Err(err) = process_frames(&mut transport) {
            eprintln!("send response processor stopped: {err}");
        }
    });

    let trace_reader = trace::Reader::new(trace_input);
    let dir_reader = TraceDirReader::new(trace_reader, Dir::In);
    let mut trace_report_reader = RawReportReader::new(dir_reader);
    let mut report_writer = RawReportWriter::new(writer);

    while let Some(report) = trace_report_reader.read_report()? {
        report_writer.write_report(&report)?;
        eprintln!(
            "writing report id={:#04x} len={}",
            report.id,
            report.data.len()
        );
        thread::sleep(Duration::from_secs(1));
    }

    loop {
        thread::park();
    }
}
