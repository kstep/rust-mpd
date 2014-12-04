
use libc;
use std::c_str::ToCStr;
use std::time::duration::Duration;
use std::ptr;

use connection::{mpd_connection, MpdConnection};

#[repr(C)] struct mpd_settings;

#[link(name = "mpdclient")]
extern {
    fn mpd_connection_get_settings(connection: *const mpd_connection) -> *const mpd_settings;

    fn mpd_settings_new(host: *const u8, port: libc::c_uint, timeout_ms: libc::c_uint, reserved: *const u8, password: *const u8) -> *mut mpd_settings;
    fn mpd_settings_free(settings: *mut mpd_settings);
    fn mpd_settings_get_host(settings: *const mpd_settings) -> *const u8;
    fn mpd_settings_get_port(settings: *const mpd_settings) -> libc::c_uint;
    fn mpd_settings_get_timeout_ms(settings: *const mpd_settings) -> libc::c_uint;
    fn mpd_settings_get_password(settings: *const mpd_settings) -> *const u8;
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
