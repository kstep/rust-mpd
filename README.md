# rust-mpd <a href="https://travis-ci.org/kstep/rust-mpd"><img src="https://img.shields.io/travis/kstep/rust-mpd.png?style=flat-square" /></a> <a href="https://crates.io/crates/mpd"><img src="https://img.shields.io/crates/d/mpd.png?style=flat-square" /></a> <a href="https://crates.io/crates/mpd"><img src="https://img.shields.io/crates/v/mpd.png?style=flat-square" /></a> <a href="https://crates.io/crates/mpd"><img src="https://img.shields.io/crates/l/mpd.png?style=flat-square" /></a><a href=http://docs.rs/mpd/><img src="https://docs.rs/mpd/badge.svg" /></a>

Pure Rust version of [libmpdclient](http://www.musicpd.org/libs/libmpdclient/).

[Full documentation](http://docs.rs/mpd/)

## Example

Add to `Cargo.toml`:

```toml
[dependencies]
mpd = "*"
```

Then just use:

```rust
extern crate mpd;

use mpd::Client;
use std::net::TcpStream;

let mut conn = Client::connect("127.0.0.1:6600").unwrap();
conn.volume(100).unwrap();
conn.load("My Lounge Playlist", ..).unwrap();
conn.play().unwrap();
println!("Status: {:?}", conn.status());
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
