//! The module describes DB and playback statistics

use crate::convert::FromIter;
use crate::error::Error;

use rustc_serialize::{Encodable, Encoder};
use std::time::Duration;

/// DB and playback statistics
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    /// number of artists in DB
    pub artists: u32,
    /// number of albums in DB
    pub albums: u32,
    /// number of songs in DB
    pub songs: u32,
    /// total MPD uptime, seconds resolution
    pub uptime: Duration,
    /// total playback time, seconds resolution
    pub playtime: Duration,
    /// total playback time for all songs in DB, seconds resolution
    pub db_playtime: Duration,
    /// last DB update timestamp in seconds since Epoch, seconds resolution
    pub db_update: Duration,
}

impl Encodable for Stats {
    fn encode<S: Encoder>(&self, e: &mut S) -> Result<(), S::Error> {
        e.emit_struct("Stats", 7, |e| {
            e.emit_struct_field("artists", 0, |e| self.artists.encode(e))?;
            e.emit_struct_field("albums", 1, |e| self.albums.encode(e))?;
            e.emit_struct_field("songs", 2, |e| self.songs.encode(e))?;
            e.emit_struct_field("uptime", 3, |e| self.uptime.as_secs().encode(e))?;
            e.emit_struct_field("playtime", 4, |e| self.playtime.as_secs().encode(e))?;
            e.emit_struct_field("db_playtime", 5, |e| self.db_playtime.as_secs().encode(e))?;
            e.emit_struct_field("db_update", 6, |e| self.db_update.as_secs().encode(e))?;
            Ok(())
        })
    }
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            artists: 0,
            albums: 0,
            songs: 0,
            uptime: Duration::from_secs(0),
            playtime: Duration::from_secs(0),
            db_playtime: Duration::from_secs(0),
            db_update: Duration::from_secs(0),
        }
    }
}

impl FromIter for Stats {
    /// build stats from iterator
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Stats, Error> {
        let mut result = Stats::default();

        for res in iter {
            let line = res?;
            match &*line.0 {
                "artists" => result.artists = line.1.parse()?,
                "albums" => result.albums = line.1.parse()?,
                "songs" => result.songs = line.1.parse()?,
                "uptime" => result.uptime = Duration::from_secs(line.1.parse()?),
                "playtime" => result.playtime = Duration::from_secs(line.1.parse()?),
                "db_playtime" => result.db_playtime = Duration::from_secs(line.1.parse()?),
                "db_update" => result.db_update = Duration::from_secs(line.1.parse()?),
                _ => (),
            }
        }

        Ok(result)
    }
}
