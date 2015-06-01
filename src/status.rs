//! The module defines MPD status data structures

use std::collections::BTreeMap;
use std::str::FromStr;
use std::convert::From;
use std::fmt;
use time::Duration;

use error::{Error, ProtoError, ParseError};
use song::{Id, QueuePlace};

/// MPD status
#[derive(Debug, PartialEq, Clone)]
pub struct Status {
    /// volume (0-100, or -1 if volume is unavailable (e.g. for HTTPD ouput type)
    pub volume: i8,
    /// repeat mode
    pub repeat: bool,
    /// random mode
    pub random: bool,
    /// single mode
    pub single: bool,
    /// consume mode
    pub consume: bool,
    /// queue version number
    pub queue_version: u32,
    /// queue length
    pub queue_len: u32,
    /// playback state
    pub state: State,
    /// currently playing song place in the queue
    pub song: Option<QueuePlace>,
    /// next song to play place in the queue
    pub nextsong: Option<QueuePlace>,
    /// time current song played, and total song duration (in seconds resolution)
    pub time: Option<(Duration, Duration)>,
    /// elapsed play time current song played (in milliseconds resolution)
    pub elapsed: Option<Duration>,
    /// current song duration
    pub duration: Option<Duration>,
    /// current song bitrate, kbps
    pub bitrate: Option<u32>,
    /// crossfade timeout, seconds
    pub crossfade: Option<Duration>,
    /// mixramp threshold, dB
    pub mixrampdb: f32,
    /// mixramp duration, seconds
    pub mixrampdelay: Option<Duration>,
    /// current audio playback format
    pub audio: Option<AudioFormat>,
    /// current DB updating job number (if DB updating is in progress)
    pub updating_db: Option<u32>,
    /// last player error (if happened, can be reset with `clearerror()` method)
    pub error: Option<String>,
    /// replay gain mode
    pub replaygain: Option<ReplayGain>
}

impl Status {
    /// build status from map
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
            crossfade: get_field!(map, opt "xfade").map(Duration::seconds),
            mixrampdb: 0.0, //get_field!(map, "mixrampdb"),
            mixrampdelay: None, //get_field!(map, opt "mixrampdelay").map(|v: f64| Duration::milliseconds((v * 1000.0) as i64)),
            audio: get_field!(map, opt "audio"),
            updating_db: get_field!(map, opt "updating_db"),
            error: map.get("error").map(|v| v.to_owned()),
            replaygain: get_field!(map, opt "replay_gain_mode"),
        })
    }
}

/// Audio playback format
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AudioFormat {
    /// sample rate, kbps
    pub rate: u32,
    /// sample resolution in bits, can be 0 for floating point resolution
    pub bits: u8,
    /// number of channels
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

/// Playback state
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum State {
    /// player stopped
    Stop,
    /// player is playing
    Play,
    /// player paused
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

/// Replay gain mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplayGain {
    /// off
    Off,
    /// track
    Track,
    /// album
    Album,
    /// auto
    Auto
}

impl FromStr for ReplayGain {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<ReplayGain, ParseError> {
        use self::ReplayGain::*;
        match s {
            "off" => Ok(Off),
            "track" => Ok(Track),
            "album" => Ok(Album),
            "auto" => Ok(Auto),
            _ => Err(ParseError::BadValue(s.to_owned()))
        }
    }
}

impl fmt::Display for ReplayGain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ReplayGain::*;
        f.write_str(match *self {
            Off => "off",
            Track => "track",
            Album => "album",
            Auto => "auto",
        })
    }
}
