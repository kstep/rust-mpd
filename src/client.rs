//! This module defines client data structure — the main entry point to MPD communication
//!
//! Almost every method of the `Client` structure corresponds to some command in [MPD protocol][proto].
//!
//! [proto]: http://www.musicpd.org/doc/protocol/

use bufstream::BufStream;

use crate::convert::*;
use crate::error::{Error, ParseError, ProtoError, Result};
use crate::message::{Channel, Message};
use crate::mount::{Mount, Neighbor};
use crate::output::Output;
use crate::playlist::Playlist;
use crate::plugin::Plugin;
use crate::proto::*;
use crate::search::{Query, Term, Window};
use crate::song::{Id, Song};
use crate::stats::Stats;
use crate::status::{ReplayGain, Status};
use crate::sticker::Sticker;
use crate::version::Version;

use std::collections::HashMap;
use std::convert::From;
use std::io::{BufRead, Lines, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};

// Client {{{

/// Client connection
#[derive(Debug)]
pub struct Client<S = TcpStream>
where S: Read + Write
{
    socket: BufStream<S>,
    /// MPD protocol version
    pub version: Version,
}

impl Default for Client<TcpStream> {
    fn default() -> Client<TcpStream> {
        Client::<TcpStream>::connect("127.0.0.1:6600").unwrap()
    }
}

impl Client<TcpStream> {
    /// Connect client to some IP address
    pub fn connect<A: ToSocketAddrs>(addr: A) -> Result<Client<TcpStream>> {
        TcpStream::connect(addr).map_err(Error::Io).and_then(Client::new)
    }
}

impl<S: Read + Write> Client<S> {
    // Constructors {{{
    /// Create client from some arbitrary pre-connected socket
    pub fn new(socket: S) -> Result<Client<S>> {
        let mut socket = BufStream::new(socket);

        let mut banner = String::new();
        socket.read_line(&mut banner)?;

        if !banner.starts_with("OK MPD ") {
            return Err(From::from(ProtoError::BadBanner));
        }

        let version = banner[7..].trim().parse::<Version>()?;

        Ok(Client { socket, version })
    }
    // }}}

    // Playback options & status {{{
    /// Get MPD status
    pub fn status(&mut self) -> Result<Status> {
        self.run_command("command_list_begin", ())
            .and_then(|_| self.run_command("status", ()))
            .and_then(|_| self.run_command("replay_gain_status", ()))
            .and_then(|_| self.run_command("command_list_end", ()))
            .and_then(|_| self.read_struct())
    }

    /// Get MPD playing statistics
    pub fn stats(&mut self) -> Result<Stats> {
        self.run_command("stats", ()).and_then(|_| self.read_struct())
    }

    /// Clear error state
    pub fn clearerror(&mut self) -> Result<()> {
        self.run_command("clearerror", ()).and_then(|_| self.expect_ok())
    }

    /// Set volume
    pub fn volume(&mut self, volume: i8) -> Result<()> {
        self.run_command("setvol", volume).and_then(|_| self.expect_ok())
    }

    /// Set repeat state
    pub fn repeat(&mut self, value: bool) -> Result<()> {
        self.run_command("repeat", value as u8).and_then(|_| self.expect_ok())
    }

    /// Set random state
    pub fn random(&mut self, value: bool) -> Result<()> {
        self.run_command("random", value as u8).and_then(|_| self.expect_ok())
    }

    /// Set single state
    pub fn single(&mut self, value: bool) -> Result<()> {
        self.run_command("single", value as u8).and_then(|_| self.expect_ok())
    }

    /// Set consume state
    pub fn consume(&mut self, value: bool) -> Result<()> {
        self.run_command("consume", value as u8).and_then(|_| self.expect_ok())
    }

    /// Set crossfade time in seconds
    pub fn crossfade<T: ToSeconds>(&mut self, value: T) -> Result<()> {
        self.run_command("crossfade", value.to_seconds()).and_then(|_| self.expect_ok())
    }

