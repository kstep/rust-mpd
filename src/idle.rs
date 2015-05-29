use std::fmt;
use std::str::FromStr;

use error::ParseError;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Subsystem {
    /// database: the song database has been modified after update.
    Database,
    /// update: a database update has started or finished. If the database was modified during the update, the database event is also emitted.
    Update,
    /// stored_playlist: a stored playlist has been modified, renamed, created or deleted
    Playlist,
    /// playlist: the current playlist has been modified
    Queue,
    /// player: the player has been started, stopped or seeked
    Player,
    /// mixer: the volume has been changed
    Mixer,
    /// output: an audio output has been enabled or disabled
    Output,
    /// options: options like repeat, random, crossfade, replay gain
    Options,
    /// sticker: the sticker database has been modified.
    Sticker,
    /// subscription: a client has subscribed or unsubscribed to a channel
    Subscription,
    /// message: a message was received on a channel this client is subscribed to; this event is only emitted when the queue is empty
    Message
}

impl FromStr for Subsystem {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Subsystem, ParseError> {
        use self::Subsystem::*;
        match s {
            "database" => Ok(Database),
            "update" => Ok(Update),
            "stored_playlist" => Ok(Playlist),
            "playlist" => Ok(Queue),
            "player" => Ok(Player),
            "mixer" => Ok(Mixer),
            "output" => Ok(Output),
            "options" => Ok(Options),
            "sticker" => Ok(Sticker),
            "subscription" => Ok(Subscription),
            "message" => Ok(Message),
            _ => Err(ParseError::BadValue(s.to_owned()))
        }
    }
}

impl fmt::Display for Subsystem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Subsystem::*;
        f.write_str(match *self {
            Database => "database",
            Update => "update",
            Playlist => "stored_playlist",
            Queue => "playlist",
            Player => "player",
            Mixer => "mixer",
            Output => "output",
            Options => "options",
            Sticker => "sticker",
            Subscription => "subscription",
            Message => "message"
        })
    }
}

