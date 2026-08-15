use crate::panels::steal;
use crate::{player_color, terrain_color};
use catan::{BuildingKind, EdgeId, GameStatus, Layout, TileId, VertexId};
use eframe::egui;
use eframe::egui::{Align2, Color32, FontId, Pos2, Sense, Shape, Stroke, Ui};
use catan_protocol::ClientMessage;
use crate::app::UiState;
use crate::game_view::GameView;

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum BuildMode {
    None,
    Road,
    Settlement,
    City,
}

pub(crate) fn show(
    ui: &mut Ui,
    game: &GameView,
    ui_state: &mut UiState,
) -> Vec<ClientMessage> {
    let mut actions = Vec::new();
    egui::CentralPanel::default().show(ui, |ui| {
        actions.extend(steal::show(ui, game));

        ui.label(format!("Statut : {:?}", game.status()));

        // On réserve toute la place restante et on récupère un pinceau.
        let available = ui.available_size();
        let (response, painter) = ui.allocate_painter(available, Sense::click());

        // Le plateau est centré sur (0,0) en coordonnées hexagonales :
        // il suffit de placer l'origine au centre de la zone de dessin.
        let center = response.rect.center();
        let layout = Layout {
            hex_size: ui_state.hex_size(),
            origin: (center.x, center.y),
        };

        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll != 0.0 {
            ui_state.adjust_hex_size_with_scroll(scroll);
        }

        let board = game.board();
        let topo = board.topology();

        for (index, tile) in board.tiles().iter().enumerate() {
            let tile_id = TileId::new(index);

            // Les 6 coins de la tuile sont les positions de ses sommets :
            // aucune fonction de coins supplémentaire n'est nécessaire.
            let corners: Vec<Pos2> = topo.tile_vertices()[index]
                .iter()
                .map(|&v| {
                    let (x, y) = layout.vertex_position(topo, v);
                    Pos2::new(x, y)
                })
                .collect();

            painter.add(Shape::convex_polygon(
                corners,
                terrain_color(tile.terrain()),
                Stroke::new(2.0, Color32::from_rgb(40, 40, 50)),
            ));

            // Le jeton numéroté, au centre de la tuile.
            if let Some(token) = tile.number() {
                let (cx, cy) = layout.tile_position(topo, tile_id);
                let pos = Pos2::new(cx, cy);
                painter.circle_filled(pos, ui_state.hex_size() * 0.35, Color32::from_rgb(240, 235, 220));

                // 6 et 8 sont les numéros "rouges" : les plus probables.
                let color = match token.value() {
                    6 | 8 => Color32::from_rgb(180, 30, 30),
                    _ => Color32::from_rgb(30, 30, 30),
                };

                let size = match token.value() {
                    6 | 8 => 0.4,
                    5 | 9 => 0.36,
                    4 | 10 => 0.32,
                    3 | 11 => 0.28,
                    _ => 0.24,
                };

                painter.text(
                    pos,
                    Align2::CENTER_CENTER,
                    token.value().to_string(),
                    FontId::proportional(ui_state.hex_size() * size),
                    color,
                );
            }

            // Le voleur.
            if board.robber() == tile_id {
                let (cx, cy) = layout.tile_position(topo, tile_id);
                painter.circle_filled(
                    Pos2::new(cx, cy - ui_state.hex_size() * 0.50),
                    ui_state.hex_size() * 0.30,
                    Color32::from_rgb(20, 20, 20),
                );
            }
        }

        // Les routes d'abord (elles passent sous les bâtiments)
        for (index, owner) in board.roads().iter().enumerate() {
            if let Some(player) = owner {
                let ((ax, ay), (bx, by)) = layout.edge_position(topo, EdgeId::new(index));
                painter.line_segment(
                    [Pos2::new(ax, ay), Pos2::new(bx, by)],
                    Stroke::new(
                        ui_state.hex_size() * 0.15,
                        player_color(game.get_player(*player).unwrap()),
                    ),
                );
            }
        }

        // Puis les colonies et les villes
        for (index, building) in board.buildings().iter().enumerate() {
            if let Some(b) = building {
                let (x, y) = layout.vertex_position(topo, VertexId::new(index));
                let pos = Pos2::new(x, y);
                let color = player_color(game.get_player(b.owner()).unwrap());
                match b.kind() {
                    BuildingKind::Settlement => painter.circle_filled(pos, ui_state.hex_size() * 0.3, color),
                    BuildingKind::City => painter.rect_filled(
                        egui::Rect::from_center_size(
                            pos,
                            egui::vec2(ui_state.hex_size() * 0.6, ui_state.hex_size() * 0.6),
                        ),
                        2.0,
                        color,
                    ),
                };
            }
        }

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
        {
            let radius = ui_state.hex_size() * 0.30;
            match game.status() {
                GameStatus::Starting => {}
                GameStatus::FirstPlacementSettlement | GameStatus::SecondPlacementSettlement => {
                    if let Some(vertex_location) = layout.pick_vertex(topo, (pos.x, pos.y), radius)
                    {
                        actions.push(ClientMessage::BuildSettlement(vertex_location));
                    }
                }
                GameStatus::FirstPlacementRoad | GameStatus::SecondPlacementRoad => {
                    if let Some(edge_location) = layout.pick_edge(topo, (pos.x, pos.y), radius) {
                        actions.push(ClientMessage::BuildRoad(edge_location));
                    }
                }
                GameStatus::AwaitingRoll => {}
                GameStatus::AwaitingDiscard { .. } => {}
                GameStatus::AwaitingSteal => {}
                GameStatus::AwaitingNewRobberLocation => {
                    if let Some(tile_location) = layout.pick_tile(topo, (pos.x, pos.y)) {
                        actions.push(ClientMessage::RobberLocation(tile_location));
                    }
                }
                GameStatus::PlayingActions => match ui_state.build_mode() {
                    BuildMode::None => {}
                    BuildMode::Road => {
                        if let Some(edge_location) = layout.pick_edge(topo, (pos.x, pos.y), radius)
                        {
                            actions.push(ClientMessage::BuildRoad(edge_location));
                        }
                    }
                    BuildMode::Settlement => {
                        if let Some(vertex_location) =
                            layout.pick_vertex(topo, (pos.x, pos.y), radius)
                        {
                            actions.push(ClientMessage::BuildSettlement(vertex_location));
                        }
                    }
                    BuildMode::City => {
                        if let Some(vertex_location) =
                            layout.pick_vertex(topo, (pos.x, pos.y), radius)
                        {
                            actions.push(ClientMessage::UpgradeCity(vertex_location))
                        }
                    }
                },
                GameStatus::End { .. } => {}
            }
        }
    });

    actions
}
