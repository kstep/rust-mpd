
use error::MpdResult;
use connection::{FromConn, MpdConnection, mpd_connection};
use songs::{MpdSong, MpdSongs, mpd_song};
use std::c_str::CString;
use libc;

extern {
    fn mpd_run_get_queue_song_pos(connection: *mut mpd_connection, pos: libc::c_uint) -> *mut mpd_song;
    fn mpd_run_get_queue_song_id(connection: *mut mpd_connection, id: libc::c_uint) -> *mut mpd_song;
    fn mpd_run_move_id(connection: *mut mpd_connection, from: libc::c_uint, to: libc::c_uint) -> bool;
    fn mpd_run_swap_id(connection: *mut mpd_connection, id1: libc::c_uint, id2: libc::c_uint) -> bool;
    fn mpd_run_swap(connection: *mut mpd_connection, pos1: libc::c_uint, pos2: libc::c_uint) -> bool;
    fn mpd_run_add_id(connection: *mut mpd_connection, file: *const i8) -> libc::c_int;
    fn mpd_run_add_id_to(connection: *mut mpd_connection, uri: *const i8, to: libc::c_uint) -> libc::c_int;
    fn mpd_send_list_queue_meta(connection: *mut mpd_connection) -> bool;
    fn mpd_send_list_queue_range_meta(connection: *mut mpd_connection, start: libc::c_uint, end: libc::c_uint) -> bool;
}

pub struct MpdQueue<'a> {
    pub conn: &'a MpdConnection
}

impl<'a> MpdQueue<'a> {
    pub fn from_conn(conn: &'a MpdConnection) -> MpdQueue<'a> {
        MpdQueue { conn: conn }
    }

    pub fn get_by_id(&self, id: uint) -> MpdResult<MpdSong> {
        let song = unsafe { mpd_run_get_queue_song_id(self.conn.conn, id as libc::c_uint) };
        if song.is_null() {
            Err(FromConn::from_conn(self.conn).unwrap())
        } else {
            Ok(MpdSong { song: song })
        }
    }

    pub fn get(&self, index: uint) -> MpdResult<MpdSong> {
        let song = unsafe { mpd_run_get_queue_song_pos(self.conn.conn, index as libc::c_uint) };
        if song.is_null() {
            Err(FromConn::from_conn(self.conn).unwrap())
        } else {
            Ok(MpdSong { song: song })
        }
    }

    pub fn move_to(&mut self, pos: uint, song: &MpdSong) -> MpdResult<()> {
        if unsafe { mpd_run_move_id(self.conn.conn, song.id() as libc::c_uint, pos as libc::c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn insert<T: ToSongUri>(&mut self, pos: uint, song: &T) -> MpdResult<uint> {
        let uid = unsafe { mpd_run_add_id_to(self.conn.conn, song.song_uri().as_ptr(), pos as libc::c_uint) };
        if uid < 0 {
            Err(FromConn::from_conn(self.conn).unwrap())
        } else {
            Ok(uid as uint)
        }
    }

    pub fn push<T: ToSongUri>(&mut self, song: &T) -> MpdResult<uint> {
        let uid = unsafe { mpd_run_add_id(self.conn.conn, song.song_uri().as_ptr()) };
        if uid < 0 {
            Err(FromConn::from_conn(self.conn).unwrap())
        } else {
            Ok(uid as uint)
        }
    }

    pub fn swap_id(&mut self, song1: &MpdSong, song2: &MpdSong) -> MpdResult<()> {
        if unsafe { mpd_run_swap_id(self.conn.conn, song1.id() as libc::c_uint, song2.id() as libc::c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn swap<T: ToSongPos>(&mut self, song1: T, song2: T) -> MpdResult<()> {
        if unsafe { mpd_run_swap(self.conn.conn, song1.song_pos() as libc::c_uint, song2.song_pos() as libc::c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn songs(&self) -> MpdResult<MpdSongs> {
        if unsafe { mpd_send_list_queue_meta(self.conn.conn) } {
            Ok(MpdSongs { conn: self.conn })
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn songs_at(&self, start: uint, end: uint) -> MpdResult<MpdSongs> {
        if unsafe { mpd_send_list_queue_range_meta(self.conn.conn, start as libc::c_uint, end as libc::c_uint) } {
            Ok(MpdSongs { conn: self.conn })
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn len(&self) -> MpdResult<uint> {
        self.conn.status().map(|s| s.queue_len())
    }
}

pub trait ToSongPos {
    fn song_pos(&self) -> uint;
}

impl ToSongPos for uint {
    #[inline] fn song_pos(&self) -> uint { *self }
}

impl ToSongPos for MpdSong {
    #[inline] fn song_pos(&self) -> uint { self.pos() }
}

pub trait ToSongUri {
    fn song_uri(&self) -> CString;
}

impl ToSongUri for MpdSong {
    #[inline] fn song_uri(&self) -> CString {
        self.uri().to_c_str()
    }
}

impl ToSongUri for String {
    #[inline] fn song_uri(&self) -> CString {
        self.to_c_str()
    }
}
