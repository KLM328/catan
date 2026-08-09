use crate::panels::{actions, board, dice, end, hand, infos, next_player, message};
use catan::{
    EdgeId, Game, GameError, Player, PlayerColor, PlayerId, ResourceCounts, Roll, Scenario, Steal,
    TileId, VertexId,
};
use eframe::egui;
use crate::panels::board::BuildMode;

pub enum UiAction {
    Roll,
    NextPlayer,
    BuildSettlement(VertexId),
    BuildRoad(EdgeId),
    UpgradeCity(VertexId),
    MoveRobber(TileId),
    Steal(Option<Steal>),
    Discard(PlayerId, ResourceCounts), //playerId à retiré quand client-serveur
}

pub(crate) struct CatanApp {
    game: Game,
    hex_size: f32,
    last_roll: Option<Roll>,
    message: Option<(String, f64)>,
    build_mode: BuildMode,
    discard_selection: ResourceCounts,
}

impl CatanApp {
    pub(crate) fn new() -> Self {
        let scenario = Scenario::standard();
        let terrains = scenario.terrains().to_vec();

        let mut game = Game::new(
            scenario,
            vec![
                Player::new(PlayerColor::Orange),
                Player::new(PlayerColor::Red),
                Player::new(PlayerColor::White),
            ],
        )
        .expect("création de la partie");

        while let Err(GameError::TiedRolls) =
            game.set_players_order(vec![Roll::random(), Roll::random(), Roll::random()])
        {}

        game.start(&terrains).expect("mise en place du plateau");

        Self {
            game,
            hex_size: 80.0,
            last_roll: None,
            message: None,
            build_mode: BuildMode::None,
            discard_selection: ResourceCounts::default(),
        }
    }

    fn apply(&mut self, action: UiAction, now : f64) {
        let player = self.game.current_player();
        let result = match action {
            UiAction::Roll => {
                let roll = Roll::random();
                self.last_roll = Some(roll);
                self.game.apply_roll(roll).map(|_| ())
            }
            UiAction::NextPlayer => {
                self.game.next_player()
            }
            UiAction::BuildSettlement(vertex_id) => {
                self.game.build_settlement(player, vertex_id)
            }
            UiAction::BuildRoad(edge_id) => {
                self.game.build_road(player, edge_id)
            }
            UiAction::UpgradeCity(vertex_id) => {
                self.game.upgrade_settlement_to_city(player, vertex_id)
            }
            UiAction::MoveRobber(tile_id) => self.game.move_robber(player, tile_id),
            UiAction::Steal(steal_option) => self.game.steal(player, steal_option),
            UiAction::Discard(player, resources) => {
                self.discard_selection = ResourceCounts::default();
                self.game.discard(player, resources)
            }
        };
        if let Err(e) = result {
            self.message = Some((format!("{e}"), now));
        } else {
            self.build_mode = BuildMode::None;
        }
    }
}

impl eframe::App for CatanApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut actions = Vec::new();
        let ctx = ui.ctx().clone();

        // Taille physique de la fenêtre, indépendante du zoom courant :
        // screen_rect rétrécit quand pixels_per_point augmente, le produit est stable.
        let physical_h = ctx.content_rect().height() * ctx.pixels_per_point();
        let native_ppp = ctx.native_pixels_per_point().unwrap_or(1.0);
        let target = (physical_h / native_ppp / 1080.0).clamp(0.5, 2.0);

        if (ctx.zoom_factor() - target).abs() > 0.01 {
            ctx.set_zoom_factor(target);
        }

        if ui.input(|i| i.key_pressed(egui::Key::F11)) {
            let full = ui.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Fullscreen(!full));
        }

        infos::show(ui, &self.game);

        actions.extend(board::show(
            ui,
            &self.game,
            &mut self.hex_size,
            &self.build_mode,
        ));

        actions.extend(dice::show(ui, &self.game, &mut self.last_roll));
        actions.extend(next_player::show(ui, &self.game));
        actions.extend(hand::show(ui, &self.game, &mut self.discard_selection));

        actions::show(ui, &self.game, &mut self.build_mode);

        end::show(ui, &self.game);

        let now = ui.input(|i| i.time);
        message::show(ui, &self.message);
        for action in actions {
            self.apply(action, now);
        }
    }
}
