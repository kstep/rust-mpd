use std::fmt::{Show, Error, Formatter};
use time::Timespec;
use libc::{time_t, c_char, c_uint};
use std::ptr;
use std::c_str::CString;

use error::MpdResult;
use connection::{FromConn, MpdConnection, mpd_connection};
use rustc_serialize::{Encoder, Encodable};
use songs::{MpdSongs, ToSongUri};

#[repr(C)] struct mpd_playlist;

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_playlist_dup(playlist: *const mpd_playlist) -> *mut mpd_playlist;
    fn mpd_recv_playlist(playlist: *mut mpd_connection) -> *mut mpd_playlist;
    fn mpd_playlist_free(playlist: *mut mpd_playlist);
    fn mpd_playlist_get_last_modified(playlist: *const mpd_playlist) -> time_t;
    fn mpd_playlist_get_path(playlist: *const mpd_playlist) -> *const c_char;

    fn mpd_send_list_playlists(connection: *mut mpd_connection) -> bool;
    fn mpd_send_list_playlist(connection: *mut mpd_connection, name: *const c_char) -> bool;

    fn mpd_run_playlist_add(connection: *mut mpd_connection, name: *const c_char, path: *const c_char) -> bool;
    fn mpd_run_playlist_clear(connection: *mut mpd_connection, name: *const c_char) -> bool;
    fn mpd_run_playlist_move(connection: *mut mpd_connection, name: *const c_char, from: c_uint, to: c_uint) -> bool;
    fn mpd_run_playlist_delete(connection: *mut mpd_connection, name: *const c_char, pos: c_uint) -> bool;
    fn mpd_run_rename(connection: *mut mpd_connection, from: *const c_char, to: *const c_char) -> bool;
    fn mpd_run_rm(connection: *mut mpd_connection, name: *const c_char) -> bool;
    fn mpd_run_load(connection: *mut mpd_connection, name: *const c_char) -> bool;
    fn mpd_run_save(connection: *mut mpd_connection, name: *const c_char) -> bool;
}

pub struct MpdPlaylists<'a> {
    conn: &'a MpdConnection
}

impl<'a, S: Encoder<E>, E> Encodable<S, E> for MpdPlaylists<'a> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(0, |s| self.enumerate().fold(Ok(()), |r, (i, v)| r.and_then(|()| s.emit_seq_elt(i, |s| v.encode(s)))))
    }
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

impl<'a> Iterator<MpdResult<MpdPlaylist<'a>>> for MpdPlaylists<'a> {
    fn next(&mut self) -> Option<MpdResult<MpdPlaylist<'a>>> {
        match MpdPlaylist::from_conn(self.conn) {
            Some(pl) => Some(Ok(pl)),
            None => match FromConn::from_conn(self.conn) {
                None => None,
                Some(e) => Some(Err(e))
            }
        }
    }
}

#[unsafe_destructor]
impl<'a> Drop for MpdPlaylist<'a> {
    fn drop(&mut self) {
        if !self.pl.is_null() {
            unsafe { mpd_playlist_free(self.pl) }
        }
    }
}

impl<'a> Clone for MpdPlaylist<'a> {
    fn clone(&self) -> MpdPlaylist<'a> {
        if self.pl.is_null() {
            return MpdPlaylist { pl: ptr::null::<mpd_playlist>() as *mut _, conn: self.conn, path: self.path.clone() };
        }

        let pl = unsafe { mpd_playlist_dup(self.pl as *const _) };
        if pl.is_null() {
            panic!("Out of memory!")
        }

        MpdPlaylist { pl: pl, conn: self.conn, path: self.path.clone() }
    }
}

impl<'a> Show for MpdPlaylist<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"MpdPlaylist { "));
        try!(f.write(b"path: "));
        try!(self.path().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

pub struct MpdPlaylist<'a> {
    pl: *mut mpd_playlist,
    path: CString,
    conn: &'a MpdConnection
}

impl<'a> MpdPlaylist<'a> {
    pub fn from_conn<'a>(conn: &'a MpdConnection) -> Option<MpdPlaylist<'a>> {
        let pl = unsafe { mpd_recv_playlist(conn.conn) };
        if pl.is_null() {
            None
        } else {
            Some(MpdPlaylist { pl: pl, conn: conn, path: unsafe { CString::new(mpd_playlist_get_path(pl as *const _), false) } })
        }
    }

    pub fn new<'a>(conn: &'a MpdConnection, path: &str) -> MpdPlaylist<'a> {
        MpdPlaylist { pl: ptr::null::<mpd_playlist>() as *mut _, path: path.to_c_str(), conn: conn }
    }

    pub fn path(&self) -> String {
        unsafe { String::from_raw_buf(self.path.as_ptr() as *const u8) }
    }

    pub fn last_mod(&self) -> Timespec { Timespec::new(if self.pl.is_null() { 0 } else { unsafe { mpd_playlist_get_last_modified(self.pl as *const _) } }, 0) }

    pub fn iter<'a>(&'a self) -> MpdResult<MpdSongs<'a>> {
        if unsafe { mpd_send_list_playlist(self.conn.conn, self.path.as_ptr()) } {
            Ok(MpdSongs { conn: self.conn })
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn push<T: ToSongUri>(&mut self, song: T) -> MpdResult<()> {
        if unsafe { mpd_run_playlist_add(self.conn.conn, self.path.as_ptr(), song.song_uri().as_ptr()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn clear(&mut self) -> MpdResult<()> {
        if unsafe { mpd_run_playlist_clear(self.conn.conn, self.path.as_ptr()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn remove(&mut self, pos: uint) -> MpdResult<()> {
        if unsafe { mpd_run_playlist_delete(self.conn.conn, self.path.as_ptr(), pos as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn move_pos(&mut self, from: uint, to: uint) -> MpdResult<()> {
        if unsafe { mpd_run_playlist_move(self.conn.conn, self.path.as_ptr(), from as c_uint, to as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn rename(&mut self, name: &str) -> MpdResult<()> {
        let name = name.to_c_str();
        if unsafe { mpd_run_rename(self.conn.conn, self.path.as_ptr(), name.as_ptr()) } {
            self.path = name;
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn delete(self) -> MpdResult<()> {
        if unsafe { mpd_run_rm(self.conn.conn, self.path.as_ptr()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn load(&self) -> MpdResult<()> {
        if unsafe { mpd_run_load(self.conn.conn, self.path.as_ptr()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn save(&mut self) -> MpdResult<()> {
        if unsafe { mpd_run_save(self.conn.conn, self.path.as_ptr()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }
}

impl<'a, S: Encoder<E>, E> Encodable<S, E> for MpdPlaylist<'a> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_struct("MpdPlaylist", 2, |s| {
            s.emit_struct_field("path", 0, |s| s.emit_str(self.path()[])).and_then(|()|
            s.emit_struct_field("last_modified", 1, |s| s.emit_i64(self.last_mod().sec)))
        })
    }
}


