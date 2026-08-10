use serde::{Deserialize, Serialize};
use crate::Topology;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TileId(usize);

impl TileId {
    pub fn new(index: usize) -> TileId {
        TileId(index)
    }
    pub(crate) fn value(&self) -> usize {
        self.0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct VertexId(usize);

impl VertexId {
    pub fn new(index: usize) -> VertexId {
        VertexId(index)
    }

    pub(crate) fn value(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct EdgeId(usize);

impl EdgeId {
    pub fn value(&self) -> usize {
        self.0
    }
}

impl EdgeId {
    pub fn new(index: usize) -> EdgeId {
        EdgeId(index)
    }
}

pub trait ConnectedEdges {
    fn connected_edges(self, topology: &Topology) -> Vec<EdgeId>;
}

impl ConnectedEdges for VertexId {
    fn connected_edges(self, topology: &Topology) -> Vec<EdgeId> {
        topology.edges_endpoints().iter().enumerate().filter(|(_, vertices)| vertices.contains(&self) ).map(|(index, _)| EdgeId::new(index)).collect::<Vec<EdgeId>>()
    }
}

impl ConnectedEdges for EdgeId {
    fn connected_edges(self, topology: &Topology) -> Vec<EdgeId> {
        let option_endpoint = topology.edges_endpoints().get(self.value());
        let mut connected_edges : Vec<EdgeId> = Vec::new();
        if let Some(option_endpoint) = option_endpoint {
            option_endpoint.iter().for_each(|&v| connected_edges.extend(v.connected_edges(topology)));
        }
        connected_edges
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connected_edges() {
        let topo = Topology::from_hexes(&Topology::spiral(2));
        let mut edges = VertexId::new(0).connected_edges(&topo);
        edges.sort();
        assert_eq!(edges, vec![EdgeId::new(0), EdgeId::new(1), EdgeId::new(8)]);
    }
}