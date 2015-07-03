#![allow(missing_docs)]
// TODO: unfinished functionality

use std::fmt;
use std::io::{Read, Write};
use std::borrow::Cow;
use std::convert::Into;
use client::Client;
use convert::{FromMap, ToPlaylistName};
use proto::Proto;
use song::Song;
use error::Result;

pub enum Term<'a> {
    Any,
    File,
    Base,
    LastMod,
    Tag(Cow<'a, str>)
}

pub struct Filter<'a> {
    typ: Term<'a>,
    what: Cow<'a, str>
}

impl<'a> Filter<'a> {
    fn new<W>(typ: Term<'a>, what: W) -> Filter where W: 'a + Into<Cow<'a, str>> {
        Filter {
            typ: typ,
            what: what.into()
        }
    }
}

pub struct Query<'a, S: 'a + Read + Write> {
    client: &'a mut Client<S>,
    filters: Vec<Filter<'a>>,
    groups: Option<Vec<Cow<'a, str>>>,
    window: Option<(u32, u32)>,
}

impl<'a, S: 'a + Read + Write> Query<'a, S> {
    pub fn new(client: &'a mut Client<S>) -> Query<'a, S> {
        Query {
            client: client,
            filters: Vec::new(),
            groups: None,
            window: None
        }
    }

    pub fn and<'b: 'a, V: 'b + Into<Cow<'b, str>>>(&'a mut self, term: Term<'b>, value: V) -> &'a mut Query<'a, S> {
        self.filters.push(Filter::new(term, value));
        self
    }

    pub fn limit(&'a mut self, offset: u32, limit: u32) -> &'a mut Query<'a, S> {
        self.window = Some((offset, limit));
        self
    }

    pub fn group<'b: 'a, G: 'b + Into<Cow<'b, str>>>(&'a mut self, group: G) -> &'a mut Query<'a, S> {
        match self.groups {
            None => self.groups = Some(vec![group.into()]),
            Some(ref mut groups) => groups.push(group.into())
        };
        self
    }

    pub fn find(mut self, fuzzy: bool, add: bool) -> Result<Vec<Song>> {
        let cmd = if fuzzy {
            if add {
                "searchadd"
            } else {
                "search"
            }
        } else {
            if add {
                "findadd"
            } else {
                "find"
            }
        };
        let args = self.to_string();

        self.client.run_command_fmt(format_args!("{} {}", cmd, args))
            .and_then(|_| self.client
                      .read_pairs()
                      .split("file")
                      .map(|v| v.and_then(FromMap::from_map))
                      .collect())
    }

    pub fn find_add<N: ToPlaylistName>(mut self, playlist: N) -> Result<()> {
        let args = self.to_string();
        self.client.run_command_fmt(format_args!("searchaddpl {} {}", playlist.to_name(), args))
            .and_then(|_| self.client.expect_ok())
    }

    //pub fn list(mut self, ty: &str) -> Result<Vec<???>> {
    //}
}

impl<'a> fmt::Display for Term<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Term::Any => f.write_str("any"),
            Term::File => f.write_str("file"),
            Term::Base => f.write_str("base"),
            Term::LastMod => f.write_str("modified-since"),
            Term::Tag(ref tag) => f.write_str(&*tag)
        }
    }
}

impl<'a> fmt::Display for Filter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.typ, self.what)
    }
}

impl<'a, S: 'a + Read + Write> fmt::Display for Query<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for filter in &self.filters {
            try!(filter.fmt(f));
        }

        if let Some(ref groups) = self.groups {
            for group in groups {
                try!(write!(f, "group {}", group));
            }
        }

        match self.window {
            Some((a, b)) => write!(f, " window {}:{}", a, b),
            None => Ok(())
        }
    }
}
