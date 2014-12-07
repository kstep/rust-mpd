
use libc;
use std::time::duration::Duration;
use std::fmt::{Show, Error, Formatter};
use time::Timespec;

use error::MpdResult;
use connection::{mpd_connection, MpdConnection, FromConn};
use tags::MpdTagType;

#[repr(C)] pub struct mpd_song;

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_song_dup(song: *const mpd_song) -> *mut mpd_song;
    fn mpd_song_free(song: *mut mpd_song);
    fn mpd_song_get_uri(song: *const mpd_song) -> *const u8;
    fn mpd_song_get_tag(song: *const mpd_song, typ: MpdTagType, idx: libc::c_uint) -> *const u8;
    fn mpd_song_get_duration(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_get_start(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_get_end(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_get_last_modified(song: *const mpd_song) -> libc::time_t;
    fn mpd_song_get_id(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_get_pos(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_set_pos(song: *mut mpd_song, pos: libc::c_uint);
    fn mpd_song_get_prio(song: *const mpd_song) -> libc::c_uint;
    fn mpd_recv_song(connection: *mut mpd_connection) -> *mut mpd_song;

    fn mpd_run_seek_id(connection: *mut mpd_connection, song_id: libc::c_uint, t: libc::c_uint) -> bool;
}

pub struct MpdSongs<'a> {
    pub conn: &'a mut MpdConnection
}

impl<'a> Iterator<MpdResult<MpdSong>> for MpdSongs<'a> {
    fn next(&mut self) -> Option<MpdResult<MpdSong>> {
        match FromConn::from_conn(self.conn) {
            Some(song) => Some(Ok(song)),
            None => match FromConn::from_conn(self.conn) {
                None => None,
                Some(e) => Some(Err(e))
            }
        }
    }
}

pub struct MpdSong {
    pub song: *mut mpd_song
}

impl Show for MpdSong {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"MpdSong { "));
        try!(f.write(b"uri: "));
        try!(self.uri().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

impl MpdSong {
    pub fn uri(&self) -> String { unsafe { String::from_raw_buf(mpd_song_get_uri(self.song as *const _)) } }
    pub fn tags(&self, kind: MpdTagType, index: u32) -> Option<String> {
        let tag = unsafe { mpd_song_get_tag(self.song as *const _, kind, index) };
        if tag.is_null() {
            None
        } else {
            Some(unsafe { String::from_raw_buf(tag) })
        }
    }
    pub fn duration(&self) -> Duration { Duration::seconds(unsafe { mpd_song_get_duration(self.song as *const _) } as i64) }
    pub fn id(&self) -> u32 { unsafe { mpd_song_get_id(self.song as *const _) } }
    pub fn prio(&self) -> u32 { unsafe { mpd_song_get_prio(self.song as *const _) } }
    pub fn start(&self) -> Duration { Duration::seconds(unsafe { mpd_song_get_start(self.song as *const _) } as i64) }
    pub fn end(&self) -> Option<Duration> {
        match unsafe { mpd_song_get_end(self.song as *const _) } {
            0 => None,
            s @ _ => Some(Duration::seconds(s as i64))
        }
    }
    pub fn slice(&self) -> (Duration, Option<Duration>) { (self.start(), self.end()) }
    pub fn last_mod(&self) -> Timespec { Timespec::new(unsafe { mpd_song_get_last_modified(self.song as *const _) }, 0) }
    pub fn pos(&self) -> uint { unsafe { mpd_song_get_pos(self.song as *const _) as uint } }
    pub fn set_pos(&mut self, pos: uint) { unsafe { mpd_song_set_pos(self.song, pos as libc::c_uint) } }

    pub fn seek(&mut self, conn: &mut MpdConnection, pos: Duration) -> MpdResult<()> {
        if unsafe { mpd_run_seek_id(conn.conn, self.id(), pos.num_seconds() as libc::c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(conn).unwrap())
        }
    }
}

impl FromConn for MpdSong {
    fn from_conn(conn: &MpdConnection) -> Option<MpdSong> {
        let song = unsafe { mpd_recv_song(conn.conn) };
        if song.is_null() {
            None
        } else {
            Some(MpdSong { song: song })
        }
    }
}

impl Drop for MpdSong {
    fn drop(&mut self) {
        unsafe { mpd_song_free(self.song); }
    }
}

impl Clone for MpdSong {
    fn clone(&self) -> MpdSong {
        let song = unsafe { mpd_song_dup(self.song as *const _) };
        if song.is_null() {
            panic!("Out of memory!")
        }

        MpdSong { song: song }
    }
}
