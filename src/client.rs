//! This module defines client data structure — the main entry point to MPD communication
//!
//! Almost every method of the `Client` structure corresponds to some command in [MPD protocol][proto].
//!
//! [proto]: http://www.musicpd.org/doc/protocol/

use std::io::{Read, Write, BufRead, Lines};
use std::convert::From;
use std::fmt::Arguments;
use std::net::{TcpStream, ToSocketAddrs};

use bufstream::BufStream;
use version::Version;
use error::{ProtoError, Error, Result};
use status::{Status, ReplayGain};
use stats::Stats;
use song::{Song, Id};
use output::Output;
use playlist::Playlist;
use plugin::Plugin;
use message::{Channel, Message};
use search::Query;
use mount::{Mount, Neighbor};

use convert::*;
use proto::*;

// Client {{{

/// Client connection
#[derive(Debug)]
pub struct Client<S=TcpStream> where S: Read + Write {
    socket: BufStream<S>,
    /// MPD version
    pub version: Version
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

impl<S: Read+Write> Client<S> {
    // Constructors {{{
    /// Create client from some arbitrary pre-connected socket
    pub fn new(socket: S) -> Result<Client<S>> {
        let mut socket = BufStream::new(socket);

        let mut banner = String::new();
        try!(socket.read_line(&mut banner));

        if !banner.starts_with("OK MPD ") {
            return Err(From::from(ProtoError::BadBanner));
        }

        let version = try!(banner[7..].trim().parse::<Version>());

        Ok(Client {
            socket: socket,
            version: version
        })
    }
    // }}}

    // Playback options & status {{{
    /// Get MPD status
    pub fn status(&mut self) -> Result<Status> {
        self.run_command("command_list_begin")
            .and_then(|_| self.run_command("status"))
            .and_then(|_| self.run_command("replay_gain_status"))
            .and_then(|_| self.run_command("command_list_end"))
            .and_then(|_| self.read_struct())
    }

    /// Get MPD playing statistics
    pub fn stats(&mut self) -> Result<Stats> {
        self.run_command("stats")
            .and_then(|_| self.read_struct())
    }

    /// Clear error state
    pub fn clearerror(&mut self) -> Result<()> {
        self.run_command("clearerror")
            .and_then(|_| self.expect_ok())
    }

    /// Set volume
    pub fn volume(&mut self, volume: i8) -> Result<()> {
        self.run_command_fmt(format_args!("setvol {}", volume))
            .and_then(|_| self.expect_ok())
    }

