use std::str::FromStr;
use std::io::IoError;
use std::error::{Error, FromError};
use std::collections::enum_set::CLike;
use rustc_serialize::{Encoder, Encodable};

#[deriving(Show, Copy, RustcEncodable)]
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
    Other(uint)
}

impl FromStr for MpdErrorCode {
    fn from_str(s: &str) -> Option<MpdErrorCode> {
        match s {
            "1" => Some(MpdErrorCode::NotList),
            "2" => Some(MpdErrorCode::Argument),
            "3" => Some(MpdErrorCode::Password),
            "4" => Some(MpdErrorCode::Permission),
            "5" => Some(MpdErrorCode::UnknownCmd),

            "50" => Some(MpdErrorCode::NoExist),
            "51" => Some(MpdErrorCode::PlaylistMax),
            "52" => Some(MpdErrorCode::System),
            "53" => Some(MpdErrorCode::PlaylistLoad),
            "54" => Some(MpdErrorCode::UpdateAlready),
            "55" => Some(MpdErrorCode::PlayerSync),
            "56" => Some(MpdErrorCode::Exist),

            _ => s.parse().map(|v| MpdErrorCode::Other(v))
        }
    }
}

impl CLike for MpdErrorCode {
    fn to_uint(&self) -> uint {
        match *self {
            MpdErrorCode::NotList => 1,
            MpdErrorCode::Argument => 2,
            MpdErrorCode::Password => 3,
            MpdErrorCode::Permission => 4,
            MpdErrorCode::UnknownCmd => 5,
            MpdErrorCode::NoExist => 50,
            MpdErrorCode::PlaylistMax => 51,
            MpdErrorCode::System => 52,
            MpdErrorCode::PlaylistLoad => 53,
            MpdErrorCode::UpdateAlready => 54,
            MpdErrorCode::PlayerSync => 55,
            MpdErrorCode::Exist => 56,
            MpdErrorCode::Other(num) => num
        }
    }

    fn from_uint(v: uint) -> MpdErrorCode {
        match v {
            1 => MpdErrorCode::NotList,
            2 => MpdErrorCode::Argument,
            3 => MpdErrorCode::Password,
            4 => MpdErrorCode::Permission,
            5 => MpdErrorCode::UnknownCmd,
            50 => MpdErrorCode::NoExist,
            51 => MpdErrorCode::PlaylistMax,
            52 => MpdErrorCode::System,
            53 => MpdErrorCode::PlaylistLoad,
            54 => MpdErrorCode::UpdateAlready,
            55 => MpdErrorCode::PlayerSync,
            56 => MpdErrorCode::Exist,
            _ => MpdErrorCode::Other(v)
        }
    }
}

#[deriving(Show, RustcEncodable)]
pub struct MpdServerError {
    pub code: MpdErrorCode,
    pub pos: uint,
    pub command: String,
    pub detail: String
}

#[deriving(Show)]
pub enum MpdError {
    Mpd(MpdServerError),
    Io(IoError),
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

impl Error for MpdError {
    fn description(&self) -> &str {
        match *self {
            MpdError::Io(ref err) => err.description(),
            MpdError::Mpd(ref err) => err.description(),
        }
    }

    fn detail(&self) -> Option<String> {
        match *self {
            MpdError::Mpd(ref err) => err.detail(),
            MpdError::Io(ref err) => err.detail(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MpdError::Io(ref err) => Some(err as &Error),
            MpdError::Mpd(ref err) => Some(err as &Error),
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

impl FromStr for MpdServerError {
    fn from_str(s: &str) -> Option<MpdServerError> {
        // ACK [<code>@<index>] {<command>} <description>
        if s.starts_with("ACK [") {
            let s = s[5..];
            if let (Some(atsign), Some(right_bracket)) = (s.find('@'), s.find(']')) {
                if let (Some(code), Some(pos)) = (s[..atsign].parse(), s[atsign + 1..right_bracket].parse()) {
                    let s = s[right_bracket + 1..];
                    if let (Some(left_brace), Some(right_brace)) = (s.find('{'), s.find('}')) {
                        let command = s[left_brace + 1..right_brace].to_string();
                        let detail = s[right_brace + 1..].trim().to_string();
                        return Some(MpdServerError {
                            code: code,
                            pos: pos,
                            command: command,
                            detail: detail
                        });
                    }
                }
            }
        }
        None
    }
}

pub type MpdResult<T> = Result<T, MpdError>;

//impl<S, E, T> Encodable<S, E> for MpdResult<T>
    //where S: Encoder<E>, T: Encodable<S, E> {

    //fn encode(&self, s: &mut S) -> Result<(), E> {
        //match *self {
            //Ok(ref v) => v.encode(s),
            //Err(ref e) => e.encode(s)
        //}
    //}
//}
