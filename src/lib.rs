#![feature(macro_rules, slicing_syntax, if_let)]

extern crate libc;
extern crate time;

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


#[cfg(test)]
mod test {

    use connection::MpdConnection;
    use playlists::MpdPlaylist;

    #[test]
    fn test_conn() {
        //let c = MpdConnection::new(Some("192.168.1.10"), 6600);
        let c = MpdConnection::new(None, 6600);
        let mut conn = match c {
            None => panic!("connection is None"),
            Some(Err(e)) => panic!("connection error: {}", e),
            Some(Ok(c)) => c
        };

        println!("{}", conn.set_volume(0));
        println!("{}", conn.settings());
        println!("{}", conn.status());

        let playlists: Vec<MpdPlaylist> = conn.playlists().unwrap().map(|r| r.unwrap()).collect();
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

        println!("{}", conn.stop());

        println!("{}", conn.current_song());

        println!("{}", conn.stats());

        let v = conn.version();
        panic!("{}", v);
    }

}
