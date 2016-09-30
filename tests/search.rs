extern crate mpd;

mod helpers;
use helpers::connect;
use mpd::{Query, Term};

#[test]
fn search() {
    let mut mpd = connect();
    let mut query = Query::new();
    //query.and(mpd::Term::Any, "Soul");
    let songs = mpd.find(&query);
    println!("{:?}", songs);
}

#[test]
fn find_query_format() {
    let mut query = Query::new();
    let finished = query
        .and(Term::Tag("albumartist".into()), "Mac DeMarco")
        .and(Term::Tag("album".into()), "Salad Days")
        .limit(0, 2);
    assert_eq!(&finished.to_string(),
               " albumartist \"Mac DeMarco\" album \"Salad Days\" window 0:2");
}

#[test]
fn count_query_format() {
    let mut query = Query::new();
    let finished = query
        .and(Term::Tag("artist".into()), "Courtney Barnett")
        .group("album");
    assert_eq!(&finished.to_string(),
               " artist \"Courtney Barnett\" group album");
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
