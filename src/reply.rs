//! The module describes all possible replies from MPD server.
//!
//! Also it contains most generic parser, which can handle
//! all possible server replies.

use crate::error::{ParseError, ServerError};
use std::str::FromStr;

/// All possible MPD server replies
#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    /// `OK` and `list_OK` replies
    Ok,
    /// `ACK` reply (server error)
    Ack(ServerError),
    /// a data pair reply (in `field: value` format)
    Pair(String, String),
}

impl FromStr for Reply {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Reply, ParseError> {
        if s == "OK" || s == "list_OK" {
            Ok(Reply::Ok)
        } else if let Ok(ack) = s.parse::<ServerError>() {
            Ok(Reply::Ack(ack))
        } else {
            let mut splits = s.splitn(2, ':');
            match (splits.next(), splits.next()) {
                (Some(a), Some(b)) => Ok(Reply::Pair(a.to_owned(), b.trim().to_owned())),
                _ => Err(ParseError::BadPair),
            }
        }
    }
}
