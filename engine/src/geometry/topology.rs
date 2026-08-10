use std::cmp::PartialEq;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::geometry::{Hex, HexCorner, HexDirection};
use crate::{EdgeId, VertexId};

#[cfg(test)]
use crate::TileId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topology{
    hexes: Vec<Hex>,
    tile_vertices: Vec<[VertexId; 6]>,
    tile_edges: Vec<[EdgeId; 6]>,
    edges_endpoints: Vec<[VertexId; 2]>,
    vertex_hexes : Vec<[Hex; 3]>,
    vertex_count: usize,
    edge_count: usize,
}

impl Topology {

    fn new() -> Topology{
        Topology{hexes: Vec::new(), tile_vertices: Vec::new(), tile_edges: Vec::new(), edges_endpoints : Vec::new(), vertex_hexes : Vec::new(), vertex_count: 0, edge_count: 0}
    }

    pub fn hexes(&self) -> &[Hex] {
        &self.hexes
    }

    pub fn tile_vertices(&self) -> &[[VertexId; 6]] {
        &self.tile_vertices
    }
    pub fn tile_edges(&self) -> &[[EdgeId; 6]] {
        &self.tile_edges
    }
    pub(crate) fn vertex_count(&self) -> usize {
        self.vertex_count
    }
    pub(crate) fn edge_count(&self) -> usize {
        self.edge_count
    }
    pub(crate) fn edges_endpoints(&self) -> &[[VertexId; 2]] {
        &self.edges_endpoints
    }

    pub(crate) fn vertex_hexes(&self) -> &[[Hex; 3]] {
        &self.vertex_hexes
    }

    pub(crate) fn vertex_neighbors(&self, vertex_id : VertexId) -> Vec<VertexId> {
        self.edges_endpoints.iter().filter(|[a, b]| *a == vertex_id || *b == vertex_id ).map(|[a, b]| if *a == vertex_id { *b } else { *a }).collect::<Vec<VertexId>>()
    }



    pub(crate) fn from_hexes(hexes : &[Hex]) -> Topology{
        let mut topology = Topology::new();
        topology.hexes = hexes.to_vec();
        let mut vertices: HashMap<[Hex; 3], VertexId> = HashMap::new();
        let mut edges: HashMap<[Hex; 2], EdgeId> = HashMap::new();
        let mut edges_endpoints : Vec<[VertexId; 2]> = Vec::new();
        let mut vertex_hexes : Vec<[Hex; 3]> = Vec::new();

        for hex in hexes {
            let mut new_tile_for_vertices: [VertexId; 6] = [VertexId::new(0); 6];
            let mut new_tile_for_edges : [EdgeId; 6] = [EdgeId::new(0); 6];

            for corner in HexCorner::ALL {
                let new_index_vertices = vertices.len();
                let key = hex.corner_hexes(corner);
                let vertex_id = *vertices.entry(key).or_insert(VertexId::new(new_index_vertices));
                new_tile_for_vertices[corner.value()] = vertex_id;

                if vertex_id.value() == vertex_hexes.len(){
                    vertex_hexes.push(key)
                }

            }
            for dir in HexDirection::ALL {
                let new_index_edges = edges.len();
                let edge_id = *edges.entry(hex.edge_hexes(dir)).or_insert(EdgeId::new(new_index_edges));
                new_tile_for_edges[dir.value()] = edge_id;
                if edge_id.value() == edges_endpoints.len() {
                    let mut endpoints = [new_tile_for_vertices[(dir.value() + 5) % 6],
                        new_tile_for_vertices[dir.value()]];
                    endpoints.sort();
                    edges_endpoints.push(endpoints);
                }
            }
            topology.tile_vertices.push(new_tile_for_vertices);
            topology.tile_edges.push(new_tile_for_edges);
        }
        topology.vertex_count = vertices.len();
        topology.edge_count = edges.len();
        topology.edges_endpoints = edges_endpoints;
        topology.vertex_hexes = vertex_hexes;
        topology

    }

    pub(crate) fn spiral(radius: i8) -> Vec<Hex> {
        let mut hexes = Vec::new();
        for ring in (1..=radius).rev() {
            let mut hex = Hex::new(-ring, ring);
            for direction in HexDirection::ALL {
                for _ in 0..ring {
                    hexes.push(hex);
                    hex = hex.neighbor(direction);
                }
            }
        }
        hexes.push(Hex::new(0, 0));
        hexes
    }

