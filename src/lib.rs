mod dynamic_bit_set;

mod graph_provider;
pub use graph_provider::GraphProvider;

mod dynamic_graph;

mod graph_discoverer;
pub use graph_discoverer::GraphDiscoverer;


// mod simple {
// fixme: for testing only!
// Example implementation of GraphProvider for a simple numeric graph
pub struct SimpleGraphProvider {
    max_depth: usize,
    branching_factor: usize,
}

impl SimpleGraphProvider {
    pub fn new(max_depth: usize, branching_factor: usize) -> Self {
        Self { max_depth, branching_factor }
    }
}

impl GraphProvider<i32> for SimpleGraphProvider {
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

    #[test]
    fn test_dynamic_discovery() {
        let provider = SimpleGraphProvider::new(2, 2);
        let mut discoverer = GraphDiscoverer::new(provider);

        let result = discoverer.dfs_discover(1);
        assert!(!result.is_empty());
        assert!(result.contains(&1));
        assert!(discoverer.discovered_count() > 0);
    }


    /*
    #[ignore]
    #[test]
    fn test_external_provider_discovery() {
        let provider = ExternalDataProvider::new();
        let mut discoverer = GraphDiscoverer::new(provider);

        let result = discoverer.dfs_discover("root".to_string());
        assert!(result.contains(&"root".to_string()));
        assert!(result.len() > 1);
    }
    */

    #[test]
    fn test_dfs_iterator() {
        let provider = SimpleGraphProvider::new(2, 2);
        let mut discoverer = GraphDiscoverer::new(provider);

        let result = discoverer.dfs_discover_iterator(1);
        assert!(!result.is_empty());
        assert!(result.contains(&1));
    }
}
