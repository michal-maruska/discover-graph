mod dynamic_bit_set;

mod neighbors_enumerator;
pub use neighbors_enumerator::NeighborEnumerator;

mod discoverable_graph;

mod graph_discoverer;
pub use graph_discoverer::GraphDiscoverer;

// mod simple {
// fixme: for testing only!
// Example implementation of a NeighborEnumerator for a simple numeric graph
pub struct SimpleNeighborEnumerator {
    max_depth: usize,
    branching_factor: usize,
}

impl SimpleNeighborEnumerator {
    pub fn new(max_depth: usize, branching_factor: usize) -> Self {
        Self { max_depth, branching_factor }
    }
}

impl NeighborEnumerator<i32> for SimpleNeighborEnumerator {
    fn get_neighbors(&mut self, vertex: &i32) -> Vec<i32> {
        if *vertex >= (self.max_depth as i32).pow(self.branching_factor as u32) {
            return Vec::new();
        }

        (1..=self.branching_factor)
            .map(|i| vertex * self.branching_factor as i32 + i as i32)
            .collect()
    }

    fn vertex_exists(&mut self, vertex: &i32) -> bool {
        *vertex >= 0 && *vertex <= 1000
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // discovery hands out indices; values live on the graph
    #[test]
    fn test_dynamic_discovery() {
        let enumerator = SimpleNeighborEnumerator::new(2, 2);
        let mut discoverer = GraphDiscoverer::new(enumerator);

        let order = discoverer.dfs_discover(1);
        assert!(!order.is_empty());
        assert!(discoverer.discovered_count() > 0);

        let (graph, _enumerator, order) = discoverer.dfs_discover_graph(1);
        assert!(order.iter().any(|&i| graph[i] == 1));
    }
}
