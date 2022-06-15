// Hidden internal interface
#![allow(missing_docs)]

use bufstream::BufStream;

use crate::convert::{FromIter, FromMap};
use crate::error::{Error, ProtoError, Result, ParseError};
use crate::reply::Reply;

use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Lines, Read, Write};
use std::result::Result as StdResult;
use std::str::FromStr;

pub struct Pairs<I>(pub I);

impl<I> Iterator for Pairs<I>
    where I: Iterator<Item = io::Result<String>>
{
    type Item = Result<(String, String)>;
    fn next(&mut self) -> Option<Result<(String, String)>> {
        let reply: Option<Result<Reply>> =
            self.0.next().map(|v| v.map_err(Error::Io).and_then(|s| s.parse::<Reply>().map_err(Error::Parse)));
        match reply {
            Some(Ok(Reply::Pair(a, b))) => Some(Ok((a, b))),
            None |
            Some(Ok(Reply::Ok)) => None,
            Some(Ok(Reply::Ack(e))) => Some(Err(Error::Server(e))),
            Some(Err(e)) => Some(Err(e)),
        }
    }
}

pub struct Maps<'a, I: 'a> {
    pairs: &'a mut Pairs<I>,
    sep: &'a str,
    value: Option<String>,
    done: bool,
    first: bool,
}

impl<'a, I> Iterator for Maps<'a, I>
    where I: Iterator<Item = io::Result<String>>
{
    type Item = Result<BTreeMap<String, String>>;
    fn next(&mut self) -> Option<Result<BTreeMap<String, String>>> {
        if self.done {
            return None;
        }

        let mut map = BTreeMap::new();

        if let Some(b) = self.value.take() {
            map.insert(self.sep.to_owned(), b);
        }

        loop {
            match self.pairs.next() {
                Some(Ok((a, b))) => {
                    if &*a == self.sep {
                        self.value = Some(b);
                        if self.first {
                            self.first = false;
                            return self.next();
                        }
                        break;
                    } else {
                        map.insert(a, b);
                    }
                }
                Some(Err(e)) => return Some(Err(e)),
                None => {
                    self.done = true;
                    break;
                }
            }
        }

        if map.is_empty() { None } else { Some(Ok(map)) }
    }
}

impl<I> Pairs<I>
    where I: Iterator<Item = io::Result<String>>
{
    pub fn split<'a, 'b: 'a>(&'a mut self, f: &'b str) -> Maps<'a, I> {
        Maps {
            pairs: self,
            sep: f,
            value: None,
            done: false,
            first: true,
        }
    }
}

// Client inner communication methods {{{
#[doc(hidden)]
pub trait Proto {
    type Stream: Read + Write;

    fn read_line(&mut self) -> Result<String>;
    fn read_pairs(&mut self) -> Pairs<Lines<&mut BufStream<Self::Stream>>>;

    fn run_command<I>(&mut self, command: &str, arguments: I) -> Result<()> where I: ToArguments;

    fn read_structs<'a, T>(&'a mut self, key: &'static str) -> Result<Vec<T>>
        where T: 'a + FromMap
    {
        self.read_pairs().split(key).map(|v| v.and_then(FromMap::from_map)).collect()
    }

    fn read_list(&mut self, key: &'static str) -> Result<Vec<String>> {
        self.read_pairs().filter(|r| r.as_ref().map(|&(ref a, _)| *a == key).unwrap_or(true)).map(|r| r.map(|(_, b)| b)).collect()
    }

