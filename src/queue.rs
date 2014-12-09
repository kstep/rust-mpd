
use error::MpdResult;
use connection::{FromConn, MpdConnection, mpd_connection};
use songs::{MpdSong, MpdSongs, mpd_song};
use std::c_str::CString;
use libc::{c_uint, c_int, c_char};

extern {
    fn mpd_run_get_queue_song_pos(connection: *mut mpd_connection, pos: c_uint) -> *mut mpd_song;
    fn mpd_run_get_queue_song_id(connection: *mut mpd_connection, id: c_uint) -> *mut mpd_song;
    fn mpd_run_move_id(connection: *mut mpd_connection, from: c_uint, to: c_uint) -> bool;
    fn mpd_run_move(connection: *mut mpd_connection, from: c_uint, to: c_uint) -> bool;
    fn mpd_run_move_range(connection: *mut mpd_connection, start: c_uint, end: c_uint, to: c_uint) -> bool;
    fn mpd_run_swap_id(connection: *mut mpd_connection, id1: c_uint, id2: c_uint) -> bool;
    fn mpd_run_swap(connection: *mut mpd_connection, pos1: c_uint, pos2: c_uint) -> bool;
    fn mpd_run_add_id(connection: *mut mpd_connection, file: *const c_char) -> c_int;
    fn mpd_run_add_id_to(connection: *mut mpd_connection, uri: *const c_char, to: c_uint) -> c_int;
    fn mpd_send_list_queue_meta(connection: *mut mpd_connection) -> bool;
    fn mpd_send_list_queue_range_meta(connection: *mut mpd_connection, start: c_uint, end: c_uint) -> bool;
    fn mpd_run_delete(connection: *mut mpd_connection, pos: c_uint) -> bool;
    fn mpd_run_delete_range(connection: *mut mpd_connection, start: c_uint, end: c_uint) -> bool;
    fn mpd_run_delete_id(connection: *mut mpd_connection, id: c_uint) -> bool;
}

pub struct MpdQueue<'a> {
    pub conn: &'a MpdConnection
}

impl<'a> MpdQueue<'a> {
    pub fn from_conn(conn: &'a MpdConnection) -> MpdQueue<'a> {
        MpdQueue { conn: conn }
    }

    /// Get song at some position in queue
    pub fn nth(&self, index: uint) -> MpdResult<MpdSong> {
        let song = unsafe { mpd_run_get_queue_song_pos(self.conn.conn, index as c_uint) };
        if song.is_null() {
            Err(FromConn::from_conn(self.conn).unwrap())
        } else {
            Ok(MpdSong { song: song })
        }
    }

    /// Get song by queue id
    pub fn get(&self, id: uint) -> MpdResult<MpdSong> {
        let song = unsafe { mpd_run_get_queue_song_id(self.conn.conn, id as c_uint) };
        if song.is_null() {
            Err(FromConn::from_conn(self.conn).unwrap())
        } else {
            Ok(MpdSong { song: song })
        }
    }

    /// Insert new song into queue at given position
    pub fn insert<T: ToSongUri>(&mut self, pos: uint, song: T) -> MpdResult<uint> {
        let uid = unsafe { mpd_run_add_id_to(self.conn.conn, song.song_uri().as_ptr(), pos as c_uint) };
        if uid < 0 {
            Err(FromConn::from_conn(self.conn).unwrap())
        } else {
            Ok(uid as uint)
        }
    }

    /// Add song at the end of the queue
    pub fn push<T: ToSongUri>(&mut self, song: T) -> MpdResult<uint> {
        let uid = unsafe { mpd_run_add_id(self.conn.conn, song.song_uri().as_ptr()) };
        if uid < 0 {
            Err(FromConn::from_conn(self.conn).unwrap())
        } else {
            Ok(uid as uint)
        }
    }

    /// Move song to some position in queue
    pub fn move_pos(&mut self, to: uint, from: uint) -> MpdResult<()> {
        if unsafe { mpd_run_move(self.conn.conn, from as c_uint, to as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Move songs in given range
    pub fn move_range(&mut self, pos: uint, start: uint, end: uint) -> MpdResult<()> {
        if unsafe { mpd_run_move_range(self.conn.conn, start as c_uint, end as c_uint, pos as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Move song to some position in queue by id
    pub fn move_to(&mut self, pos: uint, song: &MpdSong) -> MpdResult<()> {
        if unsafe { mpd_run_move_id(self.conn.conn, song.id() as c_uint, pos as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Swap two songs in given positions
    pub fn swap_pos(&mut self, song1: uint, song2: uint) -> MpdResult<()> {
        if unsafe { mpd_run_swap(self.conn.conn, song1 as c_uint, song2 as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Swap two songs in given positions by id
    pub fn swap(&mut self, song1: &MpdSong, song2: &MpdSong) -> MpdResult<()> {
        if unsafe { mpd_run_swap_id(self.conn.conn, song1.id() as c_uint, song2.id() as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Remove a song
    pub fn remove(&mut self, song: &MpdSong) -> MpdResult<()> {
        if unsafe { mpd_run_delete_id(self.conn.conn, song.id() as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Remove songs at given position
    pub fn remove_pos(&mut self, pos: uint) -> MpdResult<()> {
        if unsafe { mpd_run_delete(self.conn.conn, pos as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Remove songs in given range
    pub fn remove_range(&mut self, start: uint, end: uint) -> MpdResult<()> {
        if unsafe { mpd_run_delete_range(self.conn.conn, start as c_uint, end as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Iterate over songs in the queue
    pub fn iter(&self) -> MpdResult<MpdSongs> {
        if unsafe { mpd_send_list_queue_meta(self.conn.conn) } {
            Ok(MpdSongs { conn: self.conn })
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Iterate over songs in given range
    pub fn iter_range(&self, start: uint, end: uint) -> MpdResult<MpdSongs> {
        if unsafe { mpd_send_list_queue_range_meta(self.conn.conn, start as c_uint, end as c_uint) } {
            Ok(MpdSongs { conn: self.conn })
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    /// Length of the queue
    pub fn len(&self) -> MpdResult<uint> {
        self.conn.status().map(|s| s.queue_len())
    }

    /// Returns true if queue is empty
    pub fn is_empty(&self) -> MpdResult<bool> {
        self.len().map(|v| v == 0)
    }
}

pub trait ToSongUri {
    fn song_uri(&self) -> CString;
}

impl ToSongUri for MpdSong {
    #[inline] fn song_uri(&self) -> CString {
        self.uri().to_c_str()
    }
}

impl<T: ToCStr> ToSongUri for T {
    #[inline] fn song_uri(&self) -> CString {
        self.to_c_str()
    }
}
