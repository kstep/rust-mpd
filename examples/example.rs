extern crate mpd;

use mpd::{Client, Query};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut c = Client::connect("127.0.0.1:6600").unwrap();
    println!("version: {:?}", c.version);
    println!("status: {:?}", c.status());
    println!("stuff: {:?}", c.find(&Query::new(), (1, 2)));

    let now_playing = c.currentsong()?;
    if let Some(song) = now_playing {
        println!("Metadata:");
        for (k, v) in (c.readcomments(song)?).flatten() {
            println!("{}: {}", k, v);
        }
    } else {
        println!("No song playing.");
    }
    Ok(())
}
