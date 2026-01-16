#[allow(dead_code)]

use crate::graph_provider::GraphProvider;
use crate::dynamic_graph::DynamicGraph;

use petgraph::graph::{NodeIndex};
use petgraph::visit::{depth_first_search, DfsEvent, Control};
use petgraph::stable_graph::{StableDiGraph};

use tracing::debug;
use std::collections::{HashMap};
use std::hash::Hash;

/// Main discoverer that uses the dynamic graph wrapper
/// to invoke  `depth_first_search()'
pub struct GraphDiscoverer<T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: GraphProvider<T>,
{
    dynamic_graph: DynamicGraph<T, P>,
}

impl<T, P> GraphDiscoverer<T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: GraphProvider<T>,
{
    pub fn new(provider: P) -> Self {
        Self {
            dynamic_graph: DynamicGraph::new(provider),
        }
    }

    /// Perform DFS using petgraph's depth_first_search with dynamic discovery
    pub fn dfs_discover(&mut self, start: T) -> Vec<T> {
        let start_node = match self.dynamic_graph.add_start_vertex(start) {
            // inside the graph.
            Some(node) => node,
            None => return Vec::new(),
        };

        let mut discovery_order = Vec::new();

        // Now we can use petgraph's depth_first_search directly!
        // The IntoNeighbors implementation will handle discovery automatically
        depth_first_search(&self.dynamic_graph, Some(start_node), |event| {
            match event {
                DfsEvent::Discover(node_idx, _) => {
                    if let Some(vertex) = self.dynamic_graph.get_vertex(node_idx) {
                        debug!("DFS discovered: {:?}", vertex);
                    }
                    Control::<()>::Continue // too bad not default for that B type.
                },
                DfsEvent::TreeEdge(from, to) => {
                    if let (Some(from_vertex), Some(to_vertex)) =
                        (self.dynamic_graph.get_vertex(from), self.dynamic_graph.get_vertex(to)) {
                            debug!("DFS tree edge: {:?} -> {:?}", from_vertex, to_vertex);
                    }
                    Control::Continue
                },
                DfsEvent::BackEdge(from, to) => {
                    if let (Some(from_vertex), Some(to_vertex)) =
                        (self.dynamic_graph.get_vertex(from), self.dynamic_graph.get_vertex(to)) {
                        debug!("DFS back edge: {:?} -> {:?}", from_vertex, to_vertex);
                    }
                    Control::Continue
                },
                DfsEvent::Finish(node_idx, _) => {
                    if let Some(vertex) = self.dynamic_graph.get_vertex(node_idx) {
                        debug!("finished with {:?}", vertex);
                        discovery_order.push(vertex.clone());
                    }
                    // self.dynamic_graph.graph.borrow().node_weight(node_idx));
                    Control::Continue
                },
                _ => Control::Continue,
            }
        });

        discovery_order
    }

    /// Alternative: Use petgraph's Dfs iterator directly
    pub fn dfs_discover_iterator(&mut self, start: T) -> Vec<T> {
        let start_node = match self.dynamic_graph.add_start_vertex(start) {
            Some(node) => node,
            None => return Vec::new(),
        };

        let mut discovery_order = Vec::new();
        let mut dfs = petgraph::visit::Dfs::new(&self.dynamic_graph, start_node);

        while let Some(node_idx) = dfs.next(&self.dynamic_graph) {
            if let Some(vertex) = self.dynamic_graph.get_vertex(node_idx) {
                discovery_order.push(vertex);
            }
        }

        discovery_order
    }

    /// Get the discovered graph - returns immutable reference to the underlying StableDiGraph
    ///
    /// This provides access to the petgraph data structure after discovery is complete.
    /// You can use this for:
    /// - Analyzing graph structure (node_count, edge_count, etc.)
    /// - Running other petgraph algorithms on the discovered graph
    /// - Accessing node weights and edge data
    /// - Manual traversal using petgraph's iterators
    ///
    /// Note: This returns a snapshot of the StableDiGraph, not the DynamicGraph wrapper,
    /// so it won't trigger further discovery if you traverse it.
    pub fn get_graph(&self) -> StableDiGraph<T, ()> {
        self.dynamic_graph.get_graph_snapshot()
    }


    pub fn get_provider(self) -> (P, HashMap<T, NodeIndex>) {
        // Ref<'_, T>
        return (
            self.dynamic_graph.provider.into_inner(),
            self.dynamic_graph.vertex_to_node.into_inner()
        )
    }


    /// Get access to the dynamic wrapper (for further discovery operations)
    pub fn get_dynamic_graph(&self) -> &DynamicGraph<T, P> {
        &self.dynamic_graph
    }

    /// Get a mutable reference to the dynamic wrapper
    pub fn get_dynamic_graph_mut(&mut self) -> &mut DynamicGraph<T, P> {
        &mut self.dynamic_graph
    }

    /// Get discovered vertices count
    pub fn discovered_count(&self) -> usize {
        self.dynamic_graph.discovered_count()
    }

    /// Get total edges count
    pub fn edges_count(&self) -> usize {
        self.dynamic_graph.edges_count()
    }
}
