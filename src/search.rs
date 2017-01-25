#![allow(missing_docs)]
// TODO: unfinished functionality

use std::borrow::Cow;
use std::convert::Into;
use std::fmt;

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

pub struct Window(Option<(u32, u32)>);

impl From<(u32, u32)> for Window {
    fn from(window: (u32, u32)) -> Window {
        Window(Some(window))
    }
}

impl From<Option<(u32, u32)>> for Window {
    fn from(window: Option<(u32, u32)>) -> Window {
        Window(window)
    }
}

#[derive(Default)]
pub struct Query<'a> {
    filters: Vec<Filter<'a>>,
}

impl<'a> Query<'a> {
    pub fn new() -> Query<'a> {
        Query { filters: Vec::new() }
    }

    pub fn and<'b: 'a, V: 'b + Into<Cow<'b, str>>>(&'a mut self, term: Term<'b>, value: V) -> &'a mut Query<'a> {
        self.filters.push(Filter::new(term, value));
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
        Ok(())
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((a, b)) = self.0 {
            write!(f, " window {}:{}", a, b)?;
        }
        Ok(())
    }
}
