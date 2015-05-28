use time::{strptime, Duration, Tm};

use std::collections::BTreeMap;
use std::str::FromStr;
use std::convert::From;

use error::{Error, ParseError, ProtoError};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Id(pub u32);

#[derive(Debug, Copy, Clone)]
pub struct QueuePlace {
    pub id: Id,
    pub pos: u32,
    pub prio: u8
}

#[derive(Debug, Copy, Clone)]
pub struct Range(Duration, Option<Duration>);

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

#[derive(Debug)]
pub struct Song {
    pub file: String,
    pub last_mod: Tm,
    pub duration: Duration,
    pub place: Option<QueuePlace>,
    pub range: Option<Range>,
    pub tags: BTreeMap<String, String>,
}

impl Song {
    pub fn from_map(mut map: BTreeMap<String, String>) -> Result<Song, Error> {
        Ok(Song {
            file: try!(map.remove("file").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("file")))),
            last_mod: try!(map.remove("Last-Modified").ok_or(Error::Proto(ProtoError::NoField("Last-Modified")))
                           .and_then(|v| strptime(&*v, "%Y-%m-%dT%H:%M:%S%Z").map_err(From::from))),
            duration: try!(map.remove("Time").ok_or(Error::Proto(ProtoError::NoField("Time")))
                           .and_then(|v| v.parse().map(Duration::seconds).map_err(From::from))),
            range: try!(map.remove("Range").map(|v| v.parse().map(Some)).unwrap_or(Ok(None))),
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

