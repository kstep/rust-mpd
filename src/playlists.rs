use std::old_io::{standard_error, IoErrorKind, Stream};
use std::iter::FromIterator;
use std::convert::From;
use time::{Timespec, strptime, get_time};

use error::MpdResult;
use client::{MpdPair, MpdClient};
use rustc_serialize::{Encoder, Encodable};
use songs::MpdSong;
use utils::{ForceEncodable, FieldCutIter};

#[derive(Debug, RustcEncodable)]
pub struct MpdPlaylist {
    name: String,
    last_mod: Timespec
}

impl MpdPlaylist {
    pub fn new(name: &str) -> MpdPlaylist {
        MpdPlaylist {
            name: name.to_string(),
            last_mod: get_time()
        }
    }

    pub fn songs<S: Stream>(&self, client: &mut MpdClient<S>) -> MpdResult<Vec<MpdSong>> {
        client.exec_str("listplaylistinfo", &*self.name).and_then(|_| client.iter().collect())
    }

    pub fn remove<S: Stream>(&self, client: &mut MpdClient<S>, index: usize) -> MpdResult<()> {
        client.exec_arg2("playlistdelete", &*self.name, index).and_then(|_| client.ok())
    }

    pub fn push<S: Stream>(&self, client: &mut MpdClient<S>, file: &str) -> MpdResult<()> {
        client.exec_arg2("playlistadd", &*self.name, file).and_then(|_| client.ok())
    }

    pub fn rename<S: Stream>(&mut self, client: &mut MpdClient<S>, newname: &str) -> MpdResult<()> {
        client.exec_arg2("rename", &*self.name, newname).and_then(|_| client.ok()).map(|_| self.name = newname.to_string())
    }

    pub fn shift<S: Stream>(&self, client: &mut MpdClient<S>, index: usize, target: usize) -> MpdResult<()> {
        client.exec_arg3("playlistmove", &*self.name, index, target).and_then(|_| client.ok())
    }

    pub fn load<S: Stream>(&self, client: &mut MpdClient<S>) -> MpdResult<()> {
        client.exec_arg("load", &*self.name).and_then(|_| client.ok())
    }

    pub fn load_slice<S: Stream>(&self, client: &mut MpdClient<S>, slice: (usize, usize)) -> MpdResult<()> {
        client.exec_arg2("load", &*self.name, format!("{}:{}", slice.0, slice.1)).and_then(|_| client.ok())
    }

    pub fn clear<S: Stream>(&self, client: &mut MpdClient<S>) -> MpdResult<()> {
        client.exec_arg("playlistclear", &*self.name).and_then(|_| client.ok())
    }

    pub fn save<S: Stream>(&self, client: &mut MpdClient<S>) -> MpdResult<()> {
        client.exec_arg("save", &*self.name).and_then(|_| client.ok())
    }

    pub fn delete<S: Stream>(self, client: &mut MpdClient<S>) -> MpdResult<()> {
        client.exec_arg("rm", &*self.name).and_then(|_| client.ok())
    }
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<MpdPlaylist> {
    fn from_iter<T: Iterator<Item=MpdResult<MpdPair>>>(iterator: T) -> MpdResult<MpdPlaylist> {
        let mut playlist = MpdPlaylist {
            name: String::new(),
            last_mod: Timespec::new(0, 0)
        };

        let mut iter = iterator;

        for field in iter {
            let MpdPair(key, value) = try!(field);
            match &*key {
                "playlist" => playlist.name = value,
                "Last-Modified" => playlist.last_mod = try!(strptime(&*value, "%Y-%m-%dT%H:%M:%S%Z").map_err(|_| standard_error(IoErrorKind::InvalidInput))).to_timespec(),
                _ => return Err(From::from(standard_error(IoErrorKind::InvalidInput)))
            }
        }

        Ok(playlist)
    }
}

mpd_collectable!(MpdPlaylist, "playlist");
