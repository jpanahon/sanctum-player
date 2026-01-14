use crate::Playlist;

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
}
