use crate::theme;
use eframe::egui::{self, Align2, Color32, FontId, Ui};
use std::time::Duration;
use crate::UiState;

const HOLD: f64 = 3.0;   // secondes d'affichage plein
const FADE: f32 = 0.5;   // secondes de fondu

pub(crate) fn show(ui: &mut Ui, ui_state: &UiState) {
    let Some((text, shown_at)) = ui_state.message() else { return };

    let now = ui.input(|i| i.time);
    let age = now - shown_at;
    let visible = age < HOLD;

    // Sans ça, egui reste au repos et le message ne disparaîtrait jamais :
    // on programme un réveil pile à la fin du maintien.
    if visible {
        ui.ctx().request_repaint_after(Duration::from_secs_f64(HOLD - age));
    }

    let t = ui.ctx()
        .animate_bool_with_time(egui::Id::new("message_fade"), visible, FADE);
    if t <= 0.0 {
        return;
    }

    egui::Area::new(egui::Id::new("message"))
        .anchor(Align2::CENTER_TOP, egui::vec2(0.0, 28.0))
        .interactable(false)
        .show(ui.ctx(), |ui| {
            let text_color = Color32::from_rgb(204, 0,0);

            let galley = ui.painter().layout_no_wrap(
                text.clone(),
                FontId::proportional(18.0),
                text_color,
            );

            let pad = egui::vec2(20.0, 10.0);
            let (rect, _) = ui.allocate_exact_size(
                galley.size() + pad * 2.0,
                egui::Sense::hover(),
            );
            let painter = ui.painter_at(rect);

            painter.rect_filled(
                rect,
                rect.height() / 2.0,
                Color32::from_rgb(32, 28, 24).gamma_multiply(0.92 * t),
            );
            painter.rect_stroke(
                rect,
                rect.height() / 2.0,
                egui::Stroke::new(1.5, theme::OUTLINE.gamma_multiply(t)),
                egui::StrokeKind::Inside,
            );
            painter.galley(rect.min + pad, galley, text_color);
        });
}