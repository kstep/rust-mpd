//! The module describes DB and playback statistics

use time::{Duration, Timespec};

use std::collections::BTreeMap;
use std::convert::From;

use error::{Error, ProtoError};
use convert::FromMap;

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
    /// last DB update timestamp, seconds resolution
    pub db_update: Timespec,
}

impl FromMap for Stats {
    /// build stats from map
    fn from_map(map: BTreeMap<String, String>) -> Result<Stats, Error> {
        Ok(Stats {
            artists: get_field!(map, "artists"),
            albums: get_field!(map, "albums"),
            songs: get_field!(map, "songs"),
            uptime: Duration::seconds(get_field!(map, "uptime")),
            playtime: Duration::seconds(get_field!(map, "playtime")),
            db_playtime: Duration::seconds(get_field!(map, "db_playtime")),
            db_update: Timespec::new(get_field!(map, "db_update"), 0),
        })
    }
}
