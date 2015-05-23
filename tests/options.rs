extern crate mpd;
extern crate time;

use std::net::TcpStream;
use time::Duration;

fn connect() -> mpd::Client<TcpStream> {
    mpd::Client::new(TcpStream::connect(option_env!("MPD_SOCK").unwrap_or("127.0.0.1:6600")).unwrap()).unwrap()
}

#[test]
fn status() {
    let mut mpd = connect();
    let status = mpd.status().unwrap();
    println!("{:?}", status);
}

macro_rules! test_options_impl {
    ($name:ident, $val1:expr, $tval1:expr, $val2:expr, $tval2:expr) => {
        #[test]
        fn $name() {
            let mut mpd = connect();
            mpd.$name($val1).unwrap();
            assert_eq!(mpd.status().unwrap().$name, $tval1);
            mpd.$name($val2).unwrap();
            assert_eq!(mpd.status().unwrap().$name, $tval2);
        }
    }
}

macro_rules! test_option {
    ($name:ident, $val1:expr, $val2:expr) => {
        test_options_impl!($name, $val1, $val1, $val2, $val2);
    };
    ($name:ident, $val1:expr => $tval1:expr, $val2:expr => $tval2:expr) => {
        test_options_impl!($name, $val1, $tval1, $val2, $tval2);
    };
}

test_option!(volume, 100, 0);
test_option!(consume, true, false);
test_option!(single, true, false);
test_option!(random, true, false);
test_option!(repeat, true, false);
test_option!(crossfade, 1000 => Some(1000), 0 => None);
//test_option!(mixrampdb, 1.0f32, 0.0f32);
//test_option!(mixrampdelay, 1 => Some(Duration::seconds(1)), 0 => None);

#[test]
fn replaygain() {
    let mut mpd = connect();
    if mpd.version >= mpd::Version(0, 16, 0) {
        mpd.replaygain(mpd::ReplayGain::Track).unwrap();
        assert_eq!(mpd.get_replaygain().unwrap(), mpd::ReplayGain::Track);
        mpd.replaygain(mpd::ReplayGain::Off).unwrap();
        assert_eq!(mpd.get_replaygain().unwrap(), mpd::ReplayGain::Off);
    }
}
