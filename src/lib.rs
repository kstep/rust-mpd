#![feature(macro_rules, slicing_syntax, unsafe_destructor)]

extern crate libc;
extern crate time;
extern crate serialize;

pub mod connection;
pub mod error;
pub mod queue;
pub mod settings;
pub mod status;
pub mod stats;
pub mod outputs;
pub mod tags;
pub mod songs;
pub mod playlists;
pub mod idle;


