
use libc;
use std::c_str::ToCStr;
use std::time::duration::Duration;
use std::fmt::{Show, Error, Formatter};
use std::ptr;

use common::FromConn;
use connection::mpd_connection;

#[repr(C)] pub struct mpd_settings;

pub enum MpdSettings {
    Owned(*mut mpd_settings),
    Borrowed(*const mpd_settings)
}

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

impl Show for MpdSettings {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"MpdSettings { host: "));
        try!(self.host().fmt(f));
        try!(f.write(b", port: "));
        try!(self.port().fmt(f));
        try!(f.write(b", timeout: "));
        try!(self.timeout().fmt(f));
        try!(f.write(b", password: "));
        try!(self.password().fmt(f));
        try!(f.write(b" }"));
        Ok(())
    }
}

impl FromConn for MpdSettings {
    fn from_conn(connection: *mut mpd_connection) -> Option<MpdSettings> {
        let settings = unsafe { mpd_connection_get_settings(connection as *const _) };
        if settings.is_null() { return None; }
        Some(MpdSettings::Borrowed(settings))
    }
}

impl Drop for MpdSettings {
    fn drop(&mut self) {
        if let MpdSettings::Owned(p) = *self {
            unsafe { mpd_settings_free(p); }
        }
    }
}

impl MpdSettings {
    #[inline] unsafe fn as_ref(&self) -> *const mpd_settings {
        match *self {
            MpdSettings::Owned(p) => p as *const _,
            MpdSettings::Borrowed(p) => p
        }
    }

    pub fn host(&self) -> Option<String> {
        let host = unsafe { mpd_settings_get_host(self.as_ref()) };
        if host == ptr::null() { return None; }
        Some(unsafe { String::from_raw_buf(host) })
    }

    pub fn port(&self) -> u32 {
        unsafe { mpd_settings_get_port(self.as_ref()) }
    }

    pub fn timeout(&self) -> Duration {
        Duration::milliseconds(unsafe { mpd_settings_get_timeout_ms(self.as_ref()) as i64 })
    }

    pub fn password(&self) -> Option<String> {
        let password = unsafe { mpd_settings_get_password(self.as_ref()) };
        if password == ptr::null() { return None; }
        Some(unsafe { String::from_raw_buf(password) })
    }

    pub fn new(host: Option<String>, port: u32, timeout: Duration, password: Option<String>) -> Option<MpdSettings> {
        let host = host.map(|v| v.to_c_str());
        let password = password.map(|v| v.to_c_str());

        Some(MpdSettings::Owned(unsafe {
            mpd_settings_new(
                match host {
                    Some(h) => h.as_ptr() as *const u8,
                    None => ptr::null()
                },
                port,
                timeout.num_milliseconds() as u32, ptr::null(),
                match password {
                    Some(p) => p.as_ptr() as *const u8,
                    None => ptr::null()
                })
        }))
    }
}
