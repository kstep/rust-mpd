
use libc::{c_uint, c_int, c_float, c_uchar};
use std::fmt::{Show, Error, Formatter};
use std::time::duration::Duration;
use std::str::FromStr;

use client::{FromClient, MpdClient};
use rustc_serialize::{Encoder, Encodable};

#[deriving(Show, RustcEncodable)]
pub struct AudioFormat {
    pub rate: u32,
    pub bits: u8,
    pub chans: u8
}

impl FromStr for AudioFormat {
    fn from_str(s: &str) -> Option<AudioFormat> {
        let mut it = s.split(':');
        if let (Some(rate), Some(bits), Some(chans)) = (
            it.next().and_then(|v| v.parse()),
            it.next().and_then(|v| v.parse()),
            it.next().and_then(|v| v.parse())) {
            Some(AudioFormat {
                rate: rate,
                bits: bits,
                chans: chans
            })
        } else {
            None
        }
    }
}

#[deriving(Show, RustcEncodable)]
pub enum MpdState {
    Stop,
    Play,
    Pause,
}

impl FromStr for MpdState {
    fn from_str(s: &str) -> Option<MpdState> {
        match s {
            "stop" => Some(MpdState::Stop),
            "play" => Some(MpdState::Play),
            "pause" => Some(MpdState::Pause),
            _ => None
        }
    }
}

#[deriving(Show, RustcEncodable)]
pub struct MpdStatus {
    volume: uint,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    playlist: uint,
    playlistlength: uint,
    state: MpdState,
    song: Option<uint>,
    songid: Option<uint>,
    nextsong: Option<uint>,
    nextsongid: Option<uint>,
    time: Option<u64>,
    elapsed: Option<u64>,
    duration: Option<u64>,
    bitrate: Option<uint>,
    xfade: Option<u64>,
    mixrampdb: f32,
    mixrampdelay: Option<u64>,
    audio: Option<AudioFormat>,
    updating_db: Option<uint>,
    error: Option<String>
}

