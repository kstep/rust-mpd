//! The module defines song structs and methods.

use crate::convert::FromIter;
use crate::error::{Error, ParseError};

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// Song ID
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Id(pub u32);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        Ok(Id(u32::deserialize(deserializer)?))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_u32(self.0)
    }
}

/// Song place in the queue
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct QueuePlace {
    /// song ID
    pub id: Id,
    /// absolute zero-based song position
    pub pos: u32,
    /// song priority, if present, defaults to 0
    pub prio: u8,
}

/// Song range
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Range(pub Duration, pub Option<Duration>);

impl Default for Range {
    fn default() -> Range {
        Range(Duration::from_secs(0), None)
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.as_secs().fmt(f)?;
        f.write_str(":")?;
        if let Some(v) = self.1 {
            v.as_secs().fmt(f)?;
        }
        Ok(())
    }
}

impl FromStr for Range {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Range, ParseError> {
        let mut splits = s.split('-').flat_map(|v| v.parse().into_iter());
        match (splits.next(), splits.next()) {
            (Some(s), Some(e)) => Ok(Range(Duration::from_secs(s), Some(Duration::from_secs(e)))),
            (None, Some(e)) => Ok(Range(Duration::from_secs(0), Some(Duration::from_secs(e)))),
            (Some(s), None) => Ok(Range(Duration::from_secs(s), None)),
            (None, None) => Ok(Range(Duration::from_secs(0), None)),
        }
    }
}

/// Song data
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Song {
    /// filename
    pub file: String,
    /// name (for streams)
    pub name: Option<String>,
    /// title
    pub title: Option<String>,
    /// last modification time
    pub last_mod: Option<String>,
    /// artist
    pub artist: Option<String>,
    /// duration (in seconds resolution)
    pub duration: Option<Duration>,
    /// place in the queue (if queued for playback)
    pub place: Option<QueuePlace>,
    /// range to play (if queued for playback and range was set)
    pub range: Option<Range>,
    /// arbitrary tags, like album, artist etc
    pub tags: Vec<(String, String)>,
}

impl FromIter for Song {
    /// build song from map
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Song, Error> {
        let mut result = Song::default();

        for res in iter {
            let line = res?;
            match &*line.0 {
                "file" => result.file = line.1.to_owned(),
                "Title" => result.title = Some(line.1.to_owned()),
                "Last-Modified" => result.last_mod = Some(line.1.to_owned()),
                "Artist" => result.artist = Some(line.1.to_owned()),
                "Name" => result.name = Some(line.1.to_owned()),
                // Deprecated in MPD.
                "Time" => (),
                "duration" => result.duration = Some(Duration::try_from_secs_f64(line.1.parse()?)?),
                "Range" => result.range = Some(line.1.parse()?),
                "Id" => match result.place {
                    None => result.place = Some(QueuePlace { id: Id(line.1.parse()?), pos: 0, prio: 0 }),
                    Some(ref mut place) => place.id = Id(line.1.parse()?),
                },
                "Pos" => match result.place {
                    None => result.place = Some(QueuePlace { pos: line.1.parse()?, id: Id(0), prio: 0 }),
                    Some(ref mut place) => place.pos = line.1.parse()?,
                },
                "Prio" => match result.place {
                    None => result.place = Some(QueuePlace { prio: line.1.parse()?, id: Id(0), pos: 0 }),
                    Some(ref mut place) => place.prio = line.1.parse()?,
                },
                _ => {
                    result.tags.push((line.0, line.1));
                }
            }
        }

        Ok(result)
    }
}
