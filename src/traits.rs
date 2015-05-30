use std::ops;
use time::Duration;
use output::Output;
use playlist::Playlist;
use song::Id;

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
        &*self
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

pub trait ToSeconds {
    fn to_seconds(self) -> f64;
}

impl ToSeconds for i64 {
    fn to_seconds(self) -> f64 {
        self as f64
    }
}

impl ToSeconds for f64 {
    fn to_seconds(self) -> f64 {
        self
    }
}

impl ToSeconds for Duration {
    fn to_seconds(self) -> f64 {
        self.num_milliseconds() as f64 / 1000.0
    }
}

pub trait IsId {
    fn is_id() -> bool { false }
}

pub trait ToQueueRangeOrPlace : IsId {
    fn to_range(self) -> String;
}

pub trait ToQueueRange {
    fn to_range(self) -> String;
}

impl<T: ToQueuePlace> ToQueueRangeOrPlace for T {
    fn to_range(self) -> String {
        format!("{}", self.to_place())
    }
}

impl ToQueueRange for ops::Range<u32> {
    fn to_range(self) -> String {
        format!("{}:{}", self.start, self.end)
    }
}

impl ToQueueRangeOrPlace for ops::Range<u32> {
    fn to_range(self) -> String {
        ToQueueRange::to_range(self)
    }
}

impl ToQueueRange for ops::RangeTo<u32> {
    fn to_range(self) -> String {
        format!(":{}", self.end)
    }
}

impl ToQueueRangeOrPlace for ops::RangeTo<u32> {
    fn to_range(self) -> String {
        ToQueueRange::to_range(self)
    }
}

impl ToQueueRange for ops::RangeFrom<u32> {
    fn to_range(self) -> String {
        format!("{}:", self.start)
    }
}

impl ToQueueRangeOrPlace for ops::RangeFrom<u32> {
    fn to_range(self) -> String {
        ToQueueRange::to_range(self)
    }
}

impl ToQueueRange for ops::RangeFull {
    fn to_range(self) -> String {
        String::new()
    }
}

impl ToQueueRangeOrPlace for ops::RangeFull {
    fn to_range(self) -> String {
        ToQueueRange::to_range(self)
    }
}

pub trait ToQueuePlace : IsId {
    fn to_place(self) -> u32;
}

impl ToQueuePlace for Id {
    fn to_place(self) -> u32 {
        self.0
    }
}

impl ToQueuePlace for u32 {
    fn to_place(self) -> u32 {
        self
    }
}

impl IsId for u32 {}
impl IsId for ops::Range<u32> {}
impl IsId for ops::RangeTo<u32> {}
impl IsId for ops::RangeFrom<u32> {}
impl IsId for ops::RangeFull {}
impl IsId for Id {
    fn is_id() -> bool {
        true
    }
}

pub trait ToOutputId {
    fn to_output_id(self) -> u32;
}

impl ToOutputId for u32 {
    fn to_output_id(self) -> u32 {
        self
    }
}
impl ToOutputId for Output {
    fn to_output_id(self) -> u32 {
        self.id
    }
}
