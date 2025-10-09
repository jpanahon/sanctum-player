use eframe::egui;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    let stream_handle = rodio::OutputStreamBuilder::open_default_stream()?;
    let sink = rodio::Sink::connect_new(stream_handle.mixer());

    let play_symbols = ["▶", "⏸"];
    let mut play_state = play_symbols[0];

    eframe::run_simple_native("Sanctum Player", options, move |ctx, _frame| {
        egui::TopBottomPanel::bottom("play_bar").show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                let play_button = egui::Button::new(play_state)
                    .min_size(egui::Vec2::new(40.0, 40.0))
                    .corner_radius(100)
                    .frame(true);

                if ui.add(play_button).clicked() {
                    if sink.is_paused() {
                        sink.play();
                        play_state = play_symbols[1];
                    } else {
                        sink.pause();
                        play_state = play_symbols[0];
                    }
                }
            })
        }); // egui::TopBottomPanel

        egui::SidePanel::left("playlists").show(ctx, |ui| {
            ui.heading("Playlists");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for entry in
                    std::fs::read_dir("/home/morose/Music/My Playlist/").expect("Music folder found!")
                {
                    let entry = entry.expect("Entries found!");
                    let path = entry.path();
                    let tag = audiotags::Tag::new()
                        .read_from_path(&path)
                        .expect("No tag found!");

                    if let Some(title) = tag.title() {
                        if ui.label(format!("{}", title)).clicked() {
                            let song_file = std::fs::File::open(path).unwrap();
                            let decoder = rodio::Decoder::try_from(song_file).unwrap();
                            sink.append(decoder);
                            play_state = play_symbols[1];
                        }
                    }
                }
            });
        });
    }) // run_simple_native
}
