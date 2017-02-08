extern crate mpd;

mod helpers;
use helpers::connect;

#[test]
/// Creating a sticker and then getting that sticker returns the value that was set.
fn set_sticker() {
    let mut mpd = connect();

    static VALUE: &'static str = "value";

    mpd.set_sticker("song", "empty.flac", "test_sticker", VALUE).unwrap();

    let sticker = mpd.sticker("song", "empty.flac", "test_sticker").unwrap();
    assert_eq!(sticker, VALUE);
}
