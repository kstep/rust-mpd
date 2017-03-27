//! The module defines decoder plugin data structures

use convert::FromIter;
use error::Error;

/// Decoder plugin
#[derive(Clone, Debug, PartialEq, RustcEncodable)]
pub struct Plugin {
    /// name
    pub name: String,
    /// supported file suffixes (extensions)
    pub suffixes: Vec<String>,
    /// supported MIME-types
    pub mime_types: Vec<String>,
}

impl FromIter for Vec<Plugin> {
    fn from_iter<I: Iterator<Item = Result<(String, String), Error>>>(iter: I) -> Result<Self, Error> {
        let mut result = Vec::new();
        let mut plugin: Option<Plugin> = None;
        for reply in iter {
            let (a, b) = try!(reply);
            match &*a {
                "plugin" => {
                    plugin.map(|p| result.push(p));

                    plugin = Some(Plugin {
                                      name: b,
                                      suffixes: Vec::new(),
                                      mime_types: Vec::new(),
                                  });
                }
                "mime_type" => {
                    plugin.as_mut().map(|p| p.mime_types.push(b));
                }
                "suffix" => {
                    plugin.as_mut().map(|p| p.suffixes.push(b));
                }
                _ => unreachable!(),
            }
        }
        plugin.map(|p| result.push(p));
        Ok(result)
    }
}
