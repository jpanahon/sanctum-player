use crate::playlist::{Playlist, Sort};
use std::fs;
use std::fs::File;
use std::io::Write;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Config {
    current_playlist: usize,
    playlists: Vec<Playlist>,
    last_track_index: usize,
    volume: u32,
    pub cache_path: String,
}

impl Config {
    pub fn get_playlists(&self) -> &Vec<Playlist> {
        &self.playlists
    }

    pub fn get_last_track(&self) -> usize {
        self.last_track_index
    }

    pub fn get_volume(&self) -> u32 {
        self.volume
    }

    pub fn set_volume(&mut self, new_volume: u32) {
        self.volume = new_volume;
    }

    // pub fn add_playlist(&mut self, new_playlist: Playlist) {
    //     self.playlists.push(new_playlist);
    // }

    pub fn current_playlist(&self) -> usize {
        self.current_playlist
    }

    pub fn set_playlist(&mut self, new_playlist: usize) {
        self.current_playlist = new_playlist;
    }

    pub fn set_track(&mut self, last_index: usize) {
        self.last_track_index = last_index
    }

    pub fn update_playlist(&mut self, playlist: Playlist) {
        self.playlists[self.current_playlist] = playlist;
    }

    pub fn create(&self, playlist_name: String, playlist_path: String) {
        let new_config = Self {
            current_playlist: 0,
            playlists: [Playlist {
                name: playlist_name,
                path: playlist_path,
                sort_order: Sort::Track { reverse: false },
            }]
            .to_vec(),
            last_track_index: 0,
            volume: 50,
            cache_path: String::from("~/.cache/sanctum/").to_owned(),
        };

        let config_json = serde_json::to_string(&new_config).expect("Can't parse to string");

        fs::create_dir("~/.config/sanctum/").ok();

        let mut config_file =
            File::create("~/.config/sanctum/config.json").expect("Can't make new config file");

        config_file.write_all(config_json.as_bytes()).unwrap();
    }
}
