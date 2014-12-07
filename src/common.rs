use libc;
use std::error::Error;

#[repr(C)] pub struct mpd_connection;

#[link(name = "mpdclient")]
extern {
    fn mpd_connection_get_error(connection: *const mpd_connection) -> MpdErrorKind;
    fn mpd_connection_get_error_message(connection: *const mpd_connection) -> *const u8;
    fn mpd_connection_get_server_error(connection: *const mpd_connection) -> MpdServerErrorKind;
    fn mpd_connection_get_server_error_location(connection: *const mpd_connection) -> libc::c_uint;
    fn mpd_connection_get_system_error(connection: *const mpd_connection) -> libc::c_int;
    fn mpd_connection_clear_error(connection: *mut mpd_connection) -> bool;
}

//#[repr(C)]
//pub struct mpd_pair {
    //name: *const u8,
    //value: *const u8
//}

#[repr(C)]
#[allow(dead_code)]
#[deriving(Show)]
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
#[deriving(Show)]
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

#[deriving(Show)]
pub enum MpdError {
    Server { kind: MpdServerErrorKind, index: u32, desc: String },
    System { code: i32, desc: String },
    Other { kind: MpdErrorKind, desc: String }
}

pub trait FromConn {
    fn from_conn(connection: *mut mpd_connection) -> Option<Self>;
}

impl FromConn for MpdError {
    fn from_conn(connection: *mut mpd_connection) -> Option<MpdError> {
        unsafe {
            let error = mpd_connection_get_error(connection as *const _);

            let err = match error {
                MpdErrorKind::Success => return None,
                MpdErrorKind::System => MpdError::System {
                    code: mpd_connection_get_system_error(connection as *const _),
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection as *const _)),
                },
                MpdErrorKind::Server => MpdError::Server {
                    kind: mpd_connection_get_server_error(connection as *const _),
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection as *const _)),
                    index: mpd_connection_get_server_error_location(connection as *const _),
                },
                _ => MpdError::Other {
                    kind: error,
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection as *const _)),
                }
            };

            mpd_connection_clear_error(connection);
            Some(err)
        }
    }
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
