use eframe::egui;

use lofty::prelude::*;
use lofty::probe::Probe;

use std::collections::HashMap;
use std::collections::HashSet;

mod config;
use config::Config;

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

        let tag = match tag_file.primary_tag() {
            Some(primary_tag) => primary_tag,
            None => tag_file
                .first_tag()
                .expect(format!("No tags found!: {}", song_path).as_str()),
        };

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

struct Sanctum {
    player: Player,
    volume: u32,
    config: Config,
    playlists: Vec<Playlist>,
    songs: Vec<Song>,
    covers: HashMap<String, egui::TextureHandle>,
    loading_covers: HashSet<String>,
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
            prev_index: config.get_last_track(),
            ..Default::default()
        };

        let volume = config.get_volume();
        player.volume(volume);

        let covers: HashMap<String, egui::TextureHandle> = HashMap::new();
        let loading_covers: HashSet<String> = HashSet::new();

        let mut songs = load_songs(current_playlist.path.clone());
        songs.sort_unstable_by_key(|item| item.artist.clone());

        Self {
            config: config,
            player: player,
            volume: volume,
            playlists: playlists,
            songs: songs,
            covers: covers,
            loading_covers: loading_covers,
        }
    }
}

impl eframe::App for Sanctum {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        let play_state;
        let play_symbols = ["▶", "⏸"];

        if self.player.idle() {
            play_state = play_symbols[0];
        } else {
            play_state = play_symbols[1];
        }

        self.player.process(&self.songs);

        egui::TopBottomPanel::bottom("play_bar").show(ctx, |ui| {
            let prev_key = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowLeft);
            let skip_key =
                egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowRight);

