
use std::io::{standard_error, IoErrorKind, Stream};
use std::iter::FromIterator;
use std::error::FromError;

use error::MpdResult;
use client::{MpdPair, MpdClient};
use utils::FieldCutIter;
use rustc_serialize::{Encoder, Encodable};

#[derive(RustcEncodable, Show)]
pub struct MpdOutput {
    pub id: usize,
    pub name: String,
    pub enabled: bool
}

impl MpdOutput {
    pub fn enable<S: Stream>(&mut self, cli: &mut MpdClient<S>) -> MpdResult<()> {
        cli.exec_arg("enableoutput", self.id).map(|_| self.enabled = true)
    }
    pub fn disable<S: Stream>(&mut self, cli: &mut MpdClient<S>) -> MpdResult<()> {
        cli.exec_arg("disableoutput", self.id).map(|_| self.enabled = false)
    }
    pub fn toggle<S: Stream>(&mut self, cli: &mut MpdClient<S>) -> MpdResult<()> {
        cli.exec_arg("toggleoutput", self.id).map(|_| self.enabled = !self.enabled)
    }
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<MpdOutput> {
    fn from_iter<T: Iterator<Item=MpdResult<MpdPair>>>(iterator: T) -> MpdResult<MpdOutput> {
        let mut output = MpdOutput {
            id: 0,
            name: "".to_string(),
            enabled: false,
        };

        let mut iter = iterator;

        for field in iter {
            let MpdPair(key, value) = try!(field);
            match key.as_slice() {
                "outputid" => output.id = value.parse().unwrap_or(0),
                "outputname" => output.name = value,
                "outputenabled" => output.enabled = value == "1",
                _ => return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))
            }
        }

        Ok(output)
    }
}

mpd_collectable!(MpdOutput, "outputid");
