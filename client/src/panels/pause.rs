use crate::game_view::GameView;
use crate::{player_color, absent_disc};
use catan::{PlayerColor};
use eframe::egui::{self, Color32, RichText, Stroke, Ui};
use crate::theme::{DISC_RADIUS, GAP};

pub(crate) fn show(ui: &mut Ui, game: &GameView) {

    if game.missing_players().is_empty() {
        return
    }

    let frame = egui::Frame::popup(ui.style())
        .fill(Color32::from_rgba_unmultiplied(18, 16, 14, 205))
        .stroke(Stroke::new(1.0, Color32::from_gray(75)))
        .corner_radius(14.0)
        .inner_margin(egui::Margin::symmetric(34, 26));

    egui::Modal::new(egui::Id::new("pause"))
        .backdrop_color(Color32::from_black_alpha(55))
        .frame(frame)
        .show(ui.ctx(), |ui| {
            let n = game.missing_players().len() as f32;
            let total = (n * DISC_RADIUS + (n - 1.0).max(0.0) * GAP).max(300.0);
            ui.set_width(total);

            ui.vertical_centered(|ui| {
                ui.add(egui::Spinner::new().size(24.0));
                ui.add_space(10.0);
                ui.label(RichText::new("Partie en pause").size(28.0));
                ui.add_space(2.0);
                ui.label(
                    RichText::new(if game.missing_players().len() > 1 {
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
                    let row = n * (DISC_RADIUS*2.6) + (n - 1.0).max(0.0) * GAP;
                    ui.add_space(((ui.available_width() - row) * 0.5).max(0.0));
                    for &id in game.missing_players() {
                        if let Some(player) = game.get_player(id) {
                            absent_disc(ui, player_color(player), PlayerColor::color_name(player.color()));
                        }
                    }
                });

                ui.add_space(18.0);
                ui.label(
                    RichText::new(if game.missing_players().len() > 1 {
                        "La partie reprendra automatiquement à leur retour."
                    } else {
                        "La partie reprendra automatiquement à son retour."
                    }).size(12.0).color(Color32::from_gray(120)),
                );
            });
        });
}