use crate::Sanctum;

pub fn sidebar(ui: &mut egui::Ui, sanc: &mut Sanctum) {
    ui.heading(egui::RichText::new("Playlists").font(egui::FontId::proportional(24.0)));

    for index in 0..sanc.playlists.len() {
        let playlist_name = sanc.playlists[index].name.clone();
        if ui
            .label(
                egui::RichText::new(playlist_name)
                .font(egui::FontId::proportional(18.0)),
            )
            .clicked()
            {
                sanc.config.set_playlist(index);
                sanc.config.set_track(0);
            }
    }
}
