#![allow(missing_docs)]
// TODO: unfinished functionality

use std::fmt;
use std::borrow::Cow;
use std::convert::Into;
use std::ops::Range;
use time::Duration;
use client::Client;
use convert::{FromMap, FromIter, ToPlaylistName};
use proto::Proto;
use song::Song;
use error::{Result, Error};

/// Songs statistics for a search query
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Count {
    /// number of songs
    songs: usize,
    /// total play time of the songs
    playtime: Duration
}

impl FromIter for Count {
    /// build count from iterator
    fn from_iter<I: Iterator<Item = Result<(String, String)>>>(iter: I) -> Result<Count> {
        let mut count = Count {
            songs: 0,
            playtime: Duration::seconds(0)
        };

        for line in iter {
            let item = try!(line);
            match &*item.0 {
                "songs" => count.songs = try!(item.1.parse()),
                "playtime" => count.playtime = Duration::seconds(try!(item.1.parse())),
                _ => ()
            }
        }

        Ok(count)
    }
}

pub struct Query<'a> {
    filters: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    groups: Option<Vec<Cow<'a, str>>>,
    window: Option<(u32, u32)>,
}

impl<'a> Query<'a> {
    pub fn new() -> Query<'a> {
        Query {
            filters: Vec::new(),
            groups: None,
            window: None,
        }
    }

    pub fn and<'b: 'a, T, V>(&'a mut self, tag: T, value: V) -> &'a mut Query<'a> 
        where T: 'b + Into<Cow<'b, str>>,
              V: 'b + Into<Cow<'b, str>>
    {
        self.filters.push((tag.into(), value.into()));
        self
    }

    pub fn limit(&'a mut self, offset: u32, limit: u32) -> &'a mut Query<'a> {
        self.window = Some((offset, limit));
        self
    }

    pub fn range(&'a mut self, range: Range<u32>) -> &'a mut Query<'a> {
        self.limit(range.start, range.end)
    }

    pub fn group<'b: 'a, G: 'b + Into<Cow<'b, str>>>(&'a mut self, group: G) -> &'a mut Query<'a> {
        match self.groups {
            None => self.groups = Some(vec![group.into()]),
            Some(ref mut groups) => groups.push(group.into()),
        };
        self
    }
}

impl<'a> fmt::Display for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for &(ref tag, ref value) in &self.filters {
            try!(write!(f, " {} \"{}\"", tag, value));
        }

        if let Some(ref groups) = self.groups {
            for group in groups {
                try!(write!(f, " group {}", group));
            }
        }

        if let Some((a, b)) = self.window {
            try!(write!(f, " window {}:{}", a, b));
        }

        Ok(())
    }
}
