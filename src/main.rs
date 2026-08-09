use eframe::egui;

use std::sync::Arc;
use std::sync::Mutex;

pub mod config;
use config::Config;

pub mod cache;
use cache::{SancCache, load_cache};

pub mod mpris;
use mpris::MprisHandler;
use mpris_server::Server;

pub mod ui;
pub mod utils;

pub mod player;
use player::Player;
use player::PlayerState;

pub mod playlist;
use playlist::{Playlist, sort_songs};

pub mod search;
use search::Search;

pub mod songs;
use songs::Song;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

pub struct Sanctum {
    player: Player,
    volume: u32,
    config: Config,
    current_playlist: Playlist,
    playlists: Vec<Playlist>,
    songs: Vec<Song>,
    song_view: Vec<usize>,
    cache: SancCache,
    search: Search,
    mpris: Server<MprisHandler>,
}

impl Sanctum {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let config_file = std::fs::read_to_string("config.json").expect("Can't find config file!");
        let config: Config = serde_json::from_str(config_file.as_str()).expect("Can't parse JSON!");
        let playlists = config.get_playlists().clone();

        let current_playlist = (config.get_playlists())[config.current_playlist()].clone();

        let cache_path = config.cache_path.clone();

        let mut sanc_cache = SancCache::new(cache_path);

        load_cache(&mut sanc_cache);

        let songs = songs::load_songs(current_playlist.path.clone());

        let mut song_view: Vec<usize> = (0..songs.len()).collect();
        sort_songs(current_playlist.clone(), &mut song_view, &songs);

        let shared_state = Arc::new(Mutex::new(PlayerState::default()));
        let mpris_state = Arc::clone(&shared_state);
        let player_state = Arc::clone(&shared_state);

        let mpris_handler = MprisHandler { state: mpris_state };

        let mut player: Player = Player::new(config.get_last_track(), player_state);

        let volume = config.get_volume();
        player.volume(volume);

        player.sink.pause();

        let mpris = futures::executor::block_on(Server::new("Sanctum.Player", mpris_handler))
            .expect("Can't make server!");

        Self {
            config,
            player,
            volume,
            current_playlist,
            playlists,
            songs,
            song_view,
            cache: sanc_cache,
            search: Search::default(),
            mpris,
        }
    }
}

impl eframe::App for Sanctum {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        while let Ok((album, image_path)) = self.cache.rx.try_recv() {
            self.cache.covers.insert(album.clone(), image_path);
            self.cache.loading_covers.remove(&album);
        }
        if !self.search.modal {
            ctx.input(|i| {
                for event in &i.events {
                    self.player.handle_keybinds(
                        i,
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

        self.player.process(&self.songs);
        self.player
            .update_state(&self.mpris, &self.cache, &self.songs);

        egui::TopBottomPanel::bottom("play_bar").show(ctx, |ui| {
            ui::playbar::playbar(ui, self.player.is_playing(), self);
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
                ui::tracklist::playlist(ui, self);
            });
        });

        let close = ctx.input(|i| i.viewport().close_requested());

        if close {
            self.config.set_track(self.player.current_index);
            self.config.update_playlist(self.current_playlist.clone());
            let new_config =
                serde_json::to_string_pretty(&self.config).expect("Can't export config!");
            std::fs::write("config.json", new_config).expect("Can't update config!");
        }
    }
}
