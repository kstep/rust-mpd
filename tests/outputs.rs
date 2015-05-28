extern crate mpd;
extern crate time;

mod helpers;
use helpers::connect;

#[test]
fn outputs() {
    let mut mpd = connect();
    println!("{:?}", mpd.outputs());
}
