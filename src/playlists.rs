use std::io::{standard_error, IoErrorKind, Stream};
use std::iter::FromIterator;
use std::error::FromError;
use time::{Timespec, strptime};

use error::MpdResult;
use client::{MpdPair, MpdClient};
use rustc_serialize::{Encoder, Encodable};
use songs::MpdSong;
use utils::{ForceEncodable, FieldCutIter};

#[derive(Show, RustcEncodable)]
pub struct MpdPlaylist {
    name: String,
    last_mod: Timespec
}

impl MpdPlaylist {
    pub fn songs<S: Stream>(&self, client: &mut MpdClient<S>) -> MpdResult<Vec<MpdSong>> {
        client.exec_str("listplaylistinfo", self.name[]).and_then(|_| client.iter().collect())
    }
}

impl FromIterator<MpdResult<MpdPair>> for MpdResult<MpdPlaylist> {
    fn from_iter<T: Iterator<MpdResult<MpdPair>>>(iterator: T) -> MpdResult<MpdPlaylist> {
        let mut playlist = MpdPlaylist {
            name: "".to_string(),
            last_mod: Timespec::new(0, 0)
        };

        let mut iter = iterator;

        for field in iter {
            let MpdPair(key, value) = try!(field);
            match key[] {
                "playlist" => playlist.name = value,
                "Last-Modified" => playlist.last_mod = try!(strptime(value[], "%Y-%m-%dT%H:%M:%S%Z").map_err(|_| standard_error(IoErrorKind::InvalidInput))).to_timespec(),
                _ => return Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))
            }
        }

        Ok(playlist)
    }
}

mpd_collectable!(MpdPlaylist, "playlist");
