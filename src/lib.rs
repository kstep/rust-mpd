#![feature(slicing_syntax, unsafe_destructor)]
#![feature(core, io, collections, std_misc, libc)]

extern crate libc;
extern crate time;
extern crate rustc_serialize;

pub mod client;
mod utils;
pub mod error;
pub mod queue;
pub mod status;
pub mod stats;
pub mod outputs;
pub mod songs;
pub mod playlists;
//pub mod idle;