    /// Set mixramp level in dB
    pub fn mixrampdb(&mut self, value: f32) -> Result<()> {
        self.run_command("mixrampdb", value).and_then(|_| self.expect_ok())
    }

    /// Set mixramp delay in seconds
    pub fn mixrampdelay<T: ToSeconds>(&mut self, value: T) -> Result<()> {
        self.run_command("mixrampdelay", value.to_seconds()).and_then(|_| self.expect_ok())
    }

    /// Set replay gain mode
    pub fn replaygain(&mut self, gain: ReplayGain) -> Result<()> {
        self.run_command("replay_gain_mode", gain).and_then(|_| self.expect_ok())
    }
    // }}}

    // Playback control {{{
    /// Start playback
    pub fn play(&mut self) -> Result<()> {
        self.run_command("play", ()).and_then(|_| self.expect_ok())
    }

    /// Start playback from given song in a queue
    pub fn switch<T: ToQueuePlace>(&mut self, place: T) -> Result<()> {
        let command = if T::is_id() { "playid" } else { "play" };
        self.run_command(command, place.to_place()).and_then(|_| self.expect_ok())
    }

    /// Switch to a next song in queue
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::should_implement_trait))]
    pub fn next(&mut self) -> Result<()> {
        self.run_command("next", ()).and_then(|_| self.expect_ok())
    }

    /// Switch to a previous song in queue
    pub fn prev(&mut self) -> Result<()> {
        self.run_command("previous", ()).and_then(|_| self.expect_ok())
    }

    /// Stop playback
    pub fn stop(&mut self) -> Result<()> {
        self.run_command("stop", ()).and_then(|_| self.expect_ok())
    }

    /// Toggle pause state
    pub fn toggle_pause(&mut self) -> Result<()> {
        self.run_command("pause", ()).and_then(|_| self.expect_ok())
    }

    /// Set pause state
    pub fn pause(&mut self, value: bool) -> Result<()> {
        self.run_command("pause", value as u8).and_then(|_| self.expect_ok())
    }

    /// Seek to a given place (in seconds) in a given song
    pub fn seek<T: ToSeconds, P: ToQueuePlace>(&mut self, place: P, pos: T) -> Result<()> {
        let command = if P::is_id() { "seekid" } else { "seek" };
        self.run_command(command, (place.to_place(), pos.to_seconds())).and_then(|_| self.expect_ok())
    }

    /// Seek to a given place (in seconds) in the current song
    pub fn rewind<T: ToSeconds>(&mut self, pos: T) -> Result<()> {
        self.run_command("seekcur", pos.to_seconds()).and_then(|_| self.expect_ok())
    }
    // }}}

    // Queue control {{{
    /// List given song or range of songs in a play queue
    pub fn songs<T: ToQueueRangeOrPlace>(&mut self, pos: T) -> Result<Vec<Song>> {
        let command = if T::is_id() { "playlistid" } else { "playlistinfo" };
        self.run_command(command, pos.to_range()).and_then(|_| self.read_structs("file"))
    }

    /// List all songs in a play queue
    pub fn queue(&mut self) -> Result<Vec<Song>> {
        self.run_command("playlistinfo", ()).and_then(|_| self.read_structs("file"))
    }

    /// Lists all songs in the database
    pub fn listall(&mut self) -> Result<Vec<Song>> {
        self.run_command("listall", ()).and_then(|_| self.read_structs("file"))
    }

    /// Lists all songs in the database with metadata
    pub fn listallinfo(&mut self) -> Result<Vec<Song>> {
        self.run_command("listallinfo", ()).and_then(|_| self.read_structs("file"))
    }

    /// Get current playing song
    pub fn currentsong(&mut self) -> Result<Option<Song>> {
        self.run_command("currentsong", ())
            .and_then(|_| self.read_struct::<Song>())
            .map(|s| if s.place.is_none() { None } else { Some(s) })
    }

    /// gets the song wrt to songid in the playlist
    pub fn playlistid(&mut self, id: Id) -> Result<Option<Song>> {
        self.run_command("playlistid", id)
            .and_then(|_| self.read_struct::<Song>())
            .map(|s| if s.place.is_none() { None } else { Some(s) })
    }

    /// Clear current queue
    pub fn clear(&mut self) -> Result<()> {
        self.run_command("clear", ()).and_then(|_| self.expect_ok())
    }

    /// List all changes in a queue since given version
    pub fn changes(&mut self, version: u32) -> Result<Vec<Song>> {
        self.run_command("plchanges", version).and_then(|_| self.read_structs("file"))
    }

    /// Append a song into a queue
    pub fn push<P: ToSongPath>(&mut self, path: P) -> Result<Id> {
        self.run_command("addid", path).and_then(|_| self.read_field("Id")).map(Id)
    }

    /// Insert a song into a given position in a queue
    pub fn insert<P: ToSongPath>(&mut self, path: P, pos: usize) -> Result<usize> {
        self.run_command("addid", (path, pos)).and_then(|_| self.read_field("Id"))
    }

    /// Delete a song (at some position) or several songs (in a range) from a queue
    pub fn delete<T: ToQueueRangeOrPlace>(&mut self, pos: T) -> Result<()> {
        let command = if T::is_id() { "deleteid" } else { "delete" };
        self.run_command(command, pos.to_range()).and_then(|_| self.expect_ok())
    }

    /// Move a song (at a some position) or several songs (in a range) to other position in queue
    pub fn shift<T: ToQueueRangeOrPlace>(&mut self, from: T, to: usize) -> Result<()> {
        let command = if T::is_id() { "moveid" } else { "move" };
        self.run_command(command, (from.to_range(), to)).and_then(|_| self.expect_ok())
    }

    /// Swap to songs in a queue
    pub fn swap<T: ToQueuePlace>(&mut self, one: T, two: T) -> Result<()> {
        let command = if T::is_id() { "swapid" } else { "swap" };
        self.run_command(command, (one.to_place(), two.to_place())).and_then(|_| self.expect_ok())
    }

    /// Shuffle queue in a given range (use `..` to shuffle full queue)
    pub fn shuffle<T: ToQueueRange>(&mut self, range: T) -> Result<()> {
        self.run_command("shuffle", range.to_range()).and_then(|_| self.expect_ok())
    }

    /// Set song priority in a queue
    pub fn priority<T: ToQueueRangeOrPlace>(&mut self, pos: T, prio: u8) -> Result<()> {
        let command = if T::is_id() { "prioid" } else { "prio" };
        self.run_command(command, (prio, pos.to_range())).and_then(|_| self.expect_ok())
    }

    /// Set song range (in seconds) to play
    ///
    /// Doesn't work for currently playing song.
    pub fn range<T: ToSongId, R: ToSongRange>(&mut self, song: T, range: R) -> Result<()> {
        self.run_command("rangeid", (song.to_song_id(), range.to_range())).and_then(|_| self.expect_ok())
    }

    /// Add tag to a song
    pub fn tag<T: ToSongId>(&mut self, song: T, tag: &str, value: &str) -> Result<()> {
        self.run_command("addtagid", (song.to_song_id(), tag, value)).and_then(|_| self.expect_ok())
    }

    /// Delete tag from a song
    pub fn untag<T: ToSongId>(&mut self, song: T, tag: &str) -> Result<()> {
        self.run_command("cleartagid", (song.to_song_id(), tag)).and_then(|_| self.expect_ok())
    }
    // }}}

    // Connection settings {{{
    /// Just pings MPD server, does nothing
    pub fn ping(&mut self) -> Result<()> {
        self.run_command("ping", ()).and_then(|_| self.expect_ok())
    }

    /// Close MPD connection
    pub fn close(&mut self) -> Result<()> {
        self.run_command("close", ()).and_then(|_| self.expect_ok())
    }

    /// Kill MPD server
    pub fn kill(&mut self) -> Result<()> {
        self.run_command("kill", ()).and_then(|_| self.expect_ok())
    }

    /// Login to MPD server with given password
    pub fn login(&mut self, password: &str) -> Result<()> {
        self.run_command("password", password).and_then(|_| self.expect_ok())
    }
    // }}}

    // Playlist methods {{{
    /// List all playlists
    pub fn playlists(&mut self) -> Result<Vec<Playlist>> {
        self.run_command("listplaylists", ()).and_then(|_| self.read_structs("playlist"))
    }

    /// List all songs in a playlist
    pub fn playlist<N: ToPlaylistName>(&mut self, name: N) -> Result<Vec<Song>> {
        self.run_command("listplaylistinfo", name.to_name()).and_then(|_| self.read_structs("file"))
    }

    /// Load playlist into queue
    ///
    /// You can give either full range (`..`) to load all songs in a playlist,
    /// or some partial range to load only part of playlist.
    pub fn load<T: ToQueueRange, N: ToPlaylistName>(&mut self, name: N, range: T) -> Result<()> {
        self.run_command("load", (name.to_name(), range.to_range())).and_then(|_| self.expect_ok())
    }

    /// Save current queue into playlist
    ///
    /// If playlist with given name doesn't exist, create new one.
    pub fn save<N: ToPlaylistName>(&mut self, name: N) -> Result<()> {
        self.run_command("save", name.to_name()).and_then(|_| self.expect_ok())
    }

    /// Rename playlist
    pub fn pl_rename<N: ToPlaylistName>(&mut self, name: N, newname: &str) -> Result<()> {
        self.run_command("rename", (name.to_name(), newname)).and_then(|_| self.expect_ok())
    }

    /// Clear playlist
    pub fn pl_clear<N: ToPlaylistName>(&mut self, name: N) -> Result<()> {
        self.run_command("playlistclear", name.to_name()).and_then(|_| self.expect_ok())
    }

    /// Delete playlist
    pub fn pl_remove<N: ToPlaylistName>(&mut self, name: N) -> Result<()> {
        self.run_command("rm", name.to_name()).and_then(|_| self.expect_ok())
    }

    /// Add new songs to a playlist
    pub fn pl_push<N: ToPlaylistName, P: ToSongPath>(&mut self, name: N, path: P) -> Result<()> {
        self.run_command("playlistadd", (name.to_name(), path)).and_then(|_| self.expect_ok())
    }

    /// Delete a song at a given position in a playlist
    pub fn pl_delete<N: ToPlaylistName>(&mut self, name: N, pos: u32) -> Result<()> {
        self.run_command("playlistdelete", (name.to_name(), pos)).and_then(|_| self.expect_ok())
    }

    /// Move song in a playlist from one position into another
    pub fn pl_shift<N: ToPlaylistName>(&mut self, name: N, from: u32, to: u32) -> Result<()> {
        self.run_command("playlistmove", (name.to_name(), from, to)).and_then(|_| self.expect_ok())
    }
    // }}}

    // Database methods {{{
    /// Run database rescan, i.e. remove non-existing files from DB
    /// as well as add new files to DB
    pub fn rescan(&mut self) -> Result<u32> {
        self.run_command("rescan", ()).and_then(|_| self.read_field("updating_db"))
    }

    /// Run database update, i.e. remove non-existing files from DB
    pub fn update(&mut self) -> Result<u32> {
        self.run_command("update", ()).and_then(|_| self.read_field("updating_db"))
    }
    // }}}

    // Database search {{{
    // TODO: count tag needle [...] [group] [grouptag], find type what [...] [window start:end]
    // TODO: search type what [...] [window start:end], searchadd type what [...]
    // TODO: listfiles [uri]
    // TODO: list type [filtertype] [filterwhat] [...] [group] [grouptype] [...]
    // TODO: searchaddpl name type what [...]

    /// List all songs/directories in directory
    pub fn listfiles(&mut self, song_path: &str) -> Result<Vec<(String, String)>> {
        self.run_command("listfiles", song_path).and_then(|_| self.read_pairs().collect())
    }

    /// Find songs matching Query conditions.
    pub fn find<W>(&mut self, query: &Query, window: W) -> Result<Vec<Song>>
    where W: Into<Window> {
        self.find_generic("find", query, window.into())
    }

    /// Find album art for file
    pub fn albumart<P: ToSongPath>(&mut self, path: &P) -> Result<Vec<u8>> {
        let mut buf = vec![];
        loop {
            self.run_command("albumart", (path, &*format!("{}", buf.len())))?;
            let (_, size) = self.read_pair()?;
            let (_, bytes) = self.read_pair()?;
            let mut chunk = self.read_bytes(bytes.parse()?)?;
            buf.append(&mut chunk);
            // Read empty newline
            let _ = self.read_line()?;
            let result = self.read_line()?;
            if result != "OK" {
                return Err(ProtoError::NotOk)?;
            }

            if size.parse::<usize>()? == buf.len() {
                break;
            }
        }
        Ok(buf)
    }

    /// Case-insensitively search for songs matching Query conditions.
    pub fn search<W>(&mut self, query: &Query, window: W) -> Result<Vec<Song>>
    where W: Into<Window> {
        self.find_generic("search", query, window.into())
    }

    fn find_generic(&mut self, cmd: &str, query: &Query, window: Window) -> Result<Vec<Song>> {
        self.run_command(cmd, (query, window)).and_then(|_| self.read_structs("file"))
    }

    /// Lists unique tags values of the specified type for songs matching the given query.
    // TODO: list type [filtertype] [filterwhat] [...] [group] [grouptype] [...]
    // It isn't clear if or how `group` works
    pub fn list(&mut self, term: &Term, query: &Query) -> Result<Vec<String>> {
        self.run_command("list", (term, query)).and_then(|_| self.read_pairs().map(|p| p.map(|p| p.1)).collect())
    }

    /// Find all songs in the db that match query and adds them to current playlist.
    pub fn findadd(&mut self, query: &Query) -> Result<()> {
        self.run_command("findadd", query).and_then(|_| self.expect_ok())
    }

    /// Lists the contents of a directory.
    pub fn lsinfo<P: ToSongPath>(&mut self, path: P) -> Result<Vec<Song>> {
        self.run_command("lsinfo", path).and_then(|_| self.read_structs("file"))
    }

    /// Returns raw metadata for file
    pub fn readcomments<P: ToSongPath>(&mut self, path: P) -> Result<impl Iterator<Item = Result<(String, String)>> + '_> {
        self.run_command("readcomments", path)?;
        Ok(self.read_pairs())
    }

    // }}}

    // Output methods {{{
    /// List all outputs
    pub fn outputs(&mut self) -> Result<Vec<Output>> {
        self.run_command("outputs", ()).and_then(|_| self.read_structs("outputid"))
    }

    /// Set given output enabled state
    pub fn output<T: ToOutputId>(&mut self, id: T, state: bool) -> Result<()> {
        if state {
            self.out_enable(id)
        } else {
            self.out_disable(id)
        }
    }

    /// Disable given output
    pub fn out_disable<T: ToOutputId>(&mut self, id: T) -> Result<()> {
        self.run_command("disableoutput", id.to_output_id()).and_then(|_| self.expect_ok())
    }

    /// Enable given output
    pub fn out_enable<T: ToOutputId>(&mut self, id: T) -> Result<()> {
        self.run_command("enableoutput", id.to_output_id()).and_then(|_| self.expect_ok())
    }

    /// Toggle given output
    pub fn out_toggle<T: ToOutputId>(&mut self, id: T) -> Result<()> {
        self.run_command("toggleoutput", id.to_output_id()).and_then(|_| self.expect_ok())
    }
    // }}}

    // Reflection methods {{{
    /// Get current music directory
    pub fn music_directory(&mut self) -> Result<String> {
        self.run_command("config", ()).and_then(|_| self.read_field("music_directory"))
    }

    /// List all available commands
    pub fn commands(&mut self) -> Result<Vec<String>> {
        self.run_command("commands", ()).and_then(|_| self.read_list("command"))
    }

    /// List all forbidden commands
    pub fn notcommands(&mut self) -> Result<Vec<String>> {
        self.run_command("notcommands", ()).and_then(|_| self.read_list("command"))
    }

    /// List all available URL handlers
    pub fn urlhandlers(&mut self) -> Result<Vec<String>> {
        self.run_command("urlhandlers", ()).and_then(|_| self.read_list("handler"))
    }

    /// List all supported tag types
    pub fn tagtypes(&mut self) -> Result<Vec<String>> {
        self.run_command("tagtypes", ()).and_then(|_| self.read_list("tagtype"))
    }

    /// List all available decoder plugins
    pub fn decoders(&mut self) -> Result<Vec<Plugin>> {
        self.run_command("decoders", ()).and_then(|_| self.read_struct())
    }
    // }}}

    // Messaging {{{
    /// List all channels available for current connection
    pub fn channels(&mut self) -> Result<Vec<Channel>> {
        self.run_command("channels", ())
            .and_then(|_| self.read_list("channel"))
            .map(|v| v.into_iter().map(|b| unsafe { Channel::new_unchecked(b) }).collect())
    }

    /// Read queued messages from subscribed channels
    pub fn readmessages(&mut self) -> Result<Vec<Message>> {
        self.run_command("readmessages", ()).and_then(|_| self.read_structs("channel"))
    }

    /// Send a message to a channel
    pub fn sendmessage(&mut self, channel: Channel, message: &str) -> Result<()> {
        self.run_command("sendmessage", (channel, message)).and_then(|_| self.expect_ok())
    }

    /// Subscribe to a channel
    pub fn subscribe(&mut self, channel: Channel) -> Result<()> {
        self.run_command("subscribe", channel).and_then(|_| self.expect_ok())
    }

    /// Unsubscribe to a channel
    pub fn unsubscribe(&mut self, channel: Channel) -> Result<()> {
        self.run_command("unsubscribe", channel).and_then(|_| self.expect_ok())
    }
    // }}}

    // Mount methods {{{
    /// List all (virtual) mounts
    ///
    /// These mounts exist inside MPD process only, thus they can work without root permissions.
    pub fn mounts(&mut self) -> Result<Vec<Mount>> {
        self.run_command("listmounts", ()).and_then(|_| self.read_structs("mount"))
    }

    /// List all network neighbors, which can be potentially mounted
    pub fn neighbors(&mut self) -> Result<Vec<Neighbor>> {
        self.run_command("listneighbors", ()).and_then(|_| self.read_structs("neighbor"))
    }

    /// Mount given neighbor to a mount point
    ///
    /// The mount exists inside MPD process only, thus it can work without root permissions.
    pub fn mount(&mut self, path: &str, uri: &str) -> Result<()> {
        self.run_command("mount", (path, uri)).and_then(|_| self.expect_ok())
    }

    /// Unmount given active (virtual) mount
    ///
    /// The mount exists inside MPD process only, thus it can work without root permissions.
    pub fn unmount(&mut self, path: &str) -> Result<()> {
        self.run_command("unmount", path).and_then(|_| self.expect_ok())
    }
    // }}}

    // Sticker methods {{{
    /// Show sticker value for a given object, identified by type and uri
    pub fn sticker(&mut self, typ: &str, uri: &str, name: &str) -> Result<String> {
        self.run_command("sticker get", (typ, uri, name))
            // TODO: This should parse to a `Sticker` type.
            .and_then(|_| self.read_field::<Sticker>("sticker"))
            .map(|s| s.value)
    }

    /// Set sticker value for a given object, identified by type and uri
    pub fn set_sticker(&mut self, typ: &str, uri: &str, name: &str, value: &str) -> Result<()> {
        self.run_command("sticker set", (typ, uri, name, value)).and_then(|_| self.expect_ok())
    }

    /// Delete sticker from a given object, identified by type and uri
    pub fn delete_sticker(&mut self, typ: &str, uri: &str, name: &str) -> Result<()> {
        self.run_command("sticker delete", (typ, uri, name)).and_then(|_| self.expect_ok())
    }

    /// Remove all stickers from a given object, identified by type and uri
    pub fn clear_stickers(&mut self, typ: &str, uri: &str) -> Result<()> {
        self.run_command("sticker delete", (typ, uri)).and_then(|_| self.expect_ok())
    }

    /// List all stickers from a given object, identified by type and uri
    pub fn stickers(&mut self, typ: &str, uri: &str) -> Result<Vec<String>> {
        self.run_command("sticker list", (typ, uri))
            .and_then(|_| self.read_list("sticker"))
            .map(|v| v.into_iter().map(|b| b.split_once('=').map(|x| x.1.to_owned()).unwrap()).collect())
    }

    /// List all stickers from a given object in a map, identified by type and uri
    pub fn stickers_map(&mut self, typ: &str, uri: &str) -> Result<HashMap<String, String>> {
        self.run_command("sticker list", (typ, uri)).and_then(|_| self.read_list("sticker")).map(|v| {
            v.into_iter()
                .map(|b| {
                    let mut iter = b.splitn(2, '=');

                    (iter.next().unwrap().to_owned(), iter.next().unwrap().to_owned())
                })
                .collect()
        })
    }

    /// List all (file, sticker) pairs for sticker name and objects of given type
    /// from given directory (identified by uri)
    pub fn find_sticker(&mut self, typ: &str, uri: &str, name: &str) -> Result<Vec<(String, String)>> {
        self.run_command("sticker find", (typ, uri, name)).and_then(|_| {
            self.read_pairs()
                .split("file")
                .map(|rmap| {
                    rmap.map(|map| {
                        (
                            map.iter().find_map(|(k, v)| if k == "file" { Some(v.to_owned()) } else { None }).unwrap(),
                            map.iter()
                                .find_map(|(k, v)| if k == "sticker" { Some(v.to_owned()) } else { None })
                                .and_then(|s| s.split_once('=').map(|x| x.1.to_owned()))
                                .unwrap(),
                        )
                    })
                })
                .collect()
        })
    }

    /// List all files of a given type under given directory (identified by uri)
    /// with a tag set to given value
    pub fn find_sticker_eq(&mut self, typ: &str, uri: &str, name: &str, value: &str) -> Result<Vec<String>> {
        self.run_command("sticker find", (typ, uri, name, "=", value)).and_then(|_| self.read_list("file"))
    }
    // }}}
}

