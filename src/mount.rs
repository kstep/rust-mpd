//! The module describes data structures for MPD (virtual) mounts system
//!
//! This mounts has nothing to do with system-wide Unix mounts, as they are
//! implemented inside MPD only, so they doesn't require root access.
//!
//! The MPD mounts are plugin-based, so MPD can mount any resource as
//! a source of songs for its database (like network shares).
//!
//! Possible, but inactive, mounts are named "neighbors" and can be
//! listed with `neighbors()` method.

use crate::convert::FromMap;
use crate::error::{Error, ProtoError};

use std::collections::BTreeMap;

/// Mount point
#[derive(Clone, Debug, PartialEq, RustcEncodable)]
pub struct Mount {
    /// mount point name
    pub name: String,
    /// mount storage URI
    pub storage: String,
}

impl FromMap for Mount {
    fn from_map(map: BTreeMap<String, String>) -> Result<Mount, Error> {
        Ok(Mount {
            name: map
                .get("mount")
                .map(|s| s.to_owned())
                .ok_or(Error::Proto(ProtoError::NoField("mount")))?,
            storage: map
                .get("storage")
                .map(|s| s.to_owned())
                .ok_or(Error::Proto(ProtoError::NoField("storage")))?,
        })
    }
}

/// Neighbor
#[derive(Clone, Debug, PartialEq, RustcEncodable)]
pub struct Neighbor {
    /// neighbor name
    pub name: String,
    /// neighbor storage URI
    pub storage: String,
}

impl FromMap for Neighbor {
    fn from_map(map: BTreeMap<String, String>) -> Result<Neighbor, Error> {
        Ok(Neighbor {
            name: map
                .get("name")
                .map(|s| s.to_owned())
                .ok_or(Error::Proto(ProtoError::NoField("name")))?,
            storage: map
                .get("neighbor")
                .map(|s| s.to_owned())
                .ok_or(Error::Proto(ProtoError::NoField("neighbor")))?,
        })
    }
}