    fn read_struct<'a, T>(&'a mut self) -> Result<T>
        where T: 'a + FromIter,
              Self::Stream: 'a
    {
        FromIter::from_iter(self.read_pairs())
    }

    fn drain(&mut self) -> Result<()> {
        loop {
            let reply = self.read_line()?;
            match &*reply {
                "OK" | "list_OK" => break,
                _ => (),
            }
        }
        Ok(())
    }

    fn expect_ok(&mut self) -> Result<()> {
        let line = self.read_line()?;

        match line.parse::<Reply>() {
            Ok(Reply::Ok) => Ok(()),
            Ok(Reply::Ack(e)) => Err(Error::Server(e)),
            Ok(_) => Err(Error::Proto(ProtoError::NotOk)),
            Err(e) => Err(From::from(e)),
        }
    }

    fn read_pair(&mut self) -> Result<(String, String)> {
        let line = self.read_line()?;

        match line.parse::<Reply>() {
            Ok(Reply::Pair(a, b)) => Ok((a, b)),
            Ok(Reply::Ok) => Err(Error::Proto(ProtoError::NotPair)),
            Ok(Reply::Ack(e)) => Err(Error::Server(e)),
            Err(e) => Err(Error::Parse(e)),
        }
    }

    fn read_field<T: FromStr>(&mut self, field: &'static str) -> Result<T>
        where ParseError: From<T::Err>
    {
        let (a, b) = self.read_pair()?;
        self.expect_ok()?;
        if &*a == field {
            Ok(b.parse::<T>().map_err(Into::<ParseError>::into)?)
        } else {
            Err(Error::Proto(ProtoError::NoField(field)))
        }
    }
}


pub trait ToArguments {
    fn to_arguments<F, E>(&self, _: &mut F) -> StdResult<(), E> where F: FnMut(&str) -> StdResult<(), E>;
}

impl ToArguments for () {
    fn to_arguments<F, E>(&self, _: &mut F) -> StdResult<(), E>
        where F: FnMut(&str) -> StdResult<(), E>
    {
        Ok(())
    }
}

impl<'a> ToArguments for &'a str {
    fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
        where F: FnMut(&str) -> StdResult<(), E>
    {
        f(self)
    }
}

macro_rules! argument_for_display {
    ( $x:path ) => {
        impl ToArguments for $x {
            fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
                where F: FnMut(&str) -> StdResult<(), E>
                {
                    f(&self.to_string())
                }
        }
    };
}
argument_for_display!{i8}
argument_for_display!{u8}
argument_for_display!{u32}
argument_for_display!{f32}
argument_for_display!{f64}
argument_for_display!{usize}
argument_for_display!{crate::status::ReplayGain}
argument_for_display!{String}
argument_for_display!{crate::song::Id}
argument_for_display!{crate::song::Range}
argument_for_display!{crate::message::Channel}

macro_rules! argument_for_tuple {
    ( $($t:ident: $T: ident),+ ) => {
        impl<$($T : ToArguments,)*> ToArguments for ($($T,)*) {
            fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
                where F: FnMut(&str) -> StdResult<(), E>
                {
                    let ($(ref $t,)*) = *self;
                    $(
                        $t.to_arguments(f)?;
                     )*
                    Ok(())
                }
        }
    };
}
argument_for_tuple!{t0: T0}
argument_for_tuple!{t0: T0, t1: T1}
argument_for_tuple!{t0: T0, t1: T1, t2: T2}
argument_for_tuple!{t0: T0, t1: T1, t2: T2, t3: T3}
argument_for_tuple!{t0: T0, t1: T1, t2: T2, t3:T3, t4: T4}

impl<'a, T: ToArguments> ToArguments for &'a [T] {
    fn to_arguments<F, E>(&self, f: &mut F) -> StdResult<(), E>
        where F: FnMut(&str) -> StdResult<(), E>
    {
        for arg in *self {
            arg.to_arguments(f)?
        }
        Ok(())
    }
}

pub struct Quoted<'a, D: fmt::Display + 'a + ?Sized>(pub &'a D);

impl<'a, D: fmt::Display + 'a + ?Sized> fmt::Display for Quoted<'a, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let unquoted = format!("{}", self.0);
        if &unquoted == "" {
            // return Ok(());
        }
        let quoted = unquoted.replace('\\', r"\\").replace('"', r#"\""#);
        formatter.write_fmt(format_args!("\"{}\"", &quoted))
    }
}

// }}}
