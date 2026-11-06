// The one and only external API of this crate.
//
// Everything the library does downstream is stored in NodeIndex'es; the
// discovery engine never keeps vertex values of its own (the discovered
// graph owns them).  The implementer owns the knowledge of *who is out
// there*, handed to us one neighbors-vec at a time.

/// Enumerate neighbors of a given vertex, on demand.
///
/// The discovery engine calls it at most once per vertex (memoized by the
/// `DiscoverableGraph`).  It must be idempotent for a given vertex:
/// neighbors discovered are deltas merged into the graph "found so far".
pub trait NeighborEnumerator<T> {
    /// Enumerate the neighbors of a vertex.
    fn get_neighbors(&mut self, vertex: &T) -> Vec<T>;

    /// Validate a vertex before it enters the graph (optional).
    fn vertex_exists(&mut self, _vertex: &T) -> bool {
        true // Default implementation assumes all vertices exist
    }
}
