use std::error::Error;
use rustc_serialize::{Encoder, Encodable};

#[repr(C)] pub struct mpd_connection;

//#[repr(C)]
//pub struct mpd_pair {
    //name: *const c_uchar,
    //value: *const c_uchar
//}

#[repr(C)]
#[allow(dead_code)]
#[deriving(Show, RustcEncodable)]
pub enum MpdErrorKind {
    Success = 0,
    Oom = 1,
    Argument = 2,
    State = 3,
    Timeout = 4,
    System = 5,
    Resolver = 6,
    Malformed = 7,
    Closed = 8,
    Server = 9,
}

#[repr(C)]
#[allow(dead_code)]
#[deriving(Show, RustcEncodable)]
pub enum MpdServerErrorKind {
    Unknown = -1,
    NotList = 1,
    Argument = 2,
    Password = 3,
    Permission = 4,
    UnknownCmd = 5,
    NoExist = 50,
    PlaylistMax = 51,
    System = 52,
    PlaylistLoad = 53,
    UpdateAlready = 54,
    PlayerSync = 55,
    Exist = 56,
}

#[deriving(Show, RustcEncodable)]
pub enum MpdError {
    Server { kind: MpdServerErrorKind, index: uint, desc: String },
    System { code: int, desc: String },
    Other { kind: MpdErrorKind, desc: String }
}

impl Error for MpdError {
    fn description(&self) -> &str {
        match *self {
            MpdError::System { .. } => "system error",
            MpdError::Server { ref kind, .. } => match *kind {
                MpdServerErrorKind::Unknown => "unknown error",
                MpdServerErrorKind::NotList => "not a list",
                MpdServerErrorKind::Argument => "invalid argument",
                MpdServerErrorKind::Password => "invalid password",
                MpdServerErrorKind::Permission => "access denied",
                MpdServerErrorKind::UnknownCmd => "unknown command",
                MpdServerErrorKind::NoExist => "object not found",
                MpdServerErrorKind::PlaylistMax => "playlist overflow",
                MpdServerErrorKind::System => "system error",
                MpdServerErrorKind::PlaylistLoad => "playlist load error",
                MpdServerErrorKind::UpdateAlready => "database already updating",
                MpdServerErrorKind::PlayerSync => "player sync error",
                MpdServerErrorKind::Exist => "object already exists",
            },
            MpdError::Other { ref kind, .. } => match *kind {
                MpdErrorKind::Success => "success",
                MpdErrorKind::Oom => "out of memory",
                MpdErrorKind::Argument => "invalid argument",
                MpdErrorKind::State => "invalid state",
                MpdErrorKind::Timeout => "operation timed out",
                MpdErrorKind::System => "system error",
                MpdErrorKind::Resolver => "name resolution error",
                MpdErrorKind::Malformed => "malformed hostname",
                MpdErrorKind::Closed => "connection closed",
                MpdErrorKind::Server => "server error",
            }
        }
    }

    fn detail(&self) -> Option<String> {
        Some(match *self {
            MpdError::System { ref desc, .. } => desc.clone(),
            MpdError::Server { ref desc, .. } => desc.clone(),
            MpdError::Other { ref desc, .. } => desc.clone(),
        })
    }

    fn cause(&self) -> Option<&Error> { None }
}

pub type MpdResult<T> = Result<T, MpdError>;

impl<S, E, T> Encodable<S, E> for MpdResult<T>
    where S: Encoder<E>, T: Encodable<S, E> {

    fn encode(&self, s: &mut S) -> Result<(), E> {
        match *self {
            Ok(ref v) => v.encode(s),
            Err(ref e) => e.encode(s)
        }
    }
}
