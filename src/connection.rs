
use libc;
use std::time::duration::Duration;
use std::c_str::ToCStr;
use std::ptr;
use std::os::unix::prelude::{AsRawFd, Fd};

use error::{MpdResult, MpdError, MpdErrorKind, MpdServerErrorKind};
use outputs::MpdOutputs;
use playlists::MpdPlaylists;
use songs::{MpdSong, mpd_song};
use status::MpdStatus;
use settings::MpdSettings;
use stats::MpdStats;

pub use error::mpd_connection;

pub trait FromConn {
    fn from_conn(connection: &MpdConnection) -> Option<Self>;
}

impl FromConn for MpdError {
    fn from_conn(conn: &MpdConnection) -> Option<MpdError> {
        let connection = conn.conn as *const _;
        unsafe {
            let error = mpd_connection_get_error(connection);

            let err = match error {
                MpdErrorKind::Success => return None,
                MpdErrorKind::System => MpdError::System {
                    code: mpd_connection_get_system_error(connection),
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection)),
                },
                MpdErrorKind::Server => MpdError::Server {
                    kind: mpd_connection_get_server_error(connection as *const _),
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection)),
                    index: mpd_connection_get_server_error_location(connection),
                },
                _ => MpdError::Other {
                    kind: error,
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection)),
                }
            };

            mpd_connection_clear_error(conn.conn);
            Some(err)
        }
    }
}


#[link(name = "mpdclient")]
extern {
    fn mpd_connection_new(host: *const i8, port: libc::c_uint, timeout_ms: libc::c_uint) -> *mut mpd_connection;
    fn mpd_connection_free(connection: *mut mpd_connection);
    fn mpd_connection_set_timeout(connection: *mut mpd_connection, timeout_ms: libc::c_uint);
    fn mpd_connection_get_fd(connection: *const mpd_connection) -> libc::c_int;
    fn mpd_connection_get_server_version(connection: *const mpd_connection) -> *const [libc::c_uint, ..3];
    //fn mpd_connection_cmp_server_version(connection: *const mpd_connection, major: libc::c_uint, minor: libc::c_uint, patch: libc::c_uint) -> libc::c_int;

    /*
    fn mpd_send_command(connection: *mut mpd_connection, command: *const i8, ...) -> bool;

    fn mpd_response_finish(connection: *mut mpd_connection) -> bool;
    fn mpd_response_next(connection: *mut mpd_connection) -> bool;

    fn mpd_send_password(connection: *mut mpd_connection, password: *const i8) -> bool;
    */
    fn mpd_run_password(connection: *mut mpd_connection, password: *const i8) -> bool;

    /*
    fn mpd_recv_pair(connection: *mut mpd_connection) -> *mut mpd_pair;
    fn mpd_recv_pair_named(connection: *mut mpd_connection, name: *const i8) -> *mut mpd_pair;
    fn mpd_return_pair(connection: *mut mpd_connection, pair: *mut mpd_pair);
    fn mpd_enqueue_pair(connection: *mut mpd_connection, pair: *mut mpd_pair);

    fn mpd_command_list_begin(connection: *mut mpd_connection, discrete_ok: bool) -> bool;
    fn mpd_command_list_end(connection: *mut mpd_connection) -> bool;
    */

    fn mpd_run_play(connection: *mut mpd_connection) -> bool;
    fn mpd_run_play_pos(connection: *mut mpd_connection, song_pos: u32) -> bool;
    fn mpd_run_play_id(connection: *mut mpd_connection, song_id: u32) -> bool;
    fn mpd_run_pause(connection: *mut mpd_connection, mode: bool) -> bool;
    fn mpd_run_stop(connection: *mut mpd_connection) -> bool;
    fn mpd_run_next(connection: *mut mpd_connection) -> bool;
    fn mpd_run_previous(connection: *mut mpd_connection) -> bool;
    fn mpd_run_set_volume(connection: *mut mpd_connection, volume: libc::c_uint) -> bool;
    fn mpd_run_change_volume(connection: *mut mpd_connection, volume: libc::c_int) -> bool;

    fn mpd_run_current_song(connection: *mut mpd_connection) -> *mut mpd_song;

    fn mpd_run_rescan(connection: *mut mpd_connection, path: *const i8) -> libc::c_uint;
    fn mpd_run_update(connection: *mut mpd_connection, path: *const i8) -> libc::c_uint;

    fn mpd_connection_get_error(connection: *const mpd_connection) -> MpdErrorKind;
    fn mpd_connection_get_error_message(connection: *const mpd_connection) -> *const u8;
    fn mpd_connection_get_server_error(connection: *const mpd_connection) -> MpdServerErrorKind;
    fn mpd_connection_get_server_error_location(connection: *const mpd_connection) -> libc::c_uint;
    fn mpd_connection_get_system_error(connection: *const mpd_connection) -> libc::c_int;
    fn mpd_connection_clear_error(connection: *mut mpd_connection) -> bool;
}

