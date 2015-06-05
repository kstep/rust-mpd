#![deny(missing_docs)]

//! MPD client for Rust
//!
//! This crate tries to provide idiomatic Rust API for [Music Player Daemon][mpd].
//! The main entry point to the API is [`Client`](client/struct.Client.html) struct,
//! and inherent methods of the struct follow [MPD protocol][proto] for most part,
//! making use of traits to overload different parameters for convenience.
//!
//! [mpd]: http://www.musicpd.org/
//! [proto]: http://www.musicpd.org/doc/protocol/
//!
//! # Usage
//!
//! ```text
//! [dependencies]
//! mpd = "*"
//! ```
//!
//! ```rust,no_run
//! extern crate mpd;
//!
//! use mpd::Client;
//! use std::net::TcpStream;
//!
//! # fn main() {
//! let mut conn = Client::connect("127.0.0.1:6600").unwrap();
//! conn.volume(100).unwrap();
//! conn.load("My Lounge Playlist", ..).unwrap();
//! conn.play().unwrap();
//! println!("Status: {:?}", conn.status());
//! # }
//! ```

extern crate rustc_serialize;
extern crate time;
extern crate bufstream;

mod macros;
mod convert;
pub mod error;
pub mod version;
pub mod reply;
pub mod status;
pub mod song;
pub mod output;
pub mod playlist;
pub mod plugin;
pub mod stats;
pub mod search;
pub mod message;
pub mod idle;
pub mod mount;

mod proto;
pub mod client;

pub use client::Client;
pub use status::{Status, ReplayGain};
pub use version::Version;
pub use song::Song;
pub use playlist::Playlist;
pub use output::Output;
pub use plugin::Plugin;
pub use stats::Stats;
pub use search::{Term, Query, Clause};
pub use message::{Message, Channel};
pub use idle::{Idle, Subsystem};
pub use mount::{Mount, Neighbor};
