use eframe::egui;

use lofty::prelude::*;
use lofty::probe::Probe;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

mod config;
use config::Config;

pub mod player;
use player::Player;
use player::Song;

fn load_songs(main_dir: String, covers: &mut HashMap<String, Arc<egui::ColorImage>>) -> Vec<Song> {
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
            path: song_path,
            duration: seconds,
        };

        covers.entry(song.album.clone()).or_insert_with(|| {
            let image = (tag.pictures())[0].clone();
            let image_data = image::load_from_memory(image.data())
                .expect(format!("Can't load album art: {}", song.path).as_str());

            let cover_data = image_data.resize_exact(48, 48, image::imageops::Nearest);

            let image_size = [cover_data.width() as _, cover_data.height() as _];
            let image_buffer = cover_data.to_rgba8();
            let pixels = image_buffer.as_flat_samples();

            Arc::new(egui::ColorImage::from_rgba_unmultiplied(
                image_size,
                pixels.as_slice(),
            ))
        });

        songs.push(song);
    }

    songs
}

fn format_timestamp(timestamp: u64) -> String {
    let minutes = timestamp / 60;
    let seconds = timestamp % 60;

    format!("{:02}:{:02}", minutes, seconds)
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;

    let config_file = std::fs::read_to_string("config.json").expect("Can't find config file!");
    let mut config: Config = serde_json::from_str(config_file.as_str()).expect("Can't parse JSON!");

    let play_symbols = ["▶", "⏸"];

    let mut player: Player = Player {
        sink: rodio::Sink::connect_new(stream_handle.mixer()),
        current_index: config.get_last_track(),
        prev_index: config.get_last_track(),
        repeat: false,
        shuffle: false,
        track_pos: 0,
        volume: config.get_volume(),
        skip: false,
    };

    let mut player_vol: u32 = config.get_volume();

    player.sink.set_volume(player_vol as f32 / 100.);

    let current_playlist = &(config.get_playlists())[config.current_playlist()];

    let mut covers: HashMap<String, Arc<egui::ColorImage>> = HashMap::new();

    let mut songs = load_songs(current_playlist.path.clone(), &mut covers);

    songs.sort_unstable_by_key(|item| item.artist.clone());

    let mut song_queue: Vec<String> = Vec::new();

    let _ = eframe::run_simple_native("Sanctum Player", options, move |ctx, _frame| {
        egui_extras::install_image_loaders(ctx);
        ctx.request_repaint();
        let play_state;

        if player.idle() {
            play_state = play_symbols[0];
        } else {
            play_state = play_symbols[1];
        }

        player.process(&songs);

        egui::TopBottomPanel::bottom("play_bar").show(ctx, |ui| {
            let prev_key = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowLeft);
            let skip_key =
                egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowRight);

            let vol_up = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowUp);
            let vol_down = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::ArrowDown);

            let shufl_key = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S);

            if ui.input(|i| i.key_pressed(egui::Key::Space)) {
                player.playback();
            }

            if ui.input_mut(|i| i.consume_shortcut(&prev_key)) {
                player.previous(&songs);
            }

            if ui.input_mut(|i| i.consume_shortcut(&skip_key)) {
                player.skip(&songs);
            }

            if ui.input_mut(|i| i.consume_shortcut(&vol_up)) {
                player_vol += 1;
                player.volume(player_vol);
                config.set_volume(player_vol);
            }

            if ui.input_mut(|i| i.consume_shortcut(&vol_down)) {
                player_vol -= 1;
                player.volume(player_vol);
                config.set_volume(player_vol);
            }

            if ui.input_mut(|i| i.consume_shortcut(&shufl_key)) {
                player.shuffle();
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

            let shufl_color = if player.is_shuffled() {
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
                    if !player.done() {
                        let current_track = &songs[player.current_index];

                        if let Some(cover_art) = covers.get(&current_track.album) {
                            let album_art = ctx.load_texture(
                                "Album Art",
                                egui::ImageData::from(cover_art.clone()),
                                egui::TextureOptions::default(),
                            );

                            ui.add(egui::Image::new(&album_art));
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
                            player.previous(&songs);
                        }

                        if ui.add(play_button).clicked() {
                            player.playback();
                        }

                        if ui.add(skip_button).clicked() {
                            player.skip(&songs);
                        }

                        if ui.add(shufl_button).clicked() {
                            player.shuffle();
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.spacing_mut().slider_width = ui.max_rect().width() - 50.;
                        ui.style_mut().visuals.slider_trailing_fill = true;

                        let total_duration = songs[player.current_index].duration;
                        let time_slider =
                            egui::Slider::new(&mut player.track_pos, 0..=total_duration)
                                .logarithmic(false)
                                .show_value(false)
                                .clamping(egui::SliderClamping::Always)
                                .trailing_fill(true);

                        ui.add(time_slider);
                        ui.label(format!(
                            "{} / {}",
                            format_timestamp(player.track_pos),
                            format_timestamp(total_duration)
                        ));
                    });
                });

                columns[2].horizontal_centered(|ui| {
                    ui.add_space(ui.max_rect().width() - 200.);

                    ui.label("🔈");
                    ui.style_mut().visuals.slider_trailing_fill = true;

                    ui.add(egui::Slider::new(&mut player_vol, 0..=100));

                    player.volume(player_vol);
                    config.set_volume(player_vol);
                });
            });
        }); // egui::TopBottomPanel

        egui::SidePanel::left("playlists").show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(egui::RichText::new("Playlists").font(egui::FontId::proportional(24.0)));

                for index in 0..config.get_playlists().len() {
                    let playlist_name = (config.get_playlists())[index].name.clone();
                    if ui
                        .label(
                            egui::RichText::new(playlist_name)
                                .font(egui::FontId::proportional(18.0)),
                        )
                        .clicked()
                    {
                        config.set_playlist(index);
                        config.set_track(0);
                    }
                }

                for song in &song_queue {
                    ui.label(song);
                    ui.separator();
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
                            for list_index in 0..songs.len() {
                                let song = &songs[list_index];

                                ui.label(
                                    egui::RichText::new(format!("{:02}", list_index + 1))
                                        .font(egui::FontId::proportional(18.0)),
                                );

                                if let Some(cover_art) = covers.get(&song.album) {
                                    let album_art = ctx.load_texture(
                                        "Album Art",
                                        egui::ImageData::from(cover_art.clone()),
                                        egui::TextureOptions::default(),
                                    );

                                    ui.add(egui::Image::new(&album_art));
                                }

                                let song_title = egui::Button::new(
                                    egui::RichText::new(format!("{}", song.title))
                                        .font(egui::FontId::proportional(18.0)),
                                )
                                .frame(false);

                                let song_title = ui.add(song_title);

                                if song_title.clicked() {
                                    if song_queue.len() > 0 {
                                        song_queue.clear();
                                    }

                                    player.set_index(list_index);
                                    config.set_track(list_index);

                                    if !player.idle() {
                                        if player.sink.is_paused() {
                                            player.sink.play();
                                        }

                                        player.sink.skip_one();
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
            let new_config = serde_json::to_string_pretty(&config).expect("Can't export config!");
            std::fs::write("config.json", new_config).expect("Can't update config!");
        }
    }); // run_simple_native

    Ok(())
}
