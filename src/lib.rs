#![feature(macro_rules, slicing_syntax, if_let)]

extern crate time;
extern crate libc;
extern crate collections;

use std::time::duration::Duration;
use std::ptr;
use std::c_str::ToCStr;
use collections::string::String;
use time::Timespec;

#[repr(C)] struct mpd_connection;
#[repr(C)] struct mpd_settings;
#[repr(C)] struct mpd_status;
#[repr(C)] struct mpd_song;
#[repr(C)] struct mpd_playlist;
#[repr(C)] struct mpd_output;

#[repr(C)]
#[deriving(Show)]
pub enum TagType {
    Unknown = -1,
    Artist = 0,
    Album = 1,
    AlbumArtist = 2,
    Title = 3,
    Track = 4,
    Name = 5,
    Genre = 6,
    Date = 7,
    Composer = 8,
    Performer = 9,
    Comment = 10,
    Disc = 11,

    MbArtistId = 12,
    MbAlbumId = 13,
    MbAlbumArtistId = 14,
    MbTrackId = 15,
}

impl TagType {
    fn name(&self) -> Option<String> {
        let name = unsafe { mpd_tag_name(*self) };
        if name == ptr::null() {
            None
        } else {
            Some(unsafe { String::from_raw_buf(name) })
        }
    }
}

impl std::str::FromStr for TagType {
    fn from_str(s: &str) -> Option<TagType> {
        Some(s.with_c_str(|s| unsafe { mpd_tag_name_parse(s as *const u8) }))
    }
}


#[repr(C)]
#[deriving(Show)]
pub enum MpdErrorKind {
    Success = 0,
    Oom = 1,
    Argument = 2,
    State = 3,
    Timeout = 4,
    System = 5,
    Resolver = 6,
    Malformed = 7,
    Closed = 8,
    Server = 9,
}

#[repr(C)]
#[deriving(Show)]
struct mpd_audio_format {
    sample_rate: u32,
    bits: u8,
    channels: u8,

    reserved0: u16,
    reserved1: u32
}

#[repr(C)]
#[deriving(Show)]
struct mpd_pair {
    name: *const u8,
    value: *const u8
}

#[repr(C)]
#[deriving(Show)]
pub enum MpdState {
    Unknown = 0,
    Stop = 1,
    Play = 2,
    Pause = 3,
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
            Err(MpdError::from_connection(conn.conn).unwrap())
        }
    }
}

impl std::fmt::Show for Playlist {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        try!(f.write(b"Playlist { "));
        try!(f.write(b"path: "));
        try!(self.path().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

impl std::fmt::Show for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        try!(f.write(b"Output { "));
        try!(f.write(b"name: "));
        try!(self.name().fmt(f));
        try!(f.write(b", id: "));
        try!(self.id().fmt(f));
        try!(f.write(b", enabled: "));
        try!(self.enabled().fmt(f));
        try!(f.write(b" }"));
        Ok(())
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

pub struct Playlists<'a> {
    conn: &'a MpdConnection
}

pub struct Songs<'a> {
    conn: &'a MpdConnection
}

impl<'a> Iterator<MpdResult<Playlist>> for Playlists<'a> {
    fn next(&mut self) -> Option<MpdResult<Playlist>> {
        match Playlist::from_connection(self.conn.conn) {
            Some(s) => Some(Ok(s)),
            None => match MpdError::from_connection(self.conn.conn) {
                Some(e) => Some(Err(e)),
                None => None
            }
        }
    }
}

impl<'a> Iterator<MpdResult<Song>> for Songs<'a> {
    fn next(&mut self) -> Option<MpdResult<Song>> {
        match Song::from_connection(self.conn.conn) {
            Some(s) => Some(Ok(s)),
            None => match MpdError::from_connection(self.conn.conn) {
                Some(e) => Some(Err(e)),
                None => None
            }
        }
    }
}

pub struct Song {
    song: *mut mpd_song
}

