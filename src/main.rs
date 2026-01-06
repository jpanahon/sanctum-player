use eframe::egui;

use lofty::prelude::*;
use lofty::probe::Probe;

use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc;
use std::time::SystemTime;

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
pub mod config;
use config::Config;

pub mod mpris;
use mpris::MprisHandler;
use mpris::MprisState;

use mpris_server::Server;

pub mod ui;

pub mod player;
use player::Player;
use player::Playlist;
use player::Song;

pub mod search;
use search::Search;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    eframe::run_native(
        "Sanctum Player",
        options,
        Box::new(|cc| Ok(Box::new(Sanctum::new(cc)))),
    )
}

fn hash_album(album: String) -> String {
    URL_SAFE.encode(album).to_string()
}

fn dehash_album(album: String) -> String {
    let decoded = URL_SAFE
        .decode(&album)
        .expect(format!("Can't decode string: {}", album.clone()).as_str());
    String::from_utf8(decoded).expect("Can't decode!")
}

fn format_date(created: SystemTime) -> String {
    let age = match SystemTime::now().duration_since(created) {
        Ok(d) => d,
        Err(_) => {
            let date_time: DateTime<Local> = created.into();
            return date_time.format("%d/%m/%y").to_string();
        }
    };

    let secs = age.as_secs();

    const MIN: u64 = 60;
    const HOUR: u64 = 60 * MIN;
    const DAY: u64 = 24 * HOUR;
    const WEEK: u64 = 7 * DAY;

    if secs < MIN {
        "just now".to_string()
    } else if secs < HOUR {
        format!("{} min ago", secs / MIN)
    } else if secs < DAY {
        format!("{} hrs ago", secs / HOUR)
    } else if secs < WEEK {
        format!("{} days ago", secs / DAY)
    } else {
        let date_time: DateTime<Local> = created.into();
        date_time.format("%d/%m/%y").to_string()
    }
}

fn load_songs(main_dir: String) -> Vec<Song> {
    let mut songs: Vec<Song> = Vec::new();

    for entry in std::fs::read_dir(main_dir).expect("Music folder not found!") {
        let entry = entry.expect("Entries found!");
        let path = entry.path();

        let song_path = path.display().to_string();

        let tag_file = Probe::open(path.as_path())
            .expect(format!("Can't find file: {}", song_path).as_str())
            .read()
            .expect(format!("Can't read file: {}", song_path).as_str());

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
            cover: (tag.pictures())[0].clone(),
            path: song_path,
            duration: seconds,
            search_key: format!(
                "{} {} {}",
                tag.title().as_deref().unwrap_or("Unknown").to_lowercase(),
                tag.artist().as_deref().unwrap_or("Unknown").to_lowercase(),
                tag.album().as_deref().unwrap_or("Unknown").to_lowercase(),
            ),
            created: created_time,
            created_date: created_date,
        };

        songs.push(song);
    }

    songs
}

fn load_cover_art(ui: &mut egui::Ui, cache: &mut SancCache, song: &Song) {
    let cache_path = std::path::Path::new(&cache.path);

    if !cache_path.exists() {
        std::fs::create_dir(cache_path).expect("Can't create cache folder!");
    } else {
        for entry in std::fs::read_dir(cache_path).expect("Cache folder not found!") {
            let entry = entry.expect("Entry not found!");
            let path = entry.path();

            let file_name = path.file_stem().expect("Can't get file name!");
            cache.covers.insert(
                dehash_album(file_name.display().to_string()),
                entry.path().display().to_string(),
            );
        }
    }

    let response = if let Some(cover_art) = cache.covers.get(&song.album) {
        ui.add_sized([48., 48.], egui::Image::new(format!("file://{cover_art}")))
    } else {
        ui.allocate_response(egui::vec2(48., 48.), egui::Sense::hover())
    };

    let is_visible = response.rect.intersects(ui.clip_rect());

    if is_visible
        && !cache.covers.contains_key(&song.album)
        && !cache.loading_covers.contains(&song.album)
    {
        cache.loading_covers.insert(song.album.clone());

        cache.covers.entry(song.album.clone()).or_insert_with(|| {
            let image = song.cover.data();
            let image_data = image::load_from_memory(image)
                .expect(format!("Can't load album art: {}", song.path).as_str());

            let cover_data = image_data.resize_exact(256, 256, image::imageops::Nearest);

            let image_path = format!("{}/{}.jpg", cache.path, hash_album(song.album.clone()));

            cover_data
                .save(image_path.clone())
                .expect(format!("Can't save image: {}", song.album.clone()).as_str());

            image_path
        });

        cache.loading_covers.remove(&song.album);
    }
}

