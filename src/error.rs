use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    InvalidChecksum,
    InvalidData(String),
    UnknownCommand(super::lingo::LingoCmdId),
    UnknownPayload,
    UnexpectedEof,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn invalid<T>(message: impl Into<String>) -> Result<T> {
        Err(Self::InvalidData(message.into()))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::InvalidChecksum => write!(f, "invalid checksum"),
            Self::InvalidData(message) => f.write_str(message),
            Self::UnknownCommand(id) => write!(f, "unknown command {id}"),
            Self::UnknownPayload => write!(f, "payload not known"),
            Self::UnexpectedEof => write!(f, "unexpected EOF"),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        if err.kind() == io::ErrorKind::UnexpectedEof {
            Self::UnexpectedEof
        } else {
            Self::Io(err)
        }
    }
}