    #[cfg(test)]
    pub(crate) fn adjacent_tile_pairs(&self) -> Vec<(TileId, TileId)> {
        let mut owners: Vec<Vec<TileId>> = vec![Vec::new(); self.edge_count];

        for (tile, edges) in self.tile_edges.iter().enumerate() {
            for edge in edges {
                owners[edge.value()].push(TileId::new(tile));
            }
        }

        owners.iter()
            .filter(|tiles| tiles.len() == 2)
            .map(|tiles| (tiles[0], tiles[1]))
            .collect()
    }


    #[cfg(test)]
    pub(crate) fn test_topology() -> Topology{
        let mut hexes = Vec::new();
        hexes.push(Hex::new(0, 0));
        hexes.push(hexes[0].neighbor(HexDirection::new(0).unwrap()));
        hexes.push(hexes[0].neighbor(HexDirection::new(1).unwrap()));
        Topology::from_hexes(&hexes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_from_hexes_radius_2() {
        let hexes = Topology::spiral(2);
        let topo = Topology::from_hexes(&hexes);
        assert_eq!(topo.tile_vertices.len(), 19);
        assert_eq!(topo.vertex_count, 54);
        assert_eq!(topo.edge_count, 72);
    }

    #[test]
    fn test_from_hexes_radius_3() {
        let hexes = Topology::spiral(3);
        let topo = Topology::from_hexes(&hexes);
        assert_eq!(topo.tile_vertices.len(), 37);
        assert_eq!(topo.vertex_count, 96);
        assert_eq!(topo.edge_count, 132);
    }

    #[test]
    fn test_hexagon_radius_2() {
        let hexes = Topology::spiral(2);
        println!("{:?}", hexes);
        assert_eq!(hexes.len(), 19);

    }

    #[test]
    fn test_hexagon_radius_1() {
        let hexes = Topology::spiral(1);
        println!("{:?}", hexes);
        assert_eq!(hexes.len(), 7);
    }

    #[test]
    fn test_edges_endpoints() {
        let hexes = Topology::spiral(2);
        let topo = Topology::from_hexes(&hexes);
        assert_eq!(topo.edges_endpoints.len(), topo.edge_count());
        let topo = Topology::from_hexes(&[Hex::new(0, 0)]);
        assert_eq!(topo.edges_endpoints, vec![
            [VertexId::new(0), VertexId::new(5)],
            [VertexId::new(0), VertexId::new(1)],
            [VertexId::new(1), VertexId::new(2)],
            [VertexId::new(2), VertexId::new(3)],
            [VertexId::new(3), VertexId::new(4)],
            [VertexId::new(4), VertexId::new(5)],
        ]);
    }

    #[test]
    fn every_vertex_has_two_or_three_edges() {
        let topo = Topology::from_hexes(&Topology::spiral(2));
        let mut degree = vec![0usize; topo.vertex_count()];
        for e in &topo.edges_endpoints {
            assert_ne!(e[0], e[1], "une arête ne peut pas relier un sommet à lui-même");
            for v in e { degree[v.value()] += 1; }
        }
        assert!(degree.iter().all(|&d| d == 2 || d == 3));
        assert_eq!(degree.iter().filter(|&&d| d == 2).count(), 18);
        assert_eq!(degree.iter().filter(|&&d| d == 3).count(), 36);
    }

    #[test]
    fn every_tile_agrees_on_edge_endpoints() {
        let topo = Topology::from_hexes(&Topology::spiral(2));
        for (tile, edges) in topo.tile_edges().iter().enumerate() {
            let vertices = topo.tile_vertices()[tile];
            for dir in 0..6 {
                let mut expected = [vertices[(dir + 5) % 6], vertices[dir]];
                expected.sort();
                assert_eq!(topo.edges_endpoints[edges[dir].value()], expected,
                           "tuile {tile}, direction {dir}");
            }
        }
    }

    #[test]
    fn adjacent_pairs_match_hex_geometry() {
        let topo = Topology::from_hexes(&Topology::spiral(2));
        let pairs = topo.adjacent_tile_pairs();

        assert_eq!(pairs.len(), 42);

        for (a, b) in pairs {
            let (ha, hb) = (topo.hexes()[a.value()], topo.hexes()[b.value()]);
            assert!(HexDirection::ALL.into_iter().any(|d| ha.neighbor(d) == hb),
                    "{ha:?} et {hb:?} ne sont pas voisins");
        }
    }

    #[test]
    fn test_vertex_neighbors() {
        let topo = Topology::from_hexes(&Topology::spiral(2));

        let mut neighbors = topo.vertex_neighbors(VertexId::new(0));
        neighbors.sort();
        assert_eq!(neighbors, vec![VertexId::new(1), VertexId::new(5), VertexId::new(7)]);
    }


}