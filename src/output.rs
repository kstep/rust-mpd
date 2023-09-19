//! The module describes output

use crate::convert::FromMap;
use crate::error::{Error, ProtoError};
use std::collections::BTreeMap;
use std::convert::From;

/// Sound output
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    /// id
    pub id: u32,
    /// name of the output plugin
    pub plugin: String,
    /// name
    pub name: String,
    /// enabled state
    pub enabled: bool,
}

impl FromMap for Output {
    fn from_map(map: BTreeMap<String, String>) -> Result<Output, Error> {
        Ok(Output {
            id: get_field!(map, "outputid"),
            plugin: get_field!(map, "plugin"),
            name: map.get("outputname").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("outputname")))?,
            enabled: get_field!(map, bool "outputenabled"),
        })
    }
}
