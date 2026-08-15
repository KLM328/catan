use eframe::egui::Color32;
use catan::{Player, PlayerColor, Resource, Terrain};
use catan_protocol::PlayerInfo;

pub(crate) const OUTLINE: Color32 = Color32::from_rgb(38, 34, 30);
pub(crate) const CARD_W: f32 = 100.0;
pub(crate) const CARD_H: f32 = 155.0;
pub(crate) const GAP: f32 = 10.0;
pub(crate) const SIDE_PANEL_W: f32 = 300.0;

pub(crate) const BUTTON_W_H : f32 = 100.0;

pub(crate) fn resource_color(r: Resource) -> Color32 {
    match r {
        Resource::Wood => terrain_color(Terrain::Forest),
        Resource::Brick => terrain_color(Terrain::Hills),
        Resource::Stone => terrain_color(Terrain::Mountain),
        Resource::Wheat => terrain_color(Terrain::Fields),
        Resource::Wool => terrain_color(Terrain::Pasture),
    }
}

pub(crate) fn player_color(player: &PlayerInfo) -> Color32 {
    match player.color() {
        PlayerColor::Blue => Color32::from_rgb(0, 0, 250),
        PlayerColor::Red => Color32::from_rgb(185, 5, 20),
        PlayerColor::White => Color32::from_rgb(255, 255, 210),
        PlayerColor::Orange => Color32::from_rgb(255, 110, 0),
        PlayerColor::Green => Color32::from_rgb(0, 153, 0),
        PlayerColor::Brown => Color32::from_rgb(110, 80, 5)
    }
}

pub(crate) fn terrain_color(terrain: Terrain) -> Color32 {
    match terrain {
        Terrain::Desert => Color32::from_rgb(224, 201, 138),
        Terrain::Forest => Color32::from_rgb(45, 90, 39),
        Terrain::Mountain => Color32::from_rgb(122, 122, 140),
        Terrain::Hills => Color32::from_rgb(181, 90, 48),
        Terrain::Pasture => Color32::from_rgb(143, 193, 93),
        Terrain::Fields => Color32::from_rgb(232, 193, 74),
    }
}