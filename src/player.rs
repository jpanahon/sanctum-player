use crate::Config;
use crate::MprisState;
use rand::Rng;
use std::time::Duration;

pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover: lofty::picture::Picture,
    pub path: String,
    pub duration: u64,

    pub search_key: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Playlist {
    pub name: String,
    pub path: String,
}

pub struct Player {
    pub _stream_handle: rodio::OutputStream,
    pub sink: rodio::Sink,
    pub current_index: usize,
    pub prev_index: usize,
    pub shuffle: bool,
    pub repeat: bool,
    pub track_pos: u64,
    pub skip: bool,
}

impl Default for Player {
    fn default() -> Self {
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("Can't find speaker!");

        let sink = rodio::Sink::connect_new(&stream_handle.mixer());

        Self {
            _stream_handle: stream_handle,
            sink: sink,
            current_index: 0,
            prev_index: 0,
            shuffle: false,
            repeat: false,
            track_pos: 0,
            skip: false,
        }
    }
}
impl Player {
    pub fn handle_keybinds(
        &mut self,
        i: &eframe::egui::InputState,
        event: &egui::Event,
        volume: &mut u32,
        config: &mut Config,
        songs: &Vec<Song>,
    ) {
        if let egui::Event::Key {
            key: egui::Key::Space,
            pressed: true,
            repeat: false,
            ..
        } = event
        {
            self.playback();
        }

        if i.modifiers.ctrl {
            if let egui::Event::Key {
                key: egui::Key::ArrowLeft,
                pressed: true,
                repeat: false,
                ..
            } = event
            {
                self.previous(&songs);
            }

            if let egui::Event::Key {
                key: egui::Key::ArrowRight,
                pressed: true,
                repeat: false,
                ..
            } = event
            {
                self.skip(&songs);
            }

            if i.key_pressed(egui::Key::ArrowUp) {
                *volume += 1;
                self.volume(*volume);
            }

            if i.key_pressed(egui::Key::ArrowDown) {
                *volume -= 1;
                self.volume(*volume);
                config.set_volume(*volume);
            }

            if let egui::Event::Key {
                key: egui::Key::S,
                pressed: true,
                repeat: false,
                ..
            } = event
            {
                self.shuffle();
            }
        }
    }
    pub fn set_index(&mut self, index: usize) {
        self.current_index = index;

        if self.sink.is_paused() {
            self.sink.play();
        }

        if !self.idle() {
            self.sink.skip_one();
        }
    }

    pub fn idle(&self) -> bool {
        self.sink.empty() || self.sink.is_paused()
    }

    pub fn process(&mut self, songs: &Vec<Song>) {
        self.track_pos = self.sink.get_pos().as_secs();

        let max_duration = songs[self.current_index].duration;

        if self.sink.empty() && self.track_pos == max_duration && !self.repeat {
            if (self.current_index + 1) > songs.len() - 1 {
                self.current_index = 0;
            } else {
                self.current_index += 1;
            }
        }

        if self.repeat {
            if self.sink.empty() {
                self.play(songs);
            }
        }

        if self.current_index != self.prev_index {
            if self.shuffle {
                let mut rng = rand::rng();
                self.current_index = rng.random_range(0..=songs.len() - 1);
            }

            self.play(songs);

            if self.skip {
                self.sink.skip_one();
                self.skip = false;
            }

            self.prev_index = self.current_index;
        }
    }

    pub fn playback(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    fn play(&mut self, songs: &Vec<Song>) {
        let song_path = &songs[self.current_index].path;
        let song_file = std::fs::File::open(song_path).unwrap();
        let decoder = rodio::Decoder::try_from(song_file).expect("Unable to make decoder!");

        self.sink.append(decoder);
    }

    pub fn skip(&mut self, songs: &Vec<Song>) {
        if self.sink.len() > 1 {
            self.sink.skip_one();
        } else if (self.current_index + 1) == songs.len() {
            self.current_index = 0;
        } else {
            self.current_index += 1;
        }

        self.skip = true;
    }

    pub fn previous(&mut self, songs: &Vec<Song>) {
        if self.current_index == 0 {
            self.current_index = songs.len() - 1;
        } else {
            self.current_index -= 1;
        }

        self.skip = true;
    }

    pub fn volume(&mut self, new_volume: u32) {
        if new_volume as f32 != self.sink.volume() {
            self.sink.set_volume(new_volume as f32 / 100.);
        }
    }

    pub fn done(&self) -> bool {
        self.sink.empty()
    }

    pub fn shuffle(&mut self) {
        self.shuffle = !self.shuffle;
    }

    pub fn set_shuffle(&mut self, toggle: bool) {
        self.shuffle = toggle;
    }

    pub fn repeat(&mut self) {
        self.repeat = !self.repeat;
    }

    pub fn is_shuffled(&self) -> bool {
        self.shuffle
    }

    pub fn seek(&mut self) {
        let new_pos = Duration::from_secs(self.track_pos);
        self.sink.try_seek(new_pos).expect("Can't seek!");
    }

    pub fn seek_to(&mut self, seconds: i64) {
        let new_pos = Duration::from_secs(seconds as u64);
        self.sink.try_seek(new_pos).expect("Can't seek!");
    }

    pub fn handle_mpris(&mut self, state: MprisState, songs: &Vec<Song>) {
        match state {
            MprisState::Play => self.sink.play(),
            MprisState::Pause => self.playback(),
            MprisState::PlayPause => self.playback(),
            MprisState::Volume(vol) => self.volume(vol as u32),
            MprisState::Next => self.skip(songs),
            MprisState::Previous => self.previous(songs),
            MprisState::Shuffle(toggle_shuffle) => self.set_shuffle(toggle_shuffle),
            MprisState::Loop => self.repeat(),
            MprisState::Metadata => (),
            MprisState::Seek(pos) => self.seek_to(pos),
        }
    }
}