    /// Set repeat state
    pub fn repeat(&mut self, value: bool) -> Result<()> {
        self.run_command_fmt(format_args!("repeat {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    /// Set random state
    pub fn random(&mut self, value: bool) -> Result<()> {
        self.run_command_fmt(format_args!("random {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    /// Set single state
    pub fn single(&mut self, value: bool) -> Result<()> {
        self.run_command_fmt(format_args!("single {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    /// Set consume state
    pub fn consume(&mut self, value: bool) -> Result<()> {
        self.run_command_fmt(format_args!("consume {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    /// Set crossfade time in seconds
    pub fn crossfade<T: ToSeconds>(&mut self, value: T) -> Result<()> {
        self.run_command_fmt(format_args!("crossfade {}", value.to_seconds()))
            .and_then(|_| self.expect_ok())
    }

    /// Set mixramp level in dB
    pub fn mixrampdb(&mut self, value: f32) -> Result<()> {
        self.run_command_fmt(format_args!("mixrampdb {}", value))
            .and_then(|_| self.expect_ok())
    }

    /// Set mixramp delay in seconds
    pub fn mixrampdelay<T: ToSeconds>(&mut self, value: T) -> Result<()> {
        self.run_command_fmt(format_args!("mixrampdelay {}", value.to_seconds()))
            .and_then(|_| self.expect_ok())
    }

    /// Set replay gain mode
    pub fn replaygain(&mut self, gain: ReplayGain) -> Result<()> {
        self.run_command_fmt(format_args!("replay_gain_mode {}", gain))
            .and_then(|_| self.expect_ok())
    }
    // }}}

    // Playback control {{{
    /// Start playback
    pub fn play(&mut self) -> Result<()> {
        self.run_command("play").and_then(|_| self.expect_ok())
    }

    /// Start playback from given song in a queue
    pub fn switch<T: ToQueuePlace>(&mut self, place: T) -> Result<()> {
        self.run_command_fmt(format_args!("play{} {}", if T::is_id() { "id" } else { "" }, place.to_place()))
            .and_then(|_| self.expect_ok())
    }

    /// Switch to a next song in queue
    pub fn next(&mut self) -> Result<()> {
        self.run_command("next")
            .and_then(|_| self.expect_ok())
    }

    /// Switch to a previous song in queue
    pub fn prev(&mut self) -> Result<()> {
        self.run_command("previous")
            .and_then(|_| self.expect_ok())
    }

    /// Stop playback
    pub fn stop(&mut self) -> Result<()> {
        self.run_command("stop")
            .and_then(|_| self.expect_ok())
    }

    /// Set pause state
    pub fn pause(&mut self, value: bool) -> Result<()> {
        self.run_command_fmt(format_args!("pause {}", value as u8))
            .and_then(|_| self.expect_ok())
    }

    /// Seek to a given place (in seconds) in a given song
    pub fn seek<T: ToSeconds, P: ToQueuePlace>(&mut self, place: P, pos: T) -> Result<()> {
        self.run_command_fmt(format_args!("seek{} {} {}", if P::is_id() { "id" } else { "" }, place.to_place(), pos.to_seconds()))
            .and_then(|_| self.expect_ok())
    }

    /// Seek to a given place (in seconds) in the current song
    pub fn rewind<T: ToSeconds>(&mut self, pos: T) -> Result<()> {
        self.run_command_fmt(format_args!("seekcur {}", pos.to_seconds()))
            .and_then(|_| self.expect_ok())
    }
    // }}}

    // Queue control {{{
    /// List given song or range of songs in a play queue
    pub fn songs<T: ToQueueRangeOrPlace>(&mut self, pos: T) -> Result<Vec<Song>> {
        self.run_command_fmt(format_args!("playlist{} {}", if T::is_id() { "id" } else { "info" }, pos.to_range()))
            .and_then(|_| self.read_pairs().split("file").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// List all songs in a play queue
    pub fn queue(&mut self) -> Result<Vec<Song>> {
        self.run_command("playlistinfo")
            .and_then(|_| self.read_pairs().split("file").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// Get current playing song
    pub fn currentsong(&mut self) -> Result<Option<Song>> {
        self.run_command("currentsong")
            .and_then(|_| self.read_map())
            .and_then(|m| if m.is_empty() {
                Ok(None)
            } else {
                self.read_struct().map(Some)
            })
    }

    /// Clear current queue
    pub fn clear(&mut self) -> Result<()> {
        self.run_command("clear")
            .and_then(|_| self.expect_ok())
    }

    /// List all changes in a queue since given version
    pub fn changes(&mut self, version: u32) -> Result<Vec<Song>> {
        self.run_command_fmt(format_args!("plchanges {}", version))
            .and_then(|_| self.read_pairs().split("file").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// Append a song into a queue
    pub fn push<P: AsRef<str>>(&mut self, path: P) -> Result<Id> {
        self.run_command_fmt(format_args!("addid \"{}\"", path.as_ref()))
            .and_then(|_| self.read_field("Id"))
            .and_then(|v| self.expect_ok()
                      .and_then(|_| v.parse().map_err(From::from).map(Id)))
    }

    /// Insert a song into a given position in a queue
    pub fn insert<P: AsRef<str>>(&mut self, path: P, pos: usize) -> Result<usize> {
        self.run_command_fmt(format_args!("addid \"{}\" {}", path.as_ref(), pos))
            .and_then(|_| self.read_field("Id"))
            .and_then(|v| self.expect_ok()
                      .and_then(|_| v.parse().map_err(From::from)))
    }

    /// Delete a song (at some position) or several songs (in a range) from a queue
    pub fn delete<T: ToQueueRangeOrPlace>(&mut self, pos: T) -> Result<()> {
            self.run_command_fmt(format_args!("delete{} {}", if T::is_id() { "id" } else { "" }, pos.to_range()))
                .and_then(|_| self.expect_ok())
    }

    /// Move a song (at a some position) or several songs (in a range) to other position in queue
    pub fn shift<T: ToQueueRangeOrPlace>(&mut self, from: T, to: usize) -> Result<()> {
        self.run_command_fmt(format_args!("move{} {} {}", if T::is_id() { "id" } else { "" }, from.to_range(), to))
            .and_then(|_| self.expect_ok())
    }

    /// Swap to songs in a queue
    pub fn swap<T: ToQueuePlace>(&mut self, one: T, two: T) -> Result<()> {
        self.run_command_fmt(format_args!("swap{} {} {}", if T::is_id() { "id" } else { "" }, one.to_place(), two.to_place()))
            .and_then(|_| self.expect_ok())
    }

    /// Shuffle queue in a given range (use `..` to shuffle full queue)
    pub fn shuffle<T: ToQueueRange>(&mut self, range: T) -> Result<()> {
        self.run_command_fmt(format_args!("shuffle {}", range.to_range()))
            .and_then(|_| self.expect_ok())
    }

    /// Set song priority in a queue
    pub fn priority<T: ToQueueRangeOrPlace>(&mut self, pos: T, prio: u8) -> Result<()> {
        self.run_command_fmt(format_args!("prio{} {} {}", if T::is_id() { "id" } else { "" }, prio, pos.to_range()))
            .and_then(|_| self.expect_ok())
    }

    /// Set song range (in seconds) to play
    ///
    /// Doesn't work for currently playing song.
    pub fn range<T: ToSongId, R: ToSongRange>(&mut self, song: T, range: R) -> Result<()> {
        self.run_command_fmt(format_args!("rangeid {} {}", song.to_song_id(), range.to_range()))
            .and_then(|_| self.expect_ok())
    }

    /// Add tag to a song
    pub fn tag<T: ToSongId>(&mut self, song: T, tag: &str, value: &str) -> Result<()> {
        self.run_command_fmt(format_args!("addtagid {} {} \"{}\"", song.to_song_id(), tag, value))
            .and_then(|_| self.expect_ok())
    }

    /// Delete tag from a song
    pub fn untag<T: ToSongId>(&mut self, song: T, tag: &str) -> Result<()> {
        self.run_command_fmt(format_args!("cleartagid {} {}", song.to_song_id(), tag))
            .and_then(|_| self.expect_ok())
    }
    // }}}

    // Connection settings {{{
    /// Just pings MPD server, does nothing
    pub fn ping(&mut self) -> Result<()> {
        self.run_command("ping").and_then(|_| self.expect_ok())
    }

    /// Close MPD connection
    pub fn close(&mut self) -> Result<()> {
        self.run_command("close").and_then(|_| self.expect_ok())
    }

    /// Kill MPD server
    pub fn kill(&mut self) -> Result<()> {
        self.run_command("kill").and_then(|_| self.expect_ok())
    }

    /// Login to MPD server with given password
    pub fn login(&mut self, password: &str) -> Result<()> {
        self.run_command_fmt(format_args!("password \"{}\"", password)).and_then(|_| self.expect_ok())
    }
    // }}}

    // Playlist methods {{{
    /// List all playlists
    pub fn playlists(&mut self) -> Result<Vec<Playlist>> {
        self.run_command("listplaylists")
            .and_then(|_| self.read_pairs().split("playlist").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// List all songs in a playlist
    pub fn playlist<N: ToPlaylistName>(&mut self, name: N) -> Result<Vec<Song>> {
        self.run_command_fmt(format_args!("listplaylistinfo \"{}\"", name.to_name()))
            .and_then(|_| self.read_pairs().split("file").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// Load playlist into queue
    ///
    /// You can give either full range (`..`) to load all songs in a playlist,
    /// or some partial range to load only part of playlist.
    pub fn load<T: ToQueueRange, N: ToPlaylistName>(&mut self, name: N, range: T) -> Result<()> {
        self.run_command_fmt(format_args!("load \"{}\" {}", name.to_name(), range.to_range()))
            .and_then(|_| self.expect_ok())
    }

    /// Save current queue into playlist
    ///
    /// If playlist with given name doesn't exist, create new one.
    pub fn save<N: ToPlaylistName>(&mut self, name: N) -> Result<()> {
        self.run_command_fmt(format_args!("save \"{}\"", name.to_name()))
            .and_then(|_| self.expect_ok())
    }

    /// Rename playlist
    pub fn pl_rename<N: ToPlaylistName>(&mut self, name: N, newname: &str) -> Result<()> {
        self.run_command_fmt(format_args!("rename \"{}\" \"{}\"", name.to_name(), newname))
            .and_then(|_| self.expect_ok())
    }

    /// Clear playlist
    pub fn pl_clear<N: ToPlaylistName>(&mut self, name: N) -> Result<()> {
        self.run_command_fmt(format_args!("playlistclear \"{}\"", name.to_name()))
            .and_then(|_| self.expect_ok())
    }

    /// Delete playlist
    pub fn pl_remove<N: ToPlaylistName>(&mut self, name: N) -> Result<()> {
        self.run_command_fmt(format_args!("rm \"{}\"", name.to_name()))
            .and_then(|_| self.expect_ok())
    }

    /// Add new songs to a playlist
    pub fn pl_push<N: ToPlaylistName, P: AsRef<str>>(&mut self, name: N, path: P) -> Result<()> {
        self.run_command_fmt(format_args!("playlistadd \"{}\" \"{}\"", name.to_name(), path.as_ref()))
            .and_then(|_| self.expect_ok())
    }

    /// Delete a song at a given position in a playlist
    pub fn pl_delete<N: ToPlaylistName>(&mut self, name: N, pos: u32) -> Result<()> {
        self.run_command_fmt(format_args!("playlistdelete \"{}\" {}", name.to_name(), pos))
            .and_then(|_| self.expect_ok())
    }

    /// Move song in a playlist from one position into another
    pub fn pl_shift<N: ToPlaylistName>(&mut self, name: N, from: u32, to: u32) -> Result<()> {
        self.run_command_fmt(format_args!("playlistmove \"{}\" {} {}", name.to_name(), from, to))
            .and_then(|_| self.expect_ok())
    }
    // }}}

    // Database methods {{{
    /// Run database rescan, i.e. remove non-existing files from DB
    /// as well as add new files to DB
    pub fn rescan(&mut self) -> Result<u32> {
        self.run_command("rescan")
            .and_then(|_| self.read_field("updating_db"))
            .and_then(|v| self.expect_ok()
                      .and_then(|_| v.parse().map_err(From::from)))
    }

    /// Run database update, i.e. remove non-existing files from DB
    pub fn update(&mut self) -> Result<u32> {
        self.run_command("update")
            .and_then(|_| self.read_field("updating_db"))
            .and_then(|v| self.expect_ok()
                      .and_then(|_| v.parse().map_err(From::from)))
    }
    // }}}

    // Database search {{{
    // TODO: count tag needle [...] [group] [grouptag], find type what [...] [window start:end]
    // TODO: search type what [...] [window start:end], searchadd type what [...]
    // TODO: findadd type what [...], listallinfo [uri], listfiles [uri], lsinfo [uri]
    // TODO: list type [filtertype] [filterwhat] [...] [group] [grouptype] [...]
    // TODO: searchaddpl name type what [...], readcomments

    /// TODO: under construction
    pub fn search(&mut self, query: Query) -> Result<Vec<Song>> {
        self.run_command_fmt(format_args!("search {}", query))
            .and_then(|_| self.read_pairs().split("file").map(|v| v.and_then(FromMap::from_map)).collect())
    }
    // }}}

    // Output methods {{{
    /// List all outputs
    pub fn outputs(&mut self) -> Result<Vec<Output>> {
        self.run_command("outputs")
            .and_then(|_| self.read_pairs().split("outputid").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// Set given output enabled state
    pub fn output<T: ToOutputId>(&mut self, id: T, state: bool) -> Result<()> {
        if state { self.out_enable(id) } else { self.out_disable(id) }
    }

    /// Disable given output
    pub fn out_disable<T: ToOutputId>(&mut self, id: T) -> Result<()> {
        self.run_command_fmt(format_args!("disableoutput {}", id.to_output_id()))
            .and_then(|_| self.expect_ok())
    }

    /// Enable given output
    pub fn out_enable<T: ToOutputId>(&mut self, id: T) -> Result<()> {
        self.run_command_fmt(format_args!("enableoutput {}", id.to_output_id()))
            .and_then(|_| self.expect_ok())
    }

    /// Toggle given output
    pub fn out_toggle<T: ToOutputId>(&mut self, id: T) -> Result<()> {
        self.run_command_fmt(format_args!("toggleoutput {}", id.to_output_id()))
            .and_then(|_| self.expect_ok())
    }
    // }}}

    // Reflection methods {{{
    /// Get current music directory
    pub fn music_directory(&mut self) -> Result<String> {
        self.run_command("config")
            .and_then(|_| self.read_field("music_directory"))
            .and_then(|d| self.expect_ok().map(|_| d))
    }

    /// List all available commands
    pub fn commands(&mut self) -> Result<Vec<String>> {
        self.run_command("commands")
            .and_then(|_| self.read_pairs()
                      .filter(|r| r.as_ref()
                              .map(|&(ref a, _)| *a == "command").unwrap_or(true))
                      .map(|r| r.map(|(_, b)| b))
                      .collect())
    }

    /// List all forbidden commands
    pub fn notcommands(&mut self) -> Result<Vec<String>> {
        self.run_command("notcommands")
            .and_then(|_| self.read_pairs()
                      .filter(|r| r.as_ref()
                              .map(|&(ref a, _)| *a == "command").unwrap_or(true))
                      .map(|r| r.map(|(_, b)| b))
                      .collect())
    }

    /// List all available URL handlers
    pub fn urlhandlers(&mut self) -> Result<Vec<String>> {
        self.run_command("urlhandlers")
            .and_then(|_| self.read_pairs()
                      .filter(|r| r.as_ref()
                              .map(|&(ref a, _)| *a == "handler").unwrap_or(true))
                      .map(|r| r.map(|(_, b)| b))
                      .collect())
    }

    /// List all supported tag types
    pub fn tagtypes(&mut self) -> Result<Vec<String>> {
        self.run_command("tagtypes")
            .and_then(|_| self.read_pairs()
                      .filter(|r| r.as_ref()
                              .map(|&(ref a, _)| *a == "tagtype").unwrap_or(true))
                      .map(|r| r.map(|(_, b)| b))
                      .collect())
    }

    /// List all available decoder plugins
    pub fn decoders(&mut self) -> Result<Vec<Plugin>> {
        try!(self.run_command("decoders"));

        let mut result = Vec::new();
        let mut plugin: Option<Plugin> = None;
        for reply in self.read_pairs() {
            let (a, b) = try!(reply);
            match &*a {
                "plugin" => {
                    plugin.map(|p| result.push(p));

                    plugin = Some(Plugin {
                        name: b,
                        suffixes: Vec::new(),
                        mime_types: Vec::new()
                    });
                },
                "mime_type" => { plugin.as_mut().map(|p| p.mime_types.push(b)); }
                "suffix" => { plugin.as_mut().map(|p| p.suffixes.push(b)); }
                _ => unreachable!()
            }
        }
        plugin.map(|p| result.push(p));
        Ok(result)
    }
    // }}}

    // Messaging {{{
    /// List all channels available for current connection
    pub fn channels(&mut self) -> Result<Vec<Channel>> {
        self.run_command("channels")
            .and_then(|_| self.read_pairs()
                      .filter(|r| r.as_ref()
                              .map(|&(ref a, _)| *a == "channel").unwrap_or(true))
                      .map(|r| r.map(|(_, b)| unsafe { Channel::new_unchecked(b) }))
                      .collect())
    }

    /// Read queued messages from subscribed channels
    pub fn readmessages(&mut self) -> Result<Vec<Message>> {
        self.run_command("readmessages")
            .and_then(|_| self.read_pairs().split("channel").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// Send a message to a channel
    pub fn sendmessage(&mut self, channel: Channel, message: &str) -> Result<()> {
        self.run_command_fmt(format_args!("sendmessage {} \"{}\"", channel, message))
            .and_then(|_| self.expect_ok())
    }

    /// Subscribe to a channel
    pub fn subscribe(&mut self, channel: Channel) -> Result<()> {
        self.run_command_fmt(format_args!("subscribe {}", channel))
            .and_then(|_| self.expect_ok())
    }

    /// Unsubscribe to a channel
    pub fn unsubscribe(&mut self, channel: Channel) -> Result<()> {
        self.run_command_fmt(format_args!("unsubscribe {}", channel))
            .and_then(|_| self.expect_ok())
    }
    // }}}

    // Mount methods {{{
    /// List all (virtual) mounts
    ///
    /// These mounts exist inside MPD process only, thus they can work without root permissions.
    pub fn mounts(&mut self) -> Result<Vec<Mount>> {
        self.run_command("listmounts")
            .and_then(|_| self.read_pairs().split("mount").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// List all network neighbors, which can be potentially mounted
    pub fn neighbors(&mut self) -> Result<Vec<Neighbor>> {
        self.run_command("listneighbors")
            .and_then(|_| self.read_pairs().split("neighbor").map(|v| v.and_then(FromMap::from_map)).collect())
    }

    /// Mount given neighbor to a mount point
    ///
    /// The mount exists inside MPD process only, thus it can work without root permissions.
    pub fn mount(&mut self, path: &str, uri: &str) -> Result<()> {
        self.run_command_fmt(format_args!("mount \"{}\" \"{}\"", path, uri))
            .and_then(|_| self.expect_ok())
    }

    /// Unmount given active (virtual) mount
    ///
    /// The mount exists inside MPD process only, thus it can work without root permissions.
    pub fn unmount(&mut self, path: &str) -> Result<()> {
        self.run_command_fmt(format_args!("unmount \"{}\"", path))
            .and_then(|_| self.expect_ok())
    }
    // }}}

    // Sticker methods {{{
    /// Show sticker value for a given object, identified by type and uri
    pub fn sticker(&mut self, typ: &str, uri: &str, name: &str) -> Result<String> {
        self.run_command_fmt(format_args!("sticker set {} \"{}\" {}", typ, uri, name))
            .and_then(|_| self.read_field("sticker"))
            .and_then(|s| self.expect_ok().map(|_| s))
    }

    /// Set sticker value for a given object, identified by type and uri
    pub fn set_sticker(&mut self, typ: &str, uri: &str, name: &str, value: &str) -> Result<()> {
        self.run_command_fmt(format_args!("sticker set {} \"{}\" {} \"{}\"", typ, uri, name, value))
            .and_then(|_| self.expect_ok())
    }

    /// Delete sticker from a given object, identified by type and uri
    pub fn delete_sticker(&mut self, typ: &str, uri: &str, name: &str) -> Result<()> {
        self.run_command_fmt(format_args!("sticker delete {} \"{}\" {}", typ, uri, name))
            .and_then(|_| self.expect_ok())
    }

    /// Remove all stickers from a given object, identified by type and uri
    pub fn clear_stickers(&mut self, typ: &str, uri: &str) -> Result<()> {
        self.run_command_fmt(format_args!("sticker delete {} \"{}\"", typ, uri))
            .and_then(|_| self.expect_ok())
    }

    /// List all stickers from a given object, identified by type and uri
    pub fn stickers(&mut self, typ: &str, uri: &str) -> Result<Vec<String>> {
        self.run_command_fmt(format_args!("sticker list {} \"{}\"", typ, uri))
            .and_then(|_| self.read_pairs()
                      .filter(|r| r.as_ref()
                              .map(|&(ref a, _)| *a == "sticker").unwrap_or(true))
                      .map(|r| r.map(|(_, b)| b.splitn(2, "=").nth(1).map(|s| s.to_owned()).unwrap()))
                      .collect())
    }

    /// List all (file, sticker) pairs for sticker name and objects of given type
    /// from given directory (identified by uri)
    pub fn find_sticker(&mut self, typ: &str, uri: &str, name: &str) -> Result<Vec<(String, String)>> {
        self.run_command_fmt(format_args!("sticker find {} \"{}\" {}", typ, uri, name))
            .and_then(|_| self.read_pairs().split("file").map(|rmap| rmap.map(|mut map|
                        (map.remove("file").unwrap(),
                         map.remove("sticker").and_then(|s| s.splitn(2, "=").nth(1).map(|s| s.to_owned())).unwrap())))
                      .collect())
    }

    /// List all files of a given type under given directory (identified by uri)
    /// with a tag set to given value
    pub fn find_sticker_eq(&mut self, typ: &str, uri: &str, name: &str, value: &str) -> Result<Vec<String>> {
        self.run_command_fmt(format_args!("sticker find {} \"{}\" {} = \"{}\"", typ, uri, name, value))
            .and_then(|_| self.read_pairs()
                      .filter(|r| r.as_ref()
                              .map(|&(ref a, _)| *a == "file").unwrap_or(true))
                      .map(|r| r.map(|(_, b)| b))
                      .collect())
    }
    // }}}

}

// Helper methods {{{
impl<S: Read+Write> Proto for Client<S> {
    type Stream = S;

    fn read_line(&mut self) -> Result<String> {
        let mut buf = String::new();
        try!(self.socket.read_line(&mut buf));
        if buf.ends_with("\n") {
            buf.pop();
        }
        Ok(buf)
    }

    fn read_pairs(&mut self) -> Pairs<Lines<&mut BufStream<S>>> {
        Pairs((&mut self.socket).lines())
    }

    fn run_command(&mut self, command: &str) -> Result<()> {
        self.socket.write_all(command.as_bytes())
            .and_then(|_| self.socket.write(&[0x0a]))
            .and_then(|_| self.socket.flush())
            .map_err(From::from)
    }

    fn run_command_fmt(&mut self, command: Arguments) -> Result<()> {
        self.socket.write_fmt(command)
            .and_then(|_| self.socket.write(&[0x0a]))
            .and_then(|_| self.socket.flush())
            .map_err(From::from)
    }
}
// }}}

// }}}

