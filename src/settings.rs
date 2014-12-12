
use libc::{c_uint, c_char, c_uchar};
use std::c_str::ToCStr;
use std::time::duration::Duration;
use std::fmt::{Show, Error, Formatter};
use std::io::net::ip::Port;
use std::ptr;

use client::{MpdClient, FromClient, mpd_connection};

#[repr(C)] pub struct mpd_settings;

pub enum MpdSettings {
    Owned(*mut mpd_settings),
    Borrowed(*const mpd_settings)
}

#[link(name = "mpdclient")]
extern {
    fn mpd_connection_get_settings(connection: *const mpd_connection) -> *const mpd_settings;

    fn mpd_settings_new(host: *const c_char, port: c_uint, timeout_ms: c_uint, reserved: *const c_char, password: *const c_char) -> *mut mpd_settings;
    fn mpd_settings_free(settings: *mut mpd_settings);
    fn mpd_settings_get_host(settings: *const mpd_settings) -> *const c_uchar;
    fn mpd_settings_get_port(settings: *const mpd_settings) -> c_uint;
    fn mpd_settings_get_timeout_ms(settings: *const mpd_settings) -> c_uint;
    fn mpd_settings_get_password(settings: *const mpd_settings) -> *const c_uchar;
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

impl FromClient for MpdSettings {
    fn from_client(cli: &MpdClient) -> Option<MpdSettings> {
        let settings = unsafe { mpd_connection_get_settings(cli.conn as *const _) };
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
        if host.is_null() { return None; }
        Some(unsafe { String::from_raw_buf(host) })
    }

    pub fn port(&self) -> Port {
        unsafe { mpd_settings_get_port(self.as_ref()) as Port }
    }

    pub fn timeout(&self) -> Duration {
        Duration::milliseconds(unsafe { mpd_settings_get_timeout_ms(self.as_ref()) as i64 })
    }

    pub fn password(&self) -> Option<String> {
        let password = unsafe { mpd_settings_get_password(self.as_ref()) };
        if password.is_null() { return None; }
        Some(unsafe { String::from_raw_buf(password) })
    }

    pub fn new(host: Option<&str>, port: Port, timeout: Duration, password: Option<&str>) -> Option<MpdSettings> {
        let host = host.map(|v| v.to_c_str());
        let password = password.map(|v| v.to_c_str());

        Some(MpdSettings::Owned(unsafe {
            mpd_settings_new(
                match host {
                    Some(h) => h.as_ptr(),
                    None => ptr::null()
                },
                port as c_uint,
                timeout.num_milliseconds() as c_uint, ptr::null(),
                match password {
                    Some(p) => p.as_ptr(),
                    None => ptr::null()
                })
        }))
    }
}
