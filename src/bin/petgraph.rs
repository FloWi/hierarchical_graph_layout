// Add these to your Cargo.toml:
// [dependencies]
// petgraph = "0.6.2"

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::{Topo, EdgeRef};
use petgraph::Direction;
use std::collections::HashMap;

// NodeLayout struct for storing node positioning data
struct NodeLayout {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

// Function to perform layer assignment in a way more similar to Mermaid/Dagre
fn layered_dag_layout<N, E>(graph: &DiGraph<N, E>) -> HashMap<NodeIndex, NodeLayout> {
    // 1. Layer Assignment: Assign each node to a layer (rank)
    let node_ranks = assign_layers(graph);

    // 2. Node Ordering: Order nodes within each layer to minimize edge crossings
    let nodes_by_rank = order_nodes_within_layers(graph, &node_ranks);

    // 3. Coordinate Assignment: Assign x, y coordinates to nodes
    assign_coordinates(graph, &nodes_by_rank)
}

// Layer assignment using the longest path algorithm
fn assign_layers<N, E>(graph: &DiGraph<N, E>) -> HashMap<NodeIndex, usize> {
    let mut node_ranks = HashMap::new();

    // Find source nodes (nodes with no incoming edges)
    let mut sources = Vec::new();
    for node in graph.node_indices() {
        if graph.neighbors_directed(node, Direction::Incoming).count() == 0 {
            sources.push(node);
            node_ranks.insert(node, 0); // Source nodes at rank 0
        }
    }

    // Process nodes in topological order to assign ranks
    let mut topo = Topo::new(graph);
    while let Some(node) = topo.next(graph) {
        // If the node is already assigned, skip it
        if node_ranks.contains_key(&node) {
            continue;
        }

        // Find predecessors
        let mut max_pred_rank = 0;
        let mut has_pred = false;
        for pred in graph.neighbors_directed(node, Direction::Incoming) {
            has_pred = true;
            let pred_rank = *node_ranks.get(&pred).unwrap_or(&0);
            max_pred_rank = max_pred_rank.max(pred_rank + 1);
        }

        // Assign rank based on predecessors
        if has_pred {
            node_ranks.insert(node, max_pred_rank);
        } else {
            // Nodes with no predecessors (and not already marked as sources)
            node_ranks.insert(node, 0);
        }
    }

    // Process sink nodes to ensure they're all at the maximum rank
    let max_rank = node_ranks.values().max().cloned().unwrap_or(0);

    for rank in node_ranks.values_mut() {
        if *rank == usize::MAX {
            *rank = max_rank + 1;
        }
    }

    // Normalize ranks to start from 0
    let min_rank = *node_ranks.values().min().unwrap_or(&0);
    if min_rank > 0 {
        for rank in node_ranks.values_mut() {
            *rank -= min_rank;
        }
    }

    node_ranks
}

// Fixed version of the function with borrowing issues resolved
fn order_nodes_within_layers<N, E>(
    graph: &DiGraph<N, E>,
    node_ranks: &HashMap<NodeIndex, usize>,
) -> HashMap<usize, Vec<NodeIndex>> {
    let mut nodes_by_rank: HashMap<usize, Vec<NodeIndex>> = HashMap::new();

    // Group nodes by rank
    for (&node, &rank) in node_ranks {
        nodes_by_rank.entry(rank).or_insert_with(Vec::new).push(node);
    }

    // Find the maximum rank
    let max_rank = nodes_by_rank.keys().max().cloned().unwrap_or(0);

    // Order nodes within each rank to minimize crossings
    // Two passes: top-down and bottom-up
    for iter in 0..2 {
        let rank_range = if iter == 0 {
            (0..=max_rank).collect::<Vec<_>>() // Top-down
        } else {
            (0..=max_rank).rev().collect::<Vec<_>>() // Bottom-up
        };

        for rank in rank_range {
            // Make a copy of nodes_by_rank to avoid borrowing issues
            let nodes_by_rank_copy = nodes_by_rank.clone();

            if let Some(nodes) = nodes_by_rank.get_mut(&rank) {
                // Skip if only 0 or 1 node in this rank
                if nodes.len() <= 1 {
                    continue;
                }

                // Make a copy of the current nodes
                let current_nodes = nodes.clone();

                // Calculate barycenters for each node
                let mut node_barycenters = Vec::new();

                for &node in &current_nodes {
                    let mut sum_pos = 0.0;
                    let mut count = 0;

                    // Get connected nodes in adjacent rank
                    let connected_nodes = if iter == 0 {
                        // Top-down: look at predecessors
                        graph.neighbors_directed(node, Direction::Incoming)
                            .filter(|&pred| {
                                if let Some(&pred_rank) = node_ranks.get(&pred) {
                                    let current_rank = *node_ranks.get(&node).unwrap();
                                    pred_rank < current_rank
                                } else {
                                    false
                                }
                            })
                            .collect::<Vec<_>>()
                    } else {
                        // Bottom-up: look at successors
                        graph.neighbors_directed(node, Direction::Outgoing)
                            .filter(|&succ| {
                                if let Some(&succ_rank) = node_ranks.get(&succ) {
                                    let current_rank = *node_ranks.get(&node).unwrap();
                                    succ_rank > current_rank
                                } else {
                                    false
                                }
                            })
                            .collect::<Vec<_>>()
                    };

                    // Calculate barycenter based on positions of connected nodes
                    for &connected in &connected_nodes {
                        if let Some(&connected_rank) = node_ranks.get(&connected) {
                            // Use the copied nodes_by_rank to look up positions
                            if let Some(nodes_in_rank) = nodes_by_rank_copy.get(&connected_rank) {
                                if let Some(pos) = nodes_in_rank.iter().position(|&n| n == connected) {
                                    sum_pos += pos as f64;
                                    count += 1;
                                }
                            }
                        }
                    }

                    // Calculate final barycenter
                    let barycenter = if count > 0 {
                        sum_pos / count as f64
                    } else {
                        // Default position if no connections
                        let node_pos = current_nodes.iter().position(|&n| n == node).unwrap_or(0);
                        node_pos as f64
                    };

                    node_barycenters.push((node, barycenter));
                }

                // Sort nodes by barycenter
                node_barycenters.sort_by(|a, b| {
                    a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)
                });

                // Update node order
                *nodes = node_barycenters.into_iter().map(|(node, _)| node).collect();
            }
        }
    }