impl std::fmt::Show for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        try!(f.write(b"Song { "));
        try!(f.write(b"uri: "));
        try!(self.uri().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

impl Song {
    pub fn uri(&self) -> String { unsafe { String::from_raw_buf(mpd_song_get_uri(self.song as *const _)) } }
    pub fn tags(&self, kind: TagType, index: u32) -> Option<String> {
        let tag = unsafe { mpd_song_get_tag(self.song as *const _, kind, index) };
        if tag == ptr::null() {
            None
        } else {
            Some(unsafe { String::from_raw_buf(tag) })
        }
    }
    pub fn duration(&self) -> Duration { Duration::seconds(unsafe { mpd_song_get_duration(self.song as *const _) } as i64) }
    pub fn id(&self) -> u32 { unsafe { mpd_song_get_id(self.song as *const _) } }
    pub fn prio(&self) -> u32 { unsafe { mpd_song_get_prio(self.song as *const _) } }
    pub fn start(&self) -> u32 { unsafe { mpd_song_get_start(self.song as *const _) } }
    pub fn end(&self) -> u32 { unsafe { mpd_song_get_start(self.song as *const _) } }
    pub fn last_mod(&self) -> Timespec { Timespec::new(unsafe { mpd_song_get_last_modified(self.song as *const _) }, 0) }
    pub fn get_pos(&self) -> u32 { unsafe { mpd_song_get_pos(self.song as *const _) } }
    pub fn set_pos(&mut self, pos: u32) { unsafe { mpd_song_set_pos(self.song, pos) } }

    fn from_connection(connection: *mut mpd_connection) -> Option<Song> {
        let song = unsafe { mpd_recv_song(connection) };
        if song as *const _ == ptr::null::<mpd_song>() {
            None
        } else {
            Some(Song { song: song })
        }
    }
}

impl Drop for Song {
    fn drop(&mut self) {
        unsafe { mpd_song_free(self.song); }
    }
}

impl Clone for Song {
    fn clone(&self) -> Song {
        let song = unsafe { mpd_song_dup(self.song as *const _) };
        if song as *const _ == ptr::null::<mpd_song>() {
            panic!("Out of memory!")
        }

        Song { song: song }
    }
}

#[repr(C)]
#[deriving(Show)]
pub enum MpdServerErrorKind {
    Unknown = -1,
    NotList = 1,
    Argument = 2,
    Password = 3,
    Permission = 4,
    UnknownCmd = 5,
    NoExist = 50,
    PlaylistMax = 51,
    System = 52,
    PlaylistLoad = 53,
    UpdateAlready = 54,
    PlayerSync = 55,
    Exist = 56,
}

#[deriving(Show)]
pub enum MpdError {
    Server { kind: MpdServerErrorKind, index: u32, desc: String },
    System { code: i32, desc: String },
    Other { kind: MpdErrorKind, desc: String }
}

impl MpdError {
    fn from_connection(connection: *mut mpd_connection) -> Option<MpdError> {
        unsafe {
            let error = mpd_connection_get_error(connection as *const _);

            let err = match error {
                MpdErrorKind::Success => return None,
                MpdErrorKind::System => MpdError::System {
                    code: mpd_connection_get_system_error(connection as *const _),
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection as *const _)),
                },
                MpdErrorKind::Server => MpdError::Server {
                    kind: mpd_connection_get_server_error(connection as *const _),
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection as *const _)),
                    index: mpd_connection_get_server_error_location(connection as *const _),
                },
                _ => MpdError::Other {
                    kind: error,
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection as *const _)),
                }
            };

            mpd_connection_clear_error(connection);
            Some(err)
        }
    }
}

