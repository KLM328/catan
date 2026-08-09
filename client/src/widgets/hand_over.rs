use eframe::egui;
use eframe::egui::{Color32, FontId, Sense};
use crate::widgets::player_disc;

pub(crate) fn hand_over_button(ui: &mut egui::Ui, next: Color32, name: &str) -> egui::Response {
    const PAD: f32 = 10.0; // marge gauche et droite
    const GAP: f32 = 12.0; // entre le texte et le disque
    const RADIUS: f32 = 18.0;
    const SLIDE: f32 = 10.0; // course du disque au survol
    let text_color = Color32::from_gray(220);

    // 1. Mesurer
    let galley = ui.painter().layout_no_wrap(
        format!("Au tour de {name}"),
        FontId::proportional(25.0),
        text_color,
    );

    // 2. Allouer d'après la mesure
    let width = PAD * 2.0 + galley.size().x + GAP + RADIUS * 2.0 + SLIDE + PAD;
    let height = (galley.size().y + 24.0).max(RADIUS * 2.0 + 16.0);
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

    let c = egui::pos2(
        rect.right() - PAD - RADIUS - SLIDE + SLIDE * t,
        rect.center().y,
    );

    player_disc(&painter, c, RADIUS, next);

    response
}