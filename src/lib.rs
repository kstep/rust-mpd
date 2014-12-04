#![feature(macro_rules, slicing_syntax, if_let)]

extern crate time;
extern crate libc;
extern crate collections;

use std::time::duration::Duration;
use std::ptr;
use std::c_str::ToCStr;
use collections::string::String;

struct mpd_connection;
struct mpd_settings;
struct mpd_status;

#[repr(C)]
#[deriving(Show)]
enum mpd_error {
    success = 0,
    oom = 1,
    argument = 2,
    state = 3,
    timeout = 4,
    system = 5,
    resolver = 6,
    malformed = 7,
    closed = 8,
    server = 9,
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
enum mpd_state {
    unknown = 0,
    stop = 1,
    play = 2,
    pause = 3,
}

#[repr(C)]
#[deriving(Show)]
enum mpd_server_error {
    unk = -1,
    not_list = 1,
    arg = 2,
    password = 3,
    permission = 4,
    unknown_cmd = 5,
    no_exist = 50,
    playlist_max = 51,
    system = 52,
    playlist_load = 53,
    update_already = 54,
    player_sync = 55,
    exist = 56,
}

#[deriving(Show)]
enum MpdError {
    Server { kind: mpd_server_error, index: uint, desc: String },
    System { code: int, desc: String },
    Other { kind: mpd_error, desc: String }
}

impl MpdError {
    unsafe fn from_connection(connection: *mut mpd_connection) -> Option<MpdError> {
        let error = mpd_connection_get_error(connection as *const _);

        let err = match error {
            mpd_error::success => return None,
            mpd_error::system => MpdError::System {
                code: mpd_connection_get_system_error(connection as *const _),
                desc: String::from_raw_buf(mpd_connection_get_error_message(connection as *const _)),
            },
            mpd_error::server => MpdError::Server {
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
        return Some(err);
    }
}

impl std::error::Error for MpdError {
    fn description(&self) -> &str {
        match *self {
            MpdError::System { .. } => "system error",
            MpdError::Server { ref kind, .. } => match *kind {
                mpd_server_error::unk => "unknown error",
                mpd_server_error::not_list => "not a list",
                mpd_server_error::arg => "invalid argument",
                mpd_server_error::password => "invalid password",
                mpd_server_error::permission => "access denied",
                mpd_server_error::unknown_cmd => "unknown command",
                mpd_server_error::no_exist => "object not found",
                mpd_server_error::playlist_max => "playlist overflow",
                mpd_server_error::system => "system error",
                mpd_server_error::playlist_load => "playlist load error",
                mpd_server_error::update_already => "database already updating",
                mpd_server_error::player_sync => "player sync error",
                mpd_server_error::exist => "object already exists",
            },
            MpdError::Other { ref kind, .. } => match *kind {
                mpd_error::success => "success",
                mpd_error::oom => "out of memory",
                mpd_error::argument => "invalid argument",
                mpd_error::state => "invalid state",
                mpd_error::timeout => "operation timed out",
                mpd_error::system => "system error",
                mpd_error::resolver => "name resolution error",
                mpd_error::malformed => "malformed hostname",
                mpd_error::closed => "connection closed",
                mpd_error::server => "server error",
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

type MpdResult<T> = Result<T, MpdError>;

#[link(name = "mpdclient")]
extern {
    fn mpd_connection_new(host: *const u8, port: uint, timeout_ms: uint) -> *mut mpd_connection;
    fn mpd_connection_free(connection: *mut mpd_connection);
    fn mpd_connection_get_settings(connection: *const mpd_connection) -> *const mpd_settings;
    fn mpd_connection_set_timeout(connection: *mut mpd_connection, timeout_ms: uint);
    fn mpd_connection_get_fd(connection: *const mpd_connection) -> int;
    fn mpd_connection_get_error(connection: *const mpd_connection) -> mpd_error;
    fn mpd_connection_get_error_message(connection: *const mpd_connection) -> *const u8;
    fn mpd_connection_get_server_error(connection: *const mpd_connection) -> mpd_server_error;
    fn mpd_connection_get_server_error_location(connection: *const mpd_connection) -> uint;
    fn mpd_connection_get_system_error(connection: *const mpd_connection) -> int;
    fn mpd_connection_clear_error(connection: *mut mpd_connection) -> bool;
    fn mpd_connection_get_server_version(connection: *const mpd_connection) -> [uint, ..3];
    fn mpd_connection_cmp_server_version(connection: *const mpd_connection, major: uint, minor: uint, patch: uint) -> int;

    fn mpd_settings_new(host: *const u8, port: uint, timeout_ms: uint, reserved: *const u8, password: *const u8) -> *mut mpd_settings;
    fn mpd_settings_free(settings: *mut mpd_settings);
    fn mpd_settings_get_host(settings: *const mpd_settings) -> *const u8;
    fn mpd_settings_get_port(settings: *const mpd_settings) -> uint;
    fn mpd_settings_get_timeout_ms(settings: *const mpd_settings) -> uint;
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
    fn mpd_status_get_volume(status: *const mpd_status) -> int;
    fn mpd_status_get_repeat(status: *const mpd_status) -> bool;
    fn mpd_status_get_random(status: *const mpd_status) -> bool;
    fn mpd_status_get_single(status: *const mpd_status) -> bool;
    fn mpd_status_get_consume(status: *const mpd_status) -> bool;
    fn mpd_status_get_queue_length(status: *const mpd_status) -> uint;
    fn mpd_status_get_queue_version(status: *const mpd_status) -> uint;
    fn mpd_status_get_state(status: *const mpd_status) -> mpd_state;
    fn mpd_status_get_crossfade(status: *const mpd_status) -> uint;
    fn mpd_status_get_mixrampdb(status: *const mpd_status) -> f32;
    fn mpd_status_get_mixrampdelay(status: *const mpd_status) -> f32;
    fn mpd_status_get_song_pos(status: *const mpd_status) -> int;
    fn mpd_status_get_song_id(status: *const mpd_status) -> int;
    fn mpd_status_get_next_song_pos(status: *const mpd_status) -> int;
    fn mpd_status_get_next_song_id(status: *const mpd_status) -> int;
    fn mpd_status_get_elapsed_time(status: *const mpd_status) -> uint;
    fn mpd_status_get_elapsed_ms(status: *const mpd_status) -> uint;
    fn mpd_status_get_total_time(status: *const mpd_status) -> uint;
    fn mpd_status_get_kbit_rate(status: *const mpd_status) -> uint;
    fn mpd_status_get_audio_format(status: *const mpd_status) -> *const mpd_audio_format;
    fn mpd_status_get_update_id(status: *const mpd_status) -> uint;
    fn mpd_status_get_error(status: *const mpd_status) -> *const u8;

    fn mpd_run_play(connection: *mut mpd_connection) -> bool;
    fn mpd_run_pause(connection: *mut mpd_connection, mode: bool) -> bool;
    fn mpd_run_stop(connection: *mut mpd_connection) -> bool;
    fn mpd_run_next(connection: *mut mpd_connection) -> bool;
    fn mpd_run_previous(connection: *mut mpd_connection) -> bool;
    fn mpd_run_set_volume(connection: *mut mpd_connection, volume: uint) -> bool;
    fn mpd_run_change_volume(connection: *mut mpd_connection, volume: int) -> bool;
}

struct MpdConnection {
    conn: *mut mpd_connection
}

// rate, bits, chans
type AudioFormat = (u32, u8, u8);

#[deriving(Show)]
struct MpdStatus {
    volume: int,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    queue_length: uint,
    queue_version: uint,
    state: mpd_state,
    crossfade: uint,
    mixrampdb: f32,
    mixrampdelay: Option<f32>,
    song: Option<(int, int)>, // id, pos
    next_song: Option<(int, int)>,
    elapsed_time: Duration,
    total_time: Duration,
    kbit_rate: uint,
    audio_format: Option<AudioFormat>,
    update_id: uint,
    error: Option<String>
}

enum MpdSettings {
    Owned(*mut mpd_settings),
    Borrowed(*const mpd_settings)
}

impl std::fmt::Show for MpdSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        try!(f.write(b"MpdSettings { host: "));
        try!(self.host().fmt(f));
        try!(f.write(b", port: "));
        try!(self.port().fmt(f));
        try!(f.write(b", timeout: "));
        try!(self.timeout().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

impl Drop for MpdSettings {
    fn drop(&mut self) {
        if let MpdSettings::Owned(settings) = *self {
            unsafe { mpd_settings_free(settings); }
        }
    }
}

impl MpdSettings {
    fn new(host: Option<String>, port: uint, timeout: Duration, password: Option<String>) -> Option<MpdSettings> {
        unsafe {
            let host = host.map(|v| v.to_c_str());
            let password = password.map(|v| v.to_c_str());

            let settings = mpd_settings_new(match host {
                Some(h) => h.as_ptr() as *const u8,
                None => ptr::null()
            }, port, timeout.num_milliseconds() as uint, ptr::null(),
            match password {
                Some(p) => p.as_ptr() as *const u8,
                None => ptr::null()
            });

            if settings as *const _ == ptr::null::<mpd_settings>() { None } else {
                Some(MpdSettings::Owned(settings))
            }
        }
    }

    fn host(&self) -> Option<String> {
        unsafe {
            let host = mpd_settings_get_host(match *self {
                MpdSettings::Owned(s) => s as *const _,
                MpdSettings::Borrowed(s) => s
            });
            if host == ptr::null() { None } else { Some(String::from_raw_buf(host)) }
        }
    }

    fn port(&self) -> uint {
        unsafe {
            mpd_settings_get_port(match *self {
                MpdSettings::Owned(s) => s as *const _,
                MpdSettings::Borrowed(s) => s
            })
        }
    }

    fn timeout(&self) -> Duration {
        Duration::milliseconds(unsafe {
            mpd_settings_get_timeout_ms(match *self {
                MpdSettings::Owned(s) => s as *const _,
                MpdSettings::Borrowed(s) => s
            })
        } as i64)
    }

    fn password(&self) -> Option<String> {
        unsafe {
            let host = mpd_settings_get_password(match *self {
                MpdSettings::Owned(s) => s as *const _,
                MpdSettings::Borrowed(s) => s
            });
            if host == ptr::null() { None } else { Some(String::from_raw_buf(host)) }
        }
    }
}

macro_rules! cmd_method {
    ($cmd:ident -> $name:ident($($arg:ident: $ty:ty),+)) => {
        fn $name(&mut self, $($arg: $ty),+) -> MpdResult<()> {
            if unsafe { $cmd(self.conn, $($arg),+) } {
                Ok(())
            } else {
                Err(unsafe { MpdError::from_connection(self.conn).unwrap() })
            }
        }
    };

    ($cmd:ident -> $name:ident()) => {
        fn $name(&mut self) -> MpdResult<()> {
            if unsafe { $cmd(self.conn) } {
                Ok(())
            } else {
                Err(unsafe { MpdError::from_connection(self.conn).unwrap() })
            }
        }
    };
}

impl MpdConnection {
    fn new(host: Option<&str>, port: uint) -> Option<MpdResult<MpdConnection>> {
        MpdConnection::new_with_timeout(host, port, Duration::zero())
    }

    fn new_with_timeout(host: Option<&str>, port: uint, timeout: Duration) -> Option<MpdResult<MpdConnection>> {
        unsafe {
            let host = host.map(|v| v.to_c_str());
            let conn = mpd_connection_new(match host {
                Some(v) => v.as_ptr() as *const u8,
                None => ptr::null()
            }, port, timeout.num_milliseconds() as uint);

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

    fn settings(&self) -> Option<MpdSettings> {
        unsafe {
            let settings = mpd_connection_get_settings(self.conn as *const _);
            if settings == ptr::null() { None } else { Some(MpdSettings::Borrowed(settings)) }
        }
    }

    cmd_method!(mpd_run_play -> play())
    cmd_method!(mpd_run_pause -> pause(mode: bool))
    cmd_method!(mpd_run_stop -> stop())
    cmd_method!(mpd_run_set_volume -> set_volume(vol: uint))
    cmd_method!(mpd_run_change_volume -> change_volume(vol: int))

    fn status(&self) -> MpdResult<MpdStatus> {
        unsafe {
            let status = mpd_run_status(self.conn);
            if status as *const _ == ptr::null::<mpd_status>() {
                return Err(MpdError::from_connection(self.conn).unwrap());
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

            Ok(result)
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
    //let conn = MpdConnection::new(Some("192.168.1.10"), 6600);
    let c = MpdConnection::new(None, 6600);
    let mut conn = match c {
        None => panic!("connection is None"),
        Some(Err(e)) => panic!("connection error: {}", e),
        Some(Ok(c)) => c
    };

    println!("{}", conn.stop());
    println!("{}", conn.set_volume(0));
    panic!("{}", conn.status());
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
