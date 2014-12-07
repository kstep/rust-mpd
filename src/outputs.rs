
use libc;
use std::fmt::{Show, Error, Formatter};

use error::MpdResult;
use connection::{MpdConnection, mpd_connection, FromConn};

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

impl Show for MpdOutput {
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

pub struct MpdOutput {
    output: *mut mpd_output
}

pub struct MpdOutputs<'a> {
    conn: &'a MpdConnection
}

impl<'a> MpdOutputs<'a> {
    pub fn from_conn<'a>(conn: &'a MpdConnection) -> Option<MpdOutputs<'a>> {
        if unsafe { mpd_send_outputs(conn.conn) } {
            Some(MpdOutputs { conn: conn })
        } else {
            None
        }
    }
}

impl<'a> Iterator<MpdResult<MpdOutput>> for MpdOutputs<'a> {
    fn next(&mut self) -> Option<MpdResult<MpdOutput>> {
        match FromConn::from_conn(self.conn) {
            Some(o) => Some(Ok(o)),
            None => match FromConn::from_conn(self.conn) {
                None => None,
                Some(e) => Some(Err(e))
            }
        }
    }
}

impl FromConn for MpdOutput {
    fn from_conn(conn: &MpdConnection) -> Option<MpdOutput> {
        let output = unsafe { mpd_recv_output(conn.conn) };
        if output.is_null() {
            None
        } else {
            Some(MpdOutput { output: output })
        }
    }
}

impl MpdOutput {
    pub fn id(&self) -> u32 { unsafe { mpd_output_get_id(self.output as *const _) } }
    pub fn name(&self) -> String { unsafe { String::from_raw_buf(mpd_output_get_name(self.output as *const _)) } }
    pub fn enabled(&self) -> bool { unsafe { mpd_output_get_enabled(self.output as *const _) } }

    pub fn toggle(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_toggle_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(conn).unwrap())
        }
    }

    pub fn disable(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_disable_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(conn).unwrap())
        }
    }

    pub fn enable(&self, conn: &mut MpdConnection) -> MpdResult<()> {
        if unsafe { mpd_run_enable_output(conn.conn, self.id()) } {
            Ok(())
        } else {
            Err(FromConn::from_conn(conn).unwrap())
        }
    }
}

impl Drop for MpdOutput {
    fn drop(&mut self) {
        unsafe { mpd_output_free(self.output) }
    }
}

