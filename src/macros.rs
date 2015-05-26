#![macro_use]

macro_rules! get_field {
    ($map:expr, bool $name:expr) => {
        try!($map.get($name).ok_or(Error::Proto(ProtoError::NoField($name)))
             .map(|v| v == "1"))
    };
    ($map:expr, opt $name:expr) => {
        try!($map.get($name).map(|v| v.parse().map(Some)).unwrap_or(Ok(None)))
    };
    ($map:expr, $name:expr) => {
        try!($map.get($name).ok_or(Error::Proto(ProtoError::NoField($name)))
             .and_then(|v| v.parse().map_err(|e| Error::Parse(From::from(e)))))
    };
}
