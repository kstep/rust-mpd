
use libc;
use std::fmt::{Show, Error, Formatter};
use std::time::duration::Duration;

use common::FromConn;
use connection::mpd_connection;

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

// rate, bits, chans
pub type AudioFormat = (u32, u8, u8);

#[repr(C)]
#[deriving(Show)]
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
    fn mpd_status_get_volume(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_repeat(status: *const mpd_status) -> bool;
    fn mpd_status_get_random(status: *const mpd_status) -> bool;
    fn mpd_status_get_single(status: *const mpd_status) -> bool;
    fn mpd_status_get_consume(status: *const mpd_status) -> bool;
    fn mpd_status_get_queue_length(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_queue_version(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_state(status: *const mpd_status) -> MpdState;
    fn mpd_status_get_crossfade(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_mixrampdb(status: *const mpd_status) -> f32;
    fn mpd_status_get_mixrampdelay(status: *const mpd_status) -> f32;
    fn mpd_status_get_song_pos(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_song_id(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_next_song_pos(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_next_song_id(status: *const mpd_status) -> libc::c_int;
    //fn mpd_status_get_elapsed_time(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_elapsed_ms(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_total_time(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_kbit_rate(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_audio_format(status: *const mpd_status) -> *const mpd_audio_format;
    fn mpd_status_get_update_id(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_error(status: *const mpd_status) -> *const u8;
}

pub struct MpdStatus {
    p: *mut mpd_status
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
    fn from_conn(connection: *mut mpd_connection) -> Option<MpdStatus> {
        let status = unsafe { mpd_run_status(connection) };
        if status.is_null() {
            return None;
        }

        Some(MpdStatus { p: status })
    }
}

impl MpdStatus {
    pub fn volume(&self) -> i32 { unsafe { mpd_status_get_volume(self.p as *const _) } }
    pub fn repeat(&self) -> bool { unsafe { mpd_status_get_repeat(self.p as *const _) } }
    pub fn random(&self) -> bool { unsafe { mpd_status_get_random(self.p as *const _) } }
    pub fn single(&self) -> bool { unsafe { mpd_status_get_single(self.p as *const _) } }
    pub fn consume(&self) -> bool { unsafe { mpd_status_get_consume(self.p as *const _) } }
    pub fn state(&self) -> MpdState { unsafe { mpd_status_get_state(self.p as *const _) } }
    pub fn crossfade(&self) -> Duration { Duration::seconds(unsafe { mpd_status_get_crossfade(self.p as *const _) as i64 }) }
    pub fn queue_len(&self) -> u32 { unsafe { mpd_status_get_queue_length(self.p as *const _) } }
    pub fn queue_version(&self) -> u32 { unsafe { mpd_status_get_queue_version(self.p as *const _) } }
    pub fn mixrampdb(&self) -> f32 { unsafe { mpd_status_get_mixrampdb(self.p as *const _) } }
    pub fn mixrampdelay(&self) -> Option<f32> { let v = unsafe { mpd_status_get_mixrampdelay(self.p as *const _) }; if v < 0f32 { None } else { Some(v) } }
    pub fn song(&self) -> Option<(i32, i32)> {
        let song_id = unsafe { mpd_status_get_song_id(self.p as *const _) };
        if song_id < 0 { None } else { Some((song_id, unsafe { mpd_status_get_song_pos(self.p as *const _) })) }
    }
    pub fn next_song(&self) -> Option<(i32, i32)> {
        let song_id = unsafe { mpd_status_get_next_song_id(self.p as *const _) };
        if song_id < 0 { None } else { Some((song_id, unsafe { mpd_status_get_next_song_pos(self.p as *const _) })) }
    }
    pub fn elapsed_time(&self) -> Duration { Duration::milliseconds(unsafe { mpd_status_get_elapsed_ms(self.p as *const _) as i64 }) }
    pub fn total_time(&self) -> Duration { Duration::seconds(unsafe { mpd_status_get_total_time(self.p as *const _) as i64 }) }
    pub fn kbit_rate(&self) -> u32 { unsafe { mpd_status_get_kbit_rate(self.p as *const _) } }
    pub fn audio_format(&self) -> Option<AudioFormat> {
        let aformat = unsafe { mpd_status_get_audio_format(self.p as *const _) };
        if aformat.is_null() { None } else { Some(unsafe { ((*aformat).sample_rate, (*aformat).bits, (*aformat).channels) }) }
    }
    pub fn update_id(&self) -> u32 { unsafe { mpd_status_get_update_id(self.p as *const _) } }
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

