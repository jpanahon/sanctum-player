use eframe::egui;

use lofty::prelude::*;
use lofty::probe::Probe;

use fuzzy_matcher::skim::SkimMatcherV2;
use std::collections::HashMap;
use std::collections::HashSet;
mod config;
use config::Config;

pub mod ui;

pub mod player;
use player::Player;
use player::Playlist;
use player::Song;

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
        };

        songs.push(song);
    }

    songs
}

fn load_cover_art(
    ctx: &eframe::egui::Context,
    covers: &mut HashMap<String, egui::TextureHandle>,
    song: &Song,
) {
    covers.entry(song.album.clone()).or_insert_with(|| {
        let image = song.cover.data();
        let image_data = image::load_from_memory(image)
            .expect(format!("Can't load album art: {}", song.path).as_str());

        let cover_data = image_data.resize_exact(48, 48, image::imageops::Nearest);

        let image_size = [cover_data.width() as _, cover_data.height() as _];
        let image_buffer = cover_data.to_rgba8();
        let pixels = image_buffer.as_flat_samples();

        let color_image = egui::ColorImage::from_rgba_unmultiplied(image_size, pixels.as_slice());

        ctx.load_texture(
            song.album.clone(),
            egui::ImageData::from(color_image),
            egui::TextureOptions::default(),
        )
    });
}

fn format_timestamp(timestamp: u64) -> String {
    let minutes = timestamp / 60;
    let seconds = timestamp % 60;

    format!("{:02}:{:02}", minutes, seconds)
}

pub struct Search {
    modal: bool,
    matcher: SkimMatcherV2,
    results: Vec<(usize, i64)>,
    query: String,
}

pub struct Sanctum {
    player: Player,
    volume: u32,
    config: Config,
    playlists: Vec<Playlist>,
    songs: Vec<Song>,
    covers: HashMap<String, egui::TextureHandle>,
    loading_covers: HashSet<String>,
    search: Search,
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

        let covers: HashMap<String, egui::TextureHandle> = HashMap::new();
        let loading_covers: HashSet<String> = HashSet::new();

        let mut songs = load_songs(current_playlist.path.clone());
        songs.sort_unstable_by_key(|item| item.artist.clone());

        let matcher = SkimMatcherV2::default().ignore_case();
        let search = Search {
            modal: false,
            query: String::new(),
            matcher: matcher,
            results: Vec::new(),
        };

        Self {
            config: config,
            player: player,
            volume: volume,
            playlists: playlists,
            songs: songs,
            covers: covers,
            loading_covers: loading_covers,
            search: search,
        }
    }
}

impl eframe::App for Sanctum {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        ctx.input(|i| {
            self.player
                .handle_keybinds(i, &mut self.volume, &mut self.config, &self.songs);
        });

        let play_state;
        let play_symbols = ["▶", "⏸"];

        if self.player.idle() {
            play_state = play_symbols[0];
        } else {
            play_state = play_symbols[1];
        }

        self.player.process(&self.songs);

        egui::TopBottomPanel::bottom("play_bar").show(ctx, |ui| {
            ui::playbar::playbar(ctx, ui, &play_state, self);
        }); // egui::TopBottomPanel

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
                ui::playlist::playlist(ctx, ui, self);
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
