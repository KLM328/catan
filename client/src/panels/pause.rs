use crate::game_view::GameView;
use crate::{player_color, player_disc};
use catan::{PlayerColor, PlayerId};
use eframe::egui::{self, Color32, FontId, RichText, Sense, Stroke, Ui};

const DISC_W: f32 = 76.0;
const GAP: f32 = 12.0;

pub(crate) fn show(ui: &mut Ui, game: &GameView, missing: &[PlayerId]) {
    let frame = egui::Frame::popup(ui.style())
        .fill(Color32::from_rgba_unmultiplied(18, 16, 14, 205))
        .stroke(Stroke::new(1.0, Color32::from_gray(75)))
        .corner_radius(14.0)
        .inner_margin(egui::Margin::symmetric(34, 26));

    egui::Modal::new(egui::Id::new("pause"))
        .backdrop_color(Color32::from_black_alpha(55))
        .frame(frame)
        .show(ui.ctx(), |ui| {
            let n = missing.len() as f32;
            let total = (n * DISC_W + (n - 1.0).max(0.0) * GAP).max(300.0);
            ui.set_width(total);

            ui.vertical_centered(|ui| {
                ui.add(egui::Spinner::new().size(24.0));
                ui.add_space(10.0);
                ui.label(RichText::new("Partie en pause").size(28.0));
                ui.add_space(2.0);
                ui.label(
                    RichText::new(if missing.len() > 1 {
                        "Ces joueurs sont déconnectés"
                    } else {
                        "Ce joueur est déconnecté"
                    })
                        .size(13.0)
                        .color(ui.visuals().weak_text_color()),
                );
                ui.add_space(18.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = GAP;
                    let row = n * DISC_W + (n - 1.0).max(0.0) * GAP;
                    ui.add_space(((ui.available_width() - row) * 0.5).max(0.0));
                    for &id in missing {
                        if let Some(player) = game.get_player(id) {
                            absent_disc(ui, player_color(player), PlayerColor::color_name(player.color()));
                        }
                    }
                });

                ui.add_space(18.0);
                ui.label(
                    RichText::new(if missing.len() > 1 {
                        "La partie reprendra automatiquement à leur retour."
                    } else {
                        "La partie reprendra automatiquement à son retour."
                    }).size(12.0).color(Color32::from_gray(120)),
                );
            });
        });
}

fn absent_disc(ui: &mut Ui, color: Color32, name: &str) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(DISC_W, 64.0), Sense::hover());
    let painter = ui.painter_at(rect);
    let center = egui::pos2(rect.center().x, rect.top() + 24.0);

    // Respiration lente : 0.25 → 0.6 d'opacité.
    let t = ui.input(|i| i.time) as f32;
    let pulse = 0.25 + 0.35 * (0.5 + 0.5 * (t * 1.8).sin());

    player_disc(&painter, center, 20.0, color.gamma_multiply(pulse));

    painter.text(
        egui::pos2(rect.center().x, rect.top() + 54.0),
        egui::Align2::CENTER_CENTER,
        name,
        FontId::proportional(13.0),
        ui.visuals().weak_text_color(),
    );
}