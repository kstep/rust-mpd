use std::time::duration::Duration;
use std::io::{standard_error, IoErrorKind};
use std::iter::FromIterator;
use std::error::FromError;
use time::Timespec;
use rustc_serialize::{Encoder, Encodable};

use error::MpdResult;
use client::MpdPair;
use utils::ForceEncodable;

#[derive(Show, Copy, RustcEncodable)]
pub struct MpdStats {
    uptime: Duration,
    playtime: Duration,
    artists: uint,
    albums: uint,
    songs: uint,
    db_playtime: Duration,
    db_update: Timespec
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<MpdStats> {
    fn from_iter<T: Iterator<Item=MpdResult<MpdPair>>>(iterator: T) -> MpdResult<MpdStats> {
        let mut stats = MpdStats {
            uptime: Duration::zero(),
            playtime: Duration::zero(),
            artists: 0,
            albums: 0,
            songs: 0,
            db_playtime: Duration::zero(),
            db_update: Timespec::new(0, 0)
        };

        let mut iter = iterator;

        for field in iter {
            let MpdPair(key, value) = try!(field);
            match key[] {
                "uptime" => stats.uptime = Duration::seconds(value.parse().unwrap_or(0)),
                "playtime" => stats.playtime = Duration::seconds(value.parse().unwrap_or(0)),
                "artists" => stats.artists = value.parse().unwrap_or(0),
                "albums" => stats.albums = value.parse().unwrap_or(0),
                "songs" => stats.songs = value.parse().unwrap_or(0),
                "db_playtime" => stats.db_playtime = Duration::seconds(value.parse().unwrap_or(0)),
                "db_update" => stats.db_update = Timespec::new(value.parse().unwrap_or(0), 0),
                _ => return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))
            }
        }

        Ok(stats)
    }
}