// Helper methods {{{
impl<S: Read + Write> Proto for Client<S> {
    type Stream = S;

    fn read_bytes(&mut self, bytes: usize) -> Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(bytes);
        let mut chunk = (&mut self.socket).take(bytes as u64);
        chunk.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn read_line(&mut self) -> Result<String> {
        let mut buf = Vec::new();
        self.socket.read_until(b'\n', &mut buf)?;
        if buf.ends_with(&[b'\n']) {
            buf.pop();
        }
        let str = String::from_utf8(buf)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "stream did not contain valid UTF-8"))?;
        Ok(str)
    }

    fn read_pairs(&mut self) -> Pairs<Lines<&mut BufStream<S>>> {
        Pairs((&mut self.socket).lines())
    }

    fn read_pair(&mut self) -> Result<(String, String)> {
        let line = self.read_line()?;
        let mut split = line.split(": ");
        let key = split.next().ok_or(ParseError::BadPair)?;
        let val = split.next().ok_or(ParseError::BadPair)?;
        Ok((key.to_string(), val.to_string()))
    }

    fn run_command<I>(&mut self, command: &str, arguments: I) -> Result<()>
    where I: ToArguments {
        self.socket
            .write_all(command.as_bytes())
            .and_then(|_| arguments.to_arguments(&mut |arg| write!(self.socket, " {}", Quoted(arg))))
            .and_then(|_| self.socket.write(&[0x0a]))
            .and_then(|_| self.socket.flush())
            .map_err(From::from)
    }
}
// }}}

// }}}
