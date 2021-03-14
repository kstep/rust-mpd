//! This module defines MPD version type and parsing code

use crate::error::ParseError;
use std::str::FromStr;

// Version {{{
/// MPD version
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, RustcEncodable)]
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
// }}}
