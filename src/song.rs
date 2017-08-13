//! The module defines song structs and methods.

use convert::FromIter;

use error::{Error, ParseError};
use rustc_serialize::{Decodable, Decoder, Encodable, Encoder};

use std::collections::BTreeMap;
use std::fmt;
use std::str::FromStr;
use time::{Duration, Tm, strptime};

/// Song ID
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Default)]
pub struct Id(pub u32);

impl Encodable for Id {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        self.0.encode(e)
    }
}

impl Decodable for Id {
    fn decode<S: Decoder>(d: &mut S) -> Result<Id, S::Error> {
        d.read_u32().map(Id)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Song place in the queue
#[derive(Debug, Copy, Clone, PartialEq, Default, RustcEncodable)]
pub struct QueuePlace {
    /// song ID
    pub id: Id,
    /// absolute zero-based song position
    pub pos: u32,
    /// song priority, if present, defaults to 0
    pub prio: u8,
}

/// Song range
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Range(pub Duration, pub Option<Duration>);

impl Encodable for Range {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_tuple(2, |e| {
            e.emit_tuple_arg(0, |e| e.emit_i64(self.0.num_seconds()))?;
            e.emit_tuple_arg(1, |e| {
                e.emit_option(|e| match self.1 {
                                  Some(d) => e.emit_option_some(|e| d.num_seconds().encode(e)),
                                  None => e.emit_option_none(),
                              })
            })
        })
    }
}

impl Default for Range {
    fn default() -> Range {
        Range(Duration::seconds(0), None)
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.num_seconds().fmt(f)?;
        f.write_str(":")?;
        if let Some(v) = self.1 {
            v.num_seconds().fmt(f)?;
        }
        Ok(())
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
    /// title
    pub title: Option<String>,
    /// last modification time
    pub last_mod: Option<Tm>,
    /// artist
    pub artist: Option<String>,
    /// duration (in seconds resolution)
    pub duration: Option<Duration>,
    /// place in the queue (if queued for playback)
    pub place: Option<QueuePlace>,
    /// range to play (if queued for playback and range was set)
    pub range: Option<Range>,
    /// arbitrary tags, like album, artist etc
    pub tags: BTreeMap<String, String>,
}

impl Encodable for Song {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_struct("Song", 8, |e| {
            e.emit_struct_field("file", 0, |e| self.file.encode(e))?;
            e.emit_struct_field("name", 1, |e| self.name.encode(e))?;
            e.emit_struct_field("title", 2, |e| self.title.encode(e))?;
            e.emit_struct_field("last_mod", 3, |e| {
                    e.emit_option(|e| match self.last_mod {
                                      Some(m) => e.emit_option_some(|e| m.to_timespec().sec.encode(e)),
                                      None => e.emit_option_none(),
                                  })
                })?;
            e.emit_struct_field("artist", 4, |e| self.artist.encode(e))?;
            e.emit_struct_field("duration", 5, |e| {
                    e.emit_option(|e| match self.duration {
                                      Some(d) => e.emit_option_some(|e| d.num_seconds().encode(e)),
                                      None => e.emit_option_none(),
                                  })
                })?;
            e.emit_struct_field("place", 6, |e| self.place.encode(e))?;
            e.emit_struct_field("range", 7, |e| self.range.encode(e))?;
            e.emit_struct_field("tags", 8, |e| self.tags.encode(e))?;
            Ok(())
        })
    }
}

impl FromIter for Song {
    /// build song from map
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Song, Error> {
        let mut result = Song::default();

        for res in iter {
            let line = try!(res);
            match &*line.0 {
                "file" => result.file = line.1.to_owned(),
                "Title" => result.title = Some(line.1.to_owned()),
                "Last-Modified" => result.last_mod = try!(strptime(&*line.1, "%Y-%m-%dT%H:%M:%S%Z").map_err(ParseError::BadTime).map(Some)),
                "Artist" => result.artist = Some(line.1.to_owned()),
                "Name" => result.name = Some(line.1.to_owned()),
                "Time" => result.duration = Some(Duration::seconds(try!(line.1.parse()))),
                "Range" => result.range = Some(try!(line.1.parse())),
                "Id" => {
                    match result.place {
                        None => {
                            result.place = Some(QueuePlace {
                                                    id: Id(try!(line.1.parse())),
                                                    pos: 0,
                                                    prio: 0,
                                                })
                        }
                        Some(ref mut place) => place.id = Id(try!(line.1.parse())),
                    }
                }
                "Pos" => {
                    match result.place {
                        None => {
                            result.place = Some(QueuePlace {
                                                    pos: try!(line.1.parse()),
                                                    id: Id(0),
                                                    prio: 0,
                                                })
                        }
                        Some(ref mut place) => place.pos = try!(line.1.parse()),
                    }
                }
                "Prio" => {
                    match result.place {
                        None => {
                            result.place = Some(QueuePlace {
                                                    prio: try!(line.1.parse()),
                                                    id: Id(0),
                                                    pos: 0,
                                                })
                        }
                        Some(ref mut place) => place.prio = try!(line.1.parse()),
                    }
                }
                _ => {
                    result.tags.insert(line.0, line.1);
                }
            }
        }

        Ok(result)
    }
}
