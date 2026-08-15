use crate::{cell, player_disc, player_color};
use catan::{BuildingKind, Game, GameStatus, Player, PlayerColor, PlayerId};
use eframe::egui;
use eframe::egui::{Color32, Sense, Ui};
use std::cmp::Reverse;
use catan_protocol::PlayerInfo;
use log::warn;
use crate::game_view::GameView;

pub(crate) fn show(ui: &mut Ui, game: &GameView) {
    const WIDTH: f32 = 700.0;
    const COL: usize = 6;
    const SPACE: f32 = 10.0;

    const RADIUS: f32 = 10.0;

    let columns = [
        "Joueur", "Colonies", "Villes", "Routes", "PV carte", "PV total",
    ];
    // if let GameStatus::PlayingActions = game.status() {
    //     let winner = PlayerId::new(1);
    if let GameStatus::End { winner } = game.status() {

        egui::Modal::new(egui::Id::new("end")).show(ui.ctx(), |ui| {


            ui.set_width(WIDTH);
            ui.add_space(10.0);

            let champion = game.get_player(*winner).unwrap();
            let c = player_color(champion);

            ui.vertical_centered(|ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(70.0, 70.0), Sense::hover());
                let p = ui.painter_at(r);
                player_disc(&p, r.center(), 32.0, c);

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(format!(
                        "{} l'emporte",
                        PlayerColor::color_name(champion.color())
                    ))
                    .size(30.0)
                    .color(c),
                );
            });

            ui.add_space(18.0);
            ui.separator();
            ui.add_space(10.0);

            egui::Grid::new("recap")
                .num_columns(COL)
                .spacing([SPACE, 12.0])
                .min_col_width((WIDTH - SPACE * (COL - 1) as f32) / COL as f32)
                .striped(true)
                .show(ui, |ui| {
                    for h in columns {
                        cell(
                            ui,
                            egui::RichText::new(h)
                                .size(14.0)
                                .color(Color32::from_gray(150)),
                        );
                    }
                    ui.end_row();
                    let mut players: Vec<(PlayerId, &PlayerInfo)> = game
                        .turn_order()
                        .iter()
                        .map(|&p| (p, game.get_player(p).unwrap()))
                        .collect();
                    players.sort_by_key(|&(_ , player)| Reverse(game.score(player))); //les scores sont faux, il faudra envoyer les infos correcte via un message server


                    for (id, player) in players {
                        let color = player_color(player);
                        let board = game.board();

                        let settlements = board
                            .buildings()
                            .iter()
                            .flatten()
                            .filter(|b| b.owner() == id && b.kind() == BuildingKind::Settlement)
                            .count();
                        let cities = board
                            .buildings()
                            .iter()
                            .flatten()
                            .filter(|b| b.owner() == id && b.kind() == BuildingKind::City)
                            .count();
                        let roads = board.roads().iter().flatten().filter(|&&p| p == id).count();

                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(
                                egui::vec2(RADIUS * 4.0, RADIUS * 2.0),
                                Sense::hover(),
                            );
                            let p = ui.painter_at(r);
                            player_disc(&p, r.center(), RADIUS, color);
                            cell(
                                ui,
                                egui::RichText::new(PlayerColor::color_name(player.color()))
                                    .size(17.0)
                                    .color(color),
                            );
                        });

                        cell(ui, egui::RichText::new(settlements.to_string()).size(17.0));
                        cell(ui, egui::RichText::new(cities.to_string()).size(17.0));
                        cell(ui, egui::RichText::new(roads.to_string()).size(17.0));
                        cell(ui, egui::RichText::new(0.to_string()).size(17.0));
                        cell(
                            ui,
                            egui::RichText::new(game.score(player).to_string()).size(20.0), //les scores sont faux, il faudra envoyer les infos correcte via un message server
                        );
                        todo!("récuparation des scores finaux depuis le serveur");
                        ui.end_row();
                    }
                });
            ui.add_space(10.0);
        });
    }
}
