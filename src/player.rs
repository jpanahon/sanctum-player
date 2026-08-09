use std::sync::Arc;
use std::sync::Mutex;

use crate::Config;
use crate::MprisHandler;
use crate::cache::SancCache;
use mpris_server::{Metadata, PlaybackStatus, Property, Server, Time, TrackId};
use rand::Rng;

use crate::songs::Song;

use std::time::{Duration, Instant};

pub enum PlaybackMode {
    Normal,
    Shuffled,
    Repeat,
}

pub struct PlayerState {
    pub status: PlaybackStatus,
    pub player_pos: u64,
    pub mpris_pos: u64,
    pub mode: PlaybackMode,
    pub metadata: Metadata,
    pub skip: bool,
    pub repeat: bool,
    pub shuffle: bool,
    pub previous: bool,
    pub play: bool,
    pub pause: bool,
    pub play_pause: bool,
    pub stop: bool,
    pub volume: u64,
    pub current_index: usize,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            status: PlaybackStatus::Playing,
            player_pos: 0,
            mpris_pos: 0,
            mode: PlaybackMode::Normal,
            metadata: Metadata::new(),
            skip: false,
            shuffle: false,
            repeat: false,
            previous: false,
            volume: 100,
            play: false,
            pause: false,
            play_pause: false,
            stop: false,
            current_index: 0,
        }
    }
}
pub struct Player {
    pub _stream_handle: rodio::OutputStream,
    pub sink: rodio::Sink,
    pub track_pos: u64,
    pub current_index: usize,
    pub prev_index: usize,
    pub queue: Vec<usize>,
    pub skip: bool,
    pub last_skip: Instant,
    pub mode: PlaybackMode,
    pub state: Arc<Mutex<PlayerState>>,
}

impl Player {
    pub fn new(current_index: usize, state: Arc<Mutex<PlayerState>>) -> Self {
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("Can't find speaker!");

        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        Self {
            _stream_handle: stream_handle,
            sink,
            current_index,
            prev_index: 0,
            track_pos: 0,
            skip: false,
            last_skip: Instant::now(),
            queue: Vec::new(),
            mode: PlaybackMode::Normal,
            state,
        }
    }

    pub fn handle_keybinds(
        &mut self,
        i: &eframe::egui::InputState,
        event: &egui::Event,
        volume: &mut u32,
        config: &mut Config,
        songs: &[Song],
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
                self.previous(songs);
            }

            if let egui::Event::Key {
                key: egui::Key::ArrowRight,
                pressed: true,
                repeat: false,
                ..
            } = event
            {
                self.skip(songs);
            }

