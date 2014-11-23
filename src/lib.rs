#![feature(macro_rules, slicing_syntax, if_let)]

extern crate time;
extern crate serialize;

use std::io;
use std::io::{TcpStream, BufferedStream, IoResult, IoError, standard_error};
use std::io::net::ip::ToSocketAddr;
use std::time::Duration;
use std::collections::TreeMap;
use time::{Tm, Timespec};
use serialize::{Decoder, Decodable};
use serialize::json::Json;
use serialize::json;

#[cfg(test)]
use std::io::BufReader;

struct MpdConnection {
    stream: BufferedStream<TcpStream>
}

#[deriving(Show)]
struct DirectoryInfo {
    //directory: Path,
    directory: String,
    lastMod: Tm,
}

impl<D, E> Decodable<D, E> for DirectoryInfo where D: Decoder<E> {
    fn decode(d: &mut D) -> Result<DirectoryInfo, E> {
        d.read_struct("DirectoryInfo", 2, |d| Ok(DirectoryInfo {
            //directory: try!(d.read_struct_field("directory", 0, |d| d.read_str().map(|v| Path::new(v)))),
            directory: try!(d.read_struct_field("directory", 0, |d| d.read_str())),
            lastMod: try!(d.read_struct_field("Last-Modified", 1, |d| d.read_str().map(|v| time::strptime(v[], "%Y-%m-%dT%H:%M:%SZ").unwrap()))),
        }))
    }
}

#[deriving(Show)]
struct TrackInfo {
    //file: Path,
    file: String,
    lastMod: Tm,
    time: Duration,
    title: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    albumArtist: Option<String>,
    composer: Option<String>,
    performer: Option<String>,
    disc: Option<String>,
    track: (Option<uint>, Option<uint>),
    date: Option<Tm>,
    genre: Option<String>,
    id: Option<uint>,
    pos: Option<uint>,
}

impl<D, E> Decodable<D, E> for TrackInfo where D: Decoder<E> {
    fn decode(d: &mut D) -> Result<TrackInfo, E> {
        d.read_struct("TrackInfo", 15, |d| Ok(TrackInfo {
            //file: try!(d.read_struct_field("file", 0, |d| d.read_str().map(|v| Path::new(v)))),
            file: try!(d.read_struct_field("file", 0, |d| d.read_str())),
            lastMod: try!(d.read_struct_field("Last-Modified", 1, |d| d.read_str().map(|v| time::strptime(v[], "%Y-%m-%dT%H:%M:%SZ").unwrap()))),
            time: try!(d.read_struct_field("Time", 2, |d| d.read_i64().map(|v| Duration::seconds(v)))),

            title: try!(d.read_struct_field("Title", 3, |d| d.read_option(|d, s| if s { d.read_str().map(|v| Some(v)) } else { Ok(None) }))),
            artist: try!(d.read_struct_field("Artist", 4, |d| d.read_option(|d, s| if s { d.read_str().map(|v| Some(v)) } else { Ok(None) }))),
            album: try!(d.read_struct_field("Album", 5, |d| d.read_option(|d, s| if s { d.read_str().map(|v| Some(v)) } else { Ok(None) }))),
            albumArtist: try!(d.read_struct_field("AlbumArtist", 6, |d| d.read_option(|d, s| if s { d.read_str().map(|v| Some(v)) } else { Ok(None) }))),

            track: try!(d.read_struct_field("Track", 7, |d| d.read_option(|d, s| if s {
                d.read_str().map(|v| {
                    let mut splits = v.splitn(1, '/');
                    (splits.next().and_then(|v| from_str(v)), splits.next().and_then(|v| from_str(v)))
                })
            } else { Ok((None, None)) }))),
            date: try!(d.read_struct_field("Date", 8, |d| d.read_option(|d, s| if s {
                d.read_str().map(|v| time::strptime(v[], "%Y-%m-%dT%H:%M:%SZ")
                                 .or_else(|e| time::strptime(v[], "%Y-%m-%d"))
                                 .or_else(|e| time::strptime(v[], "%Y")).ok()) } else { Ok(None) }))),

            genre: try!(d.read_struct_field("Genre", 9, |d| d.read_option(|d, s| if s { d.read_str().map(|v| Some(v)) } else { Ok(None) }))),

            composer: try!(d.read_struct_field("Composer", 10, |d| d.read_option(|d, s| if s { d.read_str().map(|v| Some(v)) } else { Ok(None) }))),
            performer: try!(d.read_struct_field("Performer", 11, |d| d.read_option(|d, s| if s { d.read_str().map(|v| Some(v)) } else { Ok(None) }))),
            disc: try!(d.read_struct_field("Disc", 12, |d| d.read_option(|d, s| if s { d.read_str().map(|v| Some(v)) } else { Ok(None) }))),

            id: try!(d.read_struct_field("id", 13, |d| d.read_option(|d, s| if s { d.read_uint().map(|v| Some(v)) } else { Ok(None) }))),
            pos: try!(d.read_struct_field("pos", 14, |d| d.read_option(|d, s| if s { d.read_uint().map(|v| Some(v)) } else { Ok(None) })))
        }))
    }
}