    nodes_by_rank
}

// Assign x and y coordinates to nodes
fn assign_coordinates<N, E>(
    graph: &DiGraph<N, E>,
    nodes_by_rank: &HashMap<usize, Vec<NodeIndex>>,
) -> HashMap<NodeIndex, NodeLayout> {
    let mut layout = HashMap::new();

    // Constants for layout
    let horizontal_spacing = 180.0; // Space between nodes in the same rank
    let vertical_spacing = 150.0;   // Space between ranks
    let node_width = 180.0;
    let node_height = 60.0;

    // Layout direction (horizontal layout like in Mermaid)
    let is_horizontal = true; // Use LR direction

    // Assign coordinates
    for (&rank, nodes) in nodes_by_rank {
        let node_count = nodes.len();
        let total_width = node_count as f64 * (node_width + horizontal_spacing) - horizontal_spacing;
        let start_x = -total_width / 2.0;

        for (i, &node) in nodes.iter().enumerate() {
            if is_horizontal {
                // For LR layout, rank determines x, position determines y
                let x = rank as f64 * (node_width + vertical_spacing);
                let y = start_x + i as f64 * (node_width + horizontal_spacing) + node_width / 2.0;

                layout.insert(node, NodeLayout {
                    x,
                    y,
                    width: node_width,
                    height: node_height,
                });
            } else {
                // For TB layout, rank determines y, position determines x
                let x = start_x + i as f64 * (node_width + horizontal_spacing) + node_width / 2.0;
                let y = rank as f64 * (node_height + vertical_spacing);

                layout.insert(node, NodeLayout {
                    x,
                    y,
                    width: node_width,
                    height: node_height,
                });
            }
        }
    }

    // Adjust node positions for better separation
    adjust_positions(&mut layout);

    layout
}

