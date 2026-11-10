use petgraph::graph::{NodeIndex,DefaultIx};
use petgraph::visit::{IntoNeighbors, GraphBase, Visitable};
use petgraph::stable_graph::{StableDiGraph};

use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::cell::RefCell;

use fixedbitset::FixedBitSet;

use tracing::debug;

// ::crate::    global paths cannot start with `crate`
use crate::dynamic_bit_set::DynamicBitSet;
use crate::neighbors_enumerator::NeighborEnumerator;

/// A petgraph-compatible graph that **makes itself exist**,
/// vertex by vertex, edge by edge -- on demand.
///
/// It implements petgraph's traits (`GraphBase`, `Visitable`, `IntoNeighbors`)
/// so generic petgraph algorithms can treat it as an ordinary graph.
/// The trick: `neighbors(node)` does not answer from existing edges;
/// it *first* explodes the node -- calls the `NeighborEnumerator` (the one
/// and only external API) with the node's canonical vertex -- registers the
/// returned neighbors, *then* iterates the now-materialized edges.
///
/// Canonical storage:
/// - `graph` owns the one-and-only copy of every vertex value (`T`) as the
///   node weight.  The graph itself therefore *is* the reverse map
///   (`NodeIndex` -> vertex, via the weight).
/// - `vertex_to_node`: valueless index, vertex-hash -> candidate NodeIndex
///   (equality is resolved against the canonical weight kept by `graph`).
/// - `discovered`: NodeIndex-keyed set of already-exploded nodes.
///
/// Consumption: once you are done discovering, take `graph` and `provider`
/// out (the `RefCell`s die with the wrapper); the scaffolding
/// (`vertex_to_node`, `discovered`) holds nothing canonical and is dropped.
pub struct DiscoverableGraph<T, P>
where
    // Vertex type:
    T: Clone + Eq + Hash + std::fmt::Debug, // so String? Do I use another indirection?
    //
    P: NeighborEnumerator<T>,
{
    pub(crate) graph: RefCell<StableDiGraph<T, ()>>, // on indices
    // and the single owner of the `T's

    vertex_to_node: RefCell<HashMap<u64, Vec<NodeIndex>>>,
    // external vertex identity, without owning it:
    // hash(query) -> candidates, Eq resolved against the graph weights.

    // have this node's neighbors ever been requested from the enumerator?
    // NodeIndex-keyed (identity, not value) -- and a bitset, because the
    // NodeId space is dense: StableGraph never reuses nor renumbers slots.
    discovered: RefCell<FixedBitSet>,
    pub(crate) provider: RefCell<P>,
}

impl<T, P> DiscoverableGraph<T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: NeighborEnumerator<T>,
{
    pub fn new(provider: P) -> Self {
        Self {
            graph: RefCell::new(StableDiGraph::new()),
            vertex_to_node: RefCell::new(HashMap::new()),
            discovered: RefCell::new(FixedBitSet::with_capacity(0)),
            provider: RefCell::new(provider),
        }
    }

    /// Hash of a vertex value.  Deterministic within the process
    /// (`DefaultHasher::new()` is randomness-free), used as the bucket
    /// key of the valueless vertex index.
    fn vertex_hash(vertex: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        vertex.hash(&mut hasher);
        hasher.finish()
    }

    /// Get or create a node in the graph for the given vertex.
    ///
    /// Zero copies of `T` in either branch:
    /// - hit:  Eq is resolved against the canonical weight inside `graph`;
    ///         the (expendable, owned) argument is simply dropped.
    /// - miss: the argument is *moved* into the graph, which keeps the only
    ///         copy; the index records just its hash + the fresh NodeIndex.
    fn get_or_create_node(&self, vertex: T) -> NodeIndex {
        let hash = Self::vertex_hash(&vertex);

        // hit check -- pure shared borrows, no allocation at all
        {
            let buckets = self.vertex_to_node.borrow();
            if let Some(candidates) = buckets.get(&hash) {
                let graph = self.graph.borrow();
                if let Some(idx) = candidates.iter().copied()
                    .find(|&i| graph[i] == vertex)
                {
                    return idx;
                }
            }
        }

        // miss -- the vertex moves in; the graph owns it uniquely, forever
        let node_idx = self.graph.borrow_mut().add_node(vertex);
        self.vertex_to_node.borrow_mut().entry(hash).or_default().push(node_idx);
        node_idx
    }

