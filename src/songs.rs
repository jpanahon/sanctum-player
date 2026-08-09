use crate::utils::format_date;
use lofty::config::{ParseOptions, ParsingMode};
use lofty::error::LoftyError;
use lofty::file::TaggedFile;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::PathBuf;
use std::time::SystemTime;
pub struct Song {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub search_key: String,
    pub path: String,
    pub created_date: String,

    pub duration: u64,
    pub created: SystemTime,
}

pub fn get_tags(path: PathBuf, options: ParseOptions) -> Result<TaggedFile, LoftyError> {
    let tag_file = Probe::open(path.as_path())?.options(options).read()?;
    Ok(tag_file)
}

pub fn load_songs(main_dir: String) -> Vec<Song> {
    let mut songs: Vec<Song> = Vec::new();
    let parsing_options = ParseOptions::new().parsing_mode(ParsingMode::Relaxed);

    for entry in std::fs::read_dir(main_dir).expect("Music folder not found!") {
        let entry = entry.expect("Entries found!");
        let path = entry.path();

        let song_path = path.display().to_string();

        match get_tags(path, parsing_options) {
            Ok(tag_file) => {
                let tag = tag_file
                    .primary_tag()
                    .or_else(|| tag_file.first_tag())
                    .expect(&format!("No tags found!: {}", song_path));

                let properties = tag_file.properties();

                let duration = properties.duration();
                let seconds = duration.as_secs();

                let metadata = entry.metadata().expect("No metadata found!");
                let created_time = metadata.created().ok().unwrap();
                let created_date = format_date(created_time);

                let song = Song {
                    title: tag.title().as_deref().unwrap_or("Unknown").to_string(),
                    artist: tag.artist().as_deref().unwrap_or("Unknown").to_string(),
                    album: tag.album().as_deref().unwrap_or("Unknown").to_string(),
                    path: song_path,
                    duration: seconds,
                    search_key: format!(
                        "{} {} {}",
                        tag.title().as_deref().unwrap_or("Unknown").to_lowercase(),
                        tag.artist().as_deref().unwrap_or("Unknown").to_lowercase(),
                        tag.album().as_deref().unwrap_or("Unknown").to_lowercase(),
                    ),
                    created: created_time,
                    created_date,
                };

                songs.push(song);
            }
            Err(e) => eprintln!("Tag parse error for {:?}: {e}", song_path),
        }
    }

    songs
}
