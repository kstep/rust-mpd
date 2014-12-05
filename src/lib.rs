#![feature(macro_rules, slicing_syntax, if_let)]

extern crate libc;
extern crate time;

mod common;
pub mod connection;
pub mod settings;
pub mod status;
pub mod stats;
pub mod outputs;
pub mod tags;
pub mod songs;
pub mod playlists;


#[cfg(test)]
mod test {

    use common::{MpdError, MpdResult, MpdErrorKind};
    use connection::MpdConnection;
    use playlists::Playlist;

    #[test]
    fn test_conn() {
        //let c = MpdConnection::new(Some("192.168.1.10"), 6600);
        let c = MpdConnection::new(None, 6600);
        let mut conn = match c {
            None => panic!("connection is None"),
            Some(Err(e)) => panic!("connection error: {}", e),
            Some(Ok(c)) => c
        };

        println!("{}", conn.set_volume(100));
        println!("{}", conn.settings());
        println!("{}", conn.status());

        let playlists: Vec<Playlist> = conn.playlists().unwrap().collect();
        for pl in playlists.iter() {
            println!("{}", pl);
            for s in pl.songs(&mut conn).unwrap() {
                println!("{}", s);
            }
        }

        //for s in playlist.unwrap().songs(&mut conn).unwrap() {
            //println!("{}", s);
        //}

        for o in conn.outputs().unwrap() {
            println!("{}", o);
        }

        conn.play();

        println!("{}", conn.current_song());

        println!("{}", conn.stats());

        let v = conn.version();
        panic!("{}", v);
    }

}
