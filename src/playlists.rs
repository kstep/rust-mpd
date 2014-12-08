
use std::fmt::{Show, Error, Formatter};
use time::Timespec;
use libc::{time_t, c_uchar};

use error::MpdResult;
use connection::{FromConn, MpdConnection, mpd_connection};
use serialize::{Encoder, Encodable};
use songs::MpdSongs;

#[repr(C)] struct mpd_playlist;

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_playlist_dup(playlist: *const mpd_playlist) -> *mut mpd_playlist;
    fn mpd_recv_playlist(playlist: *mut mpd_connection) -> *mut mpd_playlist;
    fn mpd_playlist_free(playlist: *mut mpd_playlist);
    fn mpd_playlist_get_last_modified(playlist: *const mpd_playlist) -> time_t;
    fn mpd_playlist_get_path(playlist: *const mpd_playlist) -> *const c_uchar;

    fn mpd_send_list_playlists(connection: *mut mpd_connection) -> bool;
    fn mpd_send_list_playlist(connection: *mut mpd_connection, name: *const c_uchar) -> bool;
}

pub struct MpdPlaylists<'a> {
    conn: &'a MpdConnection
}

impl<'a> MpdPlaylists<'a> {
    pub fn from_conn<'a>(conn: &'a MpdConnection) -> Option<MpdPlaylists<'a>> {
        if unsafe { mpd_send_list_playlists(conn.conn) } {
            Some(MpdPlaylists { conn: conn })
        } else {
            None
        }
    }
}

impl<'a> Iterator<MpdResult<MpdPlaylist>> for MpdPlaylists<'a> {
    fn next(&mut self) -> Option<MpdResult<MpdPlaylist>> {
        match FromConn::from_conn(self.conn) {
            Some(pl) => Some(Ok(pl)),
            None => match FromConn::from_conn(self.conn) {
                None => None,
                Some(e) => Some(Err(e))
            }
        }
    }
}

impl Drop for MpdPlaylist {
    fn drop(&mut self) {
        unsafe { mpd_playlist_free(self.pl) }
    }
}

impl Clone for MpdPlaylist {
    fn clone(&self) -> MpdPlaylist {
        let pl = unsafe { mpd_playlist_dup(self.pl as *const _) };
        if pl.is_null() {
            panic!("Out of memory!")
        }

        MpdPlaylist { pl: pl }
    }
}

impl Show for MpdPlaylist {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"MpdPlaylist { "));
        try!(f.write(b"path: "));
        try!(self.path().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

pub struct MpdPlaylist {
    pl: *mut mpd_playlist
}

impl MpdPlaylist {
    pub fn path(&self) -> String {
        unsafe { String::from_raw_buf(mpd_playlist_get_path(self.pl as *const _)) }
    }

    pub fn last_mod(&self) -> Timespec { Timespec::new(unsafe { mpd_playlist_get_last_modified(self.pl as *const _) }, 0) }

    pub fn songs<'a>(&self, conn: &'a mut MpdConnection) -> MpdResult<MpdSongs<'a>> {
        if unsafe { mpd_send_list_playlist(conn.conn, mpd_playlist_get_path(self.pl as *const _)) } {
            Ok(MpdSongs { conn: conn })
        } else {
            Err(FromConn::from_conn(conn).unwrap())
        }
    }
}

impl<S: Encoder<E>, E> Encodable<S, E> for MpdPlaylist {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_struct("MpdPlaylist", 2, |s| {
            s.emit_struct_field("path", 0, |s| s.emit_str(self.path()[])).and_then(|()|
            s.emit_struct_field("last_modified", 1, |s| s.emit_i64(self.last_mod().sec)))
        })
    }
}

impl FromConn for MpdPlaylist {
    fn from_conn(conn: &MpdConnection) -> Option<MpdPlaylist> {
        let pl = unsafe { mpd_recv_playlist(conn.conn) };
        if pl.is_null() {
            None
        } else {
            Some(MpdPlaylist { pl: pl })
        }
    }
}

