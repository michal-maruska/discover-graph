#[allow(dead_code)]

use crate::neighbors_enumerator::NeighborEnumerator;
use crate::discoverable_graph::DiscoverableGraph;

use petgraph::graph::{NodeIndex};
use petgraph::visit::{depth_first_search, DfsEvent, Control};
use petgraph::stable_graph::{StableDiGraph};

use tracing::debug;
use std::hash::Hash;


// none of the graph objects in petgraph suits us
// So we use the depth_first_search(), which is much
// more generic (`callback_based_traversal'), by providing it parameters based
// on our Graph:

// At the beginning we have an implicit graph. And we want
// to have it explicit (numbers, edges). So we visit every
// node and edges, gradually creating and using the explicit graph
// to avoid visiting more than once.
// At the end we can retrieve the graph.

// dynamic graph  <-----> some API .. possibly costly queries for neighbors.
//
// while we search, we construct the graph itself, by calling ...
// NeighborEnumerator -- the API

/// Driver of piecewise discovery.
///
/// A `GraphDiscoverer` owns one `DiscoverableGraph` and one session:
/// run `dfs_discover` (from as many roots as you like) while you keep
/// discovering, or go one-shot with `dfs_discover_graph`.
/// Either way the output is indices; the vertex *values* live in the graph.
pub struct GraphDiscoverer<T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: NeighborEnumerator<T>,
{
    dynamic_graph: DiscoverableGraph<T, P>,
}

// maybe  DynamicGraphDiscoverer

impl<T, P> GraphDiscoverer<T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: NeighborEnumerator<T>,
{
    pub fn new(provider: P) -> Self {
        Self {
            dynamic_graph: DiscoverableGraph::new(provider),
        }
    }

    /// Perform DFS from a seed vertex.  Incremental: the discoverer (and its
    /// discovered graph) stays alive for further `dfs_discover` runs, so you
    /// can feed several roots into the same graph over time.
    ///
    /// Returns the traversal order as `Vec<NodeIndex>` -- indices, not labels;
    /// weights are on the graph (`get_dynamic_graph`, or the one-shot door).
    pub fn dfs_discover(&mut self, start: T) -> Vec<NodeIndex> {
        let Some(start_node) = self.dynamic_graph.add_start_vertex(start) else {
            // the enumerator refused the seed
            return Vec::new();
        };
        self.run_dfs(start_node)
    }

    /// One-shot discovery: seed, walk (same DFS), and exit.
    ///
    /// Consumes the discoverer, discards the scaffolding (memoize-set,
    /// valueless index -- by then both encode nothing but what the graph
    /// itself shows) and hands over the canonical parts:
    /// the graph (owner of the vertex values) plus the `NeighborEnumerator`
    /// back, which was the external API all along and may carry session
    /// state of its own.
    pub fn dfs_discover_graph(mut self, start: T)
        -> (StableDiGraph<T, ()>, P, Vec<NodeIndex>)
    {
        let order = self.dfs_discover(start);
        let DiscoverableGraph { graph, provider, .. } = self.dynamic_graph;
        (graph.into_inner(), provider.into_inner(), order)
    }

    fn run_dfs(&mut self, start_node: NodeIndex) -> Vec<NodeIndex> {
        let mut order: Vec<NodeIndex> = Vec::new();

        // Now we can use petgraph's depth_first_search directly!
        // The IntoNeighbors implementation will handle discovery automatically
        depth_first_search(
            &self.dynamic_graph,
            Some(start_node),
            // notice: not moved: `order'
            |event| {
                match event {
                    DfsEvent::Discover(node_idx, _) => {
                        debug!("DFS discovered node: {:?}", node_idx);
                        Control::<()>::Continue // too bad not default for that B type.
                    },
                    DfsEvent::TreeEdge(from, to) => {
                        debug!("DFS tree edge: {:?} -> {:?}", from, to);
                        Control::Continue
                    },
                    DfsEvent::BackEdge(from, to) => {
                        debug!("DFS back edge: {:?} -> {:?}", from, to);
                        Control::Continue
                    },
                    DfsEvent::Finish(node_idx, _) => {
                        debug!("finished with node {:?}", node_idx);
                        order.push(node_idx);
                        Control::Continue
                    },
                    _ => Control::Continue,
                }
            });

        order
    }

    /// Get access to the dynamic wrapper (for further discovery operations)
    pub fn get_dynamic_graph(&self) -> &DiscoverableGraph<T, P> {
        &self.dynamic_graph
    }

    /// Get a mutable reference to the dynamic wrapper
    pub fn get_dynamic_graph_mut(&mut self) -> &mut DiscoverableGraph<T, P> {
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
