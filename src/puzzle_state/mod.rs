use std::collections::{HashMap, HashSet};
use super::geometry;

pub struct PuzzleState {
    triangle_reqs: Vec<u32>,
    unlocked_triangles: HashSet<usize>,
    connected_edges: HashSet<(u32, u32)>, // v0, v1 sorted
    connected_edges_by_vertex: HashMap<u32, HashSet<(u32, u32)>>,
}

impl PuzzleState {
    pub fn from_data(data: &geometry::PuzzleData) -> PuzzleState {
        PuzzleState {
            triangle_reqs: vec![3; data.num_triangles()],
            unlocked_triangles: HashSet::new(),
            connected_edges: HashSet::new(),
            connected_edges_by_vertex: HashMap::new(),
        }
    }

    pub fn connect_edge(&mut self, data: &geometry::PuzzleData, edge: &(u32, u32)) {
        let edge_ordered = if edge.0 > edge.1 { (edge.1, edge.0) } else { *edge };
        if !self.connected_edges.insert(edge_ordered) { return }
        self.connected_edges_by_vertex.entry(edge.0).or_insert(HashSet::new()).insert(edge_ordered);
        self.connected_edges_by_vertex.entry(edge.1).or_insert(HashSet::new()).insert(edge_ordered);

        if let Some(triangles_with_edge) = data.triangles_with_edge(&edge_ordered) {
            for &triangle in triangles_with_edge {
                self.triangle_reqs[triangle] -= 1;
                if self.triangle_reqs[triangle] == 0 { self.unlocked_triangles.insert(triangle); }
            }
        }
    }

    pub fn disconnect_edge(&mut self, data: &geometry::PuzzleData, edge: &(u32, u32)) {
        let edge_ordered = if edge.0 > edge.1 { (edge.1, edge.0) } else { *edge };
        if !self.connected_edges.remove(&edge_ordered) { return }
        self.connected_edges_by_vertex.entry(edge.0).and_modify(|e| { e.remove(&edge_ordered); });
        self.connected_edges_by_vertex.entry(edge.1).and_modify(|e| { e.remove(&edge_ordered); });

        if let Some(triangles_with_edge) = data.triangles_with_edge(&edge_ordered) {
            for &triangle in triangles_with_edge {
                if self.triangle_reqs[triangle] == 0 { self.unlocked_triangles.remove(&triangle); }
                self.triangle_reqs[triangle] += 1;
            }
        }
    }

    pub fn disconnect_from_vertex(&mut self, data: &geometry::PuzzleData, vertex: u32) {
        if let Some(edges) = self.connected_edges_by_vertex.get(&vertex) {
            for edge in edges.clone() {
                self.disconnect_edge(data, &edge);
            }
        }
    }

    pub fn is_finished(&self) -> bool { self.unlocked_triangles.len() == self.triangle_reqs.len() }
    pub fn get_connected_edges(&self) -> &HashSet<(u32, u32)> { &self.connected_edges }
    pub fn get_unlocked_triangles(&self) -> &HashSet<usize> { &self.unlocked_triangles }
}