#[deriving(Show)]
enum State {
    PLAY,
    PAUSE,
    STOP
}

impl<D, E> Decodable<D, E> for State where D: Decoder<E> {
    fn decode(d: &mut D) -> Result<State, E> {
        d.read_str().and_then(|v| match v[] {
            "play" => Ok(State::PLAY),
            "pause" => Ok(State::PAUSE),
            "stop" => Ok(State::STOP),
            s => Err(d.error(format!("unknown state: {}", s)[]))
        })
    }
}

#[deriving(Show)]
struct AudioFormat {
    rate: u16,
    bits: u8,
    chans: u8
}

impl<D, E> Decodable<D, E> for AudioFormat where D: Decoder<E> {
    fn decode(d: &mut D) -> Result<AudioFormat, E> {
        d.read_str().map(|v| {
            let mut splits = v.splitn(3, ':');
            AudioFormat {
                rate: splits.next().and_then(|v| from_str(v)).unwrap_or(0),
                bits: splits.next().and_then(|v| from_str(v)).unwrap_or(0),
                chans: splits.next().and_then(|v| from_str(v)).unwrap_or(0)
            }
        })
    }
}

#[deriving(Show)]
struct Status {
    volume: u8,
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,
    playlist: uint,
    playlistlength: uint,
    mixrampdb: f32,
    state: State,

    xfade: Option<Duration>,

    song: Option<uint>,
    songid: Option<uint>,
    nextsong: Option<uint>,
    nextsongid: Option<uint>,

    time: Option<(Duration, Duration)>,
    elapsed: Option<Duration>,

    bitrate: Option<uint>,
    audio: Option<AudioFormat>,

    updatingDb: Option<uint>,
    error: Option<String>,
}

impl<D, E> Decodable<D, E> for Status where D: Decoder<E> {
    fn decode(d: &mut D) -> Result<Status, E> {
        d.read_struct("Status", 20, |d| Ok(Status {
            volume: try!(d.read_struct_field("volume" , 0, |d| d.read_u8())),
            repeat: try!(d.read_struct_field("repeat", 1, |d| d.read_u8())) != 0,
            random: try!(d.read_struct_field("random", 2, |d| d.read_u8())) != 0,
            single: try!(d.read_struct_field("single", 3, |d| d.read_u8())) != 0,
            consume: try!(d.read_struct_field("consume", 4, |d| d.read_u8())) != 0,
            playlist: try!(d.read_struct_field("playlist", 5, |d| d.read_uint())),
            playlistlength: try!(d.read_struct_field("playlistlength", 6, |d| d.read_uint())),
            mixrampdb: try!(d.read_struct_field("mixrampdb", 7, |d| d.read_f32())),
            state: try!(d.read_struct_field("state", 8, |d| Decodable::decode(d))),

            xfade: try!(d.read_struct_field("xfade", 9, |d| d.read_option(|d, s| if s {
                d.read_i64().map(|v| Some(Duration::seconds(v))) } else { Ok(None) }))),

            song: try!(d.read_struct_field("song", 10, |d| d.read_option(|d, s| if s {
                d.read_uint().map(|v| Some(v)) } else { Ok(None) }))),
            songid: try!(d.read_struct_field("songid", 11, |d| d.read_option(|d, s| if s {
                d.read_uint().map(|v| Some(v)) } else { Ok(None) }))),
            nextsong: try!(d.read_struct_field("nextsong", 12, |d| d.read_option(|d, s| if s {
                d.read_uint().map(|v| Some(v)) } else { Ok(None) }))),
            nextsongid: try!(d.read_struct_field("nextsongid", 13, |d| d.read_option(|d, s| if s {
                d.read_uint().map(|v| Some(v)) } else { Ok(None) }))),

            time: try!(d.read_struct_field("time", 14, |d| d.read_option(|d, s| if s {
                d.read_str().map(|v| {
                    let mut s = v.splitn(2, ':').flat_map(|v| from_str(v).map(|v| Duration::seconds(v)).into_iter());
                    s.next().into_iter().zip(s.next().into_iter()).next()
                }) } else { Ok(None) }))),
            elapsed: try!(d.read_struct_field("elapsed", 15, |d| d.read_option(|d, s| if s {
                d.read_f32().map(|v| Some(Duration::milliseconds((v * 1000.0) as i64))) } else { Ok(None) }))),

            bitrate: try!(d.read_struct_field("bitrate", 16, |d| d.read_option(|d, s| if s {
                d.read_uint().map(|v| Some(v)) } else { Ok(None) }))),
            audio: try!(d.read_struct_field("audio", 17, |d| d.read_option(|d, s| if s {
                Decodable::decode(d).map(|v| Some(v)) } else { Ok(None) }))),

            updatingDb: try!(d.read_struct_field("updating_db", 18, |d| d.read_option(|d, s| if s {
                d.read_uint().map(|v| Some(v)) } else { Ok(None) }))),
            error: try!(d.read_struct_field("error", 19, |d| d.read_option(|d, s| if s {
                d.read_str().map(|v| Some(v)) } else { Ok(None) }))),
        }))
    }
}

