
use std::time::duration::Duration;
use std::c_str::ToCStr;
use std::ptr;
use std::io::{Stream, BufferedStream};
use std::io::net::ip::{Port, ToSocketAddr};
use std::io::net::tcp::TcpStream;
use std::str::{from_str, FromStr};
use std::collections::enum_set::CLike;

use std::io::{IoError, standard_error, IoErrorKind};
use error::{MpdResult, MpdError, MpdErrorCode, MpdProtoError, MpdParserError, MpdServerError};
//use outputs::{MpdOutputs, MpdOutput};
//use playlists::MpdPlaylists;
//use songs::{MpdSong, mpd_song};
//use status::MpdStatus;
//use settings::MpdSettings;
//use stats::MpdStats;
//use queue::MpdQueue;
//use idle::{MpdIdle, MpdEvent};

pub trait FromClient {
    fn from_client<S: Stream>(client: &MpdClient<S>) -> Option<Self>;
}

trait FromMpdStr {
    fn parse(s: &str) -> Result<Self, MpdParserError>;
}

impl FromMpdStr for (String, String) {
    fn parse(s: &str) -> Result<(String, String), MpdParserError> {
        let mut it = s.splitn(1, ':').map(|v| v.trim_left_chars(' '));
        match (it.next(), it.next()) {
            (Some(a), Some(b)) => Ok((a.to_string(), b.to_string())),
            _ => Err(MpdParserError::NotAPair)
        }
    }
}

trait PairLike<E> for Sized? {
    fn map_pair<A, B>(&mut self, f1: |&E| -> A, f2: |&E| -> B) -> Option<(A, B)>;
    fn to_pair(&mut self) -> Option<(E, E)>;
}

impl<E, Iter: Iterator<E>> PairLike<E> for Iter {
    fn map_pair<A, B>(&mut self, f1: |&E| -> A, f2: |&E| -> B) -> Option<(A, B)> {
        match (self.next(), self.next()) {
            (Some(ref a), Some(ref b)) => Some((f1(a), f2(b))),
            _ => None
        }
    }
    fn to_pair(&mut self) -> Option<(E, E)> {
        match (self.next(), self.next()) {
            (Some(a), Some(b)) => Some((a, b)),
            _ => None
        }
    }
}

impl FromMpdStr for MpdServerError {
    fn parse(s: &str) -> Result<MpdServerError, MpdParserError> {
        if !s.starts_with("ACK [") {
            return Err(MpdParserError::NotAnAck);
        }

        // ACK [<code>@<index>] {<command>} <description>
    }
}

#[deriving(Show)]
struct MpdVersion(uint, uint, uint);

impl FromStr for MpdVersion {
    fn from_str(s: &str) -> Option<MpdVersion> {
        let mut parts = s.splitn(2, '.').filter_map(|v| from_str::<uint>(v));
        match (parts.next(), parts.next(), parts.next()) {
            (Some(a), Some(b), Some(c)) => Some(MpdVersion(a, b, c)),
            _ => None
        }
    }
}

pub struct MpdClient<S: Stream> {
    socket: BufferedStream<S>,
    pub version: MpdVersion
}

impl FromIterator<uint> for MpdVersion {
    fn from_iter<T: Iterator<uint>>(iter: T) -> MpdVersion {
        MpdVersion(iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap())
    }
}

impl<S: Stream> MpdClient<S> {
    pub fn new(socket: S) -> MpdResult<MpdClient<S>> {
        let mut socket = BufferedStream::new(socket);
        let banner = try!(socket.read_line());
        let version: MpdVersion = if banner.starts_with("OK MPD ") {
            match from_str(banner[7..]) {
                Some(v) => v,
                None => return Err(MpdError::Proto(MpdProtoError::InvalidInput))
            }
        } else {
            return Err(MpdError::Proto(MpdProtoError::MissingMpdBanner));
        };

        Ok(MpdClient { socket: socket, version: version })
    }

    pub fn authorize(&mut self, password: &str) -> MpdResult<()> {
        self.run_command(format!("password {}", password)[])
    }

    pub fn version(&self) -> MpdVersion {
        self.version
    }

    pub fn play(&mut self) -> MpdResult<()> { self.run_command("play") }
    pub fn stop(&mut self) -> MpdResult<()> { self.run_command("stop") }
    pub fn pause(&mut self, mode: bool) -> MpdResult<()> { self.run_command(format!("pause {}", mode as uint)[]) }