// Helper function to adjust node positions for better aesthetics
fn adjust_positions(layout: &mut HashMap<NodeIndex, NodeLayout>) {
    // This is a simplified version that avoids borrowing issues
    // Clone all the necessary data first
    let node_positions: Vec<(NodeIndex, (f64, f64))> = layout
        .iter()
        .map(|(&node, pos)| (node, (pos.x, pos.y)))
        .collect();

    // Now update positions without multiple borrows
    for (node, _) in node_positions {
        if let Some(node_layout) = layout.get_mut(&node) {
            // Apply small adjustments if needed
            // For example, add a small random offset to avoid overlaps
            // This simplified version just ensures we don't have borrowing errors
            node_layout.x += (node.index() % 5) as f64 * 0.1; // Small adjustment
        }
    }
}

fn main() {
    // Create a directed graph for your example
    let mut graph = DiGraph::<&str, &str>::new();

    // Create a mapping from node names to indices for easier reference
    let mut node_map = HashMap::new();

    // Add nodes (using a subset of your mermaid nodes for clarity)
    let nodes = vec![
        "advanced_circuitry_at_x_1_ad_75_d_44",
        "aluminum_at_x_1_ad_75_h_51",
        "aluminum_ore_at_x_1_ad_75_xd_5_a",
        "clothing_at_x_1_ad_75_k_81",
        "copper_at_x_1_ad_75_h_51",
        "copper_ore_at_x_1_ad_75_xd_5_a",
        "electronics_at_x_1_ad_75_f_49",
        "equipment_at_x_1_ad_75_k_81",
        "fabrics_at_x_1_ad_75_e_46",
        "fab_mats_at_x_1_ad_75_f_49",
        "fertilizers_at_x_1_ad_75_g_50",
        "iron_at_x_1_ad_75_h_51",
        "iron_ore_at_x_1_ad_75_xd_5_a",
        "liquid_hydrogen_at_x_1_ad_75_c_40",
        "liquid_hydrogen_at_x_1_ad_75_c_41",
        "liquid_nitrogen_at_x_1_ad_75_c_40",
        "liquid_nitrogen_at_x_1_ad_75_c_41",
        "machinery_at_x_1_ad_75_e_46",
        "microprocessors_at_x_1_ad_75_a_3",
        "plastics_at_x_1_ad_75_g_50",
        "quartz_sand_at_x_1_ad_75_h_53",
        "quartz_sand_at_x_1_ad_75_xd_5_a",
        "ship_parts_at_x_1_ad_75_d_43",
        "ship_plating_at_x_1_ad_75_d_44",
        "silicon_crystals_at_x_1_ad_75_h_53",
        "silicon_crystals_at_x_1_ad_75_xd_5_a",    ];

    for &node_name in &nodes {
        let node_idx = graph.add_node(node_name);
        node_map.insert(node_name, node_idx);
    }

    // Add edges (connections between nodes)
    let edges = vec![
        ("iron_at_x_1_ad_75_h_51", "machinery_at_x_1_ad_75_e_46"),
        ("liquid_hydrogen_at_x_1_ad_75_c_41", "plastics_at_x_1_ad_75_g_50"),
        ("silicon_crystals_at_x_1_ad_75_h_53", "electronics_at_x_1_ad_75_f_49"),
        ("copper_at_x_1_ad_75_h_51", "electronics_at_x_1_ad_75_f_49"),
        ("aluminum_at_x_1_ad_75_h_51", "equipment_at_x_1_ad_75_k_81"),
        ("plastics_at_x_1_ad_75_g_50", "equipment_at_x_1_ad_75_k_81"),
        ("aluminum_at_x_1_ad_75_h_51", "ship_plating_at_x_1_ad_75_d_44"),
        ("machinery_at_x_1_ad_75_e_46", "ship_plating_at_x_1_ad_75_d_44"),
        ("iron_at_x_1_ad_75_h_51", "fab_mats_at_x_1_ad_75_f_49"),
        ("quartz_sand_at_x_1_ad_75_h_53", "fab_mats_at_x_1_ad_75_f_49"),
        ("silicon_crystals_at_x_1_ad_75_h_53", "microprocessors_at_x_1_ad_75_a_3"),
        ("copper_at_x_1_ad_75_h_51", "microprocessors_at_x_1_ad_75_a_3"),
        ("liquid_nitrogen_at_x_1_ad_75_c_41", "fertilizers_at_x_1_ad_75_g_50"),
        ("equipment_at_x_1_ad_75_k_81", "ship_parts_at_x_1_ad_75_d_43"),
        ("electronics_at_x_1_ad_75_f_49", "ship_parts_at_x_1_ad_75_d_43"),
        ("fertilizers_at_x_1_ad_75_g_50", "fabrics_at_x_1_ad_75_e_46"),
        ("electronics_at_x_1_ad_75_f_49", "advanced_circuitry_at_x_1_ad_75_d_44"),
        ("microprocessors_at_x_1_ad_75_a_3", "advanced_circuitry_at_x_1_ad_75_d_44"),
        ("fabrics_at_x_1_ad_75_e_46", "clothing_at_x_1_ad_75_k_81"),
        ("quartz_sand_at_x_1_ad_75_xd_5_a", "quartz_sand_at_x_1_ad_75_h_53"),
        ("liquid_nitrogen_at_x_1_ad_75_c_40", "liquid_nitrogen_at_x_1_ad_75_c_41"),
        ("copper_ore_at_x_1_ad_75_xd_5_a", "copper_at_x_1_ad_75_h_51"),
        ("liquid_hydrogen_at_x_1_ad_75_c_40", "liquid_hydrogen_at_x_1_ad_75_c_41"),
        ("iron_ore_at_x_1_ad_75_xd_5_a", "iron_at_x_1_ad_75_h_51"),
        ("aluminum_ore_at_x_1_ad_75_xd_5_a", "aluminum_at_x_1_ad_75_h_51"),
        ("silicon_crystals_at_x_1_ad_75_xd_5_a", "silicon_crystals_at_x_1_ad_75_h_53"),


    ];

    for (source, target) in &edges {
        if let (Some(&source_idx), Some(&target_idx)) = (node_map.get(source), node_map.get(target)) {
            graph.add_edge(source_idx, target_idx, "");
        }
    }

    // Apply the layered DAG layout algorithm
    let layout = layered_dag_layout(&graph);

    // Print the resulting layout
    println!("Node positions after layered DAG layout:");
    for (node_idx, pos) in &layout {
        let node_name = graph[*node_idx];
        println!("Node {}: x={:.1}, y={:.1}", node_name, pos.x, pos.y);
    }

    // Output a simple DOT format for visualization
    println!("\nDOT format for visualization:");
    println!("digraph G {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box];");

    // Add nodes with positions
    for (node_idx, pos) in &layout {
        let node_name = graph[*node_idx];
        println!("  \"{}\" [pos=\"{},{}!\"];", node_name, pos.x, pos.y);
    }

    // Add edges
    for edge in graph.edge_indices() {
        let (source, target) = graph.edge_endpoints(edge).unwrap();
        let source_name = graph[source];
        let target_name = graph[target];
        println!("  \"{}\" -> \"{}\";", source_name, target_name);
    }

    println!("}}");
}
