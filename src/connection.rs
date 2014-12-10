
use libc::{c_uint, c_int, c_char, c_uchar, c_float};
use std::time::duration::Duration;
use std::c_str::ToCStr;
use std::ptr;
use std::os::unix::prelude::{AsRawFd, Fd};
use std::io::net::ip::Port;

use error::{MpdResult, MpdError, MpdErrorKind, MpdServerErrorKind};
use outputs::{MpdOutputs, MpdOutput};
use playlists::MpdPlaylists;
use songs::{MpdSong, mpd_song};
use status::MpdStatus;
use settings::MpdSettings;
use stats::MpdStats;
use queue::MpdQueue;
use idle::{MpdIdle, MpdEvent};

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
                    code: mpd_connection_get_system_error(connection) as int,
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection)),
                },
                MpdErrorKind::Server => MpdError::Server {
                    kind: mpd_connection_get_server_error(connection as *const _),
                    desc: String::from_raw_buf(mpd_connection_get_error_message(connection)),
                    index: mpd_connection_get_server_error_location(connection) as uint,
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
    fn mpd_connection_new(host: *const c_char, port: c_uint, timeout_ms: c_uint) -> *mut mpd_connection;
    fn mpd_connection_free(connection: *mut mpd_connection);
    fn mpd_connection_set_timeout(connection: *mut mpd_connection, timeout_ms: c_uint);
    fn mpd_connection_get_fd(connection: *const mpd_connection) -> c_int;
    fn mpd_connection_get_server_version(connection: *const mpd_connection) -> *const [c_uint, ..3];
    //fn mpd_connection_cmp_server_version(connection: *const mpd_connection, major: c_uint, minor: c_uint, patch: c_uint) -> c_int;

    /*
    fn mpd_send_command(connection: *mut mpd_connection, command: *const c_char, ...) -> bool;

    fn mpd_response_finish(connection: *mut mpd_connection) -> bool;
    fn mpd_response_next(connection: *mut mpd_connection) -> bool;

    fn mpd_send_password(connection: *mut mpd_connection, password: *const c_char) -> bool;
    */
    fn mpd_run_password(connection: *mut mpd_connection, password: *const c_char) -> bool;

    /*
    fn mpd_recv_pair(connection: *mut mpd_connection) -> *mut mpd_pair;
    fn mpd_recv_pair_named(connection: *mut mpd_connection, name: *const c_char) -> *mut mpd_pair;
    fn mpd_return_pair(connection: *mut mpd_connection, pair: *mut mpd_pair);
    fn mpd_enqueue_pair(connection: *mut mpd_connection, pair: *mut mpd_pair);

    fn mpd_command_list_begin(connection: *mut mpd_connection, discrete_ok: bool) -> bool;
    fn mpd_command_list_end(connection: *mut mpd_connection) -> bool;
    */

    fn mpd_run_play(connection: *mut mpd_connection) -> bool;
    fn mpd_run_play_pos(connection: *mut mpd_connection, song_pos: u32) -> bool;
    fn mpd_run_play_id(connection: *mut mpd_connection, song_id: u32) -> bool;
    fn mpd_run_toggle_pause(connection: *mut mpd_connection) -> bool;
    fn mpd_run_pause(connection: *mut mpd_connection, mode: bool) -> bool;
    fn mpd_run_stop(connection: *mut mpd_connection) -> bool;
    fn mpd_run_next(connection: *mut mpd_connection) -> bool;
    fn mpd_run_previous(connection: *mut mpd_connection) -> bool;
    fn mpd_run_set_volume(connection: *mut mpd_connection, volume: c_uint) -> bool;
    fn mpd_run_change_volume(connection: *mut mpd_connection, volume: c_int) -> bool;

    fn mpd_run_current_song(connection: *mut mpd_connection) -> *mut mpd_song;

    fn mpd_run_rescan(connection: *mut mpd_connection, path: *const c_char) -> c_uint;
    fn mpd_run_update(connection: *mut mpd_connection, path: *const c_char) -> c_uint;

    fn mpd_connection_get_error(connection: *const mpd_connection) -> MpdErrorKind;
    fn mpd_connection_get_error_message(connection: *const mpd_connection) -> *const c_uchar;
    fn mpd_connection_get_server_error(connection: *const mpd_connection) -> MpdServerErrorKind;
    fn mpd_connection_get_server_error_location(connection: *const mpd_connection) -> c_uint;
    fn mpd_connection_get_system_error(connection: *const mpd_connection) -> c_int;
    fn mpd_connection_clear_error(connection: *mut mpd_connection) -> bool;

    fn mpd_run_repeat(connection: *mut mpd_connection, mode: bool) -> bool;
    fn mpd_run_random(connection: *mut mpd_connection, mode: bool) -> bool;
    fn mpd_run_single(connection: *mut mpd_connection, mode: bool) -> bool;
    fn mpd_run_consume(connection: *mut mpd_connection, mode: bool) -> bool;
    fn mpd_run_crossfade(connection: *mut mpd_connection, seconds: c_uint) -> bool;
    fn mpd_run_mixrampdb(connection: *mut mpd_connection, db: c_float) -> bool;
    fn mpd_run_mixrampdelay(connection: *mut mpd_connection, seconds: c_float) -> bool;

    fn mpd_run_enable_output(connection: *mut mpd_connection, output_id: c_uint) -> bool;
    fn mpd_run_disable_output(connection: *mut mpd_connection, output_id: c_uint) -> bool;
    fn mpd_run_toggle_output(connection: *mut mpd_connection, output_id: c_uint) -> bool;
}

pub struct MpdConnection {
    pub conn: *mut mpd_connection
}