    pub fn volume(&mut self, vol: uint) -> MpdResult<()> { self.run_command(format!("setvol {}", vol)[]) }
    pub fn change_volume(&mut self, vol: int) -> MpdResult<()> { self.run_command(format!("volume {}", vol)[]) }

    pub fn repeat(&mut self, value: bool) -> MpdResult<()> { self.run_command(format!("repeat {}", value as uint)[]) }
    pub fn single(&mut self, value: bool) -> MpdResult<()> { self.run_command(format!("single {}", value as uint)[]) }
    pub fn consume(&mut self, value: bool) -> MpdResult<()> { self.run_command(format!("consume {}", value as uint)[]) }
    pub fn random(&mut self, value: bool) -> MpdResult<()> { self.run_command(format!("random {}", value as uint)[]) }
    pub fn crossfade(&mut self, value: Duration) -> MpdResult<()> { self.run_command(format!("crossfade {}", value.num_seconds())[]) }
    pub fn mixrampdb(&mut self, value: f32) -> MpdResult<()> { self.run_command(format!("mixrampdb {}", value)[]) }
    pub fn mixrampdelay(&mut self, value: Duration) -> MpdResult<()> { self.run_command(format!("mixrampdelay {}", value.num_seconds())[]) }

    pub fn next(&mut self) -> MpdResult<()> { self.run_command("next") }
    pub fn prev(&mut self) -> MpdResult<()> { self.run_command("previous") }

    pub fn play_pos(&mut self, pos: uint) -> MpdResult<()> { self.run_command(format!("play {}", pos)[]) }
    pub fn play_id(&mut self, id: uint) -> MpdResult<()> { self.run_command(format!("playid {}", id)[]) }

    //pub fn status(&self) -> MpdResult<MpdStatus> { FromClient::from_client(self) }
    //pub fn stats(&self) -> MpdResult<MpdStats> { FromClient::from_client(self) }
    //pub fn current_song(&self) -> MpdResult<MpdSong> { self.run_command("currentsong").and_then(|()| FromClient::from_client(self)) }

    //pub fn playlists(&self) -> MpdResult<MpdPlaylists> { FromClient::from_client(self) }
    //pub fn outputs(&self) -> MpdResult<MpdOutputs> { FromClient::from_client(self) }


    pub fn update(&mut self, path: Option<&str>) -> MpdResult<uint> {
        try!(self.run_command(format!("update {}", path.unwrap_or(""))[]));
        for res in self.socket.lines() {
            let line = try!(res);
            
        }
    }

    pub fn parse_line<'a>(s: &'a str) -> Option<MpdResult<(&'a str, &'a str)>> {
        if s == "OK\n" || s == "list_OK\n" {
            None
        } else {
            if s.starts_with("ACK [") {
                if let Some((Some((code, idx)), Some((cmd, desc)))) = s[5..].splitn(1, ']').map_pair(
                    |code_idx| code_idx.splitn(1, '@').filter_map(|v| from_str::<uint>(v)).to_pair(),
                    |cmd_desc| cmd_desc.trim_left_chars([' ', '{'][]).splitn(1, '}').map(|v| v.to_string()).to_pair()
                    ) {
                    Some(Err(MpdError::Mpd(MpdServerError {
                        code: CLike::from_uint(code),
                        pos: idx,
                        command: cmd,
                        detail: desc
                    })))
                } else {
                    Some(Err(MpdError::Parser(MpdParserError::NotAnAck)))
                }
            } else {
                let mut splits = s.splitn(1, ':');
                match (splits.next(), splits.next()) {
                    (Some(k), Some(v)) if v[0] == ' ' => Some((k, v[1..])),
                    _ => Some(Err(MpdError::Parser(MpdParserError::NotAPair)))
                }
            }
        }
    }

    pub fn rescan(&mut self, path: Option<&str>) -> MpdResult<uint> {
        try!(self.run_command(format!("rescan {}", path.unwrap_or(""))[]));
        
    }

    //pub fn queue(&self) -> MpdResult<MpdQueue> { FromClient::from_client(self) }

    //pub fn wait(&self, mask: Option<MpdEvent>) -> MpdIdle {
        //MpdIdle::from_client(self, mask)
    //}

    fn run_command(&self, command: &str) -> MpdResult<()> {
        try!(self.socket.write(command.as_bytes()))
    }
}

