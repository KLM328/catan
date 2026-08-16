use eframe::egui;
use eframe::egui::{Color32, FontId, Sense};
use crate::widgets::player_disc;

pub(crate) fn end_turn_button(ui: &mut egui::Ui) -> egui::Response {
    const PAD: f32 = 10.0; // marge gauche et droite

    let text_color = Color32::from_gray(220);

    // 1. Mesurer
    let galley = ui.painter().layout_no_wrap(
        format!("Fin du tour"),
        FontId::proportional(25.0),
        text_color,
    );

    // 2. Allouer d'après la mesure
    let width = PAD * 2.0 + galley.size().x + PAD;
    let height = galley.size().y + 24.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), Sense::click());

    // 3. Dessiner
    let t = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter_at(rect);

    painter.rect_filled(
        rect,
        height / 2.0,
        Color32::from_gray(50 + (20.0 * t) as u8),
    );

    let text_pos = egui::pos2(
        rect.left() + PAD * 2.0,
        rect.center().y - galley.size().y / 2.0,
    );
    painter.galley(text_pos, galley, text_color);
    
    response
}