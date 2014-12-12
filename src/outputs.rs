
use libc::{c_uint, c_uchar};
use std::fmt::{Show, Error, Formatter};
use std::ptr;

use error::MpdResult;
use client::{MpdClient, mpd_connection, FromClient};
use rustc_serialize::{Encoder, Encodable};

#[repr(C)] struct mpd_output;

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_output_free(output: *mut mpd_output);
    fn mpd_output_get_name(output: *const mpd_output) -> *const c_uchar;
    fn mpd_output_get_id(output: *const mpd_output) -> c_uint;
    fn mpd_output_get_enabled(output: *const mpd_output) -> bool;
    fn mpd_send_outputs(connection: *mut mpd_connection) -> bool;
    fn mpd_recv_output(connection: *mut mpd_connection) -> *mut mpd_output;

    fn mpd_run_enable_output(connection: *mut mpd_connection, output_id: c_uint) -> bool;
    fn mpd_run_disable_output(connection: *mut mpd_connection, output_id: c_uint) -> bool;
    fn mpd_run_toggle_output(connection: *mut mpd_connection, output_id: c_uint) -> bool;
}

impl<'a> Show for MpdOutput<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        try!(f.write(b"MpdOutput { "));
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

impl<'a, S: Encoder<E>, E> Encodable<S, E> for MpdOutput<'a> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_struct("MpdOutput", 3, |s| {
            s.emit_struct_field("name", 0, |s| s.emit_str(self.name()[])).and_then(|()|
            s.emit_struct_field("id", 1, |s| s.emit_uint(self.id()))).and_then(|()|
            s.emit_struct_field("enabled", 2, |s| s.emit_bool(self.enabled())))
        })
    }
}

pub struct MpdOutput<'a> {
    output: *mut mpd_output,
    id: c_uint,
    conn: &'a MpdClient
}

pub struct MpdOutputs<'a> {
    conn: &'a MpdClient
}

impl<'a, S: Encoder<E>, E> Encodable<S, E> for MpdOutputs<'a> {
    fn encode(&self, s: &mut S) -> Result<(), E> {
        s.emit_seq(0, |s| self.enumerate().fold(Ok(()), |r, (i, v)| r.and_then(|()| s.emit_seq_elt(i, |s| v.encode(s)))))
    }
}

impl<'a> MpdOutputs<'a> {
    pub fn from_client<'a>(conn: &'a MpdClient) -> Option<MpdOutputs<'a>> {
        if unsafe { mpd_send_outputs(conn.conn) } {
            Some(MpdOutputs { conn: conn })
        } else {
            None
        }
    }
}

impl<'a> Iterator<MpdResult<MpdOutput<'a>>> for MpdOutputs<'a> {
    fn next(&mut self) -> Option<MpdResult<MpdOutput<'a>>> {
        match MpdOutput::from_client(self.conn) {
            Some(o) => Some(Ok(o)),
            None => match FromClient::from_client(self.conn) {
                None => None,
                Some(e) => Some(Err(e))
            }
        }
    }
}

impl<'a> MpdOutput<'a> {
    pub fn id(&self) -> uint { self.id as uint }
    pub fn name(&self) -> String { if self.output.is_null() { "".into_string() } else { unsafe { String::from_raw_buf(mpd_output_get_name(self.output as *const _)) } } }
    pub fn enabled(&self) -> bool { if self.output.is_null() { true } else { unsafe { mpd_output_get_enabled(self.output as *const _) } } }

    pub fn enable(&mut self, enabled: bool) -> MpdResult<()> {
        if unsafe {
            if enabled {
                mpd_run_enable_output(self.conn.conn, self.id)
            } else {
                mpd_run_disable_output(self.conn.conn, self.id)
            }
        } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    pub fn toggle(&mut self) -> MpdResult<()> {
        if unsafe { mpd_run_toggle_output(self.conn.conn, self.id) } {
            Ok(())
        } else {
            Err(FromClient::from_client(self.conn).unwrap())
        }
    }

    pub fn from_client<'a>(conn: &'a MpdClient) -> Option<MpdOutput<'a>> {
        let output = unsafe { mpd_recv_output(conn.conn) };
        if output.is_null() {
            None
        } else {
            Some(MpdOutput { output: output, conn: conn, id: unsafe { mpd_output_get_id(output as *const _) } })
        }
    }

    pub fn new<'a>(conn: &'a MpdClient, id: uint) -> MpdOutput<'a> {
        MpdOutput { output: ptr::null::<mpd_output>() as *mut _, conn: conn, id: id as c_uint }
    }
}

#[unsafe_destructor]
impl<'a> Drop for MpdOutput<'a> {
    fn drop(&mut self) {
        if !self.output.is_null() {
            unsafe { mpd_output_free(self.output) }
        }
    }
}

