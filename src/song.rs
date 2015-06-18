//! The module defines song structs and methods.

use time::{strptime, Duration, Tm};

use std::collections::BTreeMap;
use std::str::FromStr;
use std::fmt;

use error::{Error, ParseError, ProtoError};
use convert::FromMap;

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
#[derive(Debug, Clone, PartialEq)]
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

impl FromMap for Song {
    /// build song from map
    fn from_map(mut map: BTreeMap<String, String>) -> Result<Song, Error> {
        Ok(Song {
            file: try!(map.remove("file").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("file")))),
            last_mod: try!(map.remove("Last-Modified")
                           .map(|v| strptime(&*v, "%Y-%m-%dT%H:%M:%S%Z").map_err(ParseError::BadTime).map(Some))
                           .unwrap_or(Ok(None))),
            name: map.remove("Name").map(|v| v.to_owned()),
            duration: pop_field!(map, opt "Time").map(Duration::seconds),
            range: pop_field!(map, opt "Range"),
            place: {
                if let (Some(id), Some(pos)) = (map.remove("Id"), map.remove("Pos")) {
                    Some(QueuePlace {
                        id: Id(try!(id.parse())),
                        pos: try!(pos.parse()),
                        prio: try!(map.remove("Prio").map(|v| v.parse()).unwrap_or(Ok(0)))
                    })
                } else {
                    None
                }
            },
            tags: map
        })
    }
}

