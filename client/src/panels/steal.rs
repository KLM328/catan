use eframe::egui;
use eframe::egui::Ui;
use catan::{Game, GameStatus, PlayerColor, PlayerId, Steal};
use crate::UiAction;
use crate::{player_color, disc_button};
use crate::game_view::GameView;

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum StealChoice {
    Pending,
    Nobody,
    Victim(PlayerId),
}

pub(crate) fn show(ui : &mut Ui, game : &GameView) -> Vec<UiAction>{
    let mut actions = Vec::new();
    if matches!(game.status(), GameStatus::AwaitingSteal) {
        // 1. On extrait les données AVANT le closure
        let mut chosen: StealChoice = StealChoice::Pending;

        if let Ok(victims) = game.steal_victims(game.current_player()) {
            const DISC_W: f32 = 78.0;
            const GAP: f32 = 10.0;

            let n = victims.len() as f32;
            let total = n.max(1.0) * DISC_W + (n - 1.0).max(0.0) * GAP;

            egui::Modal::new(egui::Id::new("steal")).show(ui.ctx(), |ui| {
                ui.set_width((total + 48.0).max(240.0));
                ui.vertical_centered(|ui| {
                    ui.heading("À qui volez-vous une carte ?");
                    ui.add_space(14.0);

                    if victims.is_empty() {
                        if disc_button(ui, None, "Personne", "à voler").clicked() {
                            chosen = StealChoice::Nobody;
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = GAP;
                            ui.add_space(((ui.available_width() - total) * 0.5).max(0.0));
                            for &v in &victims {
                                let p = &game.get_player(v).unwrap();
                                let cards = p.hand_count();
                                let sub = if cards > 1 {
                                    format!("{cards} cartes")
                                } else {
                                    format!("{cards} carte")
                                };
                                if disc_button(
                                    ui,
                                    Some(player_color(p)),
                                    PlayerColor::color_name(p.color()),
                                    sub.as_str(),
                                )
                                    .clicked()
                                {
                                    chosen = StealChoice::Victim(v);
                                }
                            }
                        });
                    }
                });
            });
        }

        let steal = match chosen {
            StealChoice::Nobody => None,
            StealChoice::Victim(victim) => todo!(),
            StealChoice::Pending => return Vec::new(),
        };
        actions.push(UiAction::Steal(steal));
    }
    actions
}

