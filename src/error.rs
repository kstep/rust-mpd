//! This module defines different errors occurring during communication with MPD.
//!
//! There're following kinds of possible errors:
//!
//!   - IO errors (due to network communication failures),
//!   - parsing errors (because of bugs in parsing server response),
//!   - protocol errors (happen when we get unexpected data from server,
//!     mostly because protocol version mismatch, network data corruption
//!     or just bugs in the client),
//!   - server errors (run-time errors coming from MPD due to some MPD
//!     errors, like database failures or sound problems)
//!
//! This module defines all necessary infrastructure to represent these kinds or errors.

use std::convert::From;
use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::num::{ParseFloatError, ParseIntError};
use std::result;
use std::str::FromStr;
use std::string::ParseError as StringParseError;
use time::ConversionRangeError as TimeConversionRangeError;
use time::ParseError as TimeParseError;

// Server errors {{{
/// Server error codes, as defined in [libmpdclient](http://www.musicpd.org/doc/libmpdclient/protocol_8h_source.html)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum ErrorCode {
    /// not a list
    NotList = 1,
    /// bad command arguments
    Argument = 2,
    /// invalid password
    Password = 3,
    /// insufficient permissions
    Permission = 4,
    /// unknown command
    UnknownCmd = 5,
    /// object doesn't exist
    NoExist = 50,
    /// maximum playlist size exceeded
    PlaylistMax = 51,
    /// general system error
    System = 52,
    /// error loading playlist
    PlaylistLoad = 53,
    /// update database is already in progress
    UpdateAlready = 54,
    /// player synchronization error
    PlayerSync = 55,
    /// object already exists
    Exist = 56,
}

