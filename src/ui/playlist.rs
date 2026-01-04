use crate::Sanctum;
use crate::format_timestamp;
use crate::load_cover_art;

pub fn playlist(ui: &mut egui::Ui, sanc: &mut Sanctum) {
    let current_index = sanc.player.current_index;
    ui.centered_and_justified(|ui| {
        egui::Grid::new("song_list")
            .min_row_height(48.)
            .striped(true)
            .with_row_color(move |index, _style| {
                if index == current_index {
                    Some(egui::Color32::from_rgb(1, 92, 128))
                } else if index % 2 == 0 {
                    Some(egui::Color32::from_rgb(32, 32, 32))
                } else {
                    None
                }
            })
            .show(ui, |ui| {
                for list_index in 0..sanc.songs.len() {
                    let song = &sanc.songs[list_index];

                    ui.label(
                        egui::RichText::new(format!("{:02}", list_index + 1))
                            .font(egui::FontId::proportional(18.0)),
                    );

                    load_cover_art(ui, &mut sanc.cache, song);

                    let song_title = egui::Button::new(
                        egui::RichText::new(format!("{}", song.title))
                            .font(egui::FontId::proportional(18.0)),
                    )
                    .frame(false);

                    let song_title = ui.add(song_title);

                    if song_title.clicked() {
                        sanc.player.set_index(list_index, &sanc.mpris);
                    }

                    song_title.context_menu(|ui| {
                        if ui.button("Add to queue").clicked() {
                            sanc.player.add_queue(list_index);
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
}
