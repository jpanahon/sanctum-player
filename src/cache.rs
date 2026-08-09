use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use image::ImageReader;
use lofty::file::TaggedFileExt;
use lofty::picture::Picture;
use lofty::probe::Probe;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Cursor;

use crate::songs::Song;
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use std::sync::mpsc::{Receiver, Sender, channel};

pub struct SancCache {
    pub path: String,
    pub covers: HashMap<String, String>,
    pub loading_covers: HashSet<String>,

    // Rayon custom bounded pool
    pub pool: ThreadPool,

    // Communication channel back to egui main thread
    pub tx: Sender<(String, String)>,
    pub rx: Receiver<(String, String)>,
}

impl SancCache {
    pub fn new(cache_path: String) -> Self {
        let (tx, rx) = channel();

        // Strict limit: Maximum 2 threads decoding images at once
        let pool = ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .expect("Failed to create Rayon pool");

        Self {
            path: cache_path,
            covers: std::collections::HashMap::new(),
            loading_covers: std::collections::HashSet::new(),
            pool,
            tx,
            rx,
        }
    }
}

fn hash_album(album: &str) -> String {
    URL_SAFE.encode(album).to_string()
}

fn dehash_album(album: String) -> String {
    let error_msg = format!("Can't decode string: {}", album);
    let decoded = URL_SAFE.decode(album).expect(&error_msg);
    String::from_utf8(decoded).expect("Can't decode!")
}

fn get_cover(song_path: &str) -> Option<Picture> {
    let tag_file = Probe::open(song_path).ok()?.read().ok()?;

    // Get primary or first tag safely
    let mut tag = tag_file
        .primary_tag()
        .or_else(|| tag_file.first_tag())?
        .clone();

    if tag.pictures().is_empty() {
        None
    } else {
        // Safely extract the Picture without cloning underlying bytes
        Some(tag.remove_picture(0))
    }
}

pub fn load_cache(cache: &mut SancCache) {
    let cache_path = std::path::Path::new(&cache.path);

    if !cache_path.exists() {
        std::fs::create_dir_all(cache_path).expect("Can't create cache folder!");
    } else if let Ok(entries) = std::fs::read_dir(cache_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_stem() {
                let album_name = dehash_album(file_name.to_string_lossy().to_string());
                cache.covers.insert(album_name, path.display().to_string());
            }
        }
    }
}

pub fn load_cover_art(ui: &mut egui::Ui, cache: &mut SancCache, song: &Song) {
    let album = &song.album;

    let response = if let Some(cover_art) = cache.covers.get(album) {
        ui.add_sized([48., 48.], egui::Image::new(format!("file://{cover_art}")))
    } else {
        ui.allocate_response(egui::vec2(48., 48.), egui::Sense::hover())
    };

    let is_visible = response.rect.intersects(ui.clip_rect());

    if is_visible && !cache.covers.contains_key(album) && !cache.loading_covers.contains(album) {
        cache.loading_covers.insert(album.clone());

        let album_key = album.clone();
        let song_path = song.path.clone();
        let cache_dir = cache.path.clone();
        let tx = cache.tx.clone();

        // Offload to our bounded 2-thread Rayon pool
        cache.pool.spawn(move || {
            // Step A: Extract Picture struct (if present)
            if let Some(picture) = get_cover(&song_path) {
                // Step B: Decode using Cursor over picture.data()
                if let Ok(reader) =
                    ImageReader::new(Cursor::new(picture.data())).with_guessed_format()
                {
                    if let Ok(image_data) = reader.decode() {
                        // Keep 256x256 for MPRIS!
                        let cover_data =
                            image_data.resize_exact(256, 256, image::imageops::Nearest);
                        let image_path = format!("{}/{}.jpg", cache_dir, hash_album(&album_key));

                        if cover_data.save(&image_path).is_ok() {
                            // Notify main egui thread
                            let _ = tx.send((album_key, image_path));
                        }
                    }
                }
            }
        });
    }
}
