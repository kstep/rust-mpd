//! The module defines song structs and methods.

use time::{strptime, Duration, Tm};

use std::collections::BTreeMap;
use std::str::FromStr;
use std::fmt;

use error::{Error, ParseError};
use convert::FromIter;

/// Song ID
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Id(pub u32);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Song place in the queue
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct QueuePlace {
    /// song ID
    pub id: Id,
    /// absolute zero-based song position
    pub pos: u32,
    /// song priority, if present, defaults to 0
    pub prio: u8
}

/// Song range
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Range(pub Duration, pub Option<Duration>);

impl Default for Range {
    fn default() -> Range {
        Range(Duration::seconds(0), None)
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.num_seconds().fmt(f)
            .and_then(|_| f.write_str(":"))
            .and_then(|_| self.1.map(|v| v.num_seconds().fmt(f)).unwrap_or(Ok(())))
    }
}

impl FromStr for Range {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Range, ParseError> {
        let mut splits = s.split('-').flat_map(|v| v.parse().into_iter());
        match (splits.next(), splits.next()) {
            (Some(s), Some(e)) => Ok(Range(Duration::seconds(s), Some(Duration::seconds(e)))),
            (None, Some(e)) => Ok(Range(Duration::zero(), Some(Duration::seconds(e)))),
            (Some(s), None) => Ok(Range(Duration::seconds(s), None)),
            (None, None) => Ok(Range(Duration::zero(), None)),
        }
    }
}

/// Song data
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Song {
    /// filename
    pub file: String,
    /// name (for streams)
    pub name: Option<String>,
    /// last modification time
    pub last_mod: Option<Tm>,
    /// duration (in seconds resolution)
    pub duration: Option<Duration>,
    /// place in the queue (if queued for playback)
    pub place: Option<QueuePlace>,
    /// range to play (if queued for playback and range was set)
    pub range: Option<Range>,
    /// arbitrary tags, like album, artist etc
    pub tags: BTreeMap<String, String>,
}

impl FromIter for Song {
    /// build song from map
    fn from_iter<I: Iterator<Item=Result<(String, String), Error>>>(iter: I) -> Result<Song, Error> {
        let mut result = Song::default();

        for res in iter {
            let line = try!(res);
            match &*line.0 {
                "file" => result.file = line.1.to_owned(),
                "Last-Modified" => result.last_mod = try!(strptime(&*line.1, "%Y-%m-%dT%H:%M:%S%Z").map_err(ParseError::BadTime).map(Some)),
                "Name" => result.name = Some(line.1.to_owned()),
                "Time" => result.duration = Some(Duration::seconds(try!(line.1.parse()))),
                "Range" => result.range = Some(try!(line.1.parse())),
                "Id" => match result.place {
                    None => result.place = Some(QueuePlace { id: Id(try!(line.1.parse())), pos: 0, prio: 0 }),
                    Some(ref mut place) => place.id = Id(try!(line.1.parse())),
                },
                "Pos" => match result.place {
                    None => result.place = Some(QueuePlace { pos: try!(line.1.parse()), id: Id(0), prio: 0 }),
                    Some(ref mut place) => place.pos = try!(line.1.parse()),
                },
                "Prio" => match result.place {
                    None => result.place = Some(QueuePlace { prio: try!(line.1.parse()), id: Id(0), pos: 0 }),
                    Some(ref mut place) => place.prio = try!(line.1.parse()),
                },
                _ => { result.tags.insert(line.0, line.1); }
            }
        }

        Ok(result)
    }
}

