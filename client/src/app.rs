use std::ops::Not;
use std::sync::Arc;
use std::time::Instant;
use crate::panels::message;
use crate::scenes::{connecting, lobby, playing, menu};
use crate::{dispatch, BuildMode, GameView};
use catan::{PlayerId, ResourceCounts, Roll};
use catan_protocol::{ClientMessage, PlayerInfo, ServerMessage};
use eframe::egui;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{watch, Notify};

pub(crate) enum AppState {
    Menu,
    Connecting,
    Lobby { players: Vec<PlayerInfo> },
    Paused(GameView),
    Playing(GameView),
}

#[derive(Clone, Copy)]
pub(crate) enum ConnectionState {
    NotConnected,
    Connecting { attempt: u32 },
    Connected,
    Retrying { at: Instant, attempt: u32 },
}

#[derive(Debug)]
enum AppError {
    SenderIsFull,
    ServerOffline
}

pub(crate) struct UiState {
    hex_size: f32,
    last_roll: Option<Roll>,
    message: Option<(String, f64)>,
    build_mode: BuildMode,
    discard_selection: ResourceCounts,
    rolls_display: Option<(Vec<(PlayerId, Roll)>, f64)>,
}

impl UiState {
    pub(crate) fn adjust_hex_size_with_scroll(&mut self, scroll: f32) {
        self.hex_size = (self.hex_size * (1.0 + scroll * 0.002)).clamp(20.0, 200.0);
    }

    pub(crate) fn hex_size(&self) -> f32 {
        self.hex_size
    }

    pub(crate) fn build_mode(&self) -> BuildMode {
        self.build_mode
    }

    pub(crate) fn last_roll(&self) -> Option<Roll> {
        self.last_roll
    }

    pub(crate) fn discard_selection(&self) -> &ResourceCounts {
        &self.discard_selection
    }

    pub(crate) fn add_discard_selection(&mut self, resource: ResourceCounts) {
        self.discard_selection.add(&resource);
    }

    pub(crate) fn remove_discard_selection(&mut self, resource: ResourceCounts) {
        self.discard_selection.remove(&resource);
    }

    pub(crate) fn switch_build_mode(&mut self, mode: BuildMode) {
        self.build_mode = if self.build_mode == mode {
            BuildMode::None
        } else {
            mode
        };
    }

    pub(crate) fn message(&self) -> Option<(String, f64)> {
        self.message.clone()
    }

    pub(crate) fn set_message(&mut self, message: String, now : f64) {
        self.message = Some((message, now));
    }

    pub(crate) fn set_last_roll(&mut self, roll: Roll) {
        self.last_roll = Some(roll);
    }

    pub(crate) fn rolls_display(&self) -> Option<(Vec<(PlayerId, Roll)>, f64)> {
        self.rolls_display.clone()
    }

    pub(crate) fn set_rolls_display(&mut self, rolls: Vec<(PlayerId, Roll)>, now : f64) {
        self.rolls_display = Some((rolls, now));
    }
    
    pub(crate) fn reset_discard_selection(&mut self)  {
        self.discard_selection = ResourceCounts::default()
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            hex_size: 80.0,
            last_roll: None,
            message: None,
            build_mode: BuildMode::None,
            discard_selection: Default::default(),
            rolls_display: None,
        }
    }
}

pub(crate) struct CatanApp {
    state: AppState,
    ui: UiState,
    tx: Sender<ClientMessage>,
    rx: Receiver<ServerMessage>,
    connection_state: watch::Receiver<ConnectionState>,
    force_retry : Arc<Notify>
}

impl CatanApp {
    pub(crate) fn new(tx: Sender<ClientMessage>, rx: Receiver<ServerMessage>, connection_state : watch::Receiver<ConnectionState>, force_retry : Arc<Notify>) -> Self {
        Self {
            state: AppState::Menu,
            ui: UiState::default(),
            tx,
            rx,
            connection_state,
            force_retry
        }
    }

    fn send(&self, message: ClientMessage) -> Result<(), AppError> {
        match self.tx.try_send(message) {
            Ok(_) => Ok(()),
            Err(TrySendError::Closed(_)) => {
                Err(AppError::ServerOffline)
            }
            Err(TrySendError::Full(_)) => Err(AppError::SenderIsFull),
        }
    }

}

impl eframe::App for CatanApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let mut messages = Vec::new();

        while let Ok(server_message) = self.rx.try_recv() {
            let now = ui.input(|i| i.time);
            dispatch::apply(&mut self.state, &mut self.ui, server_message, &mut messages, now);
        }
        let ctx = ui.ctx().clone();

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

        match &self.state {
            AppState::Menu => {
                menu::show(ui, &mut self.state, &mut messages);
            }
            AppState::Connecting => {
                let conn = *self.connection_state.borrow();
                connecting::show(ui, conn, &self.force_retry);
            }
            AppState::Lobby { players } => {
                lobby::show(ui, &self.ui, players, &mut messages);
            },
            AppState::Playing(view) => playing::show(ui, view, &mut self.ui, &mut messages),

            AppState::Paused(_) => {
                todo!()
            }
        }

        message::show(ui, &self.ui);

        while let Some(message) = messages.first().cloned() {
            match self.send(message) {
                Ok(_) => {
                    messages.remove(0);
                }
                Err(_) => {
                    break;
                }
            }
        }
    }
}
