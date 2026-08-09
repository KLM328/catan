use catan::{EdgeId, PlayerId, ResourceCounts, TileId, VertexId};

pub enum ClientMessage {
    BuildRoad(EdgeId),
    BuildSettlement(VertexId),
    UpgradeCity(VertexId),
    Discard(ResourceCounts),
    Steal(Option<PlayerId>),
    RobberLocation(TileId),
    Roll,
    EndTurn,
    Join //à réfléchir plus en détails plus tard

}