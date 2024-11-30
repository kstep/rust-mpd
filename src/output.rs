//! The module describes output

use crate::convert::FromIter;
use crate::error::{Error, ProtoError};

/// Sound output
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    /// id
    pub id: u32,
    /// name of the output plugin
    pub plugin: String,
    /// name
    pub name: String,
    /// enabled state
    pub enabled: bool,
    /// Runtime-configurable, plugin-specific attributes, such as "dop" for ALSA
    pub attributes: Vec<(String, String)>
}

impl FromIter for Output {
    // Implement FromIter directly so that we can parse plugin-specific attributes
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Output, Error> {
        let mut attributes = Vec::new();
        let mut name: Option<String> = None;  // panic if unnamed
        let mut plugin: Option<String> = None;  // panic if not found
        let mut id: u32 = 0;
        let mut enabled: bool = false;

        for res in iter {
            let line = res?;
            match &*line.0 {
                "outputid" => { id = line.1.parse::<u32>()? },
                "outputname" => { name.replace(line.1); },
                "plugin" => { plugin.replace(line.1); },
                "outputenabled" => enabled = line.1 == "1",
                "attribute" =>  {
                    let terms: Vec<&str> = line.1.split("=").collect();
                    if terms.len() != 2 {
                        return Err(Error::Proto(ProtoError::NotPair));
                    }
                    attributes.push((terms[0].to_owned(), terms[1].to_owned()));
                },
                _ => {}
            }
        }

        if name.is_none() {
            return Err(Error::Proto(ProtoError::NoField("outputname")));
        }

        if plugin.is_none() {
            return Err(Error::Proto(ProtoError::NoField("plugin")));
        }

        Ok(Self {
            id,
            plugin: plugin.unwrap(),
            name: name.unwrap(),
            enabled,
            attributes
        })
    }
}
