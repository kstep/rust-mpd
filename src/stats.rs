use time::{Duration, Timespec};

use std::collections::BTreeMap;
use std::convert::From;

use error::{Error, ProtoError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub artists: u32,
    pub albums: u32,
    pub songs: u32,
    pub uptime: Duration,
    pub playtime: Duration,
    pub db_playtime: Duration,
    pub db_update: Timespec,
}

impl Stats {
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Stats, Error> {
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
