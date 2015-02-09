extern crate core;

use std::str::FromStr;
use std::old_io::IoError;
use std::error::{Error, FromError};
use std::fmt;
use rustc_serialize::{Encoder, Encodable};
use utils::ForceEncodable;

#[derive(Debug, Copy, RustcEncodable)]
pub enum MpdErrorCode {
    NotList,
    Argument,
    Password,
    Permission,
    UnknownCmd,
    NoExist,
    PlaylistMax,
    System,
    PlaylistLoad,
    UpdateAlready,
    PlayerSync,
    Exist,
    Other(usize)
}

impl FromStr for MpdErrorCode {
    type Err = core::num::ParseIntError;
    fn from_str(s: &str) -> Result<MpdErrorCode, core::num::ParseIntError> {
        match s {
            "1" => Ok(MpdErrorCode::NotList),
            "2" => Ok(MpdErrorCode::Argument),
            "3" => Ok(MpdErrorCode::Password),
            "4" => Ok(MpdErrorCode::Permission),
            "5" => Ok(MpdErrorCode::UnknownCmd),

            "50" => Ok(MpdErrorCode::NoExist),
            "51" => Ok(MpdErrorCode::PlaylistMax),
            "52" => Ok(MpdErrorCode::System),
            "53" => Ok(MpdErrorCode::PlaylistLoad),
            "54" => Ok(MpdErrorCode::UpdateAlready),
            "55" => Ok(MpdErrorCode::PlayerSync),
            "56" => Ok(MpdErrorCode::Exist),

            _ => s.parse().map(|v| MpdErrorCode::Other(v))
        }
    }
}

#[derive(Debug, RustcEncodable)]
pub struct MpdServerError {
    pub code: MpdErrorCode,
    pub pos: usize,
    pub command: String,
    pub detail: String
}

#[derive(Debug)]
pub enum MpdError {
    Mpd(MpdServerError),
    Io(IoError),
}

impl Encodable for MpdError {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            MpdError::Mpd(ref err) => err.encode(s),
            MpdError::Io(ref err) => {
                s.emit_struct("IoError", 3, |s| {
                    s.emit_struct_field("kind", 0, |s| format!("{:?}", err.kind).encode(s)).and_then(|_|
                    s.emit_struct_field("desc", 1, |s| err.desc.encode(s))).and_then(|_|
                    s.emit_struct_field("detail", 2, |s| err.detail.encode(s)))
                })
            }
        }
    }
}

impl Error for MpdServerError {
    fn description(&self) -> &str {
        match self.code {
            MpdErrorCode::NotList => "not a list",
            MpdErrorCode::Argument => "invalid argument",
            MpdErrorCode::Password => "invalid password",
            MpdErrorCode::Permission => "access denied",
            MpdErrorCode::UnknownCmd => "unknown command",
            MpdErrorCode::NoExist => "object not found",
            MpdErrorCode::PlaylistMax => "playlist overflow",
            MpdErrorCode::System => "system error",
            MpdErrorCode::PlaylistLoad => "playlist load error",
            MpdErrorCode::UpdateAlready => "database already updating",
            MpdErrorCode::PlayerSync => "player sync error",
            MpdErrorCode::Exist => "object already exists",
            MpdErrorCode::Other(_) => "unknown error",
        }
    }
}

impl fmt::Display for MpdServerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(fmt)
    }
}

impl Error for MpdError {
    fn description(&self) -> &str {
        match *self {
            MpdError::Io(ref err) => err.description(),
            MpdError::Mpd(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MpdError::Io(ref err) => Some(err as &Error),
            MpdError::Mpd(ref err) => Some(err as &Error),
        }
    }
}

impl fmt::Display for MpdError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            MpdError::Mpd(ref err) => err.fmt(fmt),
            MpdError::Io(ref err) => err.fmt(fmt),
        }
    }
}

impl FromError<IoError> for MpdError {
    fn from_error(err: IoError) -> MpdError {
        MpdError::Io(err)
    }
}

impl FromError<MpdServerError> for MpdError {
    fn from_error(err: MpdServerError) -> MpdError {
        MpdError::Mpd(err)
    }
}

#[derive(Copy, Debug)]
pub struct ParseMpdServerError {
    kind: MpdServerResponseParseErrorKind
}

#[derive(Copy, Debug)]
enum ParseMpdServerErrorKind {
    NoCodePos
    InvalidCode,
    InvalidPos,
    NoMessage
}

impl FromStr for Option<MpdServerError> {
    type Err = ParseMpdServerError;
    fn from_str(s: &str) -> Result<Option<MpdServerError>, ParseMpdServerError> {
        // ACK [<code>@<index>] {<command>} <description>
        if s.starts_with("ACK [") {
            let s = &s[5..];
            if let (Some(atsign), Some(right_bracket)) = (s.find('@'), s.find(']')) {
                match (s[..atsign].parse(), s[atsign + 1..right_bracket].parse()) {
                    (Ok(code), Ok(pos)) => {
                        let s = &s[right_bracket + 1..];
                        if let (Some(left_brace), Some(right_brace)) = (s.find('{'), s.find('}')) {
                            let command = s[left_brace + 1..right_brace].to_string();
                            let detail = s[right_brace + 1..].trim().to_string();
                            Ok(Some(MpdServerError {
                                code: code,
                                pos: pos,
                                command: command,
                                detail: detail
                            }))
                        } else {
                            Err(ParseMpdServerError { kind: ParseMpdServerErrorKind::NoMessage })
                        }
                    }
                    (Err(_), _) => Err(ParseMpdServerError { kind: ParseMpdServerErrorKind::InvalidCode }),
                    (_, Err(_)) => Err(ParseMpdServerError { kind: ParseMpdServerErrorKind::InvalidPos }),
                }
            } else {
                Err(ParseMpdServerError { kind: ParseMpdServerErrorKind::NoCodePos })
            }
        } else {
            Ok(None)
        }
    }
}

pub type MpdResult<T> = Result<T, MpdError>;

impl<T: Encodable> ForceEncodable for MpdResult<T> {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        match *self {
            Ok(ref v) => v.encode(s),
            Err(ref e) => e.encode(s)
        }
    }
}
