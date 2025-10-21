use eframe::egui;
use std::error::Error;

struct Song<R: std::io::Read + std::io::Seek> {
    title: String,
    artist: String,
    album: String,
    decoder: Option<rodio::Decoder<R>>,
}

struct Playlist {
    name: String,
    songs: Vec<Song<std::io::BufReader<std::fs::File>>>,
}

fn load_songs(main_dir: String) -> Vec<Song<std::io::BufReader<std::fs::File>>> {
    let mut songs: Vec<Song<std::io::BufReader<std::fs::File>>> = Vec::new();
    for entry in std::fs::read_dir(main_dir).expect("Music folder found!") {
        let entry = entry.expect("Entries found!");
        let path = entry.path();
        let tag = audiotags::Tag::new()
            .read_from_path(&path)
            .expect("No tag found!");

        let song_file = std::fs::File::open(path).expect("Can't open file");
        let decoder = rodio::Decoder::try_from(song_file).expect("Can't create decoder");

        if let Some(title) = tag.title()
            && let Some(artist) = tag.artist()
            && let Some(album) = tag.album_title()
        {
            let song = Song {
                title: title.to_string(),
                artist: artist.to_string(),
                album: album.to_string(),
                decoder: Some(decoder),
            };

            songs.push(song);
        }
    }

    songs
}

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(stream_handle.mixer());

    let play_symbols = ["▶", "⏸"];
    let mut play_state = play_symbols[0];

    let mut playlist: Playlist = Playlist {
        name: "My Playlist".to_string(),
        songs: load_songs("/home/morose/Music/My Playlist/".to_string()),
    };

    let mut song_queue: Vec<String> = Vec::new();

    let _ = eframe::run_simple_native("Sanctum Player", options, move |ctx, _frame| {
        egui::TopBottomPanel::bottom("play_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let play_button = egui::Button::new(play_state)
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .corner_radius(100)
                    .frame(true);

                let prev_button = egui::Button::new("⏮︎")
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .frame(false);

                let skip_button = egui::Button::new("⏭︎")
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .frame(false);

                let shufl_button = egui::Button::new("🔀")
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .frame(false);

                let repeat_button = egui::Button::new("🔁")
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .frame(false);

                if ui.add(prev_button).clicked() {
                    println!("Previously");
                }

                if ui.add(play_button).clicked() {
                    if sink.is_paused() {
                        sink.play();
                        play_state = play_symbols[1];
                    } else {
                        sink.pause();
                        play_state = play_symbols[0];
                    }
                }

                if ui.add(skip_button).clicked() {
                    sink.skip_one();
                }

                if ui.add(shufl_button).clicked() {
                    println!("Shuffle");
                }

                if ui.add(repeat_button).clicked() {
                    println!("Repeat");
                }
            })
        }); // egui::TopBottomPanel

        egui::SidePanel::left("playlists").show(ctx, |ui| {
            ui.heading("Playlists");
            ui.label(&playlist.name);

            ui.heading("Song Queue");
            for song in &song_queue {
                ui.label(song);
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    for song in &mut playlist.songs {
                        let song_entry = egui::Label::new(
                            egui::RichText::new(format!(
                                "{}\n{}\n{}\n",
                                song.title, song.artist, song.album
                            ))
                            .font(egui::FontId::proportional(16.0)),
                        );

                        if ui.add(song_entry).clicked() {
                            if let Some(decoder) = song.decoder.take() {
                                if !sink.is_paused() {
                                    sink.append(decoder);
                                    song_queue.push(format!("{} - {}", song.title, song.artist));
                                } else {
                                    sink.append(decoder);
                                    play_state = play_symbols[1];
                                }
                            }
                        }

                        ui.separator();
                    }
                });
            });
        });
    }); // run_simple_native

    Ok(())
}
