//! The module defines decoder plugin data structures

/// Decoder plugin
#[derive(Clone, Debug, PartialEq, RustcEncodable)]
pub struct Plugin {
    /// name
    pub name: String,
    /// supported file suffixes (extensions)
    pub suffixes: Vec<String>,
    /// supported MIME-types
    pub mime_types: Vec<String>
}
