use std::time::duration::Duration;
use std::str::FromStr;
use std::io::{standard_error, IoErrorKind};
use std::error::FromError;
use std::iter::FromIterator;
use rustc_serialize::{Encoder, Encodable};

use error::MpdResult;
use songs::MpdQueuePlace;
use client::MpdPair;
use utils::ForceEncodable;

#[derive(Show, Copy, RustcEncodable)]
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

#[derive(Show, Copy, RustcEncodable)]
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

#[derive(Show, RustcEncodable)]
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
                "elapsed"        => status.elapsed = value.parse::<f32>().map(|v| Duration::milliseconds((v * 1000f32) as i64)),
                "duration"       => status.duration = value.parse().map(Duration::seconds),
                "bitrate"        => status.bitrate = value.parse(),
                "xfade"          => status.crossfade = value.parse(),
                "mixrampdb"      => status.mixrampdb = value.parse().unwrap_or(0f32),
                "mixrampdelay"   => status.mixrampdelay = value.parse(),
                "audio"          => status.audio = value.parse(),
                "updating_db"    => status.updating_db = value.parse(),
                "error"          => status.error = Some(value),
                _ => return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))
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

