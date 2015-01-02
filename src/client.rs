
use std::time::duration::Duration;
use std::c_str::ToCStr;
use std::ptr;
use std::io::{Stream, BufferedStream};
use std::io::net::ip::{Port, ToSocketAddr};
use std::io::net::tcp::TcpStream;
use std::str::FromStr;
use std::collections::enum_set::CLike;
use std::error::{Error, FromError};

use std::io::{IoError, standard_error, IoErrorKind};
use error::{MpdResult, MpdError, MpdErrorCode, MpdServerError};
//use outputs::{MpdOutputs, MpdOutput};
//use playlists::MpdPlaylists;
//use songs::{MpdSong, mpd_song};
use status::MpdStatus;
//use settings::MpdSettings;
//use stats::MpdStats;
//use queue::MpdQueue;
//use idle::{MpdIdle, MpdEvent};

struct MpdResultIterator<I: Iterator<_>> {
  inner: I
}

impl<I: Iterator<IoResult<String>>> Iterator<MpdResult<MpdPair>> for MpdResultIterator<I> {
  fn next(&mut self) -> Option<MpdResult<MpdPair>> {
    self.inner.next().map(
		|res| res.map_err(FromError::from_error).and_then(
		|s| s.parse().unwrap_or_else(
			|| Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput))))
	}
}

pub trait FromClient {
    fn from_client<S: Stream>(client: &MpdClient<S>) -> Option<Self>;
}

#[deriving(Show)]
struct MpdPair(String, String);

impl FromStr for MpdPair {
    fn from_str(s: &str) -> Option<MpdPair> {
        println!("pair? {}", s);
        let mut it = s.splitn(1, ':');
        match (it.next(), it.next()) {
            (Some(a), Some(b)) => Some(MpdPair(a.to_string(), b.trim().to_string())),
            _ => None
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

impl FromStr for MpdResult<MpdPair> {
    fn from_str(s: &str) -> Option<MpdResult<MpdPair>> {
        if s == "OK\n" || s == "list_OK\n" {
            None
        } else {
            if let Some(error) = s.parse::<MpdServerError>() {
                Some(Err(FromError::from_error(error)))
            } else {
                if let Some(pair) = s.parse::<MpdPair>() {
                    Some(Ok(pair))
                } else {
                    Some(Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput))))
                }
            }
        }
    }
}

#[deriving(Show)]
pub struct MpdVersion(uint, uint, uint);

impl FromStr for MpdVersion {
    fn from_str(s: &str) -> Option<MpdVersion> {
        let mut parts = s.splitn(2, '.').filter_map(|v| v.parse::<uint>());
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

impl<S: Stream> MpdClient<S> {
    pub fn new(socket: S) -> MpdResult<MpdClient<S>> {
        let mut socket = BufferedStream::new(socket);
        let banner = try!(socket.read_line());
        let version: MpdVersion = if banner.starts_with("OK MPD ") {
            match banner[7..].trim().parse() {
                Some(v) => v,
                None => return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))
            }
        } else {
            return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)));
        };

        Ok(MpdClient { socket: socket, version: version })
    }

    pub fn authorize(&mut self, password: &str) -> MpdResult<()> {
        self.run_command(format!("password {}", password)[])
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

    pub fn status(&self) -> MpdResult<MpdStatus> {
        try!(self.run_command("status"));
        FromClient::from_client(self)
    }
    //pub fn stats(&self) -> MpdResult<MpdStats> { FromClient::from_client(self) }
    //pub fn current_song(&self) -> MpdResult<MpdSong> { self.run_command("currentsong").and_then(|()| FromClient::from_client(self)) }

    //pub fn playlists(&self) -> MpdResult<MpdPlaylists> { FromClient::from_client(self) }
    //pub fn outputs(&self) -> MpdResult<MpdOutputs> { FromClient::from_client(self) }

    pub fn update(&mut self, path: Option<&str>) -> MpdResult<uint> {
        try!(self.run_command(format!("xpdate {}", path.unwrap_or(""))[]));
        let result = match try!(self.socket.read_line()).parse::<MpdResult<MpdPair>>() {
            Some(Ok(MpdPair(ref key, ref value))) if key[] == "updating_db" => match value.parse() {
                Some(v) => Ok(v),
                None => Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))
            },
            None => return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput))),
            Some(Ok(_)) => Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput))),
            Some(Err(e)) => return Err(FromError::from_error(e))
        };
        for s in self.socket.lines() {
            match try!(s)[] {
                "OK\n" | "OK_list\n" => break,
                _ => continue
            }
        }
        result
    }

    //pub fn rescan(&mut self, path: Option<&str>) -> MpdResult<uint> {
        //try!(self.run_command(format!("rescan {}", path.unwrap_or(""))[]));

    //}

    //pub fn queue(&self) -> MpdResult<MpdQueue> { FromClient::from_client(self) }

    //pub fn wait(&self, mask: Option<MpdEvent>) -> MpdIdle {
        //MpdIdle::from_client(self, mask)
    //}

    fn run_command(&mut self, command: &str) -> MpdResult<()> {
        try!(self.socket.write(command.as_bytes()));
        try!(self.socket.write(b"\n"));
        try!(self.socket.flush());
        Ok(())
    }
}

