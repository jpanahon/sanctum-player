use crate::Sanctum;
use crate::load_cover_art;
use fuzzy_matcher::FuzzyMatcher;

fn search_query(sanc: &Sanctum) -> Vec<(usize, i64)> {
    let query = sanc.search.query.trim();

    if query.is_empty() || query.len() < 2 {
        return Vec::new();
    }

    let matcher = &sanc.search.matcher;
    let mut results = Vec::new();

    for (index, song) in sanc.songs.iter().enumerate() {
        if let Some(score) = matcher.fuzzy_match(&song.search_key, query) {
            results.push((index, score));
        }
    }

    results.sort_by(|a, b| b.1.cmp(&a.1));

    if let Some((_, best)) = results.first() {
        let cutoff = best / 2;
        results.retain(|(_, score)| *score >= cutoff);
    }

    results.truncate(10);

    results
}

pub fn search_bar(ui: &mut egui::Ui, sanc: &mut Sanctum) {
    let search_button =
        egui::Button::new(egui::RichText::new("🔎").font(egui::FontId::proportional(18.)))
            .frame(false);
    if ui.add(search_button).clicked() {
        if !sanc.search.results.is_empty() {
            sanc.search.query.clear();
            sanc.search.results.clear();
        }

        sanc.search.modal = true;
    }

    if sanc.search.modal {
        let modal = egui::Modal::new(egui::Id::new("Search Bar")).show(ui.ctx(), |ui| {
            let search = ui.add_sized(
                [600., 48.],
                egui::TextEdit::singleline(&mut sanc.search.query)
                    .font(egui::FontId::proportional(48.)),
            );

            if search.changed() {
                sanc.search.results = search_query(sanc);
            }

            egui::ScrollArea::vertical()
                .min_scrolled_height(365.)
                .show(ui, |ui| {
                    if !sanc.search.results.is_empty() {
                        for (index, _) in sanc.search.results.iter().take(50) {
                            let song = &sanc.songs[*index];
                            ui.horizontal_wrapped(|ui| {
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
                                    load_cover_art(ui.ctx(), &mut sanc.covers, &song);
                                    sanc.loading_covers.remove(&song.album);
                                }

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
            sanc.search.modal = false;
        }
    }
}
