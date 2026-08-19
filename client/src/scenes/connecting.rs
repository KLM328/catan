use crate::{ConnectionState};
use egui::{FontId, RichText, Ui};
use std::time::Duration;
use tokio::sync::Notify;

pub(crate) fn show(ui: &mut Ui, connection_state: ConnectionState, force_retry : &Notify) {
    egui::Area::new(egui::Id::new("connecting"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.add(egui::Spinner::new().size(60.0));
                ui.add_space(8.0);
                match connection_state {
                    ConnectionState::Retrying { at, attempt } => {
                        let left = at.saturating_duration_since(std::time::Instant::now());
                        ui.label(RichText::new("Connexion en cours…").font(FontId::proportional(40.0)));
                        ui.label(
                            RichText::new(format!(
                                "Échec (essai {attempt}) — nouvelle tentative dans {} s",
                                left.as_secs_f32().ceil()
                            ))
                            .font(FontId::proportional(15.0)),
                        );
                        ui.ctx().request_repaint_after(Duration::from_millis(250));

                        ui.add_space(16.0);

                        let btn = egui::Button::new(egui::RichText::new("Réessayer").size(15.0))
                            .min_size(egui::vec2(120.0, 32.0))
                            .corner_radius(16.0)
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, ui.visuals().weak_text_color()));

                        if ui
                            .add(btn)
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            force_retry.notify_one();
                        }

                    }
                    ConnectionState::Connecting { attempt } if attempt > 1 => {
                        ui.label(RichText::new("Connexion en cours…").font(FontId::proportional(40.0)));

                        ui.label(
                            RichText::new(format!("Reconnexion… (essai {attempt})"))
                                .font(FontId::proportional(15.0)),
                        );
                    }
                    _ => {
                        ui.label(
                            RichText::new("Connexion en cours…").font(FontId::proportional(40.0)),
                        );
                    }
                }
            });
        });
}
