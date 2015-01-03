use std::time::duration::Duration;
use std::str::FromStr;
use std::io::{standard_error, IoErrorKind};
use std::error::FromError;
use rustc_serialize::{Encoder, Encodable};

use error::MpdResult;
use client::{MpdPair, ForceEncodable};

#[deriving(Show, Copy, RustcEncodable)]
pub struct AudioFormat {
    pub rate: u32,
    pub bits: u8,
    pub chans: u8
}

impl FromStr for AudioFormat {
    fn from_str(s: &str) -> Option<AudioFormat> {
        let mut it = s.split(':');
        if let (Some(rate), Some(bits), Some(chans)) = (
            it.next().and_then(|v| v.parse()),
            it.next().and_then(|v| v.parse()),
            it.next().and_then(|v| v.parse())) {
            Some(AudioFormat {
                rate: rate,
                bits: bits,
                chans: chans
            })
        } else {
            None
        }
    }
}

#[deriving(Show, Copy, RustcEncodable)]
pub enum MpdState {
    Stop,
    Play,
    Pause,
}

impl FromStr for MpdState {
    fn from_str(s: &str) -> Option<MpdState> {
        match s {
            "stop" => Some(MpdState::Stop),
            "play" => Some(MpdState::Play),
            "pause" => Some(MpdState::Pause),
            _ => None
        }
    }
}

#[deriving(Show, RustcEncodable)]
pub struct MpdStatus {
    volume: uint,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    playlist: uint,
    playlistlength: uint,
    state: MpdState,
    song: Option<uint>,
    songid: Option<uint>,
    nextsong: Option<uint>,
    nextsongid: Option<uint>,
    time: Option<Duration>,
    elapsed: Option<Duration>,
    duration: Option<Duration>,
    bitrate: Option<uint>,
    xfade: Option<u64>,
    mixrampdb: f32,
    mixrampdelay: Option<u64>,
    audio: Option<AudioFormat>,
    updating_db: Option<uint>,
    error: Option<String>
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<MpdStatus> {
    fn from_iter<T: Iterator<MpdResult<MpdPair>>>(iterator: T) -> MpdResult<MpdStatus> {
        let mut status = MpdStatus {
            volume: 0,
            repeat: false,
            random: false,
            single: false,
            consume: false,
            playlist: 0,
            playlistlength: 0,
            state: MpdState::Stop,
            song: None,
            songid: None,
            nextsong: None,
            nextsongid: None,
            time: None,
            elapsed: None,
            duration: None,
            bitrate: None,
            xfade: None,
            mixrampdb: 0.0f32,
            mixrampdelay: None,
            audio: None,
            updating_db: None,
            error: None
        };

        let mut iter = iterator;

        for field in iter {
            let MpdPair(key, value) = try!(field);
            match key[] {
                "volume"         => status.volume = value.parse().unwrap_or(0),
                "repeat"         => status.repeat = value[] == "1",
                "random"         => status.random = value[] == "1",
                "single"         => status.single = value[] == "1",
                "consume"        => status.consume = value[] == "1",
                "playlist"       => status.playlist = value.parse().unwrap_or(0),
                "playlistlength" => status.playlistlength = value.parse().unwrap_or(0),
                "state"          => status.state = value.parse().unwrap_or(MpdState::Stop),
                "song"           => status.song = value.parse(),
                "songid"         => status.songid = value.parse(),
                "nextsong"       => status.nextsong = value.parse(),
                "nextsongid"     => status.nextsongid = value.parse(),
                "time"           => status.time = value.parse().map(Duration::seconds),
                "elapsed"        => status.elapsed = value.parse().map(Duration::seconds),
                "duration"       => status.duration = value.parse().map(Duration::seconds),
                "bitrate"        => status.bitrate = value.parse(),
                "xfade"          => status.xfade = value.parse(),
                "mixrampdb"      => status.mixrampdb = value.parse().unwrap_or(0f32),
                "mixrampdelay"   => status.mixrampdelay = value.parse(),
                "audio"          => status.audio = value.parse(),
                "updating_db"    => status.updating_db = value.parse(),
                "error"          => status.error = Some(value),
                _ => return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))
            }
        }

        Ok(status)
    }
}

