use crate::Config;
use rand::Rng;

pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover: lofty::picture::Picture,
    pub path: String,
    pub duration: u64,
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
        ui: &eframe::egui::Ui,
        volume: &mut u32,
        config: &mut Config,
        songs: &Vec<Song>,
    ) {
        let prev_key = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowLeft);
        let skip_key = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowRight);

        let vol_up = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowUp);
        let vol_down = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowDown);

        let shufl_key = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S);

        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            self.playback();
        }

        if ui.input_mut(|i| i.consume_shortcut(&prev_key)) {
            self.previous(&songs);
        }

        if ui.input_mut(|i| i.consume_shortcut(&skip_key)) {
            self.skip(&songs);
        }

        if ui.input_mut(|i| i.consume_shortcut(&vol_up)) {
            *volume += 1;
            self.volume(*volume);
            config.set_volume(*volume);
        }

        if ui.input_mut(|i| i.consume_shortcut(&vol_down)) {
            *volume -= 1;
            self.volume(*volume);
            config.set_volume(*volume);
        }

        if ui.input_mut(|i| i.consume_shortcut(&shufl_key)) {
            self.shuffle();
        }
    }
    pub fn set_index(&mut self, index: usize) {
        self.current_index = index;
    }

    pub fn idle(&self) -> bool {
        self.sink.empty() || self.sink.is_paused()
    }

    pub fn process(&mut self, songs: &Vec<Song>) {
        self.track_pos = self.sink.get_pos().as_secs();

        let max_duration = songs[self.current_index].duration;

        if self.sink.empty() && self.track_pos == max_duration {
            if (self.current_index + 1) > songs.len() - 1 {
                self.current_index = 0;
            } else {
                self.current_index += 1;
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

    pub fn shuffle(&mut self) -> bool {
        self.shuffle = !self.shuffle;
        self.shuffle
    }

    pub fn is_shuffled(&self) -> bool {
        self.shuffle
    }
}
