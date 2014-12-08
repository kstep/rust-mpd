
use libc::{c_uint, c_int, c_float, c_uchar};
use std::fmt::{Show, Error, Formatter};
use std::time::duration::Duration;

use connection::{FromConn, MpdConnection, mpd_connection};
use serialize::{Encoder, Encodable};

#[repr(C)] struct mpd_status;

#[repr(C)]
#[deriving(Show)]
struct mpd_audio_format {
    sample_rate: u32,
    bits: u8,
    channels: u8,

    reserved0: u16,
    reserved1: u32
}

#[deriving(Show, Encodable)]
pub struct AudioFormat {
    pub rate: u32,
    pub bits: u8,
    pub chans: u8
}

#[repr(C)]
#[deriving(Show, Encodable)]
pub enum MpdState {
    Unknown = 0,
    Stop = 1,
    Play = 2,
    Pause = 3,
}

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_run_status(connection: *mut mpd_connection) -> *mut mpd_status;
    fn mpd_status_free(status: *mut mpd_status);
    fn mpd_status_get_volume(status: *const mpd_status) -> c_int;
    fn mpd_status_get_repeat(status: *const mpd_status) -> bool;
    fn mpd_status_get_random(status: *const mpd_status) -> bool;
    fn mpd_status_get_single(status: *const mpd_status) -> bool;
    fn mpd_status_get_consume(status: *const mpd_status) -> bool;
    fn mpd_status_get_queue_length(status: *const mpd_status) -> c_uint;
    fn mpd_status_get_queue_version(status: *const mpd_status) -> c_uint;
    fn mpd_status_get_state(status: *const mpd_status) -> MpdState;
    fn mpd_status_get_crossfade(status: *const mpd_status) -> c_uint;
    fn mpd_status_get_mixrampdb(status: *const mpd_status) -> c_float;
    fn mpd_status_get_mixrampdelay(status: *const mpd_status) -> c_float;
    fn mpd_status_get_song_pos(status: *const mpd_status) -> c_int;
    fn mpd_status_get_song_id(status: *const mpd_status) -> c_int;
    fn mpd_status_get_next_song_pos(status: *const mpd_status) -> c_int;
    fn mpd_status_get_next_song_id(status: *const mpd_status) -> c_int;
    //fn mpd_status_get_elapsed_time(status: *const mpd_status) -> c_uint;
    fn mpd_status_get_elapsed_ms(status: *const mpd_status) -> c_uint;
    fn mpd_status_get_total_time(status: *const mpd_status) -> c_uint;
    fn mpd_status_get_kbit_rate(status: *const mpd_status) -> c_uint;
    fn mpd_status_get_audio_format(status: *const mpd_status) -> *const mpd_audio_format;
    fn mpd_status_get_update_id(status: *const mpd_status) -> c_uint;
    fn mpd_status_get_error(status: *const mpd_status) -> *const c_uchar;
}

pub struct MpdStatus {
    p: *mut mpd_status
}

impl<S: Encoder<E>, E> Encodable<S, E> for MpdStatus {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_struct("MpdStatus", 19, |s| {
            s.emit_struct_field("volume", 0, |s| s.emit_int(self.volume())).and_then(|()|
            s.emit_struct_field("repeat", 1, |s| s.emit_bool(self.repeat()))).and_then(|()|
            s.emit_struct_field("random", 2, |s| s.emit_bool(self.random()))).and_then(|()|
            s.emit_struct_field("single", 3, |s| s.emit_bool(self.single()))).and_then(|()|
            s.emit_struct_field("consume", 4, |s| s.emit_bool(self.consume()))).and_then(|()|
            s.emit_struct_field("state", 5, |s| self.state().encode(s))).and_then(|()|
            s.emit_struct_field("crossfade", 6, |s| s.emit_i64(self.crossfade().num_milliseconds()))).and_then(|()|
            s.emit_struct_field("queue_length", 7, |s| s.emit_uint(self.queue_len()))).and_then(|()|
            s.emit_struct_field("queue_version", 8, |s| s.emit_uint(self.queue_version()))).and_then(|()|
            s.emit_struct_field("mixrampdb", 9, |s| s.emit_f32(self.mixrampdb()))).and_then(|()|
            s.emit_struct_field("mixrampdelay", 10, |s| s.emit_option(|s| match self.mixrampdelay() {
                Some(ref d) => s.emit_option_some(|s| s.emit_i64(d.num_milliseconds())),
                None => s.emit_option_none()
            }))).and_then(|()|
            s.emit_struct_field("song", 11, |s| s.emit_option(|s| match self.song() {
                Some(ref v) => s.emit_option_some(|s| v.encode(s)),
                None => s.emit_option_none()
            }))).and_then(|()|
            s.emit_struct_field("next_song", 12, |s| s.emit_option(|s| match self.next_song() {
                Some(ref v) => s.emit_option_some(|s| v.encode(s)),
                None => s.emit_option_none()
            }))).and_then(|()|
            s.emit_struct_field("elapsed_time", 13, |s| s.emit_i64(self.elapsed_time().num_milliseconds()))).and_then(|()|
            s.emit_struct_field("total_time", 14, |s| s.emit_i64(self.elapsed_time().num_milliseconds()))).and_then(|()|
            s.emit_struct_field("kbit_rate", 15, |s| s.emit_uint(self.kbit_rate()))).and_then(|()|
            s.emit_struct_field("audio_format", 16, |s| self.audio_format().encode(s))).and_then(|()|
            s.emit_struct_field("update_id", 17, |s| s.emit_uint(self.update_id()))).and_then(|()|
            s.emit_struct_field("error", 18, |s| s.emit_option(|s| match self.error() {
                Some(ref e) => s.emit_option_some(|s| s.emit_str(e[])),
                None => s.emit_option_none()
            })))
        })
    }
}

