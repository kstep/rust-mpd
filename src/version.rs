//! This module defines MPD version type and parsing code

use crate::error::ParseError;
use std::str::FromStr;

// Version {{{
/// MPD version
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Version(pub u16, pub u16, pub u16);

impl FromStr for Version {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Version, ParseError> {
        let mut splits = s.splitn(3, '.').map(FromStr::from_str);
        match (splits.next(), splits.next(), splits.next()) {
            (Some(Ok(a)), Some(Ok(b)), Some(Ok(c))) => Ok(Version(a, b, c)),
            (Some(Err(e)), _, _) | (_, Some(Err(e)), _) | (_, _, Some(Err(e))) => Err(ParseError::BadInteger(e)),
            _ => Err(ParseError::BadVersion),
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&format!("{}.{}.{}", self.0, self.1, self.2))
    }
}
// }}}
