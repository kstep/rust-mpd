//! The module defines structures for MPD client-to-client messaging/subscription protocol
//!
//! The MPD client-to-client messaging protocol is fairly easy one, and is based on channels.
//! Any client can subscribe to arbitrary number of channels, and some other client
//! can send messages to a channel by name. Then, at some point of time, subscribed
//! client can read all queued messages for all channels, it was subscribed to.
//!
//! Also client can get asynchronous notifications about new messages from subscribed
//! channels with `idle` command, by waiting for `message` subsystem events.

use crate::convert::FromMap;
use crate::error::{Error, ProtoError};

use std::collections::BTreeMap;
use std::fmt;

/// Message
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    /// channel
    pub channel: Channel,
    /// message payload
    pub message: String,
}

impl FromMap for Message {
    fn from_map(map: BTreeMap<String, String>) -> Result<Message, Error> {
        Ok(Message {
            channel: Channel(map.get("channel").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("channel")))?),
            message: map.get("message").map(|v| v.to_owned()).ok_or(Error::Proto(ProtoError::NoField("message")))?,
        })
    }
}

/// Channel
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub struct Channel(String);

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Channel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        Ok(Channel(String::deserialize(deserializer)?))
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.0)
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Channel {
    /// Create channel with given name
    pub fn new(name: &str) -> Option<Channel> {
        if Channel::is_valid_name(name) {
            Some(Channel(name.to_owned()))
        } else {
            None
        }
    }

    /// Create channel with arbitrary name, bypassing name validity checks
    ///
    /// Not recommened! Use [`new()`](Channel::new) method above instead.
    /// # Safety
    /// Only if [`Channel::is_valid_name(name)`](Channel::is_valid_name)
    pub unsafe fn new_unchecked(name: String) -> Channel {
        Channel(name)
    }

    /// Check if given name is a valid channel name
    ///
    /// Valid channel name can contain only English letters (`A`-`Z`, `a`-`z`),
    /// numbers (`0`-`9`), underscore, forward slash, dot and colon (`_`, `/`, `.`, `:`)
    pub fn is_valid_name(name: &str) -> bool {
        name.bytes().all(|b| {
            (0x61..=0x7a).contains(&b)
                || (0x41..=0x5a).contains(&b)
                || (0x30..=0x39).contains(&b)
                || (b == 0x5f || b == 0x2f || b == 0x2e || b == 0x3a)
        })
    }
}
