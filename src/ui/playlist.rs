use crate::Sanctum;
use crate::load_cover_art;
use crate::format_timestamp;

pub fn playlist(ctx: &eframe::egui::Context, ui: &mut egui::Ui, sanc: &mut Sanctum) {
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

                let response = if let Some(cover_art) = sanc.covers.get(&song.album)
                {
                    ui.add(egui::Image::new(cover_art))
                } else {
                    ui.allocate_response(egui::vec2(48., 48.), egui::Sense::hover())
                };

                let is_visible = response.rect.intersects(ui.clip_rect());

                if is_visible
                    && !sanc.covers.contains_key(&song.album)
                    && !sanc.loading_covers.contains(&song.album)
                    {
                        sanc.loading_covers.insert(song.album.clone());
                        load_cover_art(ctx, &mut sanc.covers, &song);
                        sanc.loading_covers.remove(&song.album);
                    }

                    let song_title = egui::Button::new(
                        egui::RichText::new(format!("{}", song.title))
                        .font(egui::FontId::proportional(18.0)),
                    )
                    .frame(false);

                    let song_title = ui.add(song_title);

                    if song_title.clicked() {
                        sanc.player.set_index(list_index);
                        sanc.config.set_track(list_index);

                        if !sanc.player.idle() {
                            if sanc.player.sink.is_paused() {
                                sanc.player.sink.play();
                            }

                            sanc.player.sink.skip_one();
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
}
