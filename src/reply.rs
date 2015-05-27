use std::str::FromStr;

use error::{ParseError, ServerError};

#[derive(Debug, Clone, PartialEq)]
pub enum Reply {
    Ok,
    Ack(ServerError),
    Pair(String, String)
}

impl FromStr for Reply {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Reply, ParseError> {
        if s == "OK" || s == "list_OK" {
            Ok(Reply::Ok)
        } else {
            if let Ok(ack) = s.parse::<ServerError>() {
                Ok(Reply::Ack(ack))
            } else {
                let mut splits = s.splitn(2, ':');
                match (splits.next(), splits.next()) {
                    (Some(a), Some(b)) => Ok(Reply::Pair(a.to_owned(), b.trim().to_owned())),
                    _ => Err(ParseError::BadPair)
                }
            }
        }
    }
}
