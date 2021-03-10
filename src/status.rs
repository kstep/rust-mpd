//! The module defines MPD status data structures

use crate::convert::FromIter;
use crate::error::{Error, ParseError};
use crate::song::{Id, QueuePlace};

use rustc_serialize::{Encodable, Encoder};
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// MPD status
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Status {
    /// volume (0-100, or -1 if volume is unavailable (e.g. for HTTPD output type)
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
    pub replaygain: Option<ReplayGain>,
}

impl Encodable for Status {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_struct("Status", 21, |e| {
            e.emit_struct_field("volume", 0, |e| self.volume.encode(e))?;
            e.emit_struct_field("repeat", 1, |e| self.repeat.encode(e))?;
            e.emit_struct_field("random", 2, |e| self.random.encode(e))?;
            e.emit_struct_field("single", 3, |e| self.single.encode(e))?;
            e.emit_struct_field("consume", 4, |e| self.consume.encode(e))?;
            e.emit_struct_field("queue_version", 5, |e| self.queue_version.encode(e))?;
            e.emit_struct_field("queue_len", 6, |e| self.queue_len.encode(e))?;
            e.emit_struct_field("state", 7, |e| self.state.encode(e))?;
            e.emit_struct_field("song", 8, |e| self.song.encode(e))?;
            e.emit_struct_field("nextsong", 9, |e| self.nextsong.encode(e))?;
            e.emit_struct_field("time", 10, |e| {
                e.emit_option(|e| match self.time {
                    Some(p) => e.emit_option_some(|e| {
                        e.emit_tuple(2, |e| {
                            e.emit_tuple_arg(0, |e| p.0.as_secs().encode(e))?;
                            e.emit_tuple_arg(1, |e| p.1.as_secs().encode(e))?;
                            Ok(())
                        })
                    }),
                    None => e.emit_option_none(),
                })
            })?;
            e.emit_struct_field("elapsed", 11, |e| {
                e.emit_option(|e| match self.elapsed {
                    Some(d) => e.emit_option_some(|e| d.as_secs().encode(e)),
                    None => e.emit_option_none(),
                })
            })?;
            e.emit_struct_field("duration", 12, |e| {
                e.emit_option(|e| match self.duration {
                    Some(d) => e.emit_option_some(|e| d.as_secs().encode(e)),
                    None => e.emit_option_none(),
                })
            })?;
            e.emit_struct_field("bitrate", 13, |e| self.bitrate.encode(e))?;
            e.emit_struct_field("crossfade", 14, |e| {
                e.emit_option(|e| match self.crossfade {
                    Some(d) => e.emit_option_some(|e| d.as_secs().encode(e)),
                    None => e.emit_option_none(),
                })
            })?;
            e.emit_struct_field("mixrampdb", 15, |e| self.mixrampdb.encode(e))?;
            e.emit_struct_field("mixrampdelay", 16, |e| {
                e.emit_option(|e| match self.mixrampdelay {
                    Some(d) => e.emit_option_some(|e| d.as_secs().encode(e)),
                    None => e.emit_option_none(),
                })
            })?;
            e.emit_struct_field("audio", 17, |e| self.audio.encode(e))?;
            e.emit_struct_field("updating_db", 18, |e| self.updating_db.encode(e))?;
            e.emit_struct_field("error", 19, |e| self.error.encode(e))?;
            e.emit_struct_field("replaygain", 20, |e| self.replaygain.encode(e))?;
            Ok(())
        })
    }
}

