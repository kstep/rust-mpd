#![feature(macro_rules, slicing_syntax)]

extern crate time;
extern crate serialize;

use std::io;
use std::io::{TcpStream, BufferedStream, IoResult, IoError, standard_error};
use std::io::net::ip::ToSocketAddr;
use std::time::Duration;
use std::collections::TreeMap;
use time::Tm;
use serialize::{Decoder, Decodable};

struct MpdConnection {
    stream: BufferedStream<TcpStream>
}

struct DirectoryInfo {
    directory: Path,
    lastMod: Tm,
}

struct TrackInfo {
    file: Path,
    lastMod: Tm,
    time: Duration,
    title: String,
    artist: Option<String>,
    album: Option<String>,
    albumArtist: Option<String>,
    track: Option<uint>,
    date: Option<uint>,
    genre: Option<String>,
    id: Option<uint>,
    pos: Option<uint>,
}

enum State {
    PLAY,
    PAUSE,
    STOP
}

struct AudioFormat {
    rate: u16,
    bits: u8,
    chans: u8
}

#[deriving(Decodable)]
struct Status {
    volume: u8,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    playlist: uint,
    playlistlength: uint,
    mixrampdb: f32,
    state: State,
    song: uint,
    songid: uint,
    time: (Option<Duration>, Option<Duration>),
    elapsed: Duration,
    bitrate: uint,
    audio: AudioFormat,
    nextsong: uint,
    nextsongid: uint,
}

struct Stats {
    uptime: Duration,
    playtime: Duration,
    artists: uint,
    albums: uint,
    songs: uint,
    dbPlaytime: Duration,
    dbUpdate: Tm,
}

struct MpdDecoder {
    stream: &mut BufferedStream<TcpStream>
}

impl Decoder<IoError> for MpdDecoder {
    fn read_nil(&mut self) -> IoResult<()> { () }

    fn read_u64(&mut self) -> IoResult<u64> {
        let mut r = 0;
        let mut skip = true;
        for b in self.stream.bytes() {
            match b {
                Ok(d) if 0x30 <= d && d <= 0x39 => { skip = false; r = r * 10 + (b & 0x0f) as uint; },
                Err(e) => return Err(e),
                Ok(0x20) => if skip { continue; } else { break; },
                _ => return Err(standard_error(io::InvalidInput))
            }
        }

        Ok(r)
    }

    fn read_uint(&mut self) -> IoResult<uint> { self.read_u64() as uint }
    fn read_u32(&mut self) -> IoResult<u32> { self.read_u64() as u32 }
    fn read_u16(&mut self) -> IoResult<u32> { self.read_u64() as u16 }
    fn read_u8(&mut self) -> IoResult<u32> { self.read_u64() as u8 }

    fn read_i64(&mut self) -> IoResult<i64> {
        let mut r = 0;
        let mut skip = true;
        let mut sign = 1;

        for b in self.stream.bytes() {
            match b {
                Ok(0x2d) if skip => { skip = false; sign = -1; },
                Ok(d) if 0x30 <= d && d <= 0x39 => { skip = false; r = r * 10 + (b & 0x0f) as uint; },
                Err(e) => return Err(e),
                Ok(0x20) => if skip { continue; } else { break; },
                _ => return Err(standard_error(io::InvalidInput))
            }
        }

        Ok(sign * r)
    }

    fn read_int(&mut self) -> IoResult<int> { self.read_i64() as int }
    fn read_i32(&mut self) -> IoResult<i32> { self.read_u64() as i32 }
    fn read_i16(&mut self) -> IoResult<i32> { self.read_u64() as i16 }
    fn read_i8(&mut self) -> IoResult<i32> { self.read_u64() as i8 }

    fn read_bool(&mut self) -> IoResult<bool> {
        match self.stream.read_u8() {
            Ok(c) if c == 0x30 => false,
            Ok(c) if c == 0x31 => true,
            Err(e) => Err(e),
            _ => Err(standard_error(io::InvalidInput))
        }
    }

    fn read_f64(&mut self) -> IoResult<f64> {
    }

    fn read_f32(&mut self) -> IoResult<f32> { self.read_f64() as f32 }

    fn read_char(&mut self) -> IoResult<char> { self.read_char() }
    fn read_str(&mut self) -> IoResult<String> { self.read_line() }
    fn read_enum<T>(&mut self, name: &str, f: |&mut MpdDecoder| -> IoResult<T>) -> IoResult<T> {
        
    }
}

impl MpdConnection {
    fn new<T: ToSocketAddr>(addr: T) -> IoResult<MpdConnection> {
       match TcpStream::connect(addr) {
           Ok(stream) => Ok(MpdConnection { stream: BufferedStream::new(stream) }),
           Err(e) => Err(e)
       }
    }

    fn playlist(&mut self) -> IoResult<Vec<Path>> {
        Err(standard_error(io::IoUnavailable))
    }

    fn playlistinfo(&mut self) -> IoResult<Vec<TrackInfo>> {
        Err(standard_error(io::IoUnavailable))
    }

    fn status(&mut self) -> IoResult<Status> {
        try!(self.stream.write(b"status\n").and_then(|()| self.stream.flush()));

        let mut result = Status {
            volume: 0,
            repeat: false,
            random: false,
            single: false,
            consume: false,
            playlist: 0,
            playlistlength: 0,
            mixrampdb: 0.0,
            state: STOP,
            song: 0,
            songid: 0,
            time: (None, None),
            elapsed: 0,
            bitrate: 0,
            audio: AudioFormat{ rate: 0, bits: 0, chans: 0 },
            nextsong: 0,
            nextsongid: 0,
        };

        for res in self.stream.lines() {
            let line = try!(res);
            if line[] == "OK" { break; } 

            if 
        }

        let map = self.stream.lines()
            .take_while(|line| line.map(|l| l[] == "OK").unwrap_or(false))
            .filter_map(|line| line.ok().map(|s| s.splitn(1, ':')).and_then(|s| match (s.next(), s.next()) {
                (Some(k), Some(v)) => Some((k.to_string(), v.trim_left_chars(' ').to_string())),
                (_, _) => None
            }))
            .collect::<TreeMap<String, String>>();
            
        Err(standard_error(io::IoUnavailable))
    } 
    fn stats(&mut self) -> IoResult<Stats> {
        Err(standard_error(io::IoUnavailable))
    }
}

#[test]
fn it_works() {
}