impl std::error::Error for MpdError {
    fn description(&self) -> &str {
        match *self {
            MpdError::System { .. } => "system error",
            MpdError::Server { ref kind, .. } => match *kind {
                MpdServerErrorKind::Unknown => "unknown error",
                MpdServerErrorKind::NotList => "not a list",
                MpdServerErrorKind::Argument => "invalid argument",
                MpdServerErrorKind::Password => "invalid password",
                MpdServerErrorKind::Permission => "access denied",
                MpdServerErrorKind::UnknownCmd => "unknown command",
                MpdServerErrorKind::NoExist => "object not found",
                MpdServerErrorKind::PlaylistMax => "playlist overflow",
                MpdServerErrorKind::System => "system error",
                MpdServerErrorKind::PlaylistLoad => "playlist load error",
                MpdServerErrorKind::UpdateAlready => "database already updating",
                MpdServerErrorKind::PlayerSync => "player sync error",
                MpdServerErrorKind::Exist => "object already exists",
            },
            MpdError::Other { ref kind, .. } => match *kind {
                MpdErrorKind::Success => "success",
                MpdErrorKind::Oom => "out of memory",
                MpdErrorKind::Argument => "invalid argument",
                MpdErrorKind::State => "invalid state",
                MpdErrorKind::Timeout => "operation timed out",
                MpdErrorKind::System => "system error",
                MpdErrorKind::Resolver => "name resolution error",
                MpdErrorKind::Malformed => "malformed hostname",
                MpdErrorKind::Closed => "connection closed",
                MpdErrorKind::Server => "server error",
            }
        }
    }

    fn detail(&self) -> Option<String> {
        Some(match *self {
            MpdError::System { ref desc, .. } => desc.clone(),
            MpdError::Server { ref desc, .. } => desc.clone(),
            MpdError::Other { ref desc, .. } => desc.clone(),
        })
    }

    fn cause(&self) -> Option<&std::error::Error> { None }
}

pub type MpdResult<T> = Result<T, MpdError>;