impl Show for MpdStatus {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"MpdStatus { volume: "));
        try!(self.volume().fmt(f));
        try!(f.write(b", repeat: "));
        try!(self.repeat().fmt(f));
        try!(f.write(b", random: "));
        try!(self.random().fmt(f));
        try!(f.write(b", single: "));
        try!(self.single().fmt(f));
        try!(f.write(b", consume: "));
        try!(self.consume().fmt(f));
        try!(f.write(b", state: "));
        try!(self.state().fmt(f));
        try!(f.write(b", crossfade: "));
        try!(self.crossfade().fmt(f));
        try!(f.write(b", queue_len: "));
        try!(self.queue_len().fmt(f));
        try!(f.write(b", queue_version: "));
        try!(self.queue_version().fmt(f));
        try!(f.write(b", mixrampdb: "));
        try!(self.mixrampdb().fmt(f));
        try!(f.write(b", mixrampdelay: "));
        try!(self.mixrampdelay().fmt(f));
        try!(f.write(b", song: "));
        try!(self.song().fmt(f));
        try!(f.write(b", next_song: "));
        try!(self.next_song().fmt(f));
        try!(f.write(b", elapsed_time: "));
        try!(self.elapsed_time().fmt(f));
        try!(f.write(b", total_time: "));
        try!(self.total_time().fmt(f));
        try!(f.write(b", kbit_rate: "));
        try!(self.kbit_rate().fmt(f));
        try!(f.write(b", audio_format: "));
        try!(self.audio_format().fmt(f));
        try!(f.write(b", update_id: "));
        try!(self.update_id().fmt(f));
        try!(f.write(b", error: "));
        try!(self.error().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

impl FromConn for MpdStatus {
    fn from_conn(conn: &MpdConnection) -> Option<MpdStatus> {
        let status = unsafe { mpd_run_status(conn.conn) };
        if status.is_null() {
            return None;
        }

        Some(MpdStatus { p: status })
    }
}

impl MpdStatus {
    pub fn volume(&self) -> int { unsafe { mpd_status_get_volume(self.p as *const _) as int } }
    pub fn repeat(&self) -> bool { unsafe { mpd_status_get_repeat(self.p as *const _) } }
    pub fn random(&self) -> bool { unsafe { mpd_status_get_random(self.p as *const _) } }
    pub fn single(&self) -> bool { unsafe { mpd_status_get_single(self.p as *const _) } }
    pub fn consume(&self) -> bool { unsafe { mpd_status_get_consume(self.p as *const _) } }
    pub fn state(&self) -> MpdState { unsafe { mpd_status_get_state(self.p as *const _) } }
    pub fn crossfade(&self) -> Duration { Duration::seconds(unsafe { mpd_status_get_crossfade(self.p as *const _) as i64 }) }
    pub fn queue_len(&self) -> uint { unsafe { mpd_status_get_queue_length(self.p as *const _) as uint } }
    pub fn queue_version(&self) -> uint { unsafe { mpd_status_get_queue_version(self.p as *const _) as uint } }
    pub fn mixrampdb(&self) -> f32 { unsafe { mpd_status_get_mixrampdb(self.p as *const _) } }
    pub fn mixrampdelay(&self) -> Option<Duration> { let v = unsafe { mpd_status_get_mixrampdelay(self.p as *const _) }; if v < 0f32 { None } else { Some(Duration::milliseconds((v * 1000f32) as i64)) } }
    pub fn song(&self) -> Option<(uint, uint)> {
        let song_id = unsafe { mpd_status_get_song_id(self.p as *const _) };
        if song_id < 0 { None } else { Some((song_id as uint, unsafe { mpd_status_get_song_pos(self.p as *const _) as uint })) }
    }
    pub fn next_song(&self) -> Option<(uint, uint)> {
        let song_id = unsafe { mpd_status_get_next_song_id(self.p as *const _) };
        if song_id < 0 { None } else { Some((song_id as uint, unsafe { mpd_status_get_next_song_pos(self.p as *const _) as uint })) }
    }
    pub fn elapsed_time(&self) -> Duration { Duration::milliseconds(unsafe { mpd_status_get_elapsed_ms(self.p as *const _) as i64 }) }
    pub fn total_time(&self) -> Duration { Duration::seconds(unsafe { mpd_status_get_total_time(self.p as *const _) as i64 }) }
    pub fn kbit_rate(&self) -> uint { unsafe { mpd_status_get_kbit_rate(self.p as *const _) as uint } }
    pub fn audio_format(&self) -> Option<AudioFormat> {
        let aformat = unsafe { mpd_status_get_audio_format(self.p as *const _) };
        if aformat.is_null() { None } else { Some(unsafe { AudioFormat { rate: (*aformat).sample_rate, bits: (*aformat).bits, chans: (*aformat).channels } }) }
    }
    pub fn update_id(&self) -> uint { unsafe { mpd_status_get_update_id(self.p as *const _) as uint } }
    pub fn error(&self) -> Option<String> {
        let error = unsafe { mpd_status_get_error(self.p as *const _) };
        if error.is_null() { None } else { Some(unsafe { String::from_raw_buf(error) }) }
    }
}

impl Drop for MpdStatus {
    fn drop(&mut self) {
        unsafe { mpd_status_free(self.p) }
    }
}

