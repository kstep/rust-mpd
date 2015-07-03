extern crate mpd;

mod helpers;
use helpers::connect;

#[test]
fn search() {
    let mut mpd = connect();
    let mut query = mpd.query();
    //query.and(mpd::Term::Any, "Soul");
    let songs = query.find(false, false);
    println!("{:?}", songs);
}

/*
#[test]
fn count() {
    let mut mpd = connect();
    let song = mpd.search(mpd::Query {
        clauses: vec![mpd::Clause(mpd::Term::Any, "Soul".to_owned())],
        window: None,
        group: None
    }).unwrap();
    println!("{:?}", song);
}
*/
