
use error::MpdResult;
use client::MpdClient;
use songs::MpdSong;
use playlists::MpdPlaylist;

pub struct MpdQueue;

pub trait MpdQueuePos {
    fn to_pos(self) -> String;
}

impl MpdQueuePos for uint {
    fn to_pos(self) -> String { self.to_string() }
}

impl MpdQueuePos for (uint, uint) {
    fn to_pos(self) -> String { format!("{}:{}", self.0, self.1) }
}

impl MpdQueue {
    pub fn clear<S: Stream>(client: &mut MpdClient<S>) -> MpdResult<()> {
        client.exec("clear").and_then(|_| client.ok())
    }

    pub fn push<S: Stream>(client: &mut MpdClient<S>, file: &str) -> MpdResult<()> {
        client.exec_arg("add", file).and_then(|_| client.ok())
    }

    pub fn insert<S: Stream>(client: &mut MpdClient<S>, index: uint, file: &str) -> MpdResult<uint> {
        let result = client.exec_arg2("addid", file, uint)
            .and_then(|_| client.iter().next().unwrap_or(Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput)))))
            .and_then(|Ok(MpdPair(ref name, ref value))| if name[] == "Id" { Ok(value.parse()) } else {
                Err(FromError::from_error(standard_error(IoErrorKind::InvalidInput))) });
        try!(client.ok());
        result
    }

    pub fn swap<S: Stream>(client: &mut MpdClient<S>, index1: uint, index2: uint) -> MpdResult<uint> {
        client.exec_arg2("swap", index1, index2).and_then(|_| client.ok())
    }

    pub fn shift<S: Stream, I: MpdQueuePos>(client: &mut MpdClient<S>, index: I, target: uint) -> MpdResult<()> {
        client.exec_arg2("move", index.to_pos(), target).and_then(|_| client.ok())
    }

    pub fn priority<S: Stream, I: MpdQueuePos>(client: &mut MpdClient<S>, index: I, prio: u8) -> MpdResult<()> {
        client.exec_arg2("prio", prio, index.to_pos()).and_then(|_| client.ok())
    }

    pub fn priorityid<S: Stream>(client: &mut MpdClient<S>, id: uint, prio: u8) -> MpdResult<()> {
        client.exec_arg2("prioid", prio, id).and_then(|_| client.ok())
    }

    pub fn rangeid<S: Stream>(client: &mut MpdClient<S>, id: uint, range: Option<(Duration, Duration)>) -> MpdResult<()> {
        client.exec_arg2("rangeid", id, range.map(format!("{}:{}", range.0.num_seconds(), range.1.num_seconds())).unwrap_or(":".to_string())).and_then(|_| client.ok())
    }

    pub fn get<S: Stream>(client: &mut MpdClient<S>, index: uint) -> MpdResult<MpdSong> {
        client.exec_arg("playlistinfo", index).and_then(|_| client.iter().collect())
    }

    pub fn slice<S: Stream>(client: &mut MpdClient<S>, slice: (uint, uint)) -> MpdResult<Vec<MpdSong>> {
        client.exec_arg("playlistinfo", format!("{}:{}", slice.0, slice.1)).and_then(|_| client.iter().collect())
    }

    pub fn remove<S: Stream, I: MpdQueuePos>(client: &mut MpdClient<S>, index: I) -> MpdResult<()> {
        client.exec_arg("delete", index.to_pos()).and_then(|_| client.ok())
    }

    pub fn removeid<S: Stream>(client: &mut MpdClient<S>, id: uint) -> MpdResult<()> {
        client.exec_arg("deleteid", id).and_then(|_| client.ok())
    }

    pub fn shiftid<S: Stream>(client: &mut MpdClient<S>, id: uint, target: uint) -> MpdResult<()> {
        client.exec_arg2("moveid", id, target).and_then(|_| client.ok())
    }

    pub fn getid<S: Stream>(client: &mut MpdClient<S>, id: uint) -> MpdResult<MpdSong> {
        client.exec_arg("playlistid", id).and_then(|_| client.iter().collect())
    }

    pub fn songs<S: Stream>(client: &mut MpdClient<S>) -> MpdResult<Vec<MpdSong>> {
        client.exec("playlistinfo").and_then(|_| client.iter().collect())
    }

    pub fn shuffle_slice<S: Stream>(client: &mut MpdClient<S>, slice: (uint, uint)) -> MpdResult<()> {
        client.exec_arg("shuffle", format!("{}:{}", slice.0, slice.1)).and_then(|_| client.ok())
    }

    pub fn shuffle<S: Stream>(client: &mut MpdClient<S>) -> MpdResult<()> {
        client.exec("shuffle").and_then(|_| client.ok())
    }

    pub fn load<S: Stream>(client: &mut MpdClient<S>, name: &str) -> MpdResult<()> {
        client.exec_arg("load", name).and_then(|_| client.ok())
    }

    pub fn load_slice<S: Stream>(client: &mut MpdClient<S>, name: &str, slice: (uint, uint)) -> MpdResult<()> {
        client.exec_arg2("load", name, format!("{}:{}", slice.0, slice.1)).and_then(|_| client.ok())
    }

    pub fn save<S: Stream>(client: &mut MpdClient<S>, name: &str) -> MpdResult<()> {
        client.exec_arg("save", name).and_then(|_| client.ok())
    }
}
