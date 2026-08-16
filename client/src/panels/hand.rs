use crate::{theme};
use catan::{GameStatus, Resource, ResourceCounts};
use eframe::egui::Ui;
use egui::{Align2, Color32, Sense, Stroke};
use catan_protocol::ClientMessage;
use crate::app::UiState;
use crate::game_view::GameView;
use crate::{player_color, resource_color};
use crate::{badge, card};

pub(crate) fn show(ui: &mut Ui, game: &GameView, ui_state: &mut UiState) -> Vec<ClientMessage>{

    let mut actions = Vec::new();

    egui::Area::new(egui::Id::new("hand"))
        .anchor(Align2::LEFT_BOTTOM, egui::vec2(24.0, -24.0))
        .show(ui.ctx(), |ui| {
            let required = match game.status() {
                GameStatus::AwaitingDiscard { must_discard } => {
                    match must_discard.iter().flatten().find(|&&(p, _)| p == game.my_id()){
                        Some(&(_, amount)) => Some(amount),
                        None => None,
                    }
                }
                _ => None
            };

            let player = game.get_player(game.my_id()).unwrap();
            let hand = game.my_hand();

            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    if let Some(required) = required && ui_state.discard_selection().count() == required && ui.button("Défausser").clicked() {
                            actions.push(ClientMessage::Discard(*ui_state.discard_selection()));
                        ui_state.reset_discard_selection();
                    }
                    let c = player_color(player);
                    let label = match required {
                        Some(n) => format!("Vous devez défausser {} / {n}",
                                           ui_state.discard_selection().count()),
                        None => String::new(),
                    };
                    ui.label(egui::RichText::new(label).size(20.0).color(c));
                });
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = theme::GAP;
                    for &resource in Resource::ALL.iter() {
                        let (card_rect, response) =
                            ui.allocate_exact_size(egui::vec2(theme::CARD_W, theme::CARD_H), Sense::click());
                        let painter = ui.painter_at(card_rect);
                        let count = hand.amount(resource);
                        let card_rect = egui::Rect::from_min_size(
                            egui::pos2(card_rect.left(), card_rect.top()),
                            egui::vec2(theme::CARD_W, theme::CARD_H),
                        );
                        let color = resource_color(resource);



                        if count == 0 {
                            painter.rect_stroke(
                                card_rect,
                                5.0,
                                Stroke::new(2.0, color.gamma_multiply(0.4)),
                                egui::StrokeKind::Inside,
                            );
                        } else {
                            card(&painter, card_rect, color);

                            let badge_pos = egui::pos2(card_rect.center().x, card_rect.bottom() - 22.0);
                            badge(&painter, badge_pos, 16.0, &count.to_string(), theme::OUTLINE, Color32::from_gray(240));

                            let selected =
                                required.map_or(0, |_| ui_state.discard_selection().amount(resource));
                            if selected > 0 {
                                // La part défaussée s'efface : on voit ce qu'on va perdre.
                                let lost = card_rect.height() * selected as f32 / count.max(1) as f32;
                                painter.rect_filled(
                                    egui::Rect::from_min_size(card_rect.left_top(), egui::vec2(theme::CARD_W, lost)),
                                    5.0,
                                    Color32::from_black_alpha(170),
                                );
                                let mark = egui::pos2(card_rect.center().x, card_rect.top() + 22.0);
                                badge(&painter, mark, 16.0, &selected.to_string(), Color32::from_rgb(150, 40, 40), Color32::from_gray(240));
                            }
                        }

                        if let Some(required) = required {
                            let selected = ui_state.discard_selection().amount(resource);
                            if response.clicked() && selected < count && ui_state.discard_selection().count() < required {
                                ui_state.add_discard_selection(ResourceCounts::from_resource(resource, 1));
                            }
                            if response.secondary_clicked() && selected > 0 {
                                ui_state.remove_discard_selection(ResourceCounts::from_resource(resource, 1));
                            }

                        }
                    }
                });
            });


        });

    actions
}