
use std::fmt::{Show, Error, Formatter};
use std::c_str::ToCStr;
use std::ptr;
use time::Timespec;
use libc;

use common::{MpdError, MpdResult, FromConnection};
use connection::{mpd_connection, MpdConnection};
use songs::{Song, Songs};

#[repr(C)] struct mpd_playlist;

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_playlist_dup(playlist: *const mpd_playlist) -> *mut mpd_playlist;
    fn mpd_recv_playlist(playlist: *mut mpd_connection) -> *mut mpd_playlist;
    fn mpd_playlist_free(playlist: *mut mpd_playlist);
    fn mpd_playlist_get_last_modified(playlist: *const mpd_playlist) -> libc::time_t;
    fn mpd_playlist_get_path(playlist: *const mpd_playlist) -> *const u8;

    fn mpd_send_list_playlists(connection: *mut mpd_connection) -> bool;
    fn mpd_send_list_playlist(connection: *mut mpd_connection, name: *const u8) -> bool;
}

pub struct Playlists<'a> {
    conn: *mut mpd_connection
}

impl<'a> FromConnection for Playlists<'a> {
    fn from_connection<'a>(connection: *mut mpd_connection) -> Option<Playlists<'a>> {
        if unsafe { mpd_send_list_playlists(connection) } {
            Some(Playlists { conn: connection })
        } else {
            None
        }
    }
}

impl<'a> Iterator<Playlist> for Playlists<'a> {
    fn next(&mut self) -> Option<Playlist> {
        Playlist::from_connection(self.conn)
    }
}

impl Drop for Playlist {
    fn drop(&mut self) {
        unsafe { mpd_playlist_free(self.pl) }
    }
}

impl Clone for Playlist {
    fn clone(&self) -> Playlist {
        let pl = unsafe { mpd_playlist_dup(self.pl as *const _) };
        if pl as *const _ == ptr::null::<mpd_playlist>() {
            panic!("Out of memory!")
        }

        Playlist { pl: pl }
    }
}

impl Show for Playlist {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"Playlist { "));
        try!(f.write(b"path: "));
        try!(self.path().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

pub struct Playlist {
    pl: *mut mpd_playlist
}

impl Playlist {
    pub fn path(&self) -> String {
        unsafe { String::from_raw_buf(mpd_playlist_get_path(self.pl as *const _)) }
    }

    pub fn last_mod(&self) -> Timespec { Timespec::new(unsafe { mpd_playlist_get_last_modified(self.pl as *const _) }, 0) }

    fn from_connection(connection: *mut mpd_connection) -> Option<Playlist> {
        let pl = unsafe { mpd_recv_playlist(connection) };
        if pl as *const _ == ptr::null::<mpd_playlist>() {
            None
        } else {
            Some(Playlist { pl: pl })
        }
    }

    pub fn songs<'a>(&self, conn: &'a mut MpdConnection) -> MpdResult<Songs<'a>> {
        if unsafe { mpd_send_list_playlist(conn.conn, mpd_playlist_get_path(self.pl as *const _)) } {
            Ok(Songs { conn: conn })
        } else {
            Err(FromConnection::from_connection(conn.conn).unwrap())
        }
    }
}


