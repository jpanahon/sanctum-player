use lofty::config::{ParseOptions, ParsingMode};
use lofty::error::LoftyError;
use lofty::file::TaggedFile;
use lofty::prelude::*;
use lofty::probe::Probe;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use ustr::Ustr;

pub struct Song {
    pub title: Ustr,
    pub artist: Ustr,
    pub album: Ustr,
    pub search_key: Ustr,
    pub path: PathBuf,
    pub duration: u64,
    pub created: SystemTime,
}

pub fn get_tags(path: &Path, options: ParseOptions) -> Result<TaggedFile, LoftyError> {
    Probe::open(path)?.options(options).read()
}

pub fn load_songs(main_dir: impl AsRef<Path>) -> Vec<Song> {
    let song_entries: Vec<_> = match std::fs::read_dir(main_dir) {
        Ok(dir) => dir.filter_map(Result::ok).collect(),
        Err(e) => {
            eprintln!("Failed to open music directory: {e}");
            return Vec::new();
        }
    };

    let mut songs: Vec<Song> = Vec::with_capacity(song_entries.len());
    let parsing_options = ParseOptions::new().parsing_mode(ParsingMode::Relaxed);

    for entry in song_entries {
        let path = entry.path();

        // Skip directories and non-files
        if !path.is_file() {
            continue;
        }

        match get_tags(&path, parsing_options) {
            Ok(tag_file) => {
                let tag = tag_file.primary_tag().or_else(|| tag_file.first_tag());

                let title_cow = tag.as_ref().and_then(|t| t.title());
                let artist_cow = tag.as_ref().and_then(|t| t.artist());
                let album_cow = tag.as_ref().and_then(|t| t.album());

                let title = title_cow.as_deref().unwrap_or("Unknown");
                let artist = artist_cow.as_deref().unwrap_or("Unknown");
                let album = album_cow.as_deref().unwrap_or("Unknown");

                let duration = tag_file.properties().duration().as_secs();

                // Fall back to epoch time if creation date is unavailable on the OS
                let created_time = entry
                    .metadata()
                    .and_then(|m| m.created())
                    .unwrap_or(SystemTime::UNIX_EPOCH);

                let search_str = format!("{title} {artist} {album}").to_lowercase();

                let song = Song {
                    title: Ustr::from(title),
                    artist: Ustr::from(artist),
                    album: Ustr::from(album),
                    search_key: Ustr::from(&search_str),
                    path,
                    duration,
                    created: created_time,
                };

                songs.push(song);
            }
            Err(e) => eprintln!(
                "Skipping invalid or untagged file {:?}: {e}",
                path.display()
            ),
        }
    }

    songs.shrink_to_fit();
    songs
}
