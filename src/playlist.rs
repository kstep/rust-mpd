//! The module defines playlist data structures

use crate::convert::FromMap;
use crate::error::{Error, ParseError, ProtoError};

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::time::Duration;

/// Playlist
#[derive(Clone, Debug, PartialEq)]
pub struct Playlist {
    /// name
    pub name: String,
    /// last modified
    pub last_mod: Duration,
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
                .ok_or(Error::Proto(ProtoError::NoField("Last-Modified")))
                .and_then(|v| {
                    let parsed: time::Date = time::parse(&*v, "%Y-%m-%dT%H:%M:%SZ").map_err(ParseError::BadTime)?;
                    Ok(std::time::Duration::try_from(parsed - time::date!(1970 - 01 - 01)).map_err(ParseError::BadTimeConversion)?)
                })?,
        })
    }
}
