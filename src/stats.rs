use libc;
use std::time::duration::Duration;
use std::fmt::{Show, Error, Formatter};
use time::Timespec;
use connection::{MpdConnection, FromConn, mpd_connection};
use serialize::{Encoder, Encodable};

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

impl FromConn for MpdStats {
    fn from_conn(conn: &MpdConnection) -> Option<MpdStats> {
        let stats = unsafe { mpd_run_stats(conn.conn) };
        if stats.is_null() {
            return None;
        }

        Some(MpdStats { p: stats })
    }
}

impl MpdStats {
    fn artists(&self) -> uint { unsafe { mpd_stats_get_number_of_artists(self.p as *const _) as uint } }
    fn albums(&self) -> uint { unsafe { mpd_stats_get_number_of_albums(self.p as *const _) as uint } }
    fn songs(&self) -> uint { unsafe { mpd_stats_get_number_of_songs(self.p as *const _) as uint } }
    fn uptime(&self) -> Duration { Duration::seconds(unsafe { mpd_stats_get_uptime(self.p as *const _) as i64 }) }
    fn db_update_time(&self) -> Timespec { Timespec::new(unsafe { mpd_stats_get_db_update_time(self.p as *const _) as i64 }, 0) } 
    fn play_time(&self) -> Duration { Duration::seconds(unsafe { mpd_stats_get_play_time(self.p as *const _) as i64 }) }
    fn db_play_time(&self) -> Duration { Duration::seconds(unsafe { mpd_stats_get_db_play_time(self.p as *const _) as i64 }) }
}

impl<S: Encoder<E>, E> Encodable<S, E> for MpdStats {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_struct("MpdStats", 7, |s| {
            s.emit_struct_field("artists", 0, |s| s.emit_uint(self.artists())).and_then(|()|
            s.emit_struct_field("albums", 1, |s| s.emit_uint(self.albums()))).and_then(|()|
            s.emit_struct_field("songs", 2, |s| s.emit_uint(self.songs()))).and_then(|()|
            s.emit_struct_field("uptime", 3, |s| s.emit_i64(self.uptime().num_milliseconds()))).and_then(|()|
            s.emit_struct_field("play_time", 4, |s| s.emit_i64(self.play_time().num_milliseconds()))).and_then(|()|
            s.emit_struct_field("db_play_time", 5, |s| s.emit_i64(self.db_play_time().num_milliseconds()))).and_then(|()|
            s.emit_struct_field("db_update_time", 6, |s| s.emit_i64(self.db_update_time().sec)))
        })
    }
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
