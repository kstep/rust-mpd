#![macro_use]

macro_rules! get_field_impl {
    ($op:ident, $map:expr, bool $name:expr) => {
        $map.$op($name).ok_or(Error::Proto(ProtoError::NoField($name))).map(|v| v == "1")?
    };
    ($op:ident, $map:expr, opt $name:expr) => {
        $map.$op($name).map(|v| v.parse().map(Some)).unwrap_or(Ok(None))?
    };
    ($op:ident, $map:expr, $name:expr) => {
        $map.$op($name)
            .ok_or(Error::Proto(ProtoError::NoField($name)))
            .and_then(|v| v.parse().map_err(|e| Error::Parse(From::from(e))))?
    };
}

macro_rules! get_field {
    ($map:expr, bool $name:expr) => { get_field_impl!(get, $map, bool $name) };
    ($map:expr, opt $name:expr) => { get_field_impl!(get, $map, opt $name) };
    ($map:expr, $name:expr) => { get_field_impl!(get, $map, $name) }
}
