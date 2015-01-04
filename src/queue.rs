
use error::MpdResult;
use client::{FromClient, MpdClient, mpd_connection};
use songs::{MpdSong, MpdSongs, ToSongUri, mpd_song};
use playlists::MpdPlaylist;

pub struct MpdQueue {
    pub conn: &'a MpdClient
}

impl<'a> MpdQueue<'a> {
    pub fn from_client(conn: &'a MpdClient) -> MpdQueue<'a> {
        MpdQueue { conn: conn }
    }

    /// Get song at some position in queue
    pub fn nth(&self, index: uint) -> MpdResult<MpdSong> {
        let song = unsafe { mpd_run_get_queue_song_pos(self.conn.conn, index as c_uint) };
        if song.is_null() {
            Err(FromClient::from_client(self.conn).unwrap())
        } else {
            Ok(MpdSong { song: song })
        }
    }

    /// Get song by queue id
    pub fn get(&self, id: uint) -> MpdResult<MpdSong> {
        let song = unsafe { mpd_run_get_queue_song_id(self.conn.conn, id as c_uint) };
        if song.is_null() {
            Err(FromClient::from_client(self.conn).unwrap())
        } else {
            Ok(MpdSong { song: song })
        }
    }

    /// Insert new song into queue at given position
    pub fn insert<T: ToSongUri>(&mut self, pos: uint, song: T) -> MpdResult<uint> {
        let uid = unsafe { mpd_run_add_id_to(self.conn.conn, song.song_uri().as_ptr(), pos as c_uint) };
        if uid < 0 {
            Err(FromClient::from_client(self.conn).unwrap())
        } else {
            Ok(uid as uint)
        }
    }

    /// Add song at the end of the queue
    pub fn push<T: ToSongUri>(&mut self, song: T) -> MpdResult<uint> {
        let uid = unsafe { mpd_run_add_id(self.conn.conn, song.song_uri().as_ptr()) };
        if uid < 0 {
            Err(FromClient::from_client(self.conn).unwrap())
        } else {
            Ok(uid as uint)
        }
    }

    /// Move song to some position in queue
    pub fn move_pos(&mut self, to: uint, from: uint) -> MpdResult<()> {
        if unsafe { mpd_run_move(self.conn.conn, from as c_uint, to as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Move songs in given range
    pub fn move_range(&mut self, pos: uint, start: uint, end: uint) -> MpdResult<()> {
        if unsafe { mpd_run_move_range(self.conn.conn, start as c_uint, end as c_uint, pos as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Move song to some position in queue by id
    pub fn move_to(&mut self, pos: uint, song: &MpdSong) -> MpdResult<()> {
        self.move_id(pos, song.id())
    }

    pub fn move_id(&mut self, pos: uint, id: uint) -> MpdResult<()> {
        if unsafe { mpd_run_move_id(self.conn.conn, id as c_uint, pos as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Swap two songs in given positions
    pub fn swap_pos(&mut self, song1: uint, song2: uint) -> MpdResult<()> {
        if unsafe { mpd_run_swap(self.conn.conn, song1 as c_uint, song2 as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Swap two songs in given positions by id
    pub fn swap(&mut self, song1: &MpdSong, song2: &MpdSong) -> MpdResult<()> {
        self.swap_id(song1.id(), song2.id())
    }

    pub fn swap_id(&mut self, song1: uint, song2: uint) -> MpdResult<()> {
        if unsafe { mpd_run_swap_id(self.conn.conn, song1 as c_uint, song2 as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    pub fn shuffle(&mut self) -> MpdResult<()> {
        if unsafe { mpd_run_shuffle(self.conn.conn) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    pub fn shuffle_range(&mut self, start: uint, end: uint) -> MpdResult<()> {
        if unsafe { mpd_run_shuffle_range(self.conn.conn, start as c_uint, end as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Remove a song
    pub fn remove(&mut self, song: &MpdSong) -> MpdResult<()> {
        self.remove_id(song.id())
    }

    pub fn remove_id(&mut self, id: uint) -> MpdResult<()> {
        if unsafe { mpd_run_delete_id(self.conn.conn, id as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Remove songs at given position
    pub fn remove_pos(&mut self, pos: uint) -> MpdResult<()> {
        if unsafe { mpd_run_delete(self.conn.conn, pos as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Remove songs in given range
    pub fn remove_range(&mut self, start: uint, end: uint) -> MpdResult<()> {
        if unsafe { mpd_run_delete_range(self.conn.conn, start as c_uint, end as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Iterate over songs in the queue
    pub fn iter(&self) -> MpdResult<MpdSongs> {
        if unsafe { mpd_send_list_queue_meta(self.conn.conn) } {
            Ok(MpdSongs { conn: self.conn })
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Iterate over songs in given range
    pub fn iter_range(&self, start: uint, end: uint) -> MpdResult<MpdSongs> {
        if unsafe { mpd_send_list_queue_range_meta(self.conn.conn, start as c_uint, end as c_uint) } {
            Ok(MpdSongs { conn: self.conn })
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
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

    /// Clear queue
    pub fn clear(&mut self) -> MpdResult<()> {
        if unsafe { mpd_run_clear(self.conn.conn) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Save queue into new playlist
    pub fn save(&mut self, name: &str) -> MpdResult<()> {
        if unsafe { mpd_run_save(self.conn.conn, name.to_c_str().as_ptr()) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Load queue from playlist
    pub fn load(&mut self, name: &str) -> MpdResult<()> {
        if unsafe { mpd_run_load(self.conn.conn, name.to_c_str().as_ptr()) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    /// Load queue from playlist object
    #[inline] pub fn load_playlist(&mut self, pl: &MpdPlaylist) -> MpdResult<()> {
        self.load(pl.path()[])
    }
}
