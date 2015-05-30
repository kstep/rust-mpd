use std::collections::BTreeMap;
use std::str::FromStr;
use std::convert::From;
use time::Duration;

use error::{Error, ProtoError, ParseError};
use song::{Id, QueuePlace};

#[derive(Debug, PartialEq, Clone)]
pub struct Status {
    pub volume: i8,
    pub repeat: bool,
    pub random: bool,
    pub single: bool,
    pub consume: bool,
    pub queue_version: u32,
    pub queue_len: u32,
    pub state: State,
    pub song: Option<QueuePlace>,
    pub nextsong: Option<QueuePlace>,
    pub time: Option<(Duration, Duration)>,
    pub elapsed: Option<Duration>,
    pub duration: Option<Duration>,
    pub bitrate: Option<u32>,
    pub crossfade: Option<u64>,
    pub mixrampdb: f32,
    pub mixrampdelay: Option<Duration>,
    pub audio: Option<AudioFormat>,
    pub updating_db: Option<u32>,
    pub error: Option<String>
}

impl Status {
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Status, Error> {
        Ok(Status {
            volume: get_field!(map, "volume"),

            repeat: get_field!(map, bool "repeat"),
            random: get_field!(map, bool "random"),
            single: get_field!(map, bool "single"),
            consume: get_field!(map, bool "consume"),

            queue_version: get_field!(map, "playlist"),
            queue_len: get_field!(map, "playlistlength"),
            state: get_field!(map, "state"),
            song: try!(map.get("song").map(|v| v.parse().map_err(ParseError::BadInteger)).and_then(|posres|
                  map.get("songid").map(|v| v.parse().map_err(ParseError::BadInteger)).map(|idres|
                    posres.and_then(|pos| idres.map(|id| Some(QueuePlace {
                      id: Id(id),
                      pos: pos,
                      prio: 0
                    }))))).unwrap_or(Ok(None))),
            nextsong: try!(map.get("nextsong").map(|v| v.parse().map_err(ParseError::BadInteger)).and_then(|posres|
                  map.get("nextsongid").map(|v| v.parse().map_err(ParseError::BadInteger)).map(|idres|
                    posres.and_then(|pos| idres.map(|id| Some(QueuePlace {
                      id: Id(id),
                      pos: pos,
                      prio: 0
                    }))))).unwrap_or(Ok(None))),
            time: try!(map.get("time").map(|time| {
                let mut splits = time.splitn(2, ':').map(|v| v.parse().map_err(ParseError::BadInteger).map(Duration::seconds));
                match (splits.next(), splits.next()) {
                    (Some(Ok(a)), Some(Ok(b))) => Ok(Some((a, b))),
                    (Some(Err(e)), _) | (_, Some(Err(e))) => Err(e),
                    _ => Ok(None)
                }
            }).unwrap_or(Ok(None))),
            // TODO: float errors don't work on stable
            elapsed: map.get("elapsed").and_then(|f| f.parse::<f32>().ok()).map(|v| Duration::milliseconds((v * 1000.0) as i64)),
            duration: get_field!(map, opt "duration").map(Duration::seconds),
            bitrate: get_field!(map, opt "bitrate"),
            crossfade: get_field!(map, opt "xfade"),
            mixrampdb: 0.0, //get_field!(map, "mixrampdb"),
            mixrampdelay: None, //get_field!(map, opt "mixrampdelay").map(|v: f64| Duration::milliseconds((v * 1000.0) as i64)),
            audio: get_field!(map, opt "audio"),
            updating_db: get_field!(map, opt "updating_db"),
            error: map.get("error").map(|v| v.to_owned()),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AudioFormat {
    pub rate: u32,
    pub bits: u8,
    pub chans: u8
}

impl FromStr for AudioFormat {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<AudioFormat, ParseError> {
        let mut it = s.split(':');
        Ok(AudioFormat {
            rate: try!(it.next().ok_or(ParseError::NoRate).and_then(|v| v.parse().map_err(ParseError::BadRate))),
            bits: try!(it.next().ok_or(ParseError::NoBits).and_then(|v| if &*v == "f" { Ok(0) } else { v.parse().map_err(ParseError::BadBits) })),
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
