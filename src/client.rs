use std::collections::BTreeMap;
use std::io::{self, Read, Write, BufRead, Lines};
use std::convert::From;
use std::fmt::Arguments;
use std::net::{TcpStream, ToSocketAddrs};

use time::Duration;
use bufstream::BufStream;
use version::Version;
use error::{ProtoError, Error, Result};
use reply::Reply;
use status::Status;
use replaygain::ReplayGain;
use song::Song;

// Iterator {{{
struct Pairs<I>(I);

impl<I> Iterator for Pairs<I> where I: Iterator<Item=io::Result<String>> {
    type Item = Result<(String, String)>;
    fn next(&mut self) -> Option<Result<(String, String)>> {
        let reply: Option<Result<Reply>> = self.0.next().map(|v| v.map_err(Error::Io).and_then(|s| s.parse::<Reply>().map_err(Error::Parse)));
        match reply {
            Some(Ok(Reply::Pair(a, b))) => Some(Ok((a, b))),
            None | Some(Ok(Reply::Ok)) => None,
            Some(Ok(Reply::Ack(e))) => Some(Err(Error::Server(e))),
            Some(Err(e)) => Some(Err(e)),
        }
    }
}

struct Maps<'a, I: 'a> {
    pairs: &'a mut Pairs<I>,
    sep: &'a str,
    value: Option<String>,
    done: bool
}

impl<'a, I> Iterator for Maps<'a, I> where I: Iterator<Item=io::Result<String>> {
    type Item = Result<BTreeMap<String, String>>;
    fn next(&mut self) -> Option<Result<BTreeMap<String, String>>> {
        if self.done {
            return None;
        }

        let mut map = BTreeMap::new();

        if let Some(b) = self.value.take() {
            map.insert(self.sep.to_owned(), b);
        }

        loop {
            match self.pairs.next() {
                Some(Ok((a, b))) => {
                    if &*a == self.sep {
                        self.value = Some(b);
                        break;
                    } else {
                        map.insert(a, b);
                    }
                },
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    self.done = true;
                    break;
                }
            }
        }

        Some(Ok(map))
    }
}

impl<I> Pairs<I> where I: Iterator<Item=io::Result<String>> {
    fn split<'a, 'b: 'a>(&'a mut self, f: &'b str) -> Maps<'a, I> {
        let mut maps = Maps {
            pairs: self,
            sep: f,
            value: None,
            done: false,
        };
        maps.next(); // swallow first separator
        maps
    }
}
// }}}

// Client {{{
#[derive(Debug)]
pub struct Client<S=TcpStream> where S: Read+Write {
    socket: BufStream<S>,
    pub version: Version
}

impl Default for Client<TcpStream> {
    fn default() -> Client<TcpStream> {
        Client::<TcpStream>::connect("127.0.0.1:6600").unwrap()
    }
}

impl<S: Read+Write> Client<S> {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Client<TcpStream>> {
        TcpStream::connect(addr).map_err(Error::Io).and_then(Client::new)
    }

