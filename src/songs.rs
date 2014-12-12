
use libc::{c_uint, time_t, c_uchar};
use std::time::duration::Duration;
use std::fmt::{Show, Error, Formatter};
use time::Timespec;
use std::iter::count;
use std::collections::BTreeMap;
use std::c_str::CString;

use error::MpdResult;
use client::{mpd_connection, MpdClient, FromClient};
use rustc_serialize::{Encoder, Encodable};
use tags::MpdTagType;

#[repr(C)] pub struct mpd_song;

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_song_dup(song: *const mpd_song) -> *mut mpd_song;
    fn mpd_song_free(song: *mut mpd_song);
    fn mpd_song_get_uri(song: *const mpd_song) -> *const c_uchar;
    fn mpd_song_get_tag(song: *const mpd_song, typ: MpdTagType, idx: c_uint) -> *const c_uchar;
    fn mpd_song_get_duration(song: *const mpd_song) -> c_uint;
    fn mpd_song_get_start(song: *const mpd_song) -> c_uint;
    fn mpd_song_get_end(song: *const mpd_song) -> c_uint;
    fn mpd_song_get_last_modified(song: *const mpd_song) -> time_t;
    fn mpd_song_get_id(song: *const mpd_song) -> c_uint;
    fn mpd_song_get_pos(song: *const mpd_song) -> c_uint;
    fn mpd_song_set_pos(song: *mut mpd_song, pos: c_uint);
    fn mpd_song_get_prio(song: *const mpd_song) -> c_uint;
    fn mpd_recv_song(connection: *mut mpd_connection) -> *mut mpd_song;

    fn mpd_run_seek_id(connection: *mut mpd_connection, song_id: c_uint, t: c_uint) -> bool;
}

pub struct MpdSongs<'a> {
    pub conn: &'a MpdClient
}

impl<'a, S: Encoder<E>, E> Encodable<S, E> for MpdSongs<'a> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(0, |s| self.enumerate().fold(Ok(()), |r, (i, v)| r.and_then(|()| s.emit_seq_elt(i, |s| v.encode(s)))))
    }
}

impl<'a> Iterator<MpdResult<MpdSong>> for MpdSongs<'a> {
    fn next(&mut self) -> Option<MpdResult<MpdSong>> {
        match FromClient::from_client(self.conn) {
            Some(song) => Some(Ok(song)),
            None => match FromClient::from_client(self.conn) {
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

impl<S: Encoder<E>, E> Encodable<S, E> for MpdSong {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_struct("MpdSong", 8, |s| {
            s.emit_struct_field("uri", 0, |s| s.emit_str(self.uri()[])).and_then(|()|
            s.emit_struct_field("duration", 1, |s| s.emit_i64(self.duration().num_milliseconds()))).and_then(|()|
            s.emit_struct_field("id", 2, |s| s.emit_uint(self.id()))).and_then(|()|
            s.emit_struct_field("prio", 3, |s| s.emit_uint(self.prio()))).and_then(|()|
            s.emit_struct_field("slice", 4, |s| s.emit_seq(
                    2, |s| s.emit_seq_elt(0, |s| s.emit_i64(self.start().num_milliseconds())).and_then(|()|
                           s.emit_seq_elt(1, |s| s.emit_option(|s| match self.end() {
                               Some(ref d) => s.emit_option_some(|s| s.emit_i64(d.num_milliseconds())),
                               None => s.emit_option_none()
                           })))
                    ))).and_then(|()|
            s.emit_struct_field("last_modified", 5, |s| s.emit_i64(self.last_mod().sec))).and_then(|()|
            s.emit_struct_field("pos", 6, |s| s.emit_uint(self.pos()))).and_then(|()|
            s.emit_struct_field("tags", 7, |s| self.first_tags().encode(s)))
        })
    }
}

impl MpdSong {
    pub fn uri(&self) -> String { unsafe { String::from_raw_buf(mpd_song_get_uri(self.song as *const _)) } }

    pub fn tag(&self, kind: MpdTagType, index: uint) -> Option<String> {
        let tag = unsafe { mpd_song_get_tag(self.song as *const _, kind, index as c_uint) };
        if tag.is_null() {
            None
        } else {
            Some(unsafe { String::from_raw_buf(tag) })
        }
    }

    pub fn tags(&self, kind: MpdTagType) -> Vec<String> {
        let song = self.song as *const _;
        count(0, 1).map(|idx| unsafe { mpd_song_get_tag(song, kind, idx) }).take_while(|v| !v.is_null())
            .map(|v| unsafe { String::from_raw_buf(v) }).collect()
    }

    pub fn all_tags(&self) -> BTreeMap<MpdTagType, Vec<String>> {
        MpdTagType::variants().iter().map(|k| (*k, self.tags(*k))).filter(|&(_, ref v)| !v.is_empty()).collect()
    }

    pub fn first_tags(&self) -> BTreeMap<MpdTagType, String> {
        MpdTagType::variants().iter().filter_map(|k| self.tag(*k, 0).map(|v| (*k, v))).collect()
    }

    pub fn duration(&self) -> Duration { Duration::seconds(unsafe { mpd_song_get_duration(self.song as *const _) } as i64) }
    pub fn id(&self) -> uint { unsafe { mpd_song_get_id(self.song as *const _) as uint } }
    pub fn prio(&self) -> uint { unsafe { mpd_song_get_prio(self.song as *const _) as uint } }
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
    pub fn set_pos(&mut self, pos: uint) { unsafe { mpd_song_set_pos(self.song, pos as c_uint) } }

    pub fn seek(&mut self, conn: &mut MpdClient, pos: Duration) -> MpdResult<()> {
        if unsafe { mpd_run_seek_id(conn.conn, self.id() as c_uint, pos.num_seconds() as c_uint) } {
            Ok(())
        } else {
            Err(FromClient::from_client(conn).unwrap())
        }
    }

    pub fn play(&self, conn: &mut MpdClient) -> MpdResult<()> {
        conn.play_id(self.id())
    }
}

impl FromClient for MpdSong {
    fn from_client(cli: &MpdClient) -> Option<MpdSong> {
        let song = unsafe { mpd_recv_song(cli.conn) };
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
