
use std::str::FromStr;

#[link(name = "mpdclient")]
extern "C" {
    fn mpd_tag_name(typ: TagType) -> *const u8;
    fn mpd_tag_name_parse(name: *const u8) -> TagType;
}

#[repr(C)]
#[deriving(Show)]
pub enum TagType {
    Unknown = -1,
    Artist = 0,
    Album = 1,
    AlbumArtist = 2,
    Title = 3,
    Track = 4,
    Name = 5,
    Genre = 6,
    Date = 7,
    Composer = 8,
    Performer = 9,
    Comment = 10,
    Disc = 11,

    MbArtistId = 12,
    MbAlbumId = 13,
    MbAlbumArtistId = 14,
    MbTrackId = 15,
}

impl TagType {
    pub fn name(&self) -> Option<String> {
        let name = unsafe { mpd_tag_name(*self) };
        if name.is_null() {
            None
        } else {
            Some(unsafe { String::from_raw_buf(name) })
        }
    }
}

impl FromStr for TagType {
    fn from_str(s: &str) -> Option<TagType> {
        Some(s.with_c_str(|s| unsafe { mpd_tag_name_parse(s as *const u8) }))
    }
}
