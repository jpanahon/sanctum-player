use crate::Sanctum;
use crate::format_timestamp;
use crate::load_cover_art;

pub fn playlist(ui: &mut egui::Ui, sanc: &mut Sanctum) {
    ui.centered_and_justified(|ui| {
        egui::Grid::new("song_list")
            .striped(true)
            .min_row_height(48.)
            .show(ui, |ui| {
                for list_index in 0..sanc.songs.len() {
                    let song = &sanc.songs[list_index];

                    ui.label(
                        egui::RichText::new(format!("{:02}", list_index + 1))
                            .font(egui::FontId::proportional(18.0)),
                    );

                    load_cover_art(ui, &mut sanc.covers, &mut sanc.loading_covers, song);

                    let song_title = egui::Button::new(
                        egui::RichText::new(format!("{}", song.title))
                            .font(egui::FontId::proportional(18.0)),
                    )
                    .frame(false);

                    let song_title = ui.add(song_title);

                    if song_title.clicked() {
                        sanc.player.set_index(list_index);
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
