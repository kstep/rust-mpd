//! The module defines playlist data structures

use time::{strptime, Tm};

use std::collections::BTreeMap;
use error::{Error, ProtoError};

/// Playlist
#[derive(Clone, Debug, PartialEq)]
pub struct Playlist {
    /// name
    pub name: String,
    /// last modified
    pub last_mod: Tm
}

impl Playlist {
    /// build playlist from map
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Playlist, Error> {
        Ok(Playlist {
            name: try!(map.get("playlist").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("playlist")))),
            last_mod: try!(map.get("Last-Modified").ok_or(Error::Proto(ProtoError::NoField("Last-Modified")))
                           .and_then(|v| strptime(&*v, "%Y-%m-%dT%H:%M:%S%Z").map_err(From::from))),
        })
    }
}
