
use error::MpdResult;
use connection::{FromConn, MpdConnection, mpd_connection};
use songs::{MpdSong, mpd_song};
use libc;

extern {
    fn mpd_run_get_queue_song_pos(connection: *mut mpd_connection, pos: libc::c_uint) -> *mut mpd_song;
    fn mpd_run_move_id(connection: *mut mpd_connection, from: libc::c_uint, to: libc::c_uint) -> bool;
    fn mpd_run_swap_id(connection: *mut mpd_connection, id1: libc::c_uint, id2: libc::c_uint) -> bool;
}

pub struct MpdQueue<'a> {
    conn: &'a MpdConnection
}

impl<'a> MpdQueue<'a> {
    pub fn from_conn(conn: &'a MpdConnection) -> MpdQueue<'a> {
        MpdQueue { conn: conn }
    }

    pub fn get(&self, index: uint) -> Option<MpdSong> {
        let song = unsafe { mpd_run_get_queue_song_pos(self.conn.conn, index as libc::c_uint) };
        if song.is_null() {
            None
        } else {
            Some(MpdSong { song: song })
        }
    }

    pub fn move_to(&mut self, song: &MpdSong, pos: uint) -> MpdResult<()> {
        if unsafe { mpd_run_move_id(self.conn.conn, song.id(), pos as libc::c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }

    pub fn swap(&mut self, song1: &MpdSong, song2: &MpdSong) -> MpdResult<()> {
        if unsafe { mpd_run_swap_id(self.conn.conn, song1.id(), song2.id()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self.conn).unwrap())
        }
    }
}
