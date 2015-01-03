
use std::time::duration::Duration;
use std::io::{standard_error, IoErrorKind};
use std::collections::BTreeMap;
use time::{Timespec, strptime};
use rustc_serialize::{Encoder, Encodable};

use error::MpdResult;
use client::{MpdPair, FieldCutIter};

#[deriving(Show, Copy, RustcEncodable)]
pub struct MpdQueuePlace {
    pub id: uint,
    pub pos: uint,
    pub prio: uint
}

#[deriving(Show)]
pub struct MpdSong {
    pub file: String,
    pub last_mod: Timespec,
    pub duration: Duration,
    pub place: Option<MpdQueuePlace>,
    pub range: (Duration, Option<Duration>),
    pub tags: BTreeMap<String, String>,
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<MpdSong> {
    fn from_iter<T: Iterator<MpdResult<MpdPair>>>(iterator: T) -> MpdResult<MpdSong> {
        let mut song = MpdSong {
            file: "".to_string(),
            last_mod: Timespec::new(0, 0),
            duration: Duration::zero(),
            place: None,
            range: (Duration::zero(), None),
            tags: BTreeMap::new(),
        };
        let mut place = MpdQueuePlace {
            id: 0,
            pos: 0,
            prio: 0
        };

        let mut iter = iterator;

        for field in iter {
            let MpdPair(key, value) = try!(field);
            match key[] {
                "file" => song.file = value,
                "Last-Modified" => song.last_mod = try!(strptime(value[], "%Y-%m-%dT%H:%M:%S%Z").map_err(|e| standard_error(IoErrorKind::InvalidInput))).to_timespec(),
                "Time" => song.duration = Duration::seconds(value.parse().unwrap_or(0)),
                "Range" => {
                    let mut splits = value[].split('-').flat_map(|v| v.parse::<i64>().into_iter());
                    match (splits.next(), splits.next()) {
                        (Some(s), Some(e)) => song.range = (Duration::seconds(s), Some(Duration::seconds(e))),
                        (None, Some(e)) => song.range = (Duration::zero(), Some(Duration::seconds(e))),
                        (Some(s), None) => song.range = (Duration::seconds(s), None),
                        (None, None) => (),
                    }
                },
                "Id" => place.id = value.parse().unwrap_or(0),
                "Pos" => place.pos = value.parse().unwrap_or(0),
                "Prio" => place.prio = value.parse().unwrap_or(0),
                _ => { song.tags.insert(key, value); }
            }
        }

        if place.id > 0 {
            song.place = Some(place);
        }

        Ok(song)
    }
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<Vec<MpdSong>> {
    fn from_iter<T: Iterator<MpdResult<MpdPair>>>(iterator: T) -> MpdResult<Vec<MpdSong>> {
        let mut iter = iterator.fuse().peekable();
        let mut result = Vec::new();

        while !iter.is_empty() {
            let song = try!(FieldCutIter::new(&mut iter, "file").collect());
            result.push(song);
        }

        Ok(result)
    }
}

