use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use egui::TextureHandle;
use image::ImageReader;
use lofty::file::TaggedFileExt;
use lofty::picture::Picture;
use lofty::probe::Probe;
use rayon::{ThreadPool, ThreadPoolBuilder};
use std::collections::{HashMap, HashSet};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use ustr::Ustr;

use crate::songs::Song;

pub struct SancCache {
    pub path: String,
    /// Maps Album -> Path on disk (for MPRIS / disk index)
    pub disk_paths: HashMap<Ustr, PathBuf>,
    /// Maps Album -> Live GPU Texture Handle (for UI rendering)
    pub textures: HashMap<Ustr, TextureHandle>,
    pub loading_covers: HashSet<Ustr>,

    // Rayon custom bounded pool
    pub pool: ThreadPool,

    // Communication channel back to egui main thread (Album, Texture Image, Disk Path)
    pub tx: Sender<(Ustr, egui::ColorImage, PathBuf)>,
    pub rx: Receiver<(Ustr, egui::ColorImage, PathBuf)>,
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
            disk_paths: HashMap::new(),
            textures: HashMap::new(),
            loading_covers: HashSet::new(),
            pool,
            tx,
            rx,
        }
    }

    /// Process finished image loads from background threads into egui GPU textures
    pub fn update(&mut self, ctx: &egui::Context) {
        while let Ok((album, color_image, disk_path)) = self.rx.try_recv() {
            self.loading_covers.remove(&album);
            self.disk_paths.insert(album, disk_path);

            // Upload raw RGBA buffer to GPU texture memory ONCE
            let handle = ctx.load_texture(
                album.as_str(),
                                          color_image,
                                          egui::TextureOptions::default(),
            );
            self.textures.insert(album, handle);
        }
    }
}

fn hash_album(album: &str) -> String {
    URL_SAFE.encode(album)
}

fn dehash_album(album: &str) -> Option<String> {
    let decoded = URL_SAFE.decode(album).ok()?;
    String::from_utf8(decoded).ok()
}

fn get_cover(song_path: &Path) -> Option<Picture> {
    let tag_file = Probe::open(song_path).ok()?.read().ok()?;

    let mut tag = tag_file
    .primary_tag()
    .or_else(|| tag_file.first_tag())?
    .clone();

    if tag.pictures().is_empty() {
        None
    } else {
        Some(tag.remove_picture(0))
    }
}

pub fn load_cache(cache: &mut SancCache) {
    let cache_path = Path::new(&cache.path);

    if !cache_path.exists() {
        std::fs::create_dir_all(cache_path).expect("Can't create cache folder!");
    } else if let Ok(entries) = std::fs::read_dir(cache_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(decoded) = dehash_album(file_name) {
                    let album_name = Ustr::from(&decoded);
                    cache.disk_paths.insert(album_name, path);
                }
            }
        }
    }
}

pub fn load_cover_art(ui: &mut egui::Ui, cache: &mut SancCache, song: &Song) {
    cache.update(ui.ctx());

    let album = song.album;

    // Render texture sized down to 48x48 on screen
    let response = if let Some(texture) = cache.textures.get(&album) {
        ui.add(
            egui::Image::from_texture(texture)
            .fit_to_exact_size(egui::vec2(48.0, 48.0))
        )
    } else {
        // Reserve 48x48 space for layout alignment while loading
        ui.allocate_response(egui::vec2(48.0, 48.0), egui::Sense::hover())
    };

    let is_visible = response.rect.intersects(ui.clip_rect());

    if is_visible && !cache.textures.contains_key(&album) && !cache.loading_covers.contains(&album) {
        cache.loading_covers.insert(album);

        let album_key = album;
        let song_path = song.path.clone();
        let cache_dir = PathBuf::from(&cache.path);
        let disk_path = cache.disk_paths.get(&album).cloned();
        let tx = cache.tx.clone();

        cache.pool.spawn(move || {
            let target_path = disk_path.unwrap_or_else(|| {
                cache_dir.join(format!("{}.jpg", hash_album(album_key.as_str())))
            });

            if target_path.exists() {
                if let Ok(reader) = ImageReader::open(&target_path) {
                    if let Ok(image_data) = reader.decode() {
                        let rgba = image_data.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());

                        let _ = tx.send((album_key, color_image, target_path));
                        return;
                    }
                }
            }

            if let Some(picture) = get_cover(&song_path) {
                if let Ok(reader) = ImageReader::new(Cursor::new(picture.data())).with_guessed_format() {
                    if let Ok(image_data) = reader.decode() {
                        // 256x256 thumbnail saved to disk for MPRIS
                        let resized = image_data.thumbnail(256, 256);

                        let _ = resized.save(&target_path);

                        let rgba = resized.to_rgba8();
                        let size = [rgba.width() as usize, rgba.height() as usize];
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba.into_raw());

                        let _ = tx.send((album_key, color_image, target_path));
                    }
                }
            }
        });
    }
}
