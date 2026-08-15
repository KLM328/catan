use eframe::egui;
use eframe::egui::{Align2, Color32, FontId, Sense, Stroke};
use catan::{Player, PlayerColor};
use catan_protocol::PlayerInfo;
use crate::game_view::GameView;
use crate::theme::player_color;
use crate::widgets::shapes::card;
use crate::widgets::player_disc;


pub(crate) fn player_row(ui: &mut egui::Ui, game : &GameView, player: &PlayerInfo, is_current: bool) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 90.0),
        Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    let color = player_color(player);

    if is_current {
        painter.rect_filled(rect, 8.0, Color32::from_gray(52));
        painter.rect_stroke(rect, 8.0, Stroke::new(2.0, color), egui::StrokeKind::Inside);
    }

    // Ligne 1 : le joueur
    let line1 = rect.top() + 24.0;
    let disc = egui::pos2(rect.left() + 32.0, line1);
    player_disc(&painter, disc, 15.0, color);
    painter.text(
        egui::pos2(rect.left() + 58.0, line1),
        Align2::LEFT_CENTER,
        PlayerColor::color_name(player.color()),
        FontId::proportional(19.0),
        Color32::from_gray(230),
    );

    // Ligne 2 : cartes à gauche, points à droite
    let line2 = rect.bottom() - 24.0;


    let resource_card = egui::Rect::from_center_size(
        egui::pos2(rect.left() + 24.0, line2),
        egui::vec2(14.0, 19.0),
    );

    card(&painter, resource_card, Color32::from_gray(185));

    painter.text(
        egui::pos2(resource_card.right() + 10.0, line2),
        Align2::LEFT_CENTER,
        player.hand_count().to_string(),
        FontId::proportional(16.0),
        Color32::from_gray(200),
    );

    let dev_card = egui::Rect::from_center_size(
        egui::pos2(rect.center().x, line2),
        egui::vec2(14.0, 19.0),
    );

    card(&painter, dev_card, Color32::from_rgb(75, 0, 130));


    painter.text(
        egui::pos2(dev_card.right() + 10.0, line2),
        Align2::LEFT_CENTER,
        0.to_string() ,
        FontId::proportional(16.0),
        Color32::from_gray(200),
    );

    painter.text(
        egui::pos2(rect.right() - 20.0, line2),
        Align2::RIGHT_CENTER,
        game.score(player).to_string(),
        FontId::proportional(26.0),
        Color32::from_gray(245),
    );
    painter.text(
        egui::pos2(rect.right() - 42.0, line2),
        Align2::RIGHT_CENTER,
        "PV",
        FontId::proportional(12.0),
        Color32::from_gray(150),
    );
}
