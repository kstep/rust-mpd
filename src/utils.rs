#![macro_use]

use std::iter::Peekable;
use std::time::duration::Duration;
use time::Timespec;
use rustc_serialize::{Encoder, Encodable};

use error::MpdResult;
use client::MpdPair;

// Work around new orphan check: as I can't implement a foreign trait for foreign struct,
// I create local trait I can do anything I want with, blanket-implement foreign trait
// for it, and implement it for all foreign structs I need.
pub trait ForceEncodable {
    fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error>;
}

impl<'a> Encodable for ForceEncodable + 'a {
    #[inline] fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        self.encode(s)
    }
}

impl ForceEncodable for Duration {
    #[inline] fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_i64(self.num_milliseconds())
    }
}
impl ForceEncodable for Timespec {
    #[inline] fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_i64(self.sec)
    }
}
impl<T: ForceEncodable> ForceEncodable for Option<T> {
    #[inline] fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_option(|s| match *self {
            Some(ref v) => s.emit_option_some(|s| v.encode(s)),
            None => s.emit_option_none()
        })
    }
}

pub struct FieldCutIter<'a, I> where I: 'a + Iterator<Item=MpdResult<MpdPair>> {
    iter: &'a mut Peekable<MpdResult<MpdPair>, I>,
    field: &'a str,
    finished: bool
}

impl<'a, I> FieldCutIter<'a, I> where I: 'a + Iterator<Item=MpdResult<MpdPair>> {
    pub fn new(iter: &'a mut Peekable<MpdResult<MpdPair>, I>, field: &'a str) -> FieldCutIter<'a, I> {
        FieldCutIter {
            iter: iter,
            field: field,
            finished: false
        }
    }
}

impl<'a, I> Iterator for FieldCutIter<'a, I> where I: 'a + Iterator<Item=MpdResult<MpdPair>> {
    type Item = MpdResult<MpdPair>;
    fn next(&mut self) -> Option<MpdResult<MpdPair>> {
        if self.finished {
            return None;
        }

        let item = self.iter.next();
        self.finished = match self.iter.peek() {
            Some(&Ok(MpdPair(ref name, _))) if name.as_slice() == self.field => true,
            None => true,
            _ => false
        };
        item
    }
}

impl<T1: ForceEncodable, T2: ForceEncodable> ForceEncodable for (T1, T2) {
    #[inline] fn encode<S: Encoder>(&self, s: &mut S) -> Result<(), S::Error> {
        s.emit_tuple(2, |s| {
            s.emit_tuple_arg(0, |s| self.0.encode(s)).and_then(|_|
            s.emit_tuple_arg(1, |s| self.1.encode(s)))
        })
    }
}

macro_rules! mpd_collectable {
    ($typ:ty, $first_field:expr) => {
        impl FromIterator<MpdResult<MpdPair>> for MpdResult<Vec<$typ>> {
            fn from_iter<I: Iterator<Item=MpdResult<MpdPair>>>(iterator: I) -> MpdResult<Vec<$typ>> {
                let mut iter = iterator.fuse().peekable();
                let mut result = Vec::new();

                while !iter.is_empty() {
                    result.push(try!(FieldCutIter::new(&mut iter, $first_field).collect()));
                }

                Ok(result)
            }
        }
    }
}
