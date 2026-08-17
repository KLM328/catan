use egui::{FontId, RichText, Ui};
use crate::app::AppState;

pub(crate) fn show(ui : &mut Ui, app_state: &mut AppState) {
    egui::Area::new(egui::Id::new("connecting"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
                ui.add(egui::Spinner::new().size(60.0));
                ui.add_space(8.0);
                ui.label(RichText::new("Connexion en cours…").font(FontId::proportional(40.0)));
                ui.add_space(16.0);

                let btn = egui::Button::new(egui::RichText::new("Annuler").size(15.0))
                    .min_size(egui::vec2(120.0, 32.0))
                    .corner_radius(16.0)
                    .fill(egui::Color32::TRANSPARENT)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().weak_text_color()));

                if ui.add(btn)
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked() {
                    *app_state = AppState::Menu;
                }
            });
        });

}