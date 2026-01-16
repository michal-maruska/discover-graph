
/// Trait for providing vertices and edges on demand
pub trait GraphProvider<T> {
    /// Get neighbors of a given vertex
    fn get_neighbors(&mut self, vertex: &T) -> Vec<T>;

    /// Check if a vertex exists (optional validation)
    fn vertex_exists(&mut self, _vertex: &T) -> bool {
        true // Default implementation assumes all vertices exist
    }
}

