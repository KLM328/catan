use crate::panels::{actions, board, dice, end, hand, infos, next_player, message};
use catan::{EdgeId, Game, GameError, GameStatus, Player, PlayerColor, PlayerId, ResourceCounts, Roll, Scenario, Steal, TileId, VertexId};
use eframe::egui;
use catan_protocol::{GameSnapshot, PlayerInfo};
use crate::{GameView, BuildMode};

pub enum UiAction {
    Roll,
    NextPlayer,
    BuildSettlement(VertexId),
    BuildRoad(EdgeId),
    UpgradeCity(VertexId),
    MoveRobber(TileId),
    Steal(Option<Steal>),
    Discard(ResourceCounts),
}

pub(crate) struct CatanApp {
    game: GameView,
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

        let mut game = Game::new(scenario);

        game.add_player(Player::new(PlayerColor::Red)).unwrap();
        game.add_player(Player::new(PlayerColor::White)).unwrap();
        game.add_player(Player::new(PlayerColor::Brown)).unwrap();


        while let Err(GameError::TiedRolls) =
            game.set_players_order(&game.sorted_player().iter().map(|&(id, _)| (id, Roll::random())).collect::<Vec<(PlayerId, Roll)>>())
        {}

        game.start(&terrains).expect("mise en place du plateau");

        Self {
            game : GameView::from(GameSnapshot {
                board : game.board().unwrap().clone(),
                scenario : Scenario::standard(),
                player_id : PlayerId::new(0),
                players : game.players().iter().map(|(&id, p)| PlayerInfo::from((p, id))).collect(),
                game_status : GameStatus::PlayingActions,
                turn_order : game.turn_order().to_vec(),
                current_turn : game.current_player_index(),
                hand : game.get_player(PlayerId::new(0)).unwrap().hand().clone(),

            }),
            hex_size: 80.0,
            last_roll: Some(Roll::new(4, 6).unwrap()),
            message: None,
            build_mode: BuildMode::None,
            discard_selection: ResourceCounts::default(),
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
            todo!()
        }
    }
}
