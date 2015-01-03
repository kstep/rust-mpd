use std::iter::Peekable;
use std::time::duration::Duration;
use time::Timespec;
use rustc_serialize::{Encoder, Encodable};

use error::MpdResult;
use client::MpdPair;

pub trait ForceEncodable<S: Encoder<E>, E> {
    fn encode(&self, s: &mut S) -> Result<(), E>;
}

impl<'a, S: Encoder<E>, E> Encodable<S, E> for ForceEncodable<S, E> + 'a {
    #[inline] fn encode(&self, s: &mut S) -> Result<(), E> {
        self.encode(s)
    }
}

impl<S: Encoder<E>, E> ForceEncodable<S, E> for Duration {
    #[inline] fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_i64(self.num_milliseconds())
    }
}
impl<S: Encoder<E>, E> ForceEncodable<S, E> for Timespec {
    #[inline] fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_i64(self.sec)
    }
}
impl<S: Encoder<E>, E, T: ForceEncodable<S, E>> ForceEncodable<S, E> for Option<T> {
    #[inline] fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_option(|s| match *self {
            Some(ref v) => s.emit_option_some(|s| v.encode(s)),
            None => s.emit_option_none()
        })
    }
}

pub struct FieldCutIter<'a, I: 'a + Iterator<MpdResult<MpdPair>>> {
    iter: &'a mut Peekable<MpdResult<MpdPair>, I>,
    field: &'a str,
    finished: bool
}

impl<'a, I: 'a + Iterator<MpdResult<MpdPair>>> FieldCutIter<'a, I> {
    pub fn new(iter: &'a mut Peekable<MpdResult<MpdPair>, I>, field: &'a str) -> FieldCutIter<'a, I> {
        FieldCutIter {
            iter: iter,
            field: field,
            finished: false
        }
    }
}

impl<'a, I: 'a + Iterator<MpdResult<MpdPair>>> Iterator<MpdResult<MpdPair>> for FieldCutIter<'a, I> {
    fn next(&mut self) -> Option<MpdResult<MpdPair>> {
        if self.finished {
            return None;
        }

        let item = self.iter.next();
        self.finished = match self.iter.peek() {
            Some(&Ok(MpdPair(ref name, _))) if name[] == self.field => true,
            None => true,
            _ => false
        };
        item
    }
}

impl<S: Encoder<E>, E, T1: ForceEncodable<S, E>, T2: ForceEncodable<S, E>> ForceEncodable<S, E> for (T1, T2) {
    #[inline] fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_tuple(2, |s| {
            self.0.encode(s).and_then(|()| self.1.encode(s))
        })
    }
}
