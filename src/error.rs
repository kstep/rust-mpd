use std::convert::From;
use std::io::Error as IoError;
use std::error::Error as StdError;
use std::str::FromStr;
use std::fmt;
use std::num::{ParseIntError, ParseFloatError};

// Server errors {{{
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ErrorCode {
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

impl FromStr for ErrorCode {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<ErrorCode, ParseError> {
        use self::ErrorCode::*;
        match try!(s.parse()) {
            1 => Ok(NotList),
            2 => Ok(Argument),
            3 => Ok(Password),
            4 => Ok(Permission),
            5 => Ok(UnknownCmd),

            50 => Ok(NoExist),
            51 => Ok(PlaylistMax),
            52 => Ok(System),
            53 => Ok(PlaylistLoad),
            54 => Ok(UpdateAlready),
            55 => Ok(PlayerSync),
            56 => Ok(Exist),

            v => Err(ParseError::BadErrorCode(v)),
        }
    }
}

impl StdError for ErrorCode {
    fn description(&self) -> &str {
        use self::ErrorCode::*;
        match *self {
            NotList => "not a list",
            Argument => "invalid argument",
            Password => "invalid password",
            Permission => "permission",
            UnknownCmd => "unknown command",

            NoExist => "item not found",
            PlaylistMax => "playlist overflow",
            System => "system",
            PlaylistLoad => "playload load",
            UpdateAlready => "already updating",
            PlayerSync => "player syncing",
            Exist => "already exists",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ServerError {
    pub code: ErrorCode,
    pub pos: u16,
    pub command: String,
    pub detail: String
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} error (`{}') at {}", self.code, self.detail, self.pos)
    }
}

impl StdError for ServerError {
    fn description(&self) -> &str {
        self.code.description()
    }
}

impl FromStr for ServerError {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<ServerError, ParseError> {
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
                            Ok(ServerError {
                                code: code,
                                pos: pos,
                                command: command,
                                detail: detail
                            })
                        } else {
                            Err(ParseError::NoMessage)
                        }
                    }
                    (Err(_), _) => Err(ParseError::BadCode),
                    (_, Err(_)) => Err(ParseError::BadPos),
                }
            } else {
                Err(ParseError::NoCodePos)
            }
        } else {
            Err(ParseError::NotAck)
        }
    }
}
// }}}

// Error {{{
#[derive(Debug)]
pub enum Error {
    Io(IoError),
    Parse(ParseError),
    Proto(ProtoError),
    Server(ServerError)
}

impl StdError for Error {
    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Parse(ref err) => Some(err),
            Error::Proto(ref err) => Some(err),
            Error::Server(ref err) => Some(err),
        }
    }
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::Parse(ref err) => err.description(),
            Error::Proto(ref err) => err.description(),
            Error::Server(ref err) => err.description(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => err.fmt(f),
            Error::Parse(ref err) => err.fmt(f),
            Error::Proto(ref err) => err.fmt(f),
            Error::Server(ref err) => err.fmt(f),
        }
    }
}

impl From<IoError> for Error {
    fn from(e: IoError) -> Error { Error::Io(e) }
}
impl From<ParseError> for Error {
    fn from(e: ParseError) -> Error { Error::Parse(e) }
}
impl From<ProtoError> for Error {
    fn from(e: ProtoError) -> Error { Error::Proto(e) }
}
impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error { Error::Parse(ParseError::BadInteger(e)) }
}
impl From<ParseFloatError> for Error {
    fn from(e: ParseFloatError) -> Error { Error::Parse(ParseError::BadFloat(e)) }
}


impl From<ServerError> for Error {
    fn from(e: ServerError) -> Error { Error::Server(e) }
}
// }}}

// Parse errors {{{
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    BadInteger(ParseIntError),
    BadFloat(ParseFloatError),
    BadValue(String),
    BadVersion,
    NotAck,
    BadPair,
    BadCode,
    BadPos,
    NoCodePos,
    NoMessage,
    NoRate,
    NoBits,
    NoChans,
    BadRate(ParseIntError),
    BadBits(ParseIntError),
    BadChans(ParseIntError),
    BadState(String),
    BadErrorCode(usize)

}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str {
        use self::ParseError::*;
        match *self {
            BadInteger(_) => "invalid integer",
            BadFloat(_) => "invalid float",
            BadValue(_) => "invalid value",
            BadVersion => "invalid version",
            NotAck => "not an ACK",
            BadPair => "invalid pair",
            BadCode => "invalid code",
            BadPos => "invalid position",
            NoCodePos => "missing code and position",
            NoMessage => "missing position",
            NoRate => "missing audio format rate",
            NoBits => "missing audio format bits",
            NoChans => "missing audio format channels",
            BadRate(_) => "invalid audio format rate",
            BadBits(_) => "invalid audio format bits",
            BadChans(_) => "invalid audio format channels",
            BadState(_) => "invalid playing state",
            BadErrorCode(_) => "unknown error code",
        }
    }
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> ParseError {
        ParseError::BadInteger(e)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(e: ParseFloatError) -> ParseError {
        ParseError::BadFloat(e)
    }
}
// }}}

// Protocol errors {{{
#[derive(Debug, Clone, PartialEq)]
pub enum ProtoError {
    NotOk,
    NotPair,
    BadBanner,
    NoField(&'static str)
}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl StdError for ProtoError {
    fn description(&self) -> &str {
        match *self {
            ProtoError::NotOk => "OK expected",
            ProtoError::NotPair => "pair expected",
            ProtoError::BadBanner => "banner error",
            ProtoError::NoField(_) => "missing field",
        }
    }
}
//}}}

