use std::collections::BTreeMap;
use error::{Error, ProtoError};
use std::convert::From;

#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub id: u32,
    pub name: String,
    pub enabled: bool
}

impl Output {
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Output, Error> {
        Ok(Output {
            id: get_field!(map, "outputid"),
            name: try!(map.get("outputname").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("outputname")))),
            enabled: get_field!(map, bool "outputenabled")
        })
    }
}
