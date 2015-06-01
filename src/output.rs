//! The module describes output

use std::collections::BTreeMap;
use error::{Error, ProtoError};
use std::convert::From;

/// Sound output
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    /// id
    pub id: u32,
    /// name
    pub name: String,
    /// enabled state
    pub enabled: bool
}

impl Output {
    /// build output from map
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Output, Error> {
        Ok(Output {
            id: get_field!(map, "outputid"),
            name: try!(map.get("outputname").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("outputname")))),
            enabled: get_field!(map, bool "outputenabled")
        })
    }
}
