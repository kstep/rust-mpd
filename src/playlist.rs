//! The module defines playlist data structures

use crate::convert::FromMap;
use crate::error::{Error, ProtoError};

use std::collections::BTreeMap;

/// Playlist
#[derive(Clone, Debug, PartialEq)]
pub struct Playlist {
    /// name
    pub name: String,
    /// last modified
    pub last_mod: String,
}

impl FromMap for Playlist {
    fn from_map(map: BTreeMap<String, String>) -> Result<Playlist, Error> {
        Ok(Playlist {
            name: map
                .get("playlist")
                .map(|v| v.to_owned())
                .ok_or(Error::Proto(ProtoError::NoField("playlist")))?,
            last_mod: map
                .get("Last-Modified")
                .map(|v| v.to_owned())
                .ok_or(Error::Proto(ProtoError::NoField("Last-Modified")))?,
        })
    }
}
