extern crate mpd;

mod helpers;
use helpers::connect;
use mpd::Query;

#[test]
fn search() {
    let mut mpd = connect();
    let mut query = Query::new();
    let query = query.and(mpd::Term::Any, "Soul");
    let songs = mpd.find(query, None);
    println!("{:?}", songs);
    assert!(songs.is_ok());
}
