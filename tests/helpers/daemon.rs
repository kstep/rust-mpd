extern crate tempdir;

use self::tempdir::TempDir;
use super::mpd;
use std::fs::{create_dir, File};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

struct MpdConfig {
    db_file: PathBuf,
    music_directory: PathBuf,
    playlist_directory: PathBuf,
    sticker_file: PathBuf,
    config_path: PathBuf,
    sock_path: PathBuf,
}

impl MpdConfig {
    pub fn new<P>(base: P) -> MpdConfig
    where
        P: AsRef<Path>,
    {
        let base = base.as_ref();
        MpdConfig {
            db_file: base.join("db"),
            music_directory: base.join("music"),
            playlist_directory: base.join("playlists"),
            sticker_file: base.join("sticker_file"),
            config_path: base.join("config"),
            sock_path: base.join("sock"),
        }
    }

    fn config_text(&self) -> String {
        format!(
            r#"
db_file "{db_file}"
log_file "/dev/null"
music_directory "{music_directory}"
playlist_directory "{playlist_directory}"
sticker_file "{sticker_file}"
bind_to_address "{sock_path}"
audio_output {{
    type "null"
    name "null"
}}
"#,
            db_file = self.db_file.display(),
            music_directory = self.music_directory.display(),
            playlist_directory = self.playlist_directory.display(),
            sticker_file = self.sticker_file.display(),
            sock_path = self.sock_path.display(),
        )
    }

    fn generate(&self) {
        create_dir(&self.music_directory).expect("Could not create music directory.");
        create_dir(&self.playlist_directory).expect("Could not create playlist directory.");
        let mut file = File::create(&self.config_path).expect("Could not create config file.");
        file.write_all(self.config_text().as_bytes()).expect("Could not write config file.");
    }
}

pub struct Daemon {
    // Saved here so it gets dropped when this does.
    _temp_dir: TempDir,
    config: MpdConfig,
    process: Child,
}

impl Drop for Daemon {
    fn drop(&mut self) {
        self.process.kill().expect("Could not kill mpd daemon.");
        self.process.wait().expect("Could not wait for mpd daemon to shutdown.");
        if let Some(ref mut stderr) = self.process.stderr {
            let mut output = String::new();
            stderr.read_to_string(&mut output).expect("Could not collect output from mpd.");
            println! {"Output from mpd:"}
            println! {"{}", output};
        }
    }
}

fn sleep() {
    use std::{thread, time};
    let ten_millis = time::Duration::from_millis(10);
    thread::sleep(ten_millis);
}

static EMPTY_FLAC_BYTES: &'static [u8] = include_bytes!("../data/empty.flac");

impl Daemon {
    pub fn start() -> Daemon {
        let temp_dir = TempDir::new("mpd-test").unwrap();
        let config = MpdConfig::new(&temp_dir);
        config.generate();

        // TODO: Factor out putting files in the music directory.
        File::create(config.music_directory.join("empty.flac"))
            .unwrap()
            .write_all(EMPTY_FLAC_BYTES)
            .unwrap();

        let process = Command::new("mpd")
            .arg("--no-daemon")
            .arg(&config.config_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Could not create mpd daemon.");

        let daemon = Daemon {
            _temp_dir: temp_dir,
            config: config,
            process: process,
        };

        // Wait until we can connect to the daemon
        let mut client;
        loop {
            if let Ok(c) = daemon.maybe_connect() {
                client = c;
                break;
            }
            sleep()
        }
        while let Some(_) = client.status().expect("Couldn't get status.").updating_db {
            sleep()
        }

        daemon
    }

    fn maybe_connect(&self) -> Result<mpd::Client<UnixStream>, mpd::error::Error> {
        let stream = UnixStream::connect(&self.config.sock_path)?;
        mpd::Client::new(stream)
    }

    pub fn connect(&self) -> mpd::Client<UnixStream> {
        self.maybe_connect().expect("Could not connect to daemon.")
    }
}
