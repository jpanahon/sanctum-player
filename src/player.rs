use rand::Rng;

#[derive(Debug, serde::Deserialize)]
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub path: String,
    pub duration: u64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct Playlist {
    pub name: String,
    pub path: String,
}

pub struct Player {
    pub sink: rodio::Sink,
    pub current_index: usize,
    pub prev_index: usize,
    pub shuffle: bool,
    pub repeat: bool,
    pub track_pos: u64,
    pub volume: u32,
    pub skip: bool,
}

impl Player {
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
        if new_volume != self.volume {
            self.volume = new_volume;
            self.sink.set_volume(self.volume as f32 / 100.);
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
