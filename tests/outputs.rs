extern crate mpd;

mod helpers;
use helpers::connect;

#[test]
fn outputs() {
    let mut mpd = connect();
    println!("{:?}", mpd.outputs());
}

#[test]
fn out_toggle() {
    let mut mpd = connect();

    mpd.out_disable(0).unwrap();
    mpd.out_enable(0).unwrap();

    if mpd.version >= mpd::Version(0, 17, 0) {
        mpd.out_toggle(0).unwrap();
    }

    mpd.output(0, true).unwrap();
}
