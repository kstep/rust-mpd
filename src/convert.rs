#![allow(missing_docs)]
//! These are inner traits to support methods overloading for the `Client`

use crate::error::Error;
use crate::output::Output;
use crate::playlist::Playlist;
use crate::proto::ToArguments;
use crate::song::{self, Id, Song};
use std::collections::BTreeMap;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

use std::time::Duration;

#[doc(hidden)]
pub trait FromMap: Sized {
    fn from_map(map: BTreeMap<String, String>) -> Result<Self, Error>;
}

#[doc(hidden)]
pub trait FromIter: Sized {
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Self, Error>;
}

impl<T: FromIter> FromMap for T {
    fn from_map(map: BTreeMap<String, String>) -> Result<Self, Error> {
        FromIter::from_iter(map.into_iter().map(Ok))
    }
}

// Playlist name polymorphisms {{{
pub trait ToPlaylistName {
    fn to_name(&self) -> &str;
}

impl ToPlaylistName for Playlist {
    fn to_name(&self) -> &str {
        &*self.name
    }
}

impl<'a> ToPlaylistName for &'a Playlist {
    fn to_name(&self) -> &str {
        &*self.name
    }
}

impl<'a> ToPlaylistName for &'a String {
    fn to_name(&self) -> &str {
        self
    }
}

impl<'a> ToPlaylistName for &'a str {
    fn to_name(&self) -> &str {
        *self
    }
}

impl ToPlaylistName for str {
    fn to_name(&self) -> &str {
        self
    }
}

impl ToPlaylistName for String {
    fn to_name(&self) -> &str {
        &*self
    }
}
// }}}

// Seconds polymorphisms {{{
pub trait ToSeconds {
    fn to_seconds(&self) -> f64;
}

impl ToSeconds for i64 {
    fn to_seconds(&self) -> f64 {
        *self as f64
    }
}

impl ToSeconds for f64 {
    fn to_seconds(&self) -> f64 {
        *self
    }
}

impl ToSeconds for Duration {
    fn to_seconds(&self) -> f64 {
        self.as_secs_f64()
    }
}
// }}}

// Queue place polymorphisms {{{

pub trait IsId {
    fn is_id() -> bool {
        false
    }
}

pub trait ToQueueRangeOrPlace: IsId {
    fn to_range(&self) -> String;
}

pub trait ToQueueRange {
    fn to_range(&self) -> String;
}

impl<T: ToQueuePlace> ToQueueRangeOrPlace for T {
    fn to_range(&self) -> String {
        format!("{}", self.to_place())
    }
}

impl ToQueueRange for Range<u32> {
    fn to_range(&self) -> String {
        format!("{}:{}", self.start, self.end)
    }
}

impl ToQueueRangeOrPlace for Range<u32> {
    fn to_range(&self) -> String {
        ToQueueRange::to_range(self)
    }
}

impl ToQueueRange for RangeTo<u32> {
    fn to_range(&self) -> String {
        format!(":{}", self.end)
    }
}

impl ToQueueRangeOrPlace for RangeTo<u32> {
    fn to_range(&self) -> String {
        ToQueueRange::to_range(self)
    }
}

impl ToQueueRange for RangeFrom<u32> {
    fn to_range(&self) -> String {
        format!("{}:", self.start)
    }
}

impl ToQueueRangeOrPlace for RangeFrom<u32> {
    fn to_range(&self) -> String {
        ToQueueRange::to_range(self)
    }
}

impl ToQueueRange for RangeFull {
    fn to_range(&self) -> String {
        ToQueueRange::to_range(&(0..))
    }
}

impl ToQueueRangeOrPlace for RangeFull {
    fn to_range(&self) -> String {
        ToQueueRange::to_range(self)
    }
}

pub trait ToQueuePlace: IsId {
    fn to_place(&self) -> u32;
}

impl ToQueuePlace for Id {
    fn to_place(&self) -> u32 {
        self.0
    }
}

impl ToQueuePlace for u32 {
    fn to_place(&self) -> u32 {
        *self
    }
}

impl IsId for u32 {}
impl IsId for Range<u32> {}
impl IsId for RangeTo<u32> {}
impl IsId for RangeFrom<u32> {}
impl IsId for RangeFull {}
impl IsId for Id {
    fn is_id() -> bool {
        true
    }
}

pub trait ToSongId {
    fn to_song_id(&self) -> Id;
}

impl ToSongId for Song {
    fn to_song_id(&self) -> Id {
        self.place.unwrap().id
    }
}

impl ToSongId for u32 {
    fn to_song_id(&self) -> Id {
        Id(*self)
    }
}

impl ToSongId for Id {
    fn to_song_id(&self) -> Id {
        *self
    }
}
// }}}

// Output id polymorphisms {{{
pub trait ToOutputId {
    fn to_output_id(&self) -> u32;
}

impl ToOutputId for u32 {
    fn to_output_id(&self) -> u32 {
        *self
    }
}
impl ToOutputId for Output {
    fn to_output_id(&self) -> u32 {
        self.id
    }
}
// }}}

// Song play range polymorphisms {{{
pub trait ToSongRange {
    fn to_range(&self) -> song::Range;
}

impl ToSongRange for Range<Duration> {
    fn to_range(&self) -> song::Range {
        song::Range(self.start, Some(self.end))
    }
}

impl ToSongRange for Range<u32> {
    fn to_range(&self) -> song::Range {
        song::Range(Duration::from_secs(self.start as u64), Some(Duration::from_secs(self.end as u64)))
    }
}

impl ToSongRange for RangeFrom<Duration> {
    fn to_range(&self) -> song::Range {
        song::Range(self.start, None)
    }
}

impl ToSongRange for RangeFrom<u32> {
    fn to_range(&self) -> song::Range {
        song::Range(Duration::from_secs(self.start as u64), None)
    }
}

impl ToSongRange for RangeTo<Duration> {
    fn to_range(&self) -> song::Range {
        song::Range(Duration::from_secs(0), Some(self.end))
    }
}

impl ToSongRange for RangeTo<u32> {
    fn to_range(&self) -> song::Range {
        song::Range(Duration::from_secs(0), Some(Duration::from_secs(self.end as u64)))
    }
}

impl ToSongRange for RangeFull {
    fn to_range(&self) -> song::Range {
        song::Range(Duration::from_secs(0), None)
    }
}

impl ToSongRange for song::Range {
    fn to_range(&self) -> song::Range {
        *self
    }
}

// }}}

pub trait ToSongPath {
    fn to_path(&self) -> &str;
}

impl ToSongPath for Song {
    fn to_path(&self) -> &str {
        &self.file
    }
}

impl<'a, T: ToSongPath> ToSongPath for &'a T {
    fn to_path(&self) -> &str {
        (*self).to_path()
    }
}

impl ToSongPath for dyn AsRef<str> {
    fn to_path(&self) -> &str {
        self.as_ref()
    }
}

impl<T: ToSongPath> ToArguments for T {
    fn to_arguments<F, E>(&self, f: &mut F) -> Result<(), E>
    where
        F: FnMut(&str) -> Result<(), E>,
    {
        self.to_path().to_arguments(f)
    }
}
