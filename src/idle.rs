use libc::{c_uint, c_char, c_uchar};
use std::fmt::{Debug, Error, Formatter};
use client::{mpd_connection, MpdClient, FromClient};
use error::MpdResult;
use std::str::FromStr;

bitflags! {
    #[derive(RustcEncodable)]
    #[repr(C)]
    flags MpdEvent: c_uint {
        const DATABASE = 0x1,
        const PLAYLIST = 0x2,
        const QUEUE = 0x4,
        const PLAYER = 0x8,
        const MIXER = 0x10,
        const OUTPUT = 0x20,
        const OPTIONS = 0x40,
        const UPDATE = 0x80,
        const STICKER = 0x100,
        const SUBSCRIPTION = 0x200,
        const MESSAGE = 0x400,
    }
}

impl Debug for MpdEvent {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"MpdEvent("));
        let name = unsafe { mpd_idle_name(*self) };
        try!(if name.is_null() {
            self.bits.fmt(f)
        } else {
            unsafe { String::from_raw_buf(name).fmt(f) }
        });
        try!(f.write(b")"));
        Ok(())
    }
}

impl FromStr for MpdEvent {
    fn from_str(s: &str) -> Option<MpdEvent> {
        let ev = s.with_c_str(|s| unsafe { mpd_idle_name_parse(s) });
        if ev.is_empty() {
            None
        } else {
            Some(ev)
        }
    }
}

#[link(name = "mpdclient")]
extern {
    fn mpd_idle_name(idle: MpdEvent) -> *const c_uchar;
    fn mpd_idle_name_parse(name: *const c_char) -> MpdEvent;
    fn mpd_run_idle(connection: *mut mpd_connection) -> MpdEvent;
    fn mpd_run_idle_mask(connection: *mut mpd_connection, mask: MpdEvent) -> MpdEvent;
    fn mpd_run_noidle(connection: *mut mpd_connection) -> bool;
}

pub struct MpdIdle<'a> {
    conn: &'a MpdClient,
    mask: Option<MpdEvent>
}

impl<'a> Iterator for MpdIdle<'a> {
    type Item = MpdResult<MpdEvent>;
    fn next(&mut self) -> Option<MpdResult<MpdEvent>> {
        let idle = unsafe {
            match self.mask {
                Some(m) => mpd_run_idle_mask(self.conn.conn, m),
                None => mpd_run_idle(self.conn.conn)
            }
        };

        if idle.is_empty() {
            FromClient::from_client(self.conn).map(|e| Err(e))
        } else {
            Some(Ok(idle))
        }
    }
}

impl<'a> MpdIdle<'a> {
    pub fn from_client<'a>(conn: &'a MpdClient, mask: Option<MpdEvent>) -> MpdIdle<'a> {
        MpdIdle { conn: conn, mask: mask }
    }

    pub fn stop(self) -> MpdResult<()> {
        if unsafe { mpd_run_noidle(self.conn.conn) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }
}