#[deriving(Show)]
struct Stats {
    uptime: Duration,
    playtime: Duration,
    artists: uint,
    albums: uint,
    songs: uint,
    dbPlaytime: Duration,
    dbUpdate: Tm,
}
impl<D, E> Decodable<D, E> for Stats where D: Decoder<E> {
    fn decode(d: &mut D) -> Result<Stats, E> {
        d.read_struct("Stats", 7, |d| Ok(Stats {
            uptime: try!(d.read_struct_field("uptime", 0, |d| d.read_i64().map(|v| Duration::seconds(v)))),
            playtime: try!(d.read_struct_field("playtime", 1, |d| d.read_i64().map(|v| Duration::seconds(v)))),
            artists: try!(d.read_struct_field("artists", 2, |d| d.read_uint())),
            albums: try!(d.read_struct_field("albums", 3, |d| d.read_uint())),
            songs: try!(d.read_struct_field("songs", 4, |d| d.read_uint())),
            dbPlaytime: try!(d.read_struct_field("db_playtime", 5, |d| d.read_i64().map(|v| Duration::seconds(v)))),
            dbUpdate: try!(d.read_struct_field("db_update", 6, |d| d.read_i64().map(|v| time::at(Timespec::new(v, 0)))))
        }))
    }
}

enum Subsystem {
    Database,
    Update,
    StoredPlaylist,
    Playlist,
    Player,
    Mixer,
    Output,
    Options,
    Sticker,
    Subscription,
    Message,
}

fn parse_mpd<B: Buffer>(buf: &mut B) -> IoResult<Json> {
    let mut arr = Vec::new();
    let mut obj = TreeMap::new();

    for res in buf.lines() {
        match res {
            Ok(line) => {
                let line = line.trim_right_chars('\n');

                if line[] == "OK" {
                    break;
                }

                let mut pair = line.splitn(1, ':');
                if let (Some(k), Some(v)) = (pair.next(), pair.next().map(|v| v[1..])) {
                    let key = k.to_string();
                    if obj.contains_key(&key) {
                        arr.push(Json::Object(obj));
                        obj = TreeMap::new();
                    }

                    obj.insert(key, Json::String(v.to_string()));
                }
            },
            Err(e) => return Err(e)
        }
    }

    arr.push(Json::Object(obj));
    Ok(Json::Array(arr))
}

fn decode_mpd<T: Decodable<json::Decoder, json::DecoderError>, B: Buffer>(buf: &mut B) -> Result<T, json::DecoderError> {
    let parsed = match parse_mpd(buf) {
        Ok(v) => v,
        Err(e) => return Err(json::DecoderError::ParseError(json::ParserError::IoError(e.kind, e.desc)))
    };

    Decodable::decode(&mut json::Decoder::new(parsed))
}

impl MpdConnection {
    fn new<T: ToSocketAddr>(addr: T) -> IoResult<MpdConnection> {
       match TcpStream::connect(addr) {
           Ok(stream) => Ok(MpdConnection { stream: BufferedStream::new(stream) }),
           Err(e) => Err(e)
       }
    }

    fn issue_command(&mut self, cmd: &str) -> IoResult<()> {
        self.stream.write_str(cmd)
            .and_then(|()| self.stream.write(b"\n"))
            .and_then(|()| self.stream.flush())
    }

    fn playlist(&mut self) -> IoResult<Vec<Path>> {
        Err(standard_error(io::IoUnavailable))
    }

    fn playlistinfo(&mut self) -> IoResult<Vec<TrackInfo>> {
        Err(standard_error(io::IoUnavailable))
    }

    fn status(&mut self) -> IoResult<Vec<Status>> {
        try!(self.issue_command("status"));
        decode_mpd(&mut self.stream).map_err(|v| panic!("{}", v))
    }
    fn stats(&mut self) -> IoResult<Vec<Stats>> {
        try!(self.issue_command("stats"));
        decode_mpd(&mut self.stream).map_err(|v| panic!("{}", v))
    }

    fn search(&mut self, typ: &str, value: &str) -> IoResult<Vec<TrackInfo>> {
        try!(self.issue_command("search file \"\""));
        decode_mpd(&mut self.stream).map_err(|v| panic!("{}", v))
    }
}

#[test]
fn test_status_parser() {
    let mut status = BufReader::new(b"\
volume: 100
repeat: 1
random: 0
single: 0
consume: 0
playlist: 2
playlistlength: 0
mixrampdb: 0.000000
state: stop
OK"[]);

    let mpdStatus: Result<Vec<Status>, json::DecoderError> = decode_mpd(&mut status);
    panic!("{}", mpdStatus);
}

#[test]
fn test_live_status() {
    let mut conn = MpdConnection::new("192.168.1.10:6600").unwrap();
    panic!("{}", conn.status());
}

#[test]
fn test_live_stats() {
    let mut conn = MpdConnection::new("192.168.1.10:6600").unwrap();
    panic!("{}", conn.stats());
}

#[test]
fn test_live_search() {
    let mut conn = MpdConnection::new("192.168.1.10:6600").unwrap();
    panic!("{}", conn.search("file", ""));
}
