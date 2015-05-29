#[derive(Clone, Debug, PartialEq)]
pub struct Plugin {
    pub name: String,
    pub suffixes: Vec<String>,
    pub mime_types: Vec<String>
}
