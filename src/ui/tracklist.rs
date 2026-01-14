use crate::Sanctum;
use crate::format_timestamp;
use crate::load_cover_art;
use crate::playlist::{Sort, sort_songs};
use egui_extras::{Column, TableBuilder};

pub fn playlist(ui: &mut egui::Ui, sanc: &mut Sanctum) {
    let col_width = ui.max_rect().width() / 3.;
    TableBuilder::new(ui)
        .striped(true)
        .column(Column::auto())
        .column(Column::exact(col_width))
        .column(Column::remainder())
        .column(Column::remainder())
        .column(Column::auto())
        .column(Column::auto())
        .header(24., |mut header| {
            header.col(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("#");
                    let sort_order = sanc.current_playlist.clone().sort_order();

                    let track_sort = match sort_order {
                        Sort::Track { reverse } => Sort::Track { reverse: !reverse },
                        _ => Sort::Track { reverse: false },
                    };

                    let label = match track_sort {
                        Sort::Track { reverse: true } => "⬇",
                        Sort::Track { reverse: false } => "⬆",
                        _ => unreachable!(),
                    };

                    if ui.button(label).clicked() {
                        sanc.current_playlist.set_sort(track_sort);
                        sort_songs(
                            sanc.current_playlist.clone(),
                            &mut sanc.song_view,
                            &sanc.songs,
                        );
                    }
                });
            });

            header.col(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Title");
                    let sort_order = sanc.current_playlist.clone().sort_order();

                    let track_sort = match sort_order {
                        Sort::Title { reverse } => Sort::Title { reverse: !reverse },
                        _ => Sort::Title { reverse: false },
                    };

                    let label = match track_sort {
                        Sort::Title { reverse: true } => "⬇",
                        Sort::Title { reverse: false } => "⬆",
                        _ => unreachable!(),
                    };

                    if ui.button(label).clicked() {
                        sanc.current_playlist.set_sort(track_sort);
                        sort_songs(
                            sanc.current_playlist.clone(),
                            &mut sanc.song_view,
                            &sanc.songs,
                        );
                    }
                });
            });

            header.col(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Artist");
                    let sort_order = sanc.current_playlist.clone().sort_order();

                    let track_sort = match sort_order {
                        Sort::Artist { reverse } => Sort::Artist { reverse: !reverse },
                        _ => Sort::Artist { reverse: false },
                    };

                    let label = match track_sort {
                        Sort::Artist { reverse: true } => "⬇",
                        Sort::Artist { reverse: false } => "⬆",
                        _ => unreachable!(),
                    };

                    if ui.button(label).clicked() {
                        sanc.current_playlist.set_sort(track_sort);
                        sort_songs(
                            sanc.current_playlist.clone(),
                            &mut sanc.song_view,
                            &sanc.songs,
                        );
                    }
                });
            });

            header.col(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Album");
                    let sort_order = sanc.current_playlist.clone().sort_order();

                    let track_sort = match sort_order {
                        Sort::Album { reverse } => Sort::Album { reverse: !reverse },
                        _ => Sort::Album { reverse: false },
                    };

                    let label = match track_sort {
                        Sort::Album { reverse: true } => "⬇",
                        Sort::Album { reverse: false } => "⬆",
                        _ => unreachable!(),
                    };

                    if ui.button(label).clicked() {
                        sanc.current_playlist.set_sort(track_sort);
                        sort_songs(
                            sanc.current_playlist.clone(),
                            &mut sanc.song_view,
                            &sanc.songs,
                        );
                    }
                });
            });

            header.col(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Date");
                    let sort_order = sanc.current_playlist.clone().sort_order();

                    let track_sort = match sort_order {
                        Sort::Date { reverse } => Sort::Date { reverse: !reverse },
                        _ => Sort::Date { reverse: false },
                    };

                    let label = match track_sort {
                        Sort::Date { reverse: true } => "⬇",
                        Sort::Date { reverse: false } => "⬆",
                        _ => unreachable!(),
                    };

                    if ui.button(label).clicked() {
                        sanc.current_playlist.set_sort(track_sort);
                        sort_songs(
                            sanc.current_playlist.clone(),
                            &mut sanc.song_view,
                            &sanc.songs,
                        );
                    }
                });
            });

            header.col(|ui| {
                ui.horizontal(|ui| {
                    ui.heading("Time");
                    let sort_order = sanc.current_playlist.clone().sort_order();

                    let track_sort = match sort_order {
                        Sort::Time { reverse } => Sort::Track { reverse: !reverse },
                        _ => Sort::Time { reverse: false },
                    };

                    let label = match track_sort {
                        Sort::Time { reverse: true } => "⬇",
                        Sort::Time { reverse: false } => "⬆",
                        _ => unreachable!(),
                    };

                    if ui.button(label).clicked() {
                        sanc.current_playlist.set_sort(track_sort);
                        sort_songs(
                            sanc.current_playlist.clone(),
                            &mut sanc.song_view,
                            &sanc.songs,
                        );
                    }
                });
            });
        })
        .body(|mut body| {
            for list_index in 0..sanc.song_view.len() {
                let view_index = &sanc.song_view[list_index];
                let song = &sanc.songs[*view_index];

                body.row(48., |mut row| {
                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{:02}", *view_index + 1))
                                    .font(egui::FontId::proportional(18.0)),
                            );
                        });
                    });

                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            load_cover_art(ui, &mut sanc.cache, song);
                            let song_title = egui::Button::new(
                                egui::RichText::new(format!("{}", song.title))
                                    .font(egui::FontId::proportional(18.0)),
                            )
                            .frame(false);

                            let song_title = ui.add(song_title);

                            if song_title.clicked() {
                                sanc.player.set_index(*view_index, &sanc.mpris);
                            }

                            song_title.context_menu(|ui| {
                                if ui.button("Add to queue").clicked() {
                                    sanc.player.add_queue(list_index);
                                }
                            });
                        });
                    });

                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}", song.artist))
                                    .font(egui::FontId::proportional(18.0)),
                            );
                        });
                    });

                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}", song.album))
                                    .font(egui::FontId::proportional(18.0)),
                            );
                        });
                    });

                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}", song.created_date))
                                    .font(egui::FontId::proportional(18.0)),
                            );
                        });
                    });

                    row.col(|ui| {
                        ui.horizontal_centered(|ui| {
                            let timestamp = format_timestamp(song.duration);

                            ui.label(
                                egui::RichText::new(format!("{}", timestamp))
                                    .font(egui::FontId::proportional(18.0)),
                            );
                        });
                    });
                });
            }
        });
}
