#![allow(missing_docs)]
// TODO: unfinished functionality

use std::fmt;
use std::io::{Read, Write};
use std::borrow::Cow;
use std::convert::Into;
use client::Client;

pub enum Term {
    Any,
    File,
    Base,
    LastMod,
    Tag(String)
}

pub struct Clause(pub Term, pub String);

pub struct Query<'a, S: 'a + Read + Write> {
    client: &'a mut Client<S>,
    clauses: Vec<Clause>,
    groups: Option<Vec<String>>,
    window: Option<(u32, Option<u32>)>,
}

impl<'a, S: 'a + Read + Write> Query<'a, S> {
    pub fn new(client: &'a mut Client<S>) -> Query<'a, S> {
        Query {
            client: client,
            clauses: Vec::new(),
            groups: None,
            window: None
        }
    }

    pub fn and<V: Into<Cow<'a, str>>>(&'a mut self, term: Term, value: V) -> &'a mut Query<'a, S> {
        self.clauses.push(Clause(term, value.into().into_owned()));
        self
    }

    pub fn limit(&'a mut self, offset: u32, limit: u32) -> &'a mut Query<'a, S> {
        self.window = Some((offset, Some(limit)));
        self
    }
}

impl fmt::Display for Term {
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

impl fmt::Display for Clause {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

impl<'a, S: 'a + Read + Write> fmt::Display for Query<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for clause in &self.clauses {
            try!(clause.fmt(f));
        }

        match self.window {
            Some((a, Some(b))) => write!(f, " window {}:{}", a, b),
            Some((a, None)) => write!(f, " window {}:", a),
            None => Ok(())
        }
    }
}
