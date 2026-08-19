use catan::{PlayerId, ResourceCounts, Roll};
use crate::panels::board::BuildMode;

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