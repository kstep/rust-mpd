use libc;
use std::time::duration::Duration;
use std::fmt::{Show, Error, Formatter};
use std::ptr;
use time::Timespec;
use common::{MpdError, MpdResult, FromConnection};
use connection::mpd_connection;

#[repr(C)] struct mpd_stats;

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_run_stats(connection: *mut mpd_connection) -> *mut mpd_stats;
    fn mpd_stats_free(stats: *mut mpd_stats);
    fn mpd_stats_get_number_of_artists(stats: *const mpd_stats) -> libc::c_uint;
    fn mpd_stats_get_number_of_albums(stats: *const mpd_stats) -> libc::c_uint;
    fn mpd_stats_get_number_of_songs(stats: *const mpd_stats) -> libc::c_uint;
    fn mpd_stats_get_uptime(stats: *const mpd_stats) -> libc::c_ulong;
    fn mpd_stats_get_db_update_time(stats: *const mpd_stats) -> libc::c_ulong;
    fn mpd_stats_get_play_time(stats: *const mpd_stats) -> libc::c_ulong;
    fn mpd_stats_get_db_play_time(stats: *const mpd_stats) -> libc::c_ulong;
}

pub struct MpdStats {
    p: *mut mpd_stats
}

impl Drop for MpdStats {
    fn drop(&mut self) {
        unsafe {
            mpd_stats_free(self.p);
        }
    }
}

impl FromConnection for MpdStats {
    fn from_connection(connection: *mut mpd_connection) -> Option<MpdStats> {
        let stats = unsafe { mpd_run_stats(connection) };
        if stats as *const _ == ptr::null::<mpd_stats>() {
            return None;
        }

        Some(MpdStats { p: stats })
    }
}

impl MpdStats {
    fn artists(&self) -> u32 { unsafe { mpd_stats_get_number_of_artists(self.p as *const _) } }
    fn albums(&self) -> u32 { unsafe { mpd_stats_get_number_of_albums(self.p as *const _) } }
    fn songs(&self) -> u32 { unsafe { mpd_stats_get_number_of_songs(self.p as *const _) } }
    fn uptime(&self) -> Duration { Duration::seconds(unsafe { mpd_stats_get_uptime(self.p as *const _) as i64 }) }
    fn db_update_time(&self) -> Timespec { Timespec::new(unsafe { mpd_stats_get_db_update_time(self.p as *const _) as i64 }, 0) } 
    fn play_time(&self) -> Duration { Duration::seconds(unsafe { mpd_stats_get_play_time(self.p as *const _) as i64 }) }
    fn db_play_time(&self) -> Duration { Duration::seconds(unsafe { mpd_stats_get_db_play_time(self.p as *const _) as i64 }) }
}

impl Show for MpdStats {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"MpdStats { artists: "));
        try!(self.artists().fmt(f));
        try!(f.write(b", albums: "));
        try!(self.albums().fmt(f));
        try!(f.write(b", songs: "));
        try!(self.songs().fmt(f));
        try!(f.write(b", uptime: "));
        try!(self.uptime().fmt(f));
        try!(f.write(b", db_update_time: "));
        try!(self.db_update_time().fmt(f));
        try!(f.write(b", play_time: "));
        try!(self.play_time().fmt(f));
        try!(f.write(b", db_play_time: "));
        try!(self.db_play_time().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}