pub struct MpdConnection {
    pub conn: *mut mpd_connection
}


impl MpdConnection {
    pub fn new(host: Option<&str>, port: u32) -> Option<MpdResult<MpdConnection>> {
        MpdConnection::new_with_timeout(host, port, Duration::zero())
    }

    pub fn new_with_timeout(host: Option<&str>, port: u32, timeout: Duration) -> Option<MpdResult<MpdConnection>> {
        unsafe {
            let host = host.map(|v| v.to_c_str());
            let conn = mpd_connection_new(match host {
                Some(v) => v.as_ptr(),
                None => ptr::null()
            }, port, timeout.num_milliseconds() as u32);

            if conn.is_null() { None } else {
                let mut result = MpdConnection { conn: conn };
                Some(match FromConn::from_conn(&mut result) {
                    None => Ok(result),
                    Some(e) => {
                        mpd_connection_free(conn);
                        Err(e)
                    }
                })
            }
        }
    }

    pub fn authorize(&mut self, password: String) -> MpdResult<()> { if ! password.with_c_str(|s| unsafe { mpd_run_password(self.conn, s) }) { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

    pub fn set_timeout(&mut self, timeout: Duration) { unsafe { mpd_connection_set_timeout(self.conn, timeout.num_milliseconds() as libc::c_uint) } }

    pub fn version(&self) -> (u32, u32, u32) {
        let version = unsafe { * mpd_connection_get_server_version(self.conn as *const _) };
        (version[0], version[1], version[2])
    }
    pub fn settings(&self) -> Option<MpdSettings> { FromConn::from_conn(self) }

    pub fn play(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_play(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn stop(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_stop(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn pause(&mut self, mode: bool) -> MpdResult<()> { if ! unsafe { mpd_run_pause(self.conn, mode) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn set_volume(&mut self, vol: u32) -> MpdResult<()> { if ! unsafe { mpd_run_set_volume(self.conn, vol) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn change_volume(&mut self, vol: i32) -> MpdResult<()> { if ! unsafe { mpd_run_change_volume(self.conn, vol) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

    pub fn next(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_next(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn prev(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_previous(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

    pub fn play_pos(&mut self, pos: u32) -> MpdResult<()> { if ! unsafe { mpd_run_play_pos(self.conn, pos) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn play_id(&mut self, pos: u32) -> MpdResult<()> { if ! unsafe { mpd_run_play_id(self.conn, pos) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

    pub fn status(&self) -> MpdResult<MpdStatus> { FromConn::from_conn(self).map(|s| Ok(s)).unwrap_or_else(|| Err(FromConn::from_conn(self).unwrap())) }
    pub fn stats(&self) -> MpdResult<MpdStats> { FromConn::from_conn(self).map(|s| Ok(s)).unwrap_or_else(|| Err(FromConn::from_conn(self).unwrap())) }
    pub fn current_song(&self) -> MpdResult<MpdSong> {
        let song = unsafe { mpd_run_current_song(self.conn) };
        if song.is_null() {
            Err(FromConn::from_conn(self).unwrap())
        } else {
            Ok(MpdSong { song: song })
        }
    }

    pub fn playlists(&self) -> MpdResult<MpdPlaylists> { MpdPlaylists::from_conn(self).map(|s| Ok(s)).unwrap_or_else(|| Err(FromConn::from_conn(self).unwrap())) }
    pub fn outputs(&self) -> MpdResult<MpdOutputs> { MpdOutputs::from_conn(self).map(|s| Ok(s)).unwrap_or_else(|| Err(FromConn::from_conn(self).unwrap())) }

    pub fn update(&mut self, path: Option<String>) -> MpdResult<u32> {
        let cpath = path.map(|p| p.to_c_str());
        match unsafe { mpd_run_update(self.conn, match cpath {
            Some(p) => p.as_ptr(),
            None => ptr::null()
        }) } {
            0 => match FromConn::from_conn(self) {
                None => Ok(0),
                Some(e) => Err(e)
            },
            uid @ _ => Ok(uid)
        }
    }

    pub fn rescan(&mut self, path: Option<String>) -> MpdResult<u32> {
        let cpath = path.map(|p| p.to_c_str());
        match unsafe { mpd_run_rescan(self.conn, match cpath {
            Some(p) => p.as_ptr(),
            None => ptr::null()
        }) } {
            0 => match FromConn::from_conn(self) {
                None => Ok(0),
                Some(e) => Err(e)
            },
            uid @ _ => Ok(uid)
        }
    }
}

impl Drop for MpdConnection {
    fn drop(&mut self) {
        unsafe { mpd_connection_free(self.conn) }
    }
}

impl AsRawFd for MpdConnection {
    fn as_raw_fd(&self) -> Fd { unsafe { mpd_connection_get_fd(self.conn as *const _) as Fd } }
}

