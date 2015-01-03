
use std::io::{standard_error, IoErrorKind};
use std::error::FromError;

use error::MpdResult;
use client::MpdPair;
use utils::FieldCutIter;
use rustc_serialize::{Encoder, Encodable};

#[deriving(RustcEncodable, Show)]
pub struct MpdOutput {
    pub id: uint,
    pub name: String,
    pub enabled: bool
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<MpdOutput> {
    fn from_iter<T: Iterator<MpdResult<MpdPair>>>(iterator: T) -> MpdResult<MpdOutput> {
        let mut output = MpdOutput {
            id: 0,
            name: "".to_string(),
            enabled: false,
        };

        let mut iter = iterator;

        for field in iter {
            let MpdPair(key, value) = try!(field);
            match key[] {
                "outputid" => output.id = value.parse().unwrap_or(0),
                "outputname" => output.name = value,
                "outputenabled" => output.enabled = value == "1",
                _ => return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))
            }
        }

        Ok(output)
    }
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<Vec<MpdOutput>> {
    fn from_iter<T: Iterator<MpdResult<MpdPair>>>(iterator: T) -> MpdResult<Vec<MpdOutput>> {
        let mut iter = iterator.fuse().peekable();
        let mut result = Vec::new();

        while !iter.is_empty() {
            let output = try!(FieldCutIter::new(&mut iter, "outputid").collect());
            result.push(output);
        }

        Ok(result)
    }
}