    pub fn new(socket: S) -> Result<Client<S>> {
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

    fn read_line(&mut self) -> Result<String> {
        let mut buf = String::new();
        try!(self.socket.read_line(&mut buf));
        if buf.ends_with("\n") {
            buf.pop();
        }
        Ok(buf)
    }

    fn read_pairs(&mut self) -> Pairs<Lines<&mut BufStream<S>>> {
        Pairs((&mut self.socket).lines())
    }

    fn read_map(&mut self) -> Result<BTreeMap<String, String>> {
        self.read_pairs().collect()
    }

    fn write_command(&mut self, command: &str) -> Result<()> {
        self.socket.write_all(command.as_bytes())
            .and_then(|_| self.socket.write(&[0x0a]))
            .and_then(|_| self.socket.flush())
            .map_err(From::from)
    }

    fn write_command_args(&mut self, command: Arguments) -> Result<()> {
        self.socket.write_fmt(command)
            .and_then(|_| self.socket.write(&[0x0a]))
            .and_then(|_| self.socket.flush())
            .map_err(From::from)
    }

    fn expect_ok(&mut self) -> Result<()> {
        let line = try!(self.read_line());

        match line.parse::<Reply>() {
            Ok(Reply::Ok) => Ok(()),
            Ok(Reply::Ack(e)) => Err(Error::Server(e)),
            Ok(_) => Err(Error::Proto(ProtoError::NotOk)),
            Err(e) => Err(From::from(e)),
        }
    }

    pub fn status(&mut self) -> Result<Status> {
        self.write_command("status")
            .and_then(|_| self.read_map())
            .and_then(Status::from_map)
    }

    pub fn clearerror(&mut self) -> Result<()> {
        self.write_command("clearerror")
            .and_then(|_| self.expect_ok())
    }

    pub fn volume(&mut self, volume: i8) -> Result<()> {
        self.write_command_args(format_args!("setvol {}", volume))
            .and_then(|_| self.expect_ok())
    }

    pub fn repeat(&mut self, value: bool) -> Result<()> {
        self.write_command_args(format_args!("repeat {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    pub fn random(&mut self, value: bool) -> Result<()> {
        self.write_command_args(format_args!("random {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    pub fn single(&mut self, value: bool) -> Result<()> {
        self.write_command_args(format_args!("single {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    pub fn consume(&mut self, value: bool) -> Result<()> {
        self.write_command_args(format_args!("consume {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    pub fn crossfade(&mut self, value: u64) -> Result<()> {
        self.write_command_args(format_args!("crossfade {}", value))
            .and_then(|_| self.expect_ok())
    }

    pub fn mixrampdb(&mut self, value: f32) -> Result<()> {
        self.write_command_args(format_args!("mixrampdb {}", value))
            .and_then(|_| self.expect_ok())
    }

    pub fn mixrampdelay<T: IntoSeconds>(&mut self, value: T) -> Result<()> {
        self.write_command_args(format_args!("mixrampdelay {}", value.into_seconds()))
            .and_then(|_| self.expect_ok())
    }

    pub fn replaygain(&mut self, gain: ReplayGain) -> Result<()> {
        self.write_command_args(format_args!("replay_gain_mode {}", gain))
            .and_then(|_| self.expect_ok())
    }

    pub fn get_replaygain(&mut self) -> Result<ReplayGain> {
        try!(self.write_command("replay_gain_status"));

        let buf = try!(self.read_line());

        let reply = try!(buf.parse::<Reply>());
        try!(self.expect_ok());

        match reply {
            Reply::Ack(e) => Err(Error::Server(e)),
            Reply::Pair(ref a, ref b) if &*a == "replay_gain_mode" => b.parse().map_err(From::from),
            _ => Err(Error::Proto(ProtoError::NoField("replay_gain_mode")))
        }
    }

    pub fn play(&mut self) -> Result<()> {
        self.write_command("play")
            .and_then(|_| self.expect_ok())
    }

    pub fn next(&mut self) -> Result<()> {
        self.write_command("next")
            .and_then(|_| self.expect_ok())
    }

    pub fn prev(&mut self) -> Result<()> {
        self.write_command("previous")
            .and_then(|_| self.expect_ok())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.write_command("stop")
            .and_then(|_| self.expect_ok())
    }

    pub fn pause(&mut self, value: bool) -> Result<()> {
        self.write_command_args(format_args!("pause {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    pub fn seek<T: IntoSeconds>(&mut self, pos: T) -> Result<()> {
        self.write_command_args(format_args!("seekcur {}", pos.into_seconds()))
            .and_then(|_| self.expect_ok())
    }

    pub fn currentsong(&mut self) -> Result<Option<Song>> {
        self.write_command("currentsong")
            .and_then(|_| self.read_map())
            .and_then(|m| if m.is_empty() {
                Ok(None)
            } else {
                Song::from_map(m).map(Some)
            })
    }

    pub fn queue(&mut self) -> Result<Vec<Song>> {
        self.write_command("playlistinfo")
            .and_then(|_| self.read_pairs().split("file").map(|v| v.and_then(Song::from_map)).collect())
    }
}

// }}}

pub trait IntoSeconds {
    fn into_seconds(self) -> f64;
}

impl IntoSeconds for i64 {
    fn into_seconds(self) -> f64 {
        self as f64
    }
}

impl IntoSeconds for f64 {
    fn into_seconds(self) -> f64 {
        self
    }
}

impl IntoSeconds for Duration {
    fn into_seconds(self) -> f64 {
        self.num_milliseconds() as f64 / 1000.0
    }
}
