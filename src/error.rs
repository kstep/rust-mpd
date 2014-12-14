use std::error::{Error, FromError};
use rustc_serialize::{Encoder, Encodable};
use std::io::{IoError, standard_error, IoErrorKind};
use std::collections::enum_set::CLike;

#[deriving(Show, RustcEncodable)]
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
pub enum MpdProtoError {
    InvalidInput,
    MissingMpdBanner
}

#[deriving(Show, RustcEncodable)]
pub enum MpdParserError {
    NotAPair,
    NotAnAck,
    NotOk
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
    Proto(MpdProtoError),
    Parser(MpdParserError),
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
            MpdError::Proto(ref err) => match *err {
                MpdProtoError::InvalidInput => "invalid input",
                MpdProtoError::MissingMpdBanner => "missing or invalid mpd banner"
            },
            MpdError::Parser(ref err) => match *err {
                MpdParserError::NotAPair => "pair expected",
                MpdParserError::NotAnAck => "pair error expected",
                MpdParserError::NotOk => "ok expected"
            }
        }
    }

    fn detail(&self) -> Option<String> {
        match *self {
            MpdError::Mpd(ref err) => err.detail(),
            MpdError::Io(ref err) => err.detail(),
            _ => None
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MpdError::Io(ref err) => Some(err as &Error),
            MpdError::Mpd(ref err) => Some(err as &Error),
            _ => None
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