impl FromStr for ErrorCode {
    type Err = ParseError;
    fn from_str(s: &str) -> result::Result<ErrorCode, ParseError> {
        use self::ErrorCode::*;
        match s.parse()? {
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

impl StdError for ErrorCode {}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ErrorCode::*;

        let desc = match *self {
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
        };

        f.write_str(desc)
    }
}

/// Server error
#[derive(Debug, Clone, PartialEq)]
pub struct ServerError {
    /// server error code
    pub code: ErrorCode,
    /// command position in command list
    pub pos: u16,
    /// command name, which caused the error
    pub command: String,
    /// detailed error description
    pub detail: String,
}

impl StdError for ServerError {}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} error (`{}') at {}", self.code, self.detail, self.pos)
    }
}

impl FromStr for ServerError {
    type Err = ParseError;
    fn from_str(s: &str) -> result::Result<ServerError, ParseError> {
        // ACK [<code>@<index>] {<command>} <description>
        if let Some(s) = s.strip_prefix("ACK [") {
            if let (Some(atsign), Some(right_bracket)) = (s.find('@'), s.find(']')) {
                match (s[..atsign].parse(), s[atsign + 1..right_bracket].parse()) {
                    (Ok(code), Ok(pos)) => {
                        let s = &s[right_bracket + 1..];
                        if let (Some(left_brace), Some(right_brace)) = (s.find('{'), s.find('}')) {
                            let command = s[left_brace + 1..right_brace].to_string();
                            let detail = s[right_brace + 1..].trim().to_string();
                            Ok(ServerError {
                                code,
                                pos,
                                command,
                                detail,
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
/// Main error type, describing all possible error classes for the crate
#[derive(Debug)]
pub enum Error {
    /// IO errors (low-level network communication failures)
    Io(IoError),
    /// parsing errors (unknown data came from server)
    Parse(ParseError),
    /// protocol errors (e.g. missing required fields in server response, no handshake message etc.)
    Proto(ProtoError),
    /// server errors (a.k.a. `ACK` responses from server)
    Server(ServerError),
}

/// Shortcut type for MPD results
pub type Result<T> = result::Result<T, Error>;

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::Parse(ref err) => Some(err),
            Error::Proto(ref err) => Some(err),
            Error::Server(ref err) => Some(err),
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
    fn from(e: IoError) -> Error {
        Error::Io(e)
    }
}
impl From<ParseError> for Error {
    fn from(e: ParseError) -> Error {
        Error::Parse(e)
    }
}
impl From<ProtoError> for Error {
    fn from(e: ProtoError) -> Error {
        Error::Proto(e)
    }
}
impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Error {
        Error::Parse(ParseError::BadInteger(e))
    }
}
impl From<ParseFloatError> for Error {
    fn from(e: ParseFloatError) -> Error {
        Error::Parse(ParseError::BadFloat(e))
    }
}
impl From<TimeParseError> for Error {
    fn from(e: TimeParseError) -> Error {
        Error::Parse(ParseError::BadTime(e))
    }
}

impl From<ServerError> for Error {
    fn from(e: ServerError) -> Error {
        Error::Server(e)
    }
}

impl From<TimeConversionRangeError> for Error {
    fn from(e: TimeConversionRangeError) -> Error {
        Error::Parse(ParseError::BadTimeConversion(e))
    }
}
// }}}

// Parse errors {{{
/// Parsing error kinds
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// invalid integer
    BadInteger(ParseIntError),
    /// invalid float
    BadFloat(ParseFloatError),
    /// some other invalid value
    BadValue(String),
    /// date/time parsing error
    BadTime(TimeParseError),
    /// date/time to duration (Unix time) conversion error
    BadTimeConversion(TimeConversionRangeError),
    /// invalid version format (should be x.y.z)
    BadVersion,
    /// the response is not an `ACK` (not an error)
    /// (this is not actually an error, just a marker
    /// to try to parse the response as some other type,
    /// like a pair)
    NotAck,
    /// invalid pair
    BadPair,
    /// invalid error code in `ACK` response
    BadCode,
    /// invalid command position in `ACK` response
    BadPos,
    /// missing command position and/or error code in `ACK` response
    NoCodePos,
    /// missing error message in `ACK` response
    NoMessage,
    /// missing bitrate in audio format field
    NoRate,
    /// missing bits in audio format field
    NoBits,
    /// missing channels in audio format field
    NoChans,
    /// invalid bitrate in audio format field
    BadRate(ParseIntError),
    /// invalid bits in audio format field
    BadBits(ParseIntError),
    /// invalid channels in audio format field
    BadChans(ParseIntError),
    /// unknown state in state status field
    BadState(String),
    /// unknown error code in `ACK` response
    BadErrorCode(usize),
}

impl StdError for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;

        let desc = match *self {
            BadInteger(_) => "invalid integer",
            BadFloat(_) => "invalid float",
            BadValue(_) => "invalid value",
            BadTime(_) => "invalid date/time",
            BadTimeConversion(_) => "invalid date/time conversion",
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
        };

        write!(f, "{}", desc)
    }
}

impl From<TimeParseError> for ParseError {
    fn from(e: TimeParseError) -> ParseError {
        ParseError::BadTime(e)
    }
}

impl From<TimeConversionRangeError> for ParseError {
    fn from(e: TimeConversionRangeError) -> ParseError {
        ParseError::BadTimeConversion(e)
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

impl From<StringParseError> for ParseError {
    fn from(e: StringParseError) -> ParseError {
        match e {}
    }
}
// }}}

// Protocol errors {{{
/// Protocol errors
///
/// They usually occur when server violate expected command response format,
/// like missing fields in answer to some command, missing closing `OK`
/// line after data stream etc.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtoError {
    /// `OK` was expected, but it was missing
    NotOk,
    /// a data pair was expected
    NotPair,
    /// invalid handshake banner received
    BadBanner,
    /// expected some field, but it was missing
    NoField(&'static str),
    /// expected sticker value, but didn't find it
    BadSticker,
}

impl StdError for ProtoError {}

impl fmt::Display for ProtoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match *self {
            ProtoError::NotOk => "OK expected",
            ProtoError::NotPair => "pair expected",
            ProtoError::BadBanner => "banner error",
            ProtoError::NoField(_) => "missing field",
            ProtoError::BadSticker => "sticker error",
        };

        write!(f, "{}", desc)
    }
}
// }}}