impl FromIter for Status {
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Status, Error> {
        let mut result = Status::default();

        for res in iter {
            let line = res?;
            match &*line.0 {
                "volume" => result.volume = line.1.parse()?,

                "repeat" => result.repeat = &*line.1 == "1",
                "random" => result.random = &*line.1 == "1",
                "single" => result.single = &*line.1 == "1",
                "consume" => result.consume = &*line.1 == "1",

                "playlist" => result.queue_version = line.1.parse()?,
                "playlistlength" => result.queue_len = line.1.parse()?,
                "state" => result.state = line.1.parse()?,
                "songid" => match result.song {
                    None => {
                        result.song = Some(QueuePlace {
                            id: Id(line.1.parse()?),
                            pos: 0,
                            prio: 0,
                        })
                    }
                    Some(ref mut place) => place.id = Id(line.1.parse()?),
                },
                "song" => match result.song {
                    None => {
                        result.song = Some(QueuePlace {
                            pos: line.1.parse()?,
                            id: Id(0),
                            prio: 0,
                        })
                    }
                    Some(ref mut place) => place.pos = line.1.parse()?,
                },
                "nextsongid" => match result.nextsong {
                    None => {
                        result.nextsong = Some(QueuePlace {
                            id: Id(line.1.parse()?),
                            pos: 0,
                            prio: 0,
                        })
                    }
                    Some(ref mut place) => place.id = Id(line.1.parse()?),
                },
                "nextsong" => match result.nextsong {
                    None => {
                        result.nextsong = Some(QueuePlace {
                            pos: line.1.parse()?,
                            id: Id(0),
                            prio: 0,
                        })
                    }
                    Some(ref mut place) => place.pos = line.1.parse()?,
                },
                "time" => {
                    let mut splits = line
                        .1
                        .splitn(2, ':')
                        .map(|v| v.parse().map_err(ParseError::BadInteger).map(Duration::from_secs));
                    result.time = match (splits.next(), splits.next()) {
                        (Some(Ok(a)), Some(Ok(b))) => Ok(Some((a, b))),
                        (Some(Err(e)), _) | (_, Some(Err(e))) => Err(e),
                        _ => Ok(None),
                    }?;
                }
                // TODO" => float errors don't work on stable
                "elapsed" => result.elapsed = line.1.parse::<f32>().ok().map(|v| Duration::from_millis((v * 1000.0) as u64)),
                "duration" => result.duration = line.1.parse::<f32>().ok().map(|v| Duration::from_millis((v * 1000.0) as u64)),
                "bitrate" => result.bitrate = Some(line.1.parse()?),
                "xfade" => result.crossfade = Some(Duration::from_secs(line.1.parse()?)),
                // "mixrampdb" => 0.0, //get_field!(map, "mixrampdb"),
                // "mixrampdelay" => None, //get_field!(map, opt "mixrampdelay").map(|v: f64| Duration::milliseconds((v * 1000.0) as i64)),
                "audio" => result.audio = Some(line.1.parse()?),
                "updating_db" => result.updating_db = Some(line.1.parse()?),
                "error" => result.error = Some(line.1.to_owned()),
                "replay_gain_mode" => result.replaygain = Some(line.1.parse()?),
                _ => (),
            }
        }

        Ok(result)
    }
}

/// Audio playback format
#[derive(Debug, Copy, Clone, PartialEq, RustcEncodable)]
pub struct AudioFormat {
    /// sample rate, kbps
    pub rate: u32,
    /// sample resolution in bits, can be 0 for floating point resolution
    pub bits: u8,
    /// number of channels
    pub chans: u8,
}

impl FromStr for AudioFormat {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<AudioFormat, ParseError> {
        let mut it = s.split(':');
        Ok(AudioFormat {
            rate: it
                .next()
                .ok_or(ParseError::NoRate)
                .and_then(|v| v.parse().map_err(ParseError::BadRate))?,
            bits: it.next().ok_or(ParseError::NoBits).and_then(|v| {
                if &*v == "f" {
                    Ok(0)
                } else {
                    v.parse().map_err(ParseError::BadBits)
                }
            })?,
            chans: it
                .next()
                .ok_or(ParseError::NoChans)
                .and_then(|v| v.parse().map_err(ParseError::BadChans))?,
        })
    }
}

/// Playback state
#[derive(Debug, Copy, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub enum State {
    /// player stopped
    Stop,
    /// player is playing
    Play,
    /// player paused
    Pause,
}

impl Default for State {
    fn default() -> State {
        State::Stop
    }
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
#[derive(Debug, Clone, Copy, PartialEq, RustcEncodable, RustcDecodable)]
pub enum ReplayGain {
    /// off
    Off,
    /// track
    Track,
    /// album
    Album,
    /// auto
    Auto,
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
            _ => Err(ParseError::BadValue(s.to_owned())),
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
