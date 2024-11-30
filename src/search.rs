#![allow(missing_docs)]
// TODO: unfinished functionality

use crate::proto::{Quoted, ToArguments};
use std::{
    io::Write,  // implements write for Vec
    borrow::Cow
};
use std::convert::Into;
use std::fmt;
use std::result::Result as StdResult;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(untagged, rename_all = "lowercase"))]
pub enum Term<'a> {
    Any,
    File,
    Base,
    #[cfg_attr(feature = "serde", serde(rename = "modified-since"))]
    LastMod,
    Tag(Cow<'a, str>),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize), serde(untagged, rename_all = "lowercase"))]
pub enum Operation {
    Equals,
    NotEquals,
    Contains,
    #[cfg_attr(feature = "serde", serde(rename = "starts_with"))]
    StartsWith
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Filter<'a> {
    typ: Term<'a>,
    what: Cow<'a, str>,
    how: Operation
}

impl<'a> Filter<'a> {
    pub fn new<W>(typ: Term<'a>, what: W) -> Filter
    where W: 'a + Into<Cow<'a, str>> {
        Filter {
            typ,
            what: what.into(),
            how: Operation::Equals
        }
    }

    pub fn new_with_op<W>(typ: Term<'a>, what: W, how: Operation) -> Filter
    where W: 'a + Into<Cow<'a, str>> {
        Filter {
            typ,
            what: what.into(),
            how
        }
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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

    pub fn and<'b: 'a, V: 'b + Into<Cow<'b, str>>>(&mut self, term: Term<'b>, value: V) -> &mut Query<'a> {
        self.filters.push(Filter::new(term, value));
        self
    }

    pub fn and_with_op<'b: 'a, V: 'b + Into<Cow<'b, str>>>(&mut self, term: Term<'b>, op: Operation, value: V) -> &mut Query<'a> {
        self.filters.push(Filter::new_with_op(term, value, op));
        self
    }
}

impl<'a> fmt::Display for Term<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Term::Any => "any",
            Term::File => "file",
            Term::Base => "base",
            Term::LastMod => "modified-since",
            Term::Tag(ref tag) => tag,
        })
    }
}

impl<'a> ToArguments for &'a Term<'a> {
    fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
    where F: FnMut(&str) -> StdResult<(), E> {
        f(&self.to_string())
    }
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match *self {
            Operation::Equals => "==",
            Operation::NotEquals => "!=",
            Operation::Contains => "contains",
            Operation::StartsWith => "starts_with"
        })
    }
}

impl ToArguments for Operation {
    fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
    where F: FnMut(&str) -> StdResult<(), E> {
        f(&self.to_string())
    }
}

impl<'a> ToArguments for &'a Filter<'a> {
    fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
    where F: FnMut(&str) -> StdResult<(), E> {
        match self.typ {
            // For some terms, the filter clause cannot have an operation
            Term::Base | Term::LastMod => {
                f(&format!(
                    "({} {})",
                    &self.typ,
                    &Quoted(&self.what).to_string()
                ))
            }
            _ => {
                f(&format!(
                    "({} {} {})",
                    &self.typ,
                    &self.how,
                    &Quoted(&self.what).to_string())
                )
            }
        }
    }
}

impl<'a> ToArguments for &'a Query<'a> {
    // Use MPD 0.21+ filter syntax
    fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
    where F: FnMut(&str) -> StdResult<(), E> {
        // Construct the query string in its entirety first before escaping
        if !self.filters.is_empty() {
            let mut qs = String::new();
            for (i, filter) in self.filters.iter().enumerate() {
                if i > 0 {
                    qs.push_str(" AND ");
                }
                // Leave escaping to the filter since terms should not be escaped or quoted
                filter.to_arguments(&mut |arg| {
                    qs.push_str(arg);
                    Ok(())
                })?;
            }
            // println!("Singly escaped query string: {}", &qs);
            f(&qs)
        } else {
            Ok(())
        }
    }
}

impl ToArguments for Window {
    fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
    where F: FnMut(&str) -> StdResult<(), E> {
        if let Some(window) = self.0 {
            f("window")?;
            f(&format! {"{}:{}", window.0, window.1})?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::proto::ToArguments;

    fn collect<I: ToArguments>(arguments: I) -> Vec<String> {
        let mut output = Vec::<String>::new();
        arguments
            .to_arguments::<_, ()>(&mut |arg| {
                output.push(arg.to_string());
                Ok(())
            })
            .unwrap();
        output
    }

    #[test]
    fn find_window_format() {
        let window: Window = (0, 2).into();
        let output = collect(window);
        assert_eq!(output, vec!["window", "0:2"]);
    }

    #[test]
    fn find_query_format() {
        let mut query = Query::new();
        let finished = query.and(Term::Tag("albumartist".into()), "Mac DeMarco").and(Term::Tag("album".into()), "Salad Days");
        let output = collect(&*finished);
        assert_eq!(output, vec!["albumartist", "Mac DeMarco", "album", "Salad Days"]);
    }

    #[test]
    fn multiple_and() {
        let mut query = Query::new();
        query.and(Term::Tag("albumartist".into()), "Mac DeMarco");
        query.and(Term::Tag("album".into()), "Salad Days");
    }
}
