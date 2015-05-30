use std::collections::BTreeMap;

use error::{Error, ProtoError};

pub struct Mount {
    pub name: String,
    pub storage: String
}

impl Mount {
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Mount, Error> {
        Ok(Mount {
            name: try!(map.get("mount").map(|s| s.to_owned()).ok_or(Error::Proto(ProtoError::NoField("mount")))),
            storage: try!(map.get("storage").map(|s| s.to_owned()).ok_or(Error::Proto(ProtoError::NoField("storage")))),
        })
    }
}

pub struct Neighbor {
    pub name: String,
    pub storage: String
}

impl Neighbor {
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Neighbor, Error> {
        Ok(Neighbor {
            name: try!(map.get("name").map(|s| s.to_owned()).ok_or(Error::Proto(ProtoError::NoField("name")))),
            storage: try!(map.get("neighbor").map(|s| s.to_owned()).ok_or(Error::Proto(ProtoError::NoField("neighbor")))),
        })
    }
}
