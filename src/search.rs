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
    Tag(Cow<'a, str>),
}

pub struct Filter<'a> {
    typ: Term<'a>,
    what: Cow<'a, str>,
}

impl<'a> Filter<'a> {
    fn new<W>(typ: Term<'a>, what: W) -> Filter
        where W: 'a + Into<Cow<'a, str>>
    {
        Filter {
            typ: typ,
            what: what.into(),
        }
    }
}

pub struct Query<'a> {
    filters: Vec<Filter<'a>>,
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

    pub fn and<'b: 'a, V: 'b + Into<Cow<'b, str>>>(&'a mut self, term: Term<'b>, value: V) -> &'a mut Query<'a> {
        self.filters.push(Filter::new(term, value));
        self
    }

    pub fn limit(&'a mut self, offset: u32, limit: u32) -> &'a mut Query<'a> {
        self.window = Some((offset, limit));
        self
    }

    pub fn group<'b: 'a, G: 'b + Into<Cow<'b, str>>>(&'a mut self, group: G) -> &'a mut Query<'a> {
        match self.groups {
            None => self.groups = Some(vec![group.into()]),
            Some(ref mut groups) => groups.push(group.into()),
        };
        self
    }
}

impl<'a> fmt::Display for Term<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Term::Any => f.write_str("any"),
            Term::File => f.write_str("file"),
            Term::Base => f.write_str("base"),
            Term::LastMod => f.write_str("modified-since"),
            Term::Tag(ref tag) => f.write_str(&*tag),
        }
    }
}

impl<'a> fmt::Display for Filter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, " {} \"{}\"", self.typ, self.what)
    }
}

impl<'a> fmt::Display for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for filter in &self.filters {
            try!(filter.fmt(f));
        }

        if let Some(ref groups) = self.groups {
            for group in groups {
                try!(write!(f, " group {}", group));
            }
        }

        match self.window {
            Some((a, b)) => write!(f, " window {}:{}", a, b),
            None => Ok(()),
        }
    }
}