fn format_timestamp(timestamp: u64) -> String {
    let minutes = timestamp / 60;
    let seconds = timestamp % 60;

    format!("{:02}:{:02}", minutes, seconds)
}
pub struct Sanctum {
    player: Player,
    volume: u32,
    config: Config,
    playlists: Vec<Playlist>,
    songs: Vec<Song>,
    cache: SancCache,
    search: Search,
    mpris: Server<MprisHandler>,
    receiver: mpsc::Receiver<MprisState>,
}

pub struct SancCache {
    covers: HashMap<String, String>,
    loading_covers: HashSet<String>,
    path: String,
}

impl Sanctum {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let config_file = std::fs::read_to_string("config.json").expect("Can't find config file!");
        let config: Config = serde_json::from_str(config_file.as_str()).expect("Can't parse JSON!");
        let playlists = config.get_playlists().clone();

        let current_playlist = (config.get_playlists())[config.current_playlist()].clone();

        let mut player: Player = Player {
            current_index: config.get_last_track(),
            ..Default::default()
        };

        player.sink.pause();

        let volume = config.get_volume();
        player.volume(volume);

        let covers: HashMap<String, String> = HashMap::new();
        let loading_covers: HashSet<String> = HashSet::new();
        let cache_path = config.cache_path.clone();

        let sanc_cache = SancCache {
            covers: covers,
            loading_covers: loading_covers,
            path: cache_path,
        };

        let mut songs = load_songs(current_playlist.path.clone());
        songs.sort_unstable_by_key(|item| std::cmp::Reverse(item.created.clone()));

        let (tx, rx) = mpsc::channel::<MprisState>();

        let mpris_handler = MprisHandler { tx: tx };

        let mpris = futures::executor::block_on(Server::new("Sanctum.Player", mpris_handler))
            .expect("Can't make server!");

        Self {
            config: config,
            player: player,
            volume: volume,
            playlists: playlists,
            songs: songs,
            cache: sanc_cache,
            search: Search::default(),
            mpris: mpris,
            receiver: rx,
        }
    }
}

impl eframe::App for Sanctum {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        while let Ok(state) = self.receiver.try_recv() {
            self.player.handle_mpris(state, &self.songs, &self.mpris);
        }

        if !self.search.modal {
            ctx.input(|i| {
                for event in &i.events {
                    self.player.handle_keybinds(
                        i,
                        &self.mpris,
                        event,
                        &mut self.volume,
                        &mut self.config,
                        &self.songs,
                    );

                    if i.modifiers.ctrl {
                        if let egui::Event::Key {
                            key: egui::Key::F,
                            pressed: true,
                            repeat: false,
                            ..
                        } = event
                        {
                            self.search.open_modal();
                        }
                    }
                }
            });
        }

        let play_state;
        let play_symbols = ["▶", "⏸"];

        if self.player.idle() {
            play_state = play_symbols[0];
        } else {
            play_state = play_symbols[1];
        }

        self.player.process(&self.songs, &self.mpris, &self.cache);

        egui::TopBottomPanel::bottom("play_bar").show(ctx, |ui| {
            ui::playbar::playbar(ui, &play_state, self);
        });

        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui::sidebar::sidebar(ui, self);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui::searchbar::search_bar(ui, self);
            });

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui::playlist::playlist(ui, self);
            });
        });

        let close = ctx.input(|i| i.viewport().close_requested());

        if close {
            self.config.set_track(self.player.current_index);
            let new_config =
                serde_json::to_string_pretty(&self.config).expect("Can't export config!");
            std::fs::write("config.json", new_config).expect("Can't update config!");
        }
    }
}
