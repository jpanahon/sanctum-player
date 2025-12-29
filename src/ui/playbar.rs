use crate::Sanctum;
use crate::format_timestamp;
use crate::load_cover_art;

pub fn playbar(ui: &mut egui::Ui, play_state: &str, sanc: &mut Sanctum) {
    let play_button =
        egui::Button::new(egui::RichText::new(play_state).font(egui::FontId::proportional(18.0)))
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

    let shufl_color = if sanc.player.is_shuffled() {
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
            if !sanc.player.done() {
                let current_track = &sanc.songs[sanc.player.current_index];

                load_cover_art(
                    ui,
                    &mut sanc.covers,
                    &mut sanc.loading_covers,
                    current_track,
                );

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
                    sanc.player.previous(&sanc.songs);
                }

                if ui.add(play_button).clicked() {
                    sanc.player.playback();
                }

                if ui.add(skip_button).clicked() {
                    sanc.player.skip(&sanc.songs);
                }

                if ui.add(shufl_button).clicked() {
                    sanc.player.shuffle();
                }
            });

            ui.horizontal(|ui| {
                ui.spacing_mut().slider_width = ui.max_rect().width() - 50.;
                ui.style_mut().visuals.slider_trailing_fill = true;

                let total_duration = sanc.songs[sanc.player.current_index].duration;
                let time_slider = egui::Slider::new(&mut sanc.player.track_pos, 0..=total_duration)
                    .logarithmic(false)
                    .show_value(false)
                    .clamping(egui::SliderClamping::Always)
                    .trailing_fill(true);

                let seek_bar = ui.add(time_slider);

                if seek_bar.drag_stopped() {
                    sanc.player.seek();
                }

                ui.label(format!(
                    "{} / {}",
                    format_timestamp(sanc.player.track_pos),
                    format_timestamp(total_duration)
                ));
            });
        });

        columns[2].horizontal_centered(|ui| {
            ui.add_space(ui.max_rect().width() - 200.);

            ui.label("🔈");
            ui.style_mut().visuals.slider_trailing_fill = true;

            ui.add(egui::Slider::new(&mut sanc.volume, 0..=100));

            sanc.player.volume(sanc.volume);
            sanc.config.set_volume(sanc.volume);
        });
    });
}
