
use time::{strptime, Tm};

use std::collections::BTreeMap;
use error::{Error, ProtoError};

#[derive(Clone, Debug, PartialEq)]
pub struct Playlist {
    pub name: String,
    pub last_mod: Tm
}

impl Playlist {
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Playlist, Error> {
        Ok(Playlist {
            name: try!(map.get("name").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("name")))),
            last_mod: try!(map.get("Last-Modified").ok_or(Error::Proto(ProtoError::NoField("Last-Modified")))
                           .and_then(|v| strptime(&*v, "%Y-%m-%dT%H:%M:%S%Z").map_err(From::from))),
        })
    }
}
