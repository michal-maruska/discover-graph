use discover_graph::{GraphDiscoverer, NeighborEnumerator};

use petgraph::graph::NodeIndex;

use discover_graph::SimpleNeighborEnumerator;

// Example with external data source
pub struct ExternalNeighborEnumerator {
    call_count: usize,
}

impl ExternalNeighborEnumerator {
    pub fn new() -> Self {
        Self { call_count: 0 }
    }

    fn fetch_neighbors(&mut self, vertex: &String) -> Vec<String> {
        self.call_count += 1;
        println!("API call #{}: fetching neighbors for '{}'", self.call_count, vertex);

        match vertex.as_str() {
            "root" => vec!["A".to_string(), "B".to_string(), "C".to_string()],
            "A" => vec!["A1".to_string(), "A2".to_string()],
            "B" => vec!["B1".to_string(), "B2".to_string(), "B3".to_string()],
            "C" => vec!["C1".to_string()],
            "A1" => vec!["A1a".to_string()],
            "A2" => vec!["A2a".to_string(), "A2b".to_string()],
            _ => Vec::new(),
        }
    }
}

impl NeighborEnumerator<String> for ExternalNeighborEnumerator {
    fn get_neighbors(&mut self, vertex: &String) -> Vec<String> {
        std::thread::sleep(std::time::Duration::from_millis(10));
        self.fetch_neighbors(vertex)
    }

    fn vertex_exists(&mut self, vertex: &String) -> bool {
        !vertex.is_empty() && vertex.len() <= 10
    }
}

fn demonstrate_graph_access() {
    println!("=== Understanding Graph Access Patterns ===");

    let enumerator = ExternalNeighborEnumerator::new();
    let discoverer = GraphDiscoverer::new(enumerator);

    // 1. One-shot discovery: indices out, canonical graph + enumerator back.
    //    (The vertices as values -- strings here -- live on the graph only.)
    let (graph, _enumerator, order) = discoverer.dfs_discover_graph("root".to_string());
    println!("Discovered: {:?}", order.iter().map(|&i| graph[i].clone()).collect::<Vec<_>>());

    // 2. The graph is an ordinary petgraph StableDiGraph afterwards:
    //    run anything petgraph has on it.
    println!("\n--- Graph Snapshot Analysis ---");
    println!("Nodes: {}, Edges: {}", graph.node_count(), graph.edge_count());
    for node_idx in graph.node_indices() {
        let neighbor_count = graph.neighbors(node_idx).count();
        println!("Vertex {:?} has {} neighbors (static view)", graph[node_idx], neighbor_count);
    }

    // 3. What used to be "static vs dynamic" here:
    println!("\n--- Key Difference ---");
    println!("petgraph on the returned graph   -> uses existing edges only");
    println!("DiscoverableGraph (IntoNeighbors) -> explodes nodes first, may discover more");
}

fn main() {
    demonstrate_graph_access();

    println!("\n=== Dynamic Graph Discovery with IntoNeighbors ===");

    println!("\n--- Simple Numeric Graph ---");
    let enumerator = SimpleNeighborEnumerator::new(3, 2);
    let mut discoverer = GraphDiscoverer::new(enumerator);
    let order = discoverer.dfs_discover(1);
    println!("Discovery order (NodeIndex): {:?}", order);
    println!("Discovered {} vertices with {} edges",
             discoverer.discovered_count(),
             discoverer.edges_count());

    println!("\n--- External Data Source ---");
    let enumerator = ExternalNeighborEnumerator::new();
    let mut discoverer = GraphDiscoverer::new(enumerator);
    let order = discoverer.dfs_discover("root".to_string());
    let snapshot = discoverer.get_dynamic_graph().get_graph_snapshot();
    let labels: Vec<&str> = order.iter().map(|&i| snapshot[i].as_str()).collect();
    println!("Discovery order: {:?}", labels);
    println!("Discovered {} vertices with {} edges",
             discoverer.discovered_count(),
             discoverer.edges_count());

    // Print final graph structure (still discovering piecewise: session API)
    println!("\nFinal graph structure:");
    for node_idx in snapshot.node_indices() {
        let neighbors: Vec<_> = snapshot.neighbors(node_idx)
            .filter_map(|n| snapshot.node_weight(n))
            .collect();
        println!("  {:?} -> {:?}", &*snapshot[node_idx], neighbors);
    }
}
