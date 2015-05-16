use std::time::duration::Duration;
use std::string::ToString;
use std::str::FromStr;
use std::old_io::{Stream, BufferedStream, Lines, IoResult, IoErrorKind, standard_error};
use std::convert::From;

use status::MpdStatus;
use stats::MpdStats;
use error::{MpdResult, MpdServerError, ParseMpdServerError, ParseMpdServerErrorKind, ParseMpdResponseError, ParseMpdPairError};
use songs::MpdSong;
//use settings::MpdSettings;
//use queue::MpdQueue;
use playlists::MpdPlaylist;
use outputs::MpdOutput;
//use idle::{MpdIdle, MpdEvent};


pub struct MpdResultIterator<I: Iterator> {
  inner: I
}

impl<I: Iterator> MpdResultIterator<I> {
    pub fn new(iter: I) -> MpdResultIterator<I> {
        MpdResultIterator { inner: iter }
    }
}

/*
impl<I> Iterator for MpdResultIterator<I> where I: Iterator<Item=IoResult<String>> {
    type Item = MpdResult<MpdPair>;
    fn next(&mut self) -> Option<MpdResult<MpdPair>> {
        match self.inner.next() {
            Some(Ok(s)) => s.parse().unwrap(), // TODO: should pass through parse error
            Some(Err(e)) => Some(Err(From::from(e))),
            None => None,
        }
    }
}
*/

#[derive(Debug)]
pub struct MpdPair(pub String, pub String);

impl FromStr for MpdPair {
    type Err = ParseMpdPairError;
    fn from_str(s: &str) -> Result<MpdPair, ParseMpdPairError> {
        let mut it = s.splitn(1, ':');
        match (it.next(), it.next()) {
            (Some(a), Some(b)) => Ok(MpdPair(a.to_string(), b.trim().to_string())),
            _ => Err(ParseMpdPairError)
        }
    }
}

impl FromStr for Option<Result<MpdPair, MpdServerError>> {
    type Err = ParseMpdResponseError;
    fn from_str(s: &str) -> Result<Option<Result<MpdPair, MpdServerError>>, ParseMpdResponseError> {
        if s == "OK\n" || s == "list_OK\n" {
            Ok(None)
        } else {
            if let Ok(error) = s.parse::<MpdServerError>() {
                Ok(Err(From::from(error)))
            } else {
                match s.parse::<MpdPair>() {
                    Ok(pair) => Ok(Ok(Some(pair))),
                    Err(error) => Err(error)
                }
            }
        }
    }
}

#[derive(Debug, Copy)]
pub struct MpdVersion(pub usize, pub usize, pub usize);

pub struct ParseMpdVersionError {
    kind: ParseMpdVersionErrorKind
}

enum ParseMpdVersionErrorKind {
    InvalidFormat,
    InvalidNumber
}

impl FromStr for MpdVersion {
    type Err = ParseMpdVersionError;
    fn from_str(s: &str) -> Result<MpdVersion, ParseMpdVersionError> {
        let mut parts = s.splitn(2, '.').map(|v| v.parse::<usize>());
        match (parts.next(), parts.next(), parts.next()) {
            (Some(Ok(a)), Some(Ok(b)), Some(Ok(c))) => Ok(MpdVersion(a, b, c)),
            (Some(Err(_)), _, _) | (_, Some(Err(_)), _) | (_, _, Some(Err(_))) => Err(ParseMpdVersionError { kind: ParseMpdVersionErrorKind::InvalidNumber }),
            _ => Err(ParseMpdVersionError { kind: ParseMpdVersionErrorKind::InvalidFormat })
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
                None => return Err(From::from(standard_error(IoErrorKind::InvalidInput)))
            }
        } else {
            return Err(From::from(standard_error(IoErrorKind::InvalidInput)));
        };

