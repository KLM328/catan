use std::time::Duration;
use egui::{Align2, Color32, FontId, Stroke, Ui};
use catan_protocol::PlayerInfo;
use crate::UiState;
use crate::theme;
use crate::widgets::{draw_die, player_disc};

pub fn show(ui: &mut Ui, players : &[PlayerInfo], ui_state: &UiState) {
    let Some((rolls, shown_at)) = ui_state.rolls_display() else { return };

    let now = ui.input(|i| i.time);
    let age = now - shown_at;
    const HOLD: f64 = 5.0;

    if age < HOLD {
        ui.ctx().request_repaint_after(Duration::from_secs_f64(HOLD - age));
    }
    let t = ui.ctx().animate_bool_with_time(
        egui::Id::new("rolls"), age < HOLD, 0.6);
    if t <= 0.0 { return; }

    egui::Area::new(egui::Id::new("rolls_area"))
        .anchor(Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            const DISC: f32 = 26.0;
            const DIE: f32 = 44.0;
            const GAP: f32 = 12.0;
            const ROW_H: f32 = 60.0;
            const PAD: f32 = 24.0;

            let w = PAD * 2.0 + DISC * 2.0 + GAP + DIE * 2.0 + GAP + 70.0;
            let h = PAD * 2.0 + ROW_H * rolls.len() as f32;

            let (rect, _) = ui.allocate_exact_size(egui::vec2(w, h), egui::Sense::hover());
            let painter = ui.painter_at(rect);

            painter.rect_filled(rect, 12.0,
                                Color32::from_rgb(28, 25, 22).gamma_multiply(0.95 * t));
            painter.rect_stroke(rect, 12.0,
                                Stroke::new(1.5, theme::OUTLINE.gamma_multiply(t)),
                                egui::StrokeKind::Inside);

            for (i, (player_id, roll)) in rolls.iter().enumerate() {
                let y = rect.top() + PAD + ROW_H * i as f32 + ROW_H / 2.0;
                let mut x = rect.left() + PAD + DISC;

                let color = players
                    .iter()
                    .find(|p| p.id() == *player_id)
                    .map(theme::player_color)
                    .unwrap_or(Color32::GRAY);

                player_disc(&painter, egui::pos2(x, y), DISC, color.gamma_multiply(t));
                x += DISC + GAP + DIE / 2.0;

                let face = Color32::from_rgb(235, 232, 226).gamma_multiply(t);
                draw_die(&painter, egui::pos2(x, y), DIE, roll.dice1(), face);
                x += DIE + GAP;
                draw_die(&painter, egui::pos2(x, y), DIE, roll.dice2(), face);
                x += DIE / 2.0 + GAP + 20.0;

                painter.text(
                    egui::pos2(x, y),
                    Align2::LEFT_CENTER,
                    roll.value().to_string(),
                    FontId::proportional(26.0),
                    Color32::from_gray(240).gamma_multiply(t),
                );
            }
        });
}