            if i.key_pressed(egui::Key::ArrowUp) {
                *volume += 1;
                self.volume(*volume);
                config.set_volume(*volume);
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

            if let egui::Event::Key {
                key: egui::Key::R,
                pressed: true,
                repeat: false,
                ..
            } = event
            {
                self.repeat();
            }
        }
    }
    pub fn set_index(&mut self, index: usize) {
        self.current_index = index;
        self.prev_index = index;

        if self.sink.is_paused() {
            self.resume();
        }

        if !self.idle() {
            self.sink.skip_one();
        }
    }

    pub fn idle(&self) -> bool {
        self.sink.empty() || self.sink.is_paused()
    }

    pub fn is_playing(&self) -> bool {
        !self.sink.is_paused() && !self.sink.empty()
    }

    pub fn process(&mut self, songs: &[Song]) {
        if songs.is_empty() {
            return;
        }

        // Safety guard against out-of-bounds indices
        if self.current_index >= songs.len() {
            self.current_index = 0;
        }

        self.track_pos = self.sink.get_pos().as_secs();

        let manual_skip = self.skip;
        let cooldown_done = self.last_skip.elapsed() > Duration::from_millis(300);

        // Audio sink is empty and cooldown has passed = track naturally finished
        let track_finished = !manual_skip && self.sink.empty() && cooldown_done;

        if track_finished || manual_skip {
            self.skip = false;
            self.last_skip = Instant::now();

            // 1. Advance the index based on queue or playback mode
            if !self.queue.is_empty() {
                let queued_idx = self.queue.remove(0);
                self.current_index = if queued_idx < songs.len() {
                    queued_idx
                } else {
                    0
                };
            } else {
                match self.mode {
                    PlaybackMode::Normal => {
                        self.current_index = (self.current_index + 1) % songs.len();
                    }
                    PlaybackMode::Shuffled => {
                        let mut rng = rand::rng();
                        self.current_index = rng.random_range(0..songs.len());
                    }
                    PlaybackMode::Repeat => {
                        // If manually skipped in repeat mode, move to next song;
                        // if track finished naturally, current_index stays the same.
                        if manual_skip {
                            self.current_index = (self.current_index + 1) % songs.len();
                        }
                    }
                }
            }

            // 2. Always start playing the newly selected track!
            self.play(songs);
            self.prev_index = self.current_index;
        }
    }

    pub fn update_state(
        &mut self,
        mpris: &Server<MprisHandler>,
        cache: &SancCache,
        songs: &[Song],
    ) {
        let song = &songs[self.current_index];

        let mut trigger_skip = false;
        let mut trigger_previous = false;
        let mut trigger_stop = false;
        let mut trigger_play = false;
        let mut trigger_pause = false;
        let mut trigger_play_pause = false;

        let mut status_changed = false;
        let mut metadata_changed = false;
        let mut pos_changed = false;
        let mut mpris_pos = 0;

        let mut new_status: PlaybackStatus = PlaybackStatus::Stopped;
        let mut new_metadata: Metadata = Metadata::new();

        if let Ok(mut state) = self.state.lock() {
            new_status = if self.is_playing() {
                PlaybackStatus::Playing
            } else if self.sink.empty() {
                PlaybackStatus::Stopped
            } else {
                PlaybackStatus::Paused
            };

            if state.status != new_status {
                state.status = new_status;
                status_changed = true;
            }

            if state.player_pos != self.track_pos {
                state.player_pos = self.track_pos;
                state.mpris_pos = self.track_pos;
            }

            if state.mpris_pos != state.player_pos {
                mpris_pos = state.mpris_pos;
                state.player_pos = state.mpris_pos;
                pos_changed = true;
            }

            state.mode = if matches!(self.mode, PlaybackMode::Normal) {
                PlaybackMode::Normal
            } else if matches!(self.mode, PlaybackMode::Repeat) {
                PlaybackMode::Repeat
            } else {
                PlaybackMode::Shuffled
            };

            new_metadata = Metadata::builder()
                .title(song.title.clone())
                .artist(vec![song.artist.clone()])
                .album(song.album.clone())
                .length(Time::from_secs(song.duration as i64))
                .trackid(TrackId::NO_TRACK)
                .build();

            if let Some(cover_art) = cache.covers.get(&song.album) {
                new_metadata.set_art_url(Some(format!("file://{}", cover_art)));
            }

            if state.metadata != new_metadata {
                state.metadata = new_metadata.clone();
                metadata_changed = true;
            }

            if state.skip {
                state.skip = false;
                self.skip = true;
                trigger_skip = true;
            }

            if state.previous {
                state.previous = false;
                self.skip = true;
                trigger_previous = true;
            }

            if state.play && !self.is_playing() {
                state.play = false;
                trigger_play = true;
            }
            if state.stop && !self.idle() {
                state.stop = false;
                trigger_stop = true;
            }

            if state.pause && self.is_playing() {
                state.pause = false;
                trigger_pause = true;
            }

            if state.play_pause {
                state.play_pause = false;
                trigger_play_pause = true;
            }
        }

        if trigger_skip {
            self.skip(songs);
        }

        if trigger_previous {
            self.previous(songs);
        }

        if trigger_play {
            self.resume();
        }

        if trigger_pause {
            self.pause();
        }

        if trigger_stop {
            self.stop();
        }

        if trigger_play_pause {
            self.playback();
        }

        if status_changed {
            futures::executor::block_on(
                mpris.properties_changed([Property::PlaybackStatus(new_status)]),
            )
            .expect("Failed to update PlaybackStatus to Playing!");
        }

        if metadata_changed {
            futures::executor::block_on(
                mpris.properties_changed([Property::Metadata(new_metadata)]),
            )
            .expect("Failed to update PlaybackStatus to Playing!");
        }

        if pos_changed {
            self.seek_to(mpris_pos as i64);
        }
    }

    pub fn playback(&mut self) {
        if self.sink.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }

    fn resume(&mut self) {
        self.sink.play();
    }

    fn pause(&mut self) {
        self.sink.pause();
    }

    fn stop(&mut self) {
        self.track_pos = 0;
        self.sink.stop();
    }

    pub fn add_queue(&mut self, index: usize) {
        self.queue.push(index);
    }

    fn play(&mut self, songs: &[Song]) {
        let song = &songs[self.current_index];
        let song_path = &song.path;

        let song_file = std::io::BufReader::new(
            std::fs::File::open(song_path)
                .unwrap_or_else(|_| panic!("Unable to open file: {song_path}")),
        );
        let decoder = rodio::Decoder::try_from(song_file).expect("Unable to make decoder!");

        self.sink.append(decoder);
    }

    pub fn skip(&mut self, songs: &[Song]) {
        if !self.queue.is_empty() {
            self.current_index = self.queue.remove(0);
        } else if (self.current_index + 1) >= songs.len() {
            self.current_index = 0;
        } else {
            self.current_index += 1;
        }

        self.skip = true;
    }

    pub fn previous(&mut self, songs: &[Song]) {
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
        self.mode = if matches!(self.mode, PlaybackMode::Shuffled) {
            PlaybackMode::Normal
        } else {
            PlaybackMode::Shuffled
        }
    }

    pub fn set_shuffle(&mut self, toggle: bool) {
        if toggle {
            self.mode = PlaybackMode::Shuffled;
        } else {
            self.mode = PlaybackMode::Normal;
        }
    }

    pub fn repeat(&mut self) {
        self.mode = if matches!(self.mode, PlaybackMode::Repeat) {
            PlaybackMode::Normal
        } else {
            PlaybackMode::Repeat
        }
    }

    pub fn is_repeat(&self) -> bool {
        let is_repeat = matches!(self.mode, PlaybackMode::Repeat);
        is_repeat
    }

    pub fn is_shuffled(&self) -> bool {
        let is_shuffled = matches!(self.mode, PlaybackMode::Shuffled);
        is_shuffled
    }

    pub fn seek(&mut self) {
        let new_pos = Duration::from_secs(self.track_pos);
        self.sink.try_seek(new_pos).expect("Can't seek!");
    }

    pub fn seek_to(&mut self, seconds: i64) {
        let new_pos = Duration::from_secs(seconds as u64);
        self.sink.try_seek(new_pos).expect("Can't seek!");
    }
}
