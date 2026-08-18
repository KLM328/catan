use egui::{Color32, FontId, RichText, Sense, Stroke};
use catan::Scenario;
use catan_protocol::{ClientMessage, ClientState, GameInfo};


const LIST_W: f32 = 560.0;
const ROW_H: f32 = 66.0;

pub(crate) fn show(
    ui: &mut egui::Ui,
    games: &[GameInfo],
    messages: &mut Vec<ClientMessage>,
) {
    egui::Area::new(egui::Id::new("menu"))
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.set_width(LIST_W);

            ui.vertical_centered(|ui| {
                ui.label(RichText::new("Catan").font(FontId::proportional(56.0)));
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("Parties disponibles")
                        .font(FontId::proportional(15.0))
                        .color(ui.visuals().weak_text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if refresh_button(ui).clicked() {
                        messages.push(ClientMessage::Sync(ClientState::Menu));
                    }
                });
            });
            ui.add_space(22.0);

            if games.is_empty() {
                empty_state(ui);
            } else {
                egui::ScrollArea::vertical()
                    .max_height(420.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for game in games {
                            if game_row(ui, game).clicked() {
                                messages.push(ClientMessage::Join {
                                    game_id: game.id,
                                    token: None,
                                });
                            }
                        }
                    });
            }

            ui.add_space(20.0);
            ui.vertical_centered(|ui| {
                if create_button(ui).clicked() {
                    messages.push(ClientMessage::CreateGame(Scenario::standard()));
                }
            });
        });
}

fn game_row(ui: &mut egui::Ui, game: &GameInfo) -> egui::Response {
    let max = game.scenario.max_player();
    let full = game.connected_players >= max;

    let sense = if full { Sense::hover() } else { Sense::click() };
    let (rect, response) = ui.allocate_exact_size(egui::vec2(LIST_W, ROW_H), sense);
    let t = ui.ctx().animate_bool(response.id, response.hovered() && !full);
    let painter = ui.painter_at(rect);

    painter.rect_filled(rect, 10.0, Color32::from_gray(34 + (16.0 * t) as u8));
    if t > 0.0 {
        painter.rect_stroke(
            rect,
            10.0,
            Stroke::new(1.0, Color32::from_gray(70 + (60.0 * t) as u8)),
            egui::StrokeKind::Inside,
        );
    }

    let dim = |c: Color32| if full { c.gamma_multiply(0.45) } else { c };

    painter.text(
        rect.left_center() + egui::vec2(20.0, -10.0),
        egui::Align2::LEFT_CENTER,
        format!("Partie {}", game.id()),
        FontId::proportional(20.0),
        dim(ui.visuals().text_color()),
    );
    painter.text(
        rect.left_center() + egui::vec2(20.0, 13.0),
        egui::Align2::LEFT_CENTER,
        format!(
            "{} points • {}–{} joueurs",
            game.scenario.max_points(),
            game.scenario.min_player(),
            max
        ),
        FontId::proportional(13.0),
        dim(ui.visuals().weak_text_color()),
    );

    // Pastilles : une par place, pleine si occupée.
    const D: f32 = 11.0;
    const GAP: f32 = 7.0;
    let width = max as f32 * D + (max - 1) as f32 * GAP;
    let mut x = rect.right() - 20.0 - width + D / 2.0;
    for i in 0..max {
        let c = egui::pos2(x, rect.center().y);
        if i < game.connected_players {
            painter.circle_filled(c, D / 2.0, dim(Color32::from_rgb(120, 190, 110)));
        } else {
            painter.circle_stroke(c, D / 2.0, Stroke::new(1.2, Color32::from_gray(80)));
        }
        x += D + GAP;
    }

    if full {
        painter.text(
            rect.right_center() + egui::vec2(-20.0, 15.0),
            egui::Align2::RIGHT_CENTER,
            "Complet",
            FontId::proportional(11.0),
            Color32::from_gray(95),
        );
    }

    response.on_hover_cursor(if full {
        egui::CursorIcon::NotAllowed
    } else {
        egui::CursorIcon::PointingHand
    })
}

fn empty_state(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(LIST_W, 110.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_stroke(
        rect,
        10.0,
        Stroke::new(1.0, Color32::from_gray(55)),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "Aucune partie en cours — créez la première",
        FontId::proportional(15.0),
        ui.visuals().weak_text_color(),
    );
}

fn create_button(ui: &mut egui::Ui) -> egui::Response {
    let btn = egui::Button::new(RichText::new("Créer une partie").size(16.0))
        .min_size(egui::vec2(220.0, 40.0))
        .corner_radius(20.0)
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0, ui.visuals().weak_text_color()));
    ui.add(btn).on_hover_cursor(egui::CursorIcon::PointingHand)
}

fn refresh_button(ui: &mut egui::Ui) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), Sense::click());
    let t = ui.ctx().animate_bool(response.id, response.hovered());
    let painter = ui.painter_at(rect);

    let c = rect.center();
    let r = 8.0;
    let spin = t * std::f32::consts::FRAC_PI_2;
    let color = ui
        .visuals()
        .weak_text_color()
        .lerp_to_gamma(ui.visuals().strong_text_color(), t);

    let start = spin - 0.4;
    let sweep = std::f32::consts::PI * 1.55;
    let pts: Vec<egui::Pos2> = (0..=32)
        .map(|i| {
            let a = start + sweep * (i as f32 / 32.0);
            c + egui::vec2(a.cos(), a.sin()) * r
        })
        .collect();
    painter.add(egui::Shape::line(pts, Stroke::new(1.8, color)));

    // Pointe de flèche à l'extrémité de l'arc.
    let end = start + sweep;
    let tip = c + egui::vec2(end.cos(), end.sin()) * r;
    let dir = egui::vec2(-end.sin(), end.cos());
    let out = egui::vec2(end.cos(), end.sin());
    painter.add(egui::Shape::convex_polygon(
        vec![
            tip + dir * 4.5,
            tip + out * 3.5 - dir * 1.5,
            tip - out * 3.5 - dir * 1.5,
        ],
        color,
        Stroke::NONE,
    ));

    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}