    /// Explode node: register all edges out (along with the vertices-neighbors).
    ///
    /// Called exactly once per node, lazily: by the `DiscoveringNeighborsIter`
    /// on its first `next()`.  Identity is the NodeIndex: the graph owns the
    /// vertex value, so the enumerator is called with a borrow straight out
    /// of the canonical weight.
    pub(crate) fn explode_node(&self, node_idx: NodeIndex) {
        // Check if already exploded, atomically in one borrow.
        // The bitset grows on demand (node slots are appended -> widen
        // the one node whose explosion we are doing right now).
        let idx = node_idx.index();
        let mut discovered = self.discovered.borrow_mut();
        if idx >= discovered.len() {
            discovered.grow(idx + 1);
        }
        if discovered.contains(idx) {
            return;
        }
        discovered.insert(idx);
        drop(discovered);

        // Get neighbors from the enumerator -- borrowing the vertex from the
        // graph; no copy crosses into or out of this block (T stays put).
        let neighbors = {
            let graph = self.graph.borrow();
            let vertex = &graph[node_idx];
            debug!("Exploding node {:?} with vertex {:?}", node_idx, vertex);
            self.provider.borrow_mut().get_neighbors(vertex)
        };

        // Mark as explored -- done above, in the same borrow: the check
        // (contains) and the registration (insert) are one atomic step,
        // and the enumerator runs exactly once per node, ever.

        debug!("Explosion of node {:?} yielded {} neighbors", node_idx, neighbors.len());

        // Add neighbors to graph
        for neighbor in neighbors {
            if self.provider.borrow_mut().vertex_exists(&neighbor) {
                // external node -> graph node (the owned `neighbor' moves in)
                // this is the `unification':
                let neighbor_node = self.get_or_create_node(neighbor);

                // Add edge if not yet registered
                let mut graph = self.graph.borrow_mut();
                if graph.find_edge(node_idx, neighbor_node).is_none() {
                    debug!("Adding edge");
                    graph.add_edge(node_idx, neighbor_node, ()); // could add a name/weight
                }
            }
        }
    }

    /// Add a seed vertex: drop a vertex into the graph *before* any
    /// traversal (the glossary's "seed" -- many seeds are allowed over a
    /// session; "start" would suggest exactly one).
    pub fn add_seed(&self, vertex: T) -> Option<NodeIndex> {
        if self.provider.borrow_mut().vertex_exists(&vertex) {
            Some(self.get_or_create_node(vertex))
        } else {
            None
        }
    }

    pub fn get_vertex(&self, node_idx: NodeIndex) -> Option<T> {
        let graph = self.graph.borrow();
        graph.node_weight(node_idx).cloned()
    }

    pub fn get_graph_snapshot(&self) -> StableDiGraph<T, ()> {
        self.graph.borrow().clone()
    }

    pub fn discovered_count(&self) -> usize {
        // fixedbitset caches only the bit *width*, not the popcount:
        // sum the set bits over the words (Block = u32 in 0.5).
        self.discovered.borrow().as_slice()
            .iter()
            .map(|w| w.count_ones() as usize)
            .sum()
    }

    pub fn edges_count(&self) -> usize {
        self.graph.borrow().edge_count()
    }
}

// Implement GraphBase for our wrapper
impl<T, P> GraphBase for DiscoverableGraph<T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: NeighborEnumerator<T>,
{
    type NodeId = NodeIndex;
    type EdgeId = petgraph::graph::EdgeIndex;
}

// Implement Visitable for our wrapper with dynamic bit set
impl<T, P> Visitable for DiscoverableGraph<T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: NeighborEnumerator<T>,
{
    type Map = DynamicBitSet;

    fn visit_map(&self) -> Self::Map {
        // Create a bit set with current graph capacity
        DynamicBitSet::with_capacity(self.graph.borrow().node_count())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.reset();
    }
}

/// Lazily-iterated neighbors of a node.  An `Iterator`, yes -- but one whose
/// very *first* `next()` causes the node to be exploded (the enumerator
/// runs, edges materialize) and *only then* walks those edges.
pub struct DiscoveringNeighborsIter<'a, T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: NeighborEnumerator<T>,
{
    d_graph: &'a DiscoverableGraph<T, P>,
    inner: petgraph::stable_graph::WalkNeighbors<DefaultIx>, // petgraph::Directed  Neighbors
    node_idx: NodeIndex,
    exploded: bool,
}


impl<'a, T, P> Iterator for DiscoveringNeighborsIter<'a, T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: NeighborEnumerator<T>,
{
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        // Trigger explosion on first access
        if !self.exploded {
            self.d_graph.explode_node(self.node_idx);
            self.exploded = true;

            self.inner = self.d_graph.graph.borrow().neighbors(self.node_idx).detach();
        }

        self.inner.next_node(&*self.d_graph.graph.borrow()) // wtf!
    }
}

// Implement IntoNeighbors for our &wrapper - this is the key!
impl<'a, T, P> IntoNeighbors for &'a DiscoverableGraph<T, P>
where
    T: Clone + Eq + Hash + std::fmt::Debug,
    P: NeighborEnumerator<T>,
{
    type Neighbors = DiscoveringNeighborsIter<'a, T, P>;

    fn neighbors(self, node_idx: Self::NodeId) -> Self::Neighbors {
        DiscoveringNeighborsIter {
            d_graph: self,
            inner: self.graph.borrow().neighbors(node_idx).detach(),
            node_idx,
            exploded: false,
        }
    }
}
