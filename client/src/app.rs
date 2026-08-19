use std::sync::Arc;
use crate::panels::{message};
use crate::scenes::{connecting, lobby, playing, menu};
use crate::{dispatch, GameView, UiState};
use catan_protocol::{ClientMessage, GameInfo, PlayerInfo, ServerMessage};
use eframe::egui;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{watch, Notify};
use crate::ConnectionState;

pub(crate) enum AppState {
    Menu{ games : Vec<GameInfo>},
    Connecting,
    Lobby { players: Vec<PlayerInfo> },
    Playing(GameView),
}

#[derive(Debug)]
enum AppError {
    SenderIsFull,
    ServerOffline
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
            state: AppState::Connecting,
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

        if !matches!(*self.connection_state.borrow(), ConnectionState::Connected) {
            self.state = AppState::Connecting;
        }


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
            AppState::Menu {games} => {
                menu::show(ui, games, &mut messages);
            }
            AppState::Connecting => {
                let conn = *self.connection_state.borrow();
                connecting::show(ui, conn, &self.force_retry);
            }
            AppState::Lobby { players } => {
                lobby::show(ui, &self.ui, players, &mut messages);
            },
            AppState::Playing(view) => playing::show(ui, view, &mut self.ui, &mut messages),
            
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
