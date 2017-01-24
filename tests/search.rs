extern crate mpd;

mod helpers;
use helpers::connect;
use mpd::{Query, Term};
use mpd::search::Window;

#[test]
fn search() {
    let mut mpd = connect();
    let mut query = Query::new();
    let query = query.and(mpd::Term::Any, "Soul");
    let songs = mpd.find(query, None);
    println!("{:?}", songs);
    assert!(songs.is_ok());
}

#[test]
fn find_query_format() {
    let mut query = Query::new();
    let finished = query.and(Term::Tag("albumartist".into()), "Mac DeMarco")
        .and(Term::Tag("album".into()), "Salad Days");
    assert_eq!(&finished.to_string(), " albumartist \"Mac DeMarco\" album \"Salad Days\"");
}

#[test]
fn find_window_format() {
    let window: Window = (0, 2).into();
    assert_eq!(&window.to_string(), " window 0:2");
}
