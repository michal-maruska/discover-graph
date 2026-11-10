# discover-graph
A standalone crate extending [Petgraph](https://github.com/petgraph/petgraph) with _dynamic_,
piecewise graph discovery. It defines the generic
NeighborEnumerator trait (the one external API: lazily enumerates a vertex's neighbors) and GraphDiscoverer,
which grows a StableGraph on demand via DFS, plus the discoverable_graph/ dynamic_bit_set helpers.
It's data-source agnostic (see GitHierarchyProvider in src/graph for one concrete enumerator of it).

Names:
- NeighborEnumerator -- the external API. Answers "neighbors of this vertex?" when asked.
- GraphDiscoverer    -- the driver. dfs_discover() = incremental (stays alive for more roots);
                       dfs_discover_graph() = one-shot, hands back (graph, enumerator, Vec<NodeIndex>).
- DiscoverableGraph  -- petgraph-traits shell over a graph that makes itself exist:
                       neighbors() first explodes the node through the enumerator, then walks edges.
- The output is indices. Vertex values (T) live exactly once, in the graph's node weights.
  Labels/lookup maps are a client-side concern, materialized from the graph after discovery.


see [git-hierarchy documentation](https://github.com/michal-maruska/git-hierarchy-rust/blob/main/description.md) for
extended guide how to use this.

