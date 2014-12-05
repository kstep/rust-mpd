
use libc;
use std::c_str::ToCStr;
use std::time::duration::Duration;
use std::ptr;

use common::FromConnection;
use connection::{mpd_connection, MpdConnection};

struct mpd_status;

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
type AudioFormat = (u32, u8, u8);

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
    fn mpd_status_get_elapsed_time(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_elapsed_ms(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_total_time(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_kbit_rate(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_audio_format(status: *const mpd_status) -> *const mpd_audio_format;
    fn mpd_status_get_update_id(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_error(status: *const mpd_status) -> *const u8;
}

#[deriving(Show)]
pub struct MpdStatus {
    volume: i32,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    queue_length: u32,
    queue_version: u32,
    state: MpdState,
    crossfade: u32,
    mixrampdb: f32,
    mixrampdelay: Option<f32>,
    song: Option<(i32, i32)>, // id, pos
    next_song: Option<(i32, i32)>,
    elapsed_time: Duration,
    total_time: Duration,
    kbit_rate: u32,
    audio_format: Option<AudioFormat>,
    update_id: u32,
    error: Option<String>
}

impl FromConnection for MpdStatus {
    fn from_connection(connection: *mut mpd_connection) -> Option<MpdStatus> {
        unsafe {
            let status = mpd_run_status(connection);
            if status as *const _ == ptr::null::<mpd_status>() {
                return None
            }

            let s = status as *const _;
            let aformat = mpd_status_get_audio_format(s);
            let error = mpd_status_get_error(s);
            let song_id = mpd_status_get_song_id(s);
            let next_song_id = mpd_status_get_next_song_id(s);
            let mixramp = mpd_status_get_mixrampdelay(s);

            let result = MpdStatus {
                volume: mpd_status_get_volume(s),
                repeat: mpd_status_get_repeat(s),
                random: mpd_status_get_random(s),
                single: mpd_status_get_single(s),
                consume: mpd_status_get_consume(s),
                queue_length: mpd_status_get_queue_length(s),
                queue_version: mpd_status_get_queue_version(s),
                state: mpd_status_get_state(s),
                crossfade: mpd_status_get_crossfade(s),
                mixrampdb: mpd_status_get_mixrampdb(s),
                mixrampdelay: if mixramp < 0f32 { None } else { Some(mixramp) },
                song: if song_id < 0 { None } else { Some((song_id, mpd_status_get_song_pos(s))) },
                next_song: if next_song_id < 0 { None } else { Some((next_song_id, mpd_status_get_next_song_pos(s))) },
                elapsed_time: Duration::milliseconds(mpd_status_get_elapsed_ms(s) as i64),
                total_time: Duration::seconds(mpd_status_get_total_time(s) as i64),
                kbit_rate: mpd_status_get_kbit_rate(s),
                audio_format: if aformat == ptr::null() { None } else { Some(((*aformat).sample_rate, (*aformat).bits, (*aformat).channels)) },
                update_id: mpd_status_get_update_id(s),
                error: if error == ptr::null() { None } else { Some(String::from_raw_buf(error)) }
            };

            mpd_status_free(status);

            Some(result)
        }
    }
}

