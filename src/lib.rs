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

    // Enumerator that knows its universe: sweep discovery must reach every
    // vertex, unify shared neighbors into one node, and call the
    // enumerator exactly once per vertex.
    struct KnownUniverse {
        universe: Vec<i32>,
        calls: std::cell::Cell<usize>,
    }

    impl NeighborEnumerator<i32> for KnownUniverse {
        fn get_neighbors(&mut self, vertex: &i32) -> Vec<i32> {
            self.calls.set(self.calls.get() + 1);
            match *vertex {
                1 => vec![2, 3],
                2 => vec![3],
                _ => Vec::new(),
            }
        }

        fn known_vertices(&mut self) -> Option<Vec<i32>> {
            Some(self.universe.clone())
        }
    }

    #[test]
    fn test_whole_graph_discovery() {
        // only vertex 1 seeded; 2 and 3 must be discovered via sweep
        let enumerator = KnownUniverse { universe: vec![1], calls: std::cell::Cell::new(0) };
        let (graph, enumerator) = GraphDiscoverer::new(enumerator).discover_whole_graph();

        assert_eq!(graph.node_count(), 3);      // 1, 2, 3 (3 unified: reached twice)
        assert_eq!(graph.edge_count(), 3);      // 1->2, 1->3, 2->3
        assert_eq!(enumerator.calls.get(), 3);  // exactly once per vertex
    }

    #[test]
    fn test_whole_graph_discovery_dedup_seed() {
        // seeding the same universe twice must not duplicate anything
        let enumerator = KnownUniverse { universe: vec![1, 1, 2], calls: std::cell::Cell::new(0) };
        let (graph, enumerator) = GraphDiscoverer::new(enumerator).discover_whole_graph();

        assert_eq!(graph.node_count(), 3);
        assert_eq!(enumerator.calls.get(), 3);
    }
}
