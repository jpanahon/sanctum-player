use crate::Sanctum;
use crate::load_cover_art;

pub fn search_bar(ui: &mut egui::Ui, sanc: &mut Sanctum) {
    let search_button =
        egui::Button::new(egui::RichText::new("🔎").font(egui::FontId::proportional(18.)))
            .frame(false);

    if ui.add(search_button).clicked() {
        sanc.search.open_modal();
    }

    if sanc.search.modal {
        let modal = egui::Modal::new(egui::Id::new("Search Bar")).show(ui.ctx(), |ui| {
            let search = ui.add_sized(
                [600., 48.],
                egui::TextEdit::singleline(&mut sanc.search.query)
                    .font(egui::FontId::proportional(48.)),
            );

            if search.changed() {
                sanc.search.handle_query(&sanc.songs);
            }

            egui::ScrollArea::vertical()
                .min_scrolled_height(365.)
                .show(ui, |ui| {
                    if !sanc.search.results.is_empty() {
                        for (index, _) in sanc.search.results.iter().take(50) {
                            let song = &sanc.songs[*index];
                            ui.horizontal_wrapped(|ui| {
                                load_cover_art(
                                    ui,
                                    &mut sanc.covers,
                                    &mut sanc.loading_covers,
                                    &song,
                                );
                                let song_title = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(format!(
                                            "{}\n{}\n{}",
                                            song.title, song.artist, song.album
                                        ))
                                        .font(egui::FontId::proportional(18.)),
                                    )
                                    .frame(false),
                                );

                                if song_title.clicked() {
                                    sanc.player.set_index(*index);
                                    sanc.search.modal = false;
                                }
                            });
                            ui.separator();
                        }
                    }
                });
        });

        if modal.should_close() {
            sanc.search.close_modal();
        }
    }
}