            let vol_up = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowUp);
            let vol_down = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowDown);

            let shufl_key = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S);

            if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                self.player.playback();
            }

            if ui.input_mut(|i| i.consume_shortcut(&prev_key)) {
                self.player.previous(&self.songs);
            }

            if ui.input_mut(|i| i.consume_shortcut(&skip_key)) {
                self.player.skip(&self.songs);
            }

            if ui.input_mut(|i| i.consume_shortcut(&vol_up)) {
                self.volume += 1;
                self.player.volume(self.volume);
                self.config.set_volume(self.volume);
            }

            if ui.input_mut(|i| i.consume_shortcut(&vol_down)) {
                self.volume -= 1;
                self.player.volume(self.volume);
                self.config.set_volume(self.volume);
            }

            if ui.input_mut(|i| i.consume_shortcut(&shufl_key)) {
                self.player.shuffle();
            }

            let play_button = egui::Button::new(
                egui::RichText::new(play_state).font(egui::FontId::proportional(18.0)),
            )
            .min_size(egui::Vec2::new(40.0, 40.0))
            .corner_radius(100)
            .frame(true);

            let prev_button =
                egui::Button::new(egui::RichText::new("⏪").font(egui::FontId::proportional(18.0)))
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .frame(false);

            let skip_button =
                egui::Button::new(egui::RichText::new("⏩").font(egui::FontId::proportional(18.0)))
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .frame(false);

            let shufl_color = if self.player.is_shuffled() {
                egui::Color32::from_rgb(1, 92, 128)
            } else {
                egui::Color32::from_rgb(180, 180, 180)
            };

            let shufl_button = egui::Button::new(
                egui::RichText::new("🔀")
                    .font(egui::FontId::proportional(18.0))
                    .color(shufl_color),
            )
            .min_size(egui::Vec2::new(40.0, 40.0))
            .frame(false);

            let repeat_button =
                egui::Button::new(egui::RichText::new("🔁").font(egui::FontId::proportional(18.0)))
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .frame(false);

            ui.columns(3, |columns| {
                columns[0].horizontal_centered(|ui| {
                    if !self.player.done() {
                        let current_track = &self.songs[self.player.current_index];

                        if let Some(cover_art) = self.covers.get(&current_track.album) {
                            ui.add(egui::Image::new(cover_art));
                        } else {
                            ui.allocate_response(egui::vec2(48., 48.), egui::Sense::hover());
                        }

                        if !self.covers.contains_key(&current_track.album)
                            && !self.loading_covers.contains(&current_track.album)
                        {
                            self.loading_covers.insert(current_track.album.clone());
                            load_cover_art(ctx, &mut self.covers, current_track);
                            self.loading_covers.remove(&current_track.album);
                        }

                        ui.heading(format!("{}\n{}", current_track.title, current_track.artist));
                    } else {
                        ui.heading("No song playing!");
                    }
                });

                columns[1].vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(ui.max_rect().width() / 3.);
                        if ui.add(repeat_button).clicked() {
                            println!("Repeat");
                        }

                        if ui.add(prev_button).clicked() {
                            self.player.previous(&self.songs);
                        }

                        if ui.add(play_button).clicked() {
                            self.player.playback();
                        }

                        if ui.add(skip_button).clicked() {
                            self.player.skip(&self.songs);
                        }

                        if ui.add(shufl_button).clicked() {
                            self.player.shuffle();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.spacing_mut().slider_width = ui.max_rect().width() - 50.;
                        ui.style_mut().visuals.slider_trailing_fill = true;

                        let total_duration = self.songs[self.player.current_index].duration;
                        let time_slider =
                            egui::Slider::new(&mut self.player.track_pos, 0..=total_duration)
                                .logarithmic(false)
                                .show_value(false)
                                .clamping(egui::SliderClamping::Always)
                                .trailing_fill(true);

                        ui.add(time_slider);
                        ui.label(format!(
                            "{} / {}",
                            format_timestamp(self.player.track_pos),
                            format_timestamp(total_duration)
                        ));
                    });
                });

                columns[2].horizontal_centered(|ui| {
                    ui.add_space(ui.max_rect().width() - 200.);

                    ui.label("🔈");
                    ui.style_mut().visuals.slider_trailing_fill = true;

                    ui.add(egui::Slider::new(&mut self.volume, 0..=100));

                    self.player.volume(self.volume);
                    self.config.set_volume(self.volume);
                });
            });
        }); // egui::TopBottomPanel

        egui::SidePanel::left("playlists").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(egui::RichText::new("Playlists").font(egui::FontId::proportional(24.0)));

                for index in 0..self.playlists.len() {
                    let playlist_name = self.playlists[index].name.clone();
                    if ui
                        .label(
                            egui::RichText::new(playlist_name)
                                .font(egui::FontId::proportional(18.0)),
                        )
                        .clicked()
                    {
                        self.config.set_playlist(index);
                        self.config.set_track(0);
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    let col_width = ui.max_rect().width() / 2.6;
                    egui::Grid::new("song_list")
                        .striped(true)
                        .min_row_height(48.)
                        .max_col_width(col_width)
                        .show(ui, |ui| {
                            for list_index in 0..self.songs.len() {
                                let song = &self.songs[list_index];

                                ui.label(
                                    egui::RichText::new(format!("{:02}", list_index + 1))
                                        .font(egui::FontId::proportional(18.0)),
                                );

                                let response = if let Some(cover_art) = self.covers.get(&song.album)
                                {
                                    ui.add(egui::Image::new(cover_art))
                                } else {
                                    ui.allocate_response(egui::vec2(48., 48.), egui::Sense::hover())
                                };

                                let is_visible = response.rect.intersects(ui.clip_rect());

                                if is_visible
                                    && !self.covers.contains_key(&song.album)
                                    && !self.loading_covers.contains(&song.album)
                                {
                                    self.loading_covers.insert(song.album.clone());
                                    load_cover_art(ctx, &mut self.covers, &song);
                                    self.loading_covers.remove(&song.album);
                                }

                                let song_title = egui::Button::new(
                                    egui::RichText::new(format!("{}", song.title))
                                        .font(egui::FontId::proportional(18.0)),
                                )
                                .frame(false);

                                let song_title = ui.add(song_title);

                                if song_title.clicked() {
                                    self.player.set_index(list_index);
                                    self.config.set_track(list_index);

                                    if !self.player.idle() {
                                        if self.player.sink.is_paused() {
                                            self.player.sink.play();
                                        }

                                        self.player.sink.skip_one();
                                    }
                                }

                                song_title.context_menu(|ui| {
                                    if ui.button("Add to queue").clicked() {
                                        println!("Test");
                                    }
                                });

                                ui.label(
                                    egui::RichText::new(format!("{}", song.artist))
                                        .font(egui::FontId::proportional(18.0)),
                                );

                                ui.label(
                                    egui::RichText::new(format!("{}", song.album))
                                        .font(egui::FontId::proportional(18.0)),
                                );

                                let timestamp = format_timestamp(song.duration);

                                ui.label(
                                    egui::RichText::new(format!("{}", timestamp))
                                        .font(egui::FontId::proportional(18.0)),
                                );

                                ui.end_row();
                            }
                        });
                });
            });
        });

        let close_request = ctx.input(|input| input.viewport().close_requested());

        if close_request {
            let new_config =
                serde_json::to_string_pretty(&self.config).expect("Can't export config!");
            std::fs::write("config.json", new_config).expect("Can't update config!");
        }
    }
}