#[link(name = "mpdclient")]
extern {
    fn mpd_connection_new(host: *const u8, port: libc::c_uint, timeout_ms: libc::c_uint) -> *mut mpd_connection;
    fn mpd_connection_free(connection: *mut mpd_connection);
    fn mpd_connection_get_settings(connection: *const mpd_connection) -> *const mpd_settings;
    fn mpd_connection_set_timeout(connection: *mut mpd_connection, timeout_ms: libc::c_uint);
    fn mpd_connection_get_fd(connection: *const mpd_connection) -> libc::c_int;
    fn mpd_connection_get_error(connection: *const mpd_connection) -> MpdErrorKind;
    fn mpd_connection_get_error_message(connection: *const mpd_connection) -> *const u8;
    fn mpd_connection_get_server_error(connection: *const mpd_connection) -> MpdServerErrorKind;
    fn mpd_connection_get_server_error_location(connection: *const mpd_connection) -> libc::c_uint;
    fn mpd_connection_get_system_error(connection: *const mpd_connection) -> libc::c_int;
    fn mpd_connection_clear_error(connection: *mut mpd_connection) -> bool;
    fn mpd_connection_get_server_version(connection: *const mpd_connection) -> [libc::c_uint, ..3];
    fn mpd_connection_cmp_server_version(connection: *const mpd_connection, major: libc::c_uint, minor: libc::c_uint, patch: libc::c_uint) -> libc::c_int;

    fn mpd_settings_new(host: *const u8, port: libc::c_uint, timeout_ms: libc::c_uint, reserved: *const u8, password: *const u8) -> *mut mpd_settings;
    fn mpd_settings_free(settings: *mut mpd_settings);
    fn mpd_settings_get_host(settings: *const mpd_settings) -> *const u8;
    fn mpd_settings_get_port(settings: *const mpd_settings) -> libc::c_uint;
    fn mpd_settings_get_timeout_ms(settings: *const mpd_settings) -> libc::c_uint;
    fn mpd_settings_get_password(settings: *const mpd_settings) -> *const u8;

    fn mpd_send_command(connection: *mut mpd_connection, command: *const u8, ...) -> bool;

    fn mpd_response_finish(connection: *mut mpd_connection) -> bool;
    fn mpd_response_next(connection: *mut mpd_connection) -> bool;

    fn mpd_send_password(connection: *mut mpd_connection, password: *const u8) -> bool;
    fn mpd_run_password(connection: *mut mpd_connection, password: *const u8) -> bool;

    fn mpd_recv_pair(connection: *mut mpd_connection) -> *mut mpd_pair;
    fn mpd_recv_pair_named(connection: *mut mpd_connection, name: *const u8) -> *mut mpd_pair;
    fn mpd_return_pair(connection: *mut mpd_connection, pair: *mut mpd_pair);
    fn mpd_enqueue_pair(connection: *mut mpd_connection, pair: *mut mpd_pair);

    fn mpd_command_list_begin(connection: *mut mpd_connection, discrete_ok: bool) -> bool;
    fn mpd_command_list_end(connection: *mut mpd_connection) -> bool;

    fn mpd_status_feed(status: *mut mpd_status, pair: *const mpd_pair);
    fn mpd_send_status(connection: *mut mpd_connection) -> bool;
    fn mpd_recv_status(connection: *mut mpd_connection) -> *mut mpd_status;
    fn mpd_run_status(connection: *mut mpd_connection) -> *mut mpd_status;
    fn mpd_status_free(status: *mut mpd_status);
    fn mpd_status_get_volume(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_repeat(status: *const mpd_status) -> bool;
    fn mpd_status_get_random(status: *const mpd_status) -> bool;
    fn mpd_status_get_single(status: *const mpd_status) -> bool;
    fn mpd_status_get_consume(status: *const mpd_status) -> bool;
    fn mpd_status_get_queue_length(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_queue_version(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_state(status: *const mpd_status) -> MpdState;
    fn mpd_status_get_crossfade(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_mixrampdb(status: *const mpd_status) -> f32;
    fn mpd_status_get_mixrampdelay(status: *const mpd_status) -> f32;
    fn mpd_status_get_song_pos(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_song_id(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_next_song_pos(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_next_song_id(status: *const mpd_status) -> libc::c_int;
    fn mpd_status_get_elapsed_time(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_elapsed_ms(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_total_time(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_kbit_rate(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_audio_format(status: *const mpd_status) -> *const mpd_audio_format;
    fn mpd_status_get_update_id(status: *const mpd_status) -> libc::c_uint;
    fn mpd_status_get_error(status: *const mpd_status) -> *const u8;

    fn mpd_run_play(connection: *mut mpd_connection) -> bool;
    fn mpd_run_play_pos(connection: *mut mpd_connection, song_pos: u32) -> bool;
    fn mpd_run_play_id(connection: *mut mpd_connection, song_id: u32) -> bool;
    fn mpd_run_pause(connection: *mut mpd_connection, mode: bool) -> bool;
    fn mpd_run_stop(connection: *mut mpd_connection) -> bool;
    fn mpd_run_next(connection: *mut mpd_connection) -> bool;
    fn mpd_run_previous(connection: *mut mpd_connection) -> bool;
    fn mpd_run_set_volume(connection: *mut mpd_connection, volume: libc::c_uint) -> bool;
    fn mpd_run_change_volume(connection: *mut mpd_connection, volume: libc::c_int) -> bool;

    fn mpd_song_dup(song: *const mpd_song) -> *mut mpd_song;
    fn mpd_song_free(song: *mut mpd_song);
    fn mpd_song_get_uri(song: *const mpd_song) -> *const u8;
    fn mpd_song_get_tag(song: *const mpd_song, typ: TagType, idx: libc::c_uint) -> *const u8;
    fn mpd_song_get_duration(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_get_start(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_get_end(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_get_last_modified(song: *const mpd_song) -> libc::time_t;
    fn mpd_song_get_id(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_get_pos(song: *const mpd_song) -> libc::c_uint;
    fn mpd_song_set_pos(song: *mut mpd_song, pos: libc::c_uint);
    fn mpd_song_get_prio(song: *const mpd_song) -> libc::c_uint;
    fn mpd_recv_song(connection: *mut mpd_connection) -> *mut mpd_song;

    fn mpd_run_current_song(connection: *mut mpd_connection) -> *mut mpd_song;

    fn mpd_tag_name(typ: TagType) -> *const u8;
    fn mpd_tag_name_parse(name: *const u8) -> TagType;

    fn mpd_playlist_dup(playlist: *const mpd_playlist) -> *mut mpd_playlist;
    fn mpd_recv_playlist(playlist: *mut mpd_connection) -> *mut mpd_playlist;
    fn mpd_playlist_free(playlist: *mut mpd_playlist);
    fn mpd_playlist_get_last_modified(playlist: *const mpd_playlist) -> libc::time_t;
    fn mpd_playlist_get_path(playlist: *const mpd_playlist) -> *const u8;

    fn mpd_send_list_playlists(connection: *mut mpd_connection) -> bool;
    fn mpd_send_list_playlist(connection: *mut mpd_connection, name: *const u8) -> bool;

    fn mpd_output_free(output: *mut mpd_output);
    fn mpd_output_get_name(output: *const mpd_output) -> *const u8;
    fn mpd_output_get_id(output: *const mpd_output) -> libc::c_uint;
    fn mpd_output_get_enabled(output: *const mpd_output) -> bool;
    fn mpd_run_enable_output(connection: *mut mpd_connection, output_id: libc::c_uint) -> bool;
    fn mpd_run_disable_output(connection: *mut mpd_connection, output_id: libc::c_uint) -> bool;
    fn mpd_run_toggle_output(connection: *mut mpd_connection, output_id: libc::c_uint) -> bool;
    fn mpd_send_outputs(connection: *mut mpd_connection) -> bool;
    fn mpd_recv_output(connection: *mut mpd_connection) -> *mut mpd_output;
}

pub struct Output {
    output: *mut mpd_output
}

pub struct Outputs<'a> {
    conn: &'a mut MpdConnection
}

impl<'a> Iterator<MpdResult<Output>> for Outputs<'a> {
    fn next(&mut self) -> Option<MpdResult<Output>> {
        match Output::from_connection(self.conn.conn) {
            Some(s) => Some(Ok(s)),
            None => match MpdError::from_connection(self.conn.conn) {
                Some(e) => Some(Err(e)),
                None => None
            }
        }
    }
}

impl Output {
    fn from_connection(connection: *mut mpd_connection) -> Option<Output> {
        let output = unsafe { mpd_recv_output(connection) };
        if output as *const _ == ptr::null::<mpd_output>() {
            None
        } else {
            Some(Output { output: output })
        }
    }

    fn id(&self) -> u32 { unsafe { mpd_output_get_id(self.output as *const _) } }
    fn name(&self) -> String { unsafe { String::from_raw_buf(mpd_output_get_name(self.output as *const _)) } }
    fn enabled(&self) -> bool { unsafe { mpd_output_get_enabled(self.output as *const _) } }

    fn toggle(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_toggle_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(MpdError::from_connection(conn.conn).unwrap())
        }
    }

    fn disable(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_disable_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(MpdError::from_connection(conn.conn).unwrap())
        }
    }

    fn enable(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_enable_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(MpdError::from_connection(conn.conn).unwrap())
        }
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        unsafe { mpd_output_free(self.output) }
    }
}

pub struct MpdConnection {
    conn: *mut mpd_connection
}

// rate, bits, chans
type AudioFormat = (u32, u8, u8);

#[deriving(Show)]
pub struct MpdStatus {
    volume: i32,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    queue_length: u32,
    queue_version: u32,
    state: MpdState,
    crossfade: u32,
    mixrampdb: f32,
    mixrampdelay: Option<f32>,
    song: Option<(i32, i32)>, // id, pos
    next_song: Option<(i32, i32)>,
    elapsed_time: Duration,
    total_time: Duration,
    kbit_rate: u32,
    audio_format: Option<AudioFormat>,
    update_id: u32,
    error: Option<String>
}

impl MpdStatus {
    fn from_connection(connection: *mut mpd_connection) -> Option<MpdStatus> {
        unsafe {
            let status = mpd_run_status(connection);
            if status as *const _ == ptr::null::<mpd_status>() {
                return None
            }

            let s = status as *const _;
            let aformat = mpd_status_get_audio_format(s);
            let error = mpd_status_get_error(s);
            let song_id = mpd_status_get_song_id(s);
            let next_song_id = mpd_status_get_next_song_id(s);
            let mixramp = mpd_status_get_mixrampdelay(s);

            let result = MpdStatus {
                volume: mpd_status_get_volume(s),
                repeat: mpd_status_get_repeat(s),
                random: mpd_status_get_random(s),
                single: mpd_status_get_single(s),
                consume: mpd_status_get_consume(s),
                queue_length: mpd_status_get_queue_length(s),
                queue_version: mpd_status_get_queue_version(s),
                state: mpd_status_get_state(s),
                crossfade: mpd_status_get_crossfade(s),
                mixrampdb: mpd_status_get_mixrampdb(s),
                mixrampdelay: if mixramp < 0f32 { None } else { Some(mixramp) },
                song: if song_id < 0 { None } else { Some((song_id, mpd_status_get_song_pos(s))) },
                next_song: if next_song_id < 0 { None } else { Some((next_song_id, mpd_status_get_next_song_pos(s))) },
                elapsed_time: Duration::milliseconds(mpd_status_get_elapsed_ms(s) as i64),
                total_time: Duration::seconds(mpd_status_get_total_time(s) as i64),
                kbit_rate: mpd_status_get_kbit_rate(s),
                audio_format: if aformat == ptr::null() { None } else { Some(((*aformat).sample_rate, (*aformat).bits, (*aformat).channels)) },
                update_id: mpd_status_get_update_id(s),
                error: if error == ptr::null() { None } else { Some(String::from_raw_buf(error)) }
            };

            mpd_status_free(status);

            Some(result)
        }
    }
}

#[deriving(Show)]
pub struct MpdSettings {
    host: Option<String>,
    port: u32,
    timeout: Duration,
    password: Option<String>,
}

impl MpdSettings {
    fn from_connection(connection: *mut mpd_connection) -> Option<MpdSettings> {
        unsafe {
            let settings = mpd_connection_get_settings(connection as *const _);
            if settings == ptr::null() { None } else {
                let host = mpd_settings_get_host(settings);
                let password = mpd_settings_get_password(settings);

                let result = MpdSettings {
                    host: if host == ptr::null() { None } else { Some(String::from_raw_buf(host)) },
                    port: mpd_settings_get_port(settings),
                    timeout: Duration::milliseconds(mpd_settings_get_timeout_ms(settings) as i64),
                    password: if password == ptr::null() { None } else { Some(String::from_raw_buf(password)) },
                };

                Some(result)
            }
        }
    }

    unsafe fn to_c_struct(&self) -> *mut mpd_settings {
        let host = self.host.clone().map(|v| v.to_c_str());
        let password = self.password.clone().map(|v| v.to_c_str());

        mpd_settings_new(match host {
            Some(h) => h.as_ptr() as *const u8,
            None => ptr::null()
        }, self.port, self.timeout.num_milliseconds() as u32, ptr::null(),
        match password {
            Some(p) => p.as_ptr() as *const u8,
            None => ptr::null()
        })
    }
}

impl MpdConnection {
    fn new(host: Option<&str>, port: u32) -> Option<MpdResult<MpdConnection>> {
        MpdConnection::new_with_timeout(host, port, Duration::zero())
    }

    fn new_with_timeout(host: Option<&str>, port: u32, timeout: Duration) -> Option<MpdResult<MpdConnection>> {
        unsafe {
            let host = host.map(|v| v.to_c_str());
            let conn = mpd_connection_new(match host {
                Some(v) => v.as_ptr() as *const u8,
                None => ptr::null()
            }, port, timeout.num_milliseconds() as u32);

            if conn as *const _ == ptr::null::<mpd_connection>() { None } else {
                Some(match MpdError::from_connection(conn) {
                    None => Ok(MpdConnection { conn: conn }),
                    Some(e) => {
                        mpd_connection_free(conn);
                        Err(e)
                    }
                })
            }
        }
    }

    pub fn authorize(&mut self, password: String) -> MpdResult<()> { if ! password.with_c_str(|s| unsafe { mpd_run_password(self.conn, s as *const u8) }) { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }

    pub fn settings(&self) -> Option<MpdSettings> { MpdSettings::from_connection(self.conn) }

    pub fn play(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_play(self.conn) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }
    pub fn stop(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_stop(self.conn) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }
    pub fn pause(&mut self, mode: bool) -> MpdResult<()> { if ! unsafe { mpd_run_pause(self.conn, mode) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }
    pub fn set_volume(&mut self, vol: u32) -> MpdResult<()> { if ! unsafe { mpd_run_set_volume(self.conn, vol) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }
    pub fn change_volume(&mut self, vol: i32) -> MpdResult<()> { if ! unsafe { mpd_run_change_volume(self.conn, vol) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }

    pub fn next(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_next(self.conn) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }
    pub fn prev(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_previous(self.conn) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }

    pub fn play_pos(&mut self, pos: u32) -> MpdResult<()> { if ! unsafe { mpd_run_play_pos(self.conn, pos) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }
    pub fn play_id(&mut self, pos: u32) -> MpdResult<()> { if ! unsafe { mpd_run_play_id(self.conn, pos) } { return Err(MpdError::from_connection(self.conn).unwrap()) } Ok(()) }

    pub fn status(&self) -> MpdResult<MpdStatus> { MpdStatus::from_connection(self.conn).map(|s| Ok(s)).unwrap_or_else(|| Err(MpdError::from_connection(self.conn).unwrap())) }
    pub fn current_song(&self) -> MpdResult<Song> {
        let song = unsafe { mpd_run_current_song(self.conn) };
        if song as *const _ == ptr::null::<mpd_song>() {
            Err(MpdError::from_connection(self.conn).unwrap())
        } else {
            Ok(Song { song: song })
        }
    }

    pub fn playlists(&mut self) -> MpdResult<Playlists> {
        if unsafe { mpd_send_list_playlists(self.conn) } {
            Ok(Playlists { conn: self })
        } else {
            Err(MpdError::from_connection(self.conn).unwrap())
        }
    }

    pub fn outputs(&mut self) -> MpdResult<Outputs> {
        if unsafe { mpd_send_outputs(self.conn) } {
            Ok(Outputs { conn: self })
        } else {
            Err(MpdError::from_connection(self.conn).unwrap())
        }
    }
}

impl Drop for MpdConnection {
    fn drop(&mut self) {
        unsafe { mpd_connection_free(self.conn) }
    }
}

#[test]
fn test_conn() {
    //let c = MpdConnection::new(Some("192.168.1.10"), 6600);
    let c = MpdConnection::new(None, 6600);
    let mut conn = match c {
        None => panic!("connection is None"),
        Some(Err(e)) => panic!("connection error: {}", e),
        Some(Ok(c)) => c
    };

    println!("{}", conn.stop());
    println!("{}", conn.set_volume(0));
    println!("{}", conn.settings());

    let mut playlist: MpdResult<Playlist> = Err(MpdError::Other { kind: MpdErrorKind::Success, desc: "".to_string() });

    for pl in conn.playlists().unwrap() {
        println!("{}", pl);
        playlist = pl;
    }

    for s in playlist.unwrap().songs(&mut conn).unwrap() {
        println!("{}", s);
    }

    for o in conn.outputs().unwrap() {
        println!("{}", o);
    }

    panic!("{}", conn.current_song());
}

//#[test]
//fn test_live_status() {
    //let mut conn = MpdConnection::new("192.168.1.10:6600").unwrap();
    //panic!("{}", conn.status());
//}

//#[test]
//fn test_live_stats() {
    //let mut conn = MpdConnection::new("192.168.1.10:6600").unwrap();
    //panic!("{}", conn.stats());
//}

//#[test]
//fn test_live_search() {
    //let mut conn = MpdConnection::new("192.168.1.10:6600").unwrap();
    //panic!("{}", conn.search("file", ""));
//}
