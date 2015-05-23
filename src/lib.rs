extern crate rustc_serialize;
extern crate time;
extern crate bufstream;

use std::collections::BTreeMap;
use std::io::{Read, Write, BufRead};
use std::io::Error as IoError;
use std::str::FromStr;
use std::num::ParseIntError;
use std::convert::From;
use std::error::Error as StdError;
use std::fmt;

use bufstream::BufStream;

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

impl From<ServerError> for Error {
    fn from(e: ServerError) -> Error { Error::Server(e) }
}
// }}}

// Parse errors {{{
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    BadNumber(ParseIntError),
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
        use ParseError::*;
        match *self {
            BadNumber(_) => "invalid number",
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
        ParseError::BadNumber(e)
    }
}
// }}}

// Protocol errors {{{
#[derive(Debug, Clone, PartialEq)]
pub enum ProtoError {
    NotOk,
    NotPair,
    BadBanner,
    NoField(String)
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

// Client {{{
#[derive(Debug)]
pub struct Client<S: Read+Write> {
    socket: BufStream<S>,
    pub version: Version
}

impl<S: Read+Write> Client<S> {
    pub fn new(socket: S) -> Result<Client<S>, Error> {
        let mut socket = BufStream::new(socket);

        let mut banner = String::new();
        try!(socket.read_line(&mut banner));

        if !banner.starts_with("OK MPD ") {
            return Err(From::from(ProtoError::BadBanner));
        }

        let version = try!(banner[7..].trim().parse::<Version>());

        Ok(Client {
            socket: socket,
            version: version
        })
    }

    pub fn status(&mut self) -> Result<Status, Error> {
        // Result<Result<Reply, Error>, IoError>
        try!(self.socket.write_all("status\n".as_bytes()));
        try!(self.socket.flush());
        let lines = try! {
            (&mut self.socket).lines().map(|v| v.map_err(From::from).and_then(|s| s.parse().map_err(From::from)))
            .take_while(|v| {
                match *v {
                    Ok(Reply::Ok) => false,
                    Err(_) => false,
                    _ => true
                }
            })
            .map(|v| match v {
                Ok(Reply::Pair(a, b)) => Ok((a, b)),
                Ok(Reply::Ack(e)) => Err(Error::Server(e)),
                Err(e) => Err(e),
                _ => Err(Error::Proto(ProtoError::NotPair))
            })
            .collect::<Result<BTreeMap<String, String>, _>>()
        };

        // TODO: handle parse errors
        Ok(Status {
            volume: lines["volume"].parse().unwrap(),

            repeat: lines["repeat"] == "1",
            random: lines["random"] == "1",
            single: lines["single"] == "1",
            consume: lines["consume"] == "1",

            queue_version: lines["playlist"].parse().unwrap(),
            queue_len: lines["playlistlength"].parse().unwrap(),
            state: lines["state"].parse().unwrap(),
            //song: Option<MpdQueuePlace>,
            //nextsong: Option<MpdQueuePlace>,
            //play_time: Option<Duration>,
            //total_time: Option<Duration>,
            //elapsed: Option<Duration>,
            //duration: Option<Duration>,
            bitrate: lines.get("bitrate").and_then(|v| v.parse().ok()),
            crossfade: lines.get("crossfade").and_then(|v| v.parse().ok()),
            mixrampdb: lines["mixrampdb"].parse().unwrap(),
            mixrampdelay: lines.get("mixrampdelay").and_then(|v| v.parse().ok()),
            audio: lines.get("audio").and_then(|v| v.parse().ok()),
            updating_db: lines.get("updating_db").and_then(|v| v.parse().ok()),
            error: lines.get("error").map(|v| v.to_owned()),
        })
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Status {
    volume: usize,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    queue_version: usize,
    queue_len: usize,
    state: State,
    //song: Option<MpdQueuePlace>,
    //nextsong: Option<MpdQueuePlace>,
    //play_time: Option<Duration>,
    //total_time: Option<Duration>,
    //elapsed: Option<Duration>,
    //duration: Option<Duration>,
    bitrate: Option<usize>,
    crossfade: Option<u64>,
    mixrampdb: f32,
    mixrampdelay: Option<u64>,
    audio: Option<AudioFormat>,
    updating_db: Option<usize>,
    error: Option<String>
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AudioFormat {
    pub rate: u16,
    pub bits: u8,
    pub chans: u8
}

impl FromStr for AudioFormat {
    type Err = Error;
    fn from_str(s: &str) -> Result<AudioFormat, Error> {
        let mut it = s.split(':');
        Ok(AudioFormat {
            rate: try!(it.next().ok_or(ParseError::NoRate).and_then(|v| v.parse().map_err(ParseError::BadRate))),
            bits: try!(it.next().ok_or(ParseError::NoBits).and_then(|v| v.parse().map_err(ParseError::BadBits))),
            chans: try!(it.next().ok_or(ParseError::NoChans).and_then(|v| v.parse().map_err(ParseError::BadChans))),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    Stop,
    Play,
    Pause
}

impl FromStr for State {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<State, ParseError> {
        match s {
            "stop" => Ok(State::Stop),
            "play" => Ok(State::Play),
            "pause" => Ok(State::Pause),
            _ => Err(ParseError::BadState(s.to_owned())),
        }
    }
}
// }}}

// Version {{{
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Version(pub u16, pub u16, pub u16);

impl FromStr for Version {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Version, ParseError> {
        let mut splits = s.splitn(3, '.').map(FromStr::from_str);
        match (splits.next(), splits.next(), splits.next()) {
            (Some(Ok(a)), Some(Ok(b)), Some(Ok(c))) => Ok(Version(a, b, c)),
            (Some(Err(e)), _, _) | (_, Some(Err(e)), _) | (_, _, Some(Err(e))) => Err(ParseError::BadNumber(e)),
            _ => Err(ParseError::BadVersion)
        }
    }
}
// }}}

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
        use ErrorCode::*;
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
        use ErrorCode::*;
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

// Reply {{{
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    Ok,
    Ack(ServerError),
    Pair(String, String)
}

impl FromStr for Reply {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Reply, ParseError> {
        if s == "OK" || s == "list_OK" {
            Ok(Reply::Ok)
        } else {
            if let Ok(ack) = s.parse::<ServerError>() {
                Ok(Reply::Ack(ack))
            } else {
                let mut splits = s.splitn(2, ':');
                match (splits.next(), splits.next()) {
                    (Some(a), Some(b)) => Ok(Reply::Pair(a.to_owned(), b.trim().to_owned())),
                    _ => Err(ParseError::BadPair)
                }
            }
        }
    }
}
// }}}
