
use libc;
use std::fmt::{Show, Error, Formatter};
use std::c_str::ToCStr;
use std::ptr;

use common::{MpdError, MpdResult, FromConnection};
use connection::{MpdConnection, mpd_connection};

#[repr(C)] struct mpd_output;

#[link(name = "mpdclient")]
extern "C" {
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

impl Show for Output {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
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

pub struct Output {
    output: *mut mpd_output
}

pub struct Outputs<'a> {
    conn: &'a mut MpdConnection
}

impl<'a> Outputs<'a> {
    pub fn from_connection<'a>(conn: &'a mut MpdConnection) -> MpdResult<Outputs<'a>> {
        if unsafe { mpd_send_outputs(conn.conn) } {
            Ok(Outputs { conn: conn })
        } else {
            Err(FromConnection::from_connection(conn.conn).unwrap())
        }
    }
}

impl<'a> Iterator<MpdResult<Output>> for Outputs<'a> {
    fn next(&mut self) -> Option<MpdResult<Output>> {
        match FromConnection::from_connection(self.conn.conn) {
            Some(s) => Some(Ok(s)),
            None => match FromConnection::from_connection(self.conn.conn) {
                Some(e) => Some(Err(e)),
                None => None
            }
        }
    }
}

impl FromConnection for Output {
    fn from_connection(connection: *mut mpd_connection) -> Option<Output> {
        let output = unsafe { mpd_recv_output(connection) };
        if output as *const _ == ptr::null::<mpd_output>() {
            None
        } else {
            Some(Output { output: output })
        }
    }
}

impl Output {
    pub fn id(&self) -> u32 { unsafe { mpd_output_get_id(self.output as *const _) } }
    pub fn name(&self) -> String { unsafe { String::from_raw_buf(mpd_output_get_name(self.output as *const _)) } }
    pub fn enabled(&self) -> bool { unsafe { mpd_output_get_enabled(self.output as *const _) } }

    pub fn toggle(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_toggle_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(FromConnection::from_connection(conn.conn).unwrap())
        }
    }

    pub fn disable(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_disable_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(FromConnection::from_connection(conn.conn).unwrap())
        }
    }

    pub fn enable(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_enable_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(FromConnection::from_connection(conn.conn).unwrap())
        }
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        unsafe { mpd_output_free(self.output) }
    }
}

