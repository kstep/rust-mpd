use std::fmt;

use std::collections::BTreeMap;

use error::{Error, ProtoError};

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub channel: Channel,
    pub message: String
}

impl Message {
    pub fn from_map(map: BTreeMap<String, String>) -> Result<Message, Error> {
        Ok(Message {
            channel: Channel(try!(map.get("channel").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("channel"))))),
            message: try!(map.get("message").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("message")))),
        })
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Channel(String);

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Channel {
    pub fn new(name: &str) -> Option<Channel> {
        if Channel::is_valid_name(name) {
            Some(Channel(name.to_owned()))
        } else {
            None
        }
    }

    pub unsafe fn new_unchecked(name: String) -> Channel {
        Channel(name)
    }

    pub fn is_valid_name(name: &str) -> bool {
        name.bytes().all(
            |b| (0x61 <= b && b <= 0x7a) || (0x41 <= b && b <= 0x5a) || (0x30 <= b && b <= 0x39) ||
            (b == 0x5f || b == 0x2f || b == 0x2e || b == 0x3a))
    }
}
