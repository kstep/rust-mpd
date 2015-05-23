use std::str::FromStr;
use std::fmt;

use error::ParseError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplayGain {
    Off,
    Track,
    Album,
    Auto
}

impl FromStr for ReplayGain {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<ReplayGain, ParseError> {
        use self::ReplayGain::*;
        match s {
            "off" => Ok(Off),
            "track" => Ok(Track),
            "album" => Ok(Album),
            "auto" => Ok(Auto),
            _ => Err(ParseError::BadValue(s.to_owned()))
        }
    }
}

impl fmt::Display for ReplayGain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ReplayGain::*;
        f.write_str(match *self {
            Off => "off",
            Track => "track",
            Album => "album",
            Auto => "auto",
        })
    }
}
