use std::time::duration::Duration;
use std::str::FromStr;
use std::old_io::{standard_error, IoErrorKind};
use std::error::Error;
use std::convert::From;
use std::iter::FromIterator;
use std::fmt;
use rustc_serialize::{Encoder, Encodable};

use error::MpdResult;
use songs::MpdQueuePlace;
use client::MpdPair;
use utils::ForceEncodable;

#[derive(Debug, Copy, RustcEncodable)]
pub struct AudioFormat {
    pub rate: u32,
    pub bits: u8,
    pub chans: u8
}

impl FromStr for AudioFormat {
    fn from_str(s: &str) -> Option<AudioFormat> {
        let mut it = s.split(':');
        if let (Some(rate), Some(bits), Some(chans)) = (
            it.next().and_then(|v| v.parse().ok()),
            it.next().and_then(|v| v.parse().ok()),
            it.next().and_then(|v| v.parse().ok())) {
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

#[derive(Debug, Copy, RustcEncodable)]
pub enum MpdState {
    Stop,
    Play,
    Pause,
}

#[derive(Debug, Copy)]
struct MpdStateParseError;

impl Error for MpdStateParseError {
    fn description(&self) -> &str {
        "state must be `play`, `stop` or `pause`"
    }
}

impl fmt::String for MpdStateParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl FromStr for MpdState {
    type Err = MpdStateParseError;
    fn from_str(s: &str) -> Result<MpdState, MpdStateParseError> {
        match s {
            "stop" => Ok(MpdState::Stop),
            "play" => Ok(MpdState::Play),
            "pause" => Ok(MpdState::Pause),
            _ => Err(MpdStateParseError)
        }
    }
}

#[derive(Debug, RustcEncodable)]
pub struct MpdStatus {
    volume: usize,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    queue_version: usize,
    queue_len: usize,
    state: MpdState,
    song: Option<MpdQueuePlace>,
    nextsong: Option<MpdQueuePlace>,
    play_time: Option<Duration>,
    total_time: Option<Duration>,
    elapsed: Option<Duration>,
    duration: Option<Duration>,
    bitrate: Option<usize>,
    crossfade: Option<u64>,
    mixrampdb: f32,
    mixrampdelay: Option<u64>,
    audio: Option<AudioFormat>,
    updating_db: Option<usize>,
    error: Option<String>
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<MpdStatus> {
    fn from_iter<T: Iterator<Item=MpdResult<MpdPair>>>(iterator: T) -> MpdResult<MpdStatus> {
        let mut status = MpdStatus {
            volume: 0,
            repeat: false,
            random: false,
            single: false,
            consume: false,
            queue_version: 0,
            queue_len: 0,
            state: MpdState::Stop,
            song: None,
            nextsong: None,
            play_time: None,
            total_time: None,
            elapsed: None,
            duration: None,
            bitrate: None,
            crossfade: None,
            mixrampdb: 0.0f32,
            mixrampdelay: None,
            audio: None,
            updating_db: None,
            error: None
        };

        let mut iter = iterator;
        let mut song_place = MpdQueuePlace { id: 0, pos: 0, prio: 0 };
        let mut next_song_place = MpdQueuePlace { id: 0, pos: 0, prio: 0 };

        for field in iter {
            let MpdPair(key, value) = try!(field);
            match &*key {
                "volume"         => status.volume = value.parse().unwrap_or(0),
                "repeat"         => status.repeat = &*value == "1",
                "random"         => status.random = &*value == "1",
                "single"         => status.single = &*value == "1",
                "consume"        => status.consume = &*value == "1",
                "playlist"       => status.queue_version = value.parse().unwrap_or(0),
                "playlistlength" => status.queue_len = value.parse().unwrap_or(0),
                "state"          => status.state = value.parse().unwrap_or(MpdState::Stop),
                "song"           => song_place.pos = value.parse().unwrap_or(0),
                "songid"         => song_place.id = value.parse().unwrap_or(0),
                "nextsong"       => next_song_place.pos = value.parse().unwrap_or(0),
                "nextsongid"     => next_song_place.id = value.parse().unwrap_or(0),
                "time"           => {
                    let mut splits = value.splitn(2, ':').flat_map(|v| v.parse::<i64>().into_iter()).map(Duration::seconds);
                    status.play_time = splits.next();
                    status.total_time = splits.next();
                },
                "elapsed"        => status.elapsed = value.parse::<f32>().map(|v| Duration::milliseconds((v * 1000f32) as i64)).ok(),
                "duration"       => status.duration = value.parse().map(Duration::seconds).ok(),
                "bitrate"        => status.bitrate = value.parse().ok(),
                "xfade"          => status.crossfade = value.parse().ok(),
                "mixrampdb"      => status.mixrampdb = value.parse().unwrap_or(0f32),
                "mixrampdelay"   => status.mixrampdelay = value.parse().ok(),
                "audio"          => status.audio = value.parse().ok(),
                "updating_db"    => status.updating_db = value.parse().ok(),
                "error"          => status.error = Some(value),
                _ => return Err(From::from(standard_error(IoErrorKind::InvalidInput)))
            }
        }

        if song_place.id > 0 {
            status.song = Some(song_place);
        }
        if next_song_place.id > 0 {
            status.nextsong = Some(next_song_place);
        }

        Ok(status)
    }
}