impl MpdConnection {
    pub fn new(host: Option<&str>, port: Port) -> Option<MpdResult<MpdConnection>> {
        MpdConnection::new_with_timeout(host, port, Duration::zero())
    }

    pub fn new_with_timeout(host: Option<&str>, port: Port, timeout: Duration) -> Option<MpdResult<MpdConnection>> {
        unsafe {
            let host = host.map(|v| v.to_c_str());
            let conn = mpd_connection_new(match host {
                Some(v) => v.as_ptr(),
                None => ptr::null()
            }, port as c_uint, timeout.num_milliseconds() as c_uint);

            if conn.is_null() { None } else {
                let mut result = MpdConnection { conn: conn };
                Some(match FromConn::from_conn(&mut result) {
                    None => Ok(result),
                    Some(e) => Err(e)
                })
            }
        }
    }

    pub fn authorize(&mut self, password: &str) -> MpdResult<()> { if ! password.with_c_str(|s| unsafe { mpd_run_password(self.conn, s) }) { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

    pub fn set_timeout(&mut self, timeout: Duration) { unsafe { mpd_connection_set_timeout(self.conn, timeout.num_milliseconds() as c_uint) } }

    pub fn version(&self) -> (uint, uint, uint) {
        let version = unsafe { * mpd_connection_get_server_version(self.conn as *const _) };
        (version[0] as uint, version[1] as uint, version[2] as uint)
    }
    pub fn settings(&self) -> Option<MpdSettings> { FromConn::from_conn(self) }

    pub fn play(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_play(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn stop(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_stop(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn pause(&mut self, mode: bool) -> MpdResult<()> { if ! unsafe { mpd_run_pause(self.conn, mode) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn toggle_pause(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_toggle_pause(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

    pub fn set_volume(&mut self, vol: u32) -> MpdResult<()> { if ! unsafe { mpd_run_set_volume(self.conn, vol) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn change_volume(&mut self, vol: i32) -> MpdResult<()> { if ! unsafe { mpd_run_change_volume(self.conn, vol) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

    pub fn set_repeat(&mut self, value: bool) -> MpdResult<()> {
        if unsafe { mpd_run_repeat(self.conn, value) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }
    pub fn set_single(&mut self, value: bool) -> MpdResult<()> {
        if unsafe { mpd_run_single(self.conn, value) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }
    pub fn set_consume(&mut self, value: bool) -> MpdResult<()> {
        if unsafe { mpd_run_consume(self.conn, value) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }
    pub fn set_random(&mut self, value: bool) -> MpdResult<()> {
        if unsafe { mpd_run_random(self.conn, value) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }
    pub fn set_crossfade(&mut self, value: Duration) -> MpdResult<()> {
        if unsafe { mpd_run_crossfade(self.conn, value.num_seconds() as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }
    pub fn set_mixrampdb(&mut self, value: f32) -> MpdResult<()> {
        if unsafe { mpd_run_mixrampdb(self.conn, value as c_float) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }
    pub fn set_mixrampdelay(&mut self, value: Duration) -> MpdResult<()> {
        if unsafe { mpd_run_mixrampdelay(self.conn, (value.num_milliseconds() as f32 / 1000f32) as c_float) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }

    pub fn next(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_next(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn prev(&mut self) -> MpdResult<()> { if ! unsafe { mpd_run_previous(self.conn) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

    pub fn play_pos(&mut self, pos: uint) -> MpdResult<()> { if ! unsafe { mpd_run_play_pos(self.conn, pos as c_uint) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }
    pub fn play_id(&mut self, id: uint) -> MpdResult<()> { if ! unsafe { mpd_run_play_id(self.conn, id as c_uint) } { return Err(FromConn::from_conn(self).unwrap()) } Ok(()) }

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

    pub fn enable_output_id(&mut self, id: uint, enabled: bool) -> MpdResult<()> {
        if unsafe {
            if enabled {
                mpd_run_enable_output(self.conn, id as c_uint)
            } else {
                mpd_run_disable_output(self.conn, id as c_uint)
            }
        } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }

    pub fn toggle_output_id(&mut self, id: uint) -> MpdResult<()> {
        if unsafe { mpd_run_toggle_output(self.conn, id as c_uint) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(self).unwrap())
        }
    }

    pub fn enable_output(&mut self, output: &MpdOutput, enabled: bool) -> MpdResult<()> {
        self.enable_output_id(output.id(), enabled)
    }

    pub fn toggle_output(&mut self, output: &MpdOutput) -> MpdResult<()> {
        self.toggle_output_id(output.id())
    }

    pub fn update(&mut self, path: Option<&str>) -> MpdResult<uint> {
        let cpath = path.map(|p| p.to_c_str());
        match unsafe { mpd_run_update(self.conn, match cpath {
            Some(p) => p.as_ptr(),
            None => ptr::null()
        }) } {
            0 => match FromConn::from_conn(self) {
                None => Ok(0),
                Some(e) => Err(e)
            },
            uid @ _ => Ok(uid as uint)
        }
    }

    pub fn rescan(&mut self, path: Option<&str>) -> MpdResult<uint> {
        let cpath = path.map(|p| p.to_c_str());
        match unsafe { mpd_run_rescan(self.conn, match cpath {
            Some(p) => p.as_ptr(),
            None => ptr::null()
        }) } {
            0 => match FromConn::from_conn(self) {
                None => Ok(0),
                Some(e) => Err(e)
            },
            uid @ _ => Ok(uid as uint)
        }
    }

    pub fn queue(&self) -> MpdQueue {
        MpdQueue { conn: self }
    }

    pub fn wait(&self, mask: Option<MpdEvent>) -> MpdIdle {
        MpdIdle::from_conn(self, mask)
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

