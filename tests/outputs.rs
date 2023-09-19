extern crate mpd;

mod helpers;
use helpers::connect;

#[test]
fn outputs() {
    let mut mpd = connect();

    let outputs = mpd.outputs().unwrap();
    assert_eq!(outputs.len(), 1);

    let null_output = outputs.first().unwrap();
    assert_eq!(null_output.id, 0);
    assert_eq!(null_output.plugin, "null");
    assert_eq!(null_output.name, "null");
    assert!(null_output.enabled);
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