        Ok(MpdClient { socket: socket, version: version })
    }

    pub fn authorize(&mut self, password: &str) -> MpdResult<()> {
        self.exec_str("password", password)
    }

    pub fn play(&mut self) -> MpdResult<()> { self.exec("play").and_then(|_| self.ok()) }
    pub fn stop(&mut self) -> MpdResult<()> { self.exec("stop").and_then(|_| self.ok()) }
    pub fn pause(&mut self, mode: bool) -> MpdResult<()> { self.exec_bool("pause", mode).and_then(|_| self.ok()) }

    pub fn volume(&mut self, vol: usize) -> MpdResult<()> { self.exec_arg("setvol", vol).and_then(|_| self.ok()) }
    pub fn change_volume(&mut self, vol: isize) -> MpdResult<()> { self.exec_arg("volume", vol).and_then(|_| self.ok()) }

    pub fn repeat(&mut self, value: bool) -> MpdResult<()> { self.exec_bool("repeat", value).and_then(|_| self.ok()) }
    pub fn single(&mut self, value: bool) -> MpdResult<()> { self.exec_bool("single", value).and_then(|_| self.ok()) }
    pub fn consume(&mut self, value: bool) -> MpdResult<()> { self.exec_bool("consume", value).and_then(|_| self.ok()) }
    pub fn random(&mut self, value: bool) -> MpdResult<()> { self.exec_bool("random", value).and_then(|_| self.ok()) }
    pub fn crossfade(&mut self, value: Duration) -> MpdResult<()> { self.exec_arg("crossfade", value.num_seconds()).and_then(|_| self.ok()) }
    pub fn mixrampdb(&mut self, value: f32) -> MpdResult<()> { self.exec_arg("mixrampdb", value).and_then(|_| self.ok()) }
    pub fn mixrampdelay(&mut self, value: Duration) -> MpdResult<()> { self.exec_arg("mixrampdelay", value.num_seconds()).and_then(|_| self.ok()) }

    pub fn next(&mut self) -> MpdResult<()> { self.exec("next").and_then(|_| self.ok()) }
    pub fn prev(&mut self) -> MpdResult<()> { self.exec("previous").and_then(|_| self.ok()) }

    pub fn play_pos(&mut self, pos: usize) -> MpdResult<()> { self.exec_arg("play", pos).and_then(|_| self.ok()) }
    pub fn play_id(&mut self, id: usize) -> MpdResult<()> { self.exec_arg("playid", id).and_then(|_| self.ok()) }

    pub fn status(&mut self) -> MpdResult<MpdStatus> {
        self.exec("status").and_then(|_| self.iter().collect())
    }
    pub fn stats(&mut self) -> MpdResult<MpdStats> {
        self.exec("stats").and_then(|_| self.iter().collect())
    }
    pub fn current_song(&mut self) -> MpdResult<MpdSong> {
        self.exec("currentsong").and_then(|_| self.iter().collect())
    }

    pub fn playlists(&mut self) -> MpdResult<Vec<MpdPlaylist>> {
        self.exec("listplaylists").and_then(|_| self.iter().collect())
    }

    pub fn outputs(&mut self) -> MpdResult<Vec<MpdOutput>> {
        self.exec("outputs").and_then(|_| self.iter().collect())
    }

    pub fn update(&mut self, rescan: bool, path: Option<&str>) -> MpdResult<usize> {
        try!(self.exec_args(if rescan { "rescan" } else { "update" }, &[path.unwrap_or("")]));
        let mut iter = self.iter();
        let result = match iter.next() {
            Some(Ok(MpdPair(ref key, ref value))) if *key == "updating_db" => match value.parse() {
                Some(v) => Ok(v),
                None => Err(From::from(standard_error(IoErrorKind::InvalidInput)))
            },
            Some(Err(e)) => return Err(From::from(e)),
            _ => return Err(From::from(standard_error(IoErrorKind::InvalidInput))),
        };
        for s in iter {
            try!(s);
        }
        result
    }

    pub fn queue(&mut self) -> MpdResult<Vec<MpdSong>> {
        self.exec("playlistinfo").and_then(|_| self.iter().collect())
    }

    pub fn load(&mut self, playlist_name: &str) -> MpdResult<()> {
        self.exec_arg("load", playlist_name).and_then(|_| self.ok())
    }

    pub fn clear(&mut self) -> MpdResult<()> {
        self.exec("clear").and_then(|_| self.ok())
    }

    //pub fn wait(&self, mask: Option<MpdEvent>) -> MpdIdle {
        //MpdIdle::from_client(self, mask)
    //}

    #[inline] pub fn iter(&mut self) -> MpdResultIterator<Lines<BufferedStream<S>>> {
        MpdResultIterator::new(self.socket.lines())
    }

    #[inline] pub fn ok(&mut self) -> MpdResult<()> {
        self.socket.read_line().map_err(From::from).and_then(|line|
        line.parse::<MpdResult<MpdPair>>()
            .map(|r| Err(r.err().unwrap_or(From::from(standard_error(IoErrorKind::InvalidInput)))))
            .unwrap_or(Ok(())))
    }

    fn exec_args(&mut self, command: &str, args: &[&str]) -> MpdResult<()> {
        try!(self.socket.write_all(command.as_bytes()));
        for arg in args.iter() {
            try!(self.socket.write_all(b" "));
            try!(self.socket.write_all(arg.as_bytes()));
        }

        try!(self.socket.write_all(b"\n"));
        try!(self.socket.flush());
        Ok(())
    }

    #[inline] pub fn exec(&mut self, command: &str) -> MpdResult<()> { self.exec_args(command, &[]) }
    #[inline] pub fn exec_bool(&mut self, command: &str, val: bool) -> MpdResult<()> { self.exec_args(command, &[if val { "1" } else { "0" }]) }
    #[inline] pub fn exec_str(&mut self, command: &str, val: &str) -> MpdResult<()> { self.exec_args(command, &[val]) }
    #[inline] pub fn exec_arg<T: ToString>(&mut self, command: &str, val: T) -> MpdResult<()> { self.exec_args(command, &[&*val.to_string()]) }
    #[inline] pub fn exec_arg2<T1: ToString, T2: ToString>(&mut self, command: &str, val1: T1, val2: T2) -> MpdResult<()> { self.exec_args(command, &[&*val1.to_string(), &*val2.to_string()]) }
    #[inline] pub fn exec_arg3<T1: ToString, T2: ToString, T3: ToString>(&mut self, command: &str, val1: T1, val2: T2, val3: T3) -> MpdResult<()> { self.exec_args(command, &[&*val1.to_string(), &*val2.to_string(), &*val3.to_string()]) }
}

