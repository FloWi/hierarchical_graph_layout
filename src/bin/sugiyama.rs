use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use rust_sugiyama::configure::{CrossingMinimization, RankingType};
use rust_sugiyama::{configure::Config, from_graph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Your existing types (assuming these are defined elsewhere)
type TradeGoodSymbol = String;
type WaypointSymbol = String;
type TradeGoodType = String;
type SupplyLevel = String;
type ActivityLevel = String;
type Point = (f64, f64);

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TechNode {
    id: String,
    name: TradeGoodSymbol,
    waypoint_symbol: WaypointSymbol,
    waypoint_type: TradeGoodType,
    supply: SupplyLevel,
    activity: ActivityLevel,
    cost: u32,
    volume: u32,
    width: f64,
    height: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    y: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TechEdge {
    source: String,
    target: String,
    cost: u32,
    activity: ActivityLevel,
    volume: u32,
    supply: SupplyLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    points: Option<Vec<Point>>,
    // Add a curve factor for each edge
    #[serde(skip_serializing_if = "Option::is_none")]
    curve_factor: Option<f64>,
}

fn main() {
    let (nodes, edges) = create_full_supply_chain();

    // Run the layout
    let (layout_nodes, layout_edges) = build_supply_chain_layout(&nodes, &edges);

    // Print the results
    println!("Node Layout:");
    for node in &layout_nodes {
        println!("Node '{}': x={:?}, y={:?}", node.name, node.x, node.y);
    }

    println!("\nEdge Routing:");
    for edge in &layout_edges {
        println!(
            "Edge '{}' -> '{}': points={:?}",
            edge.source, edge.target, edge.points
        );
    }

    let svg = output_svg(&layout_nodes, &layout_edges);

    // Write SVG to file
    use std::fs::File;
    use std::io::Write;

    match File::create("sugiyama.svg") {
        Ok(mut file) => {
            match file.write_all(svg.as_bytes()) {
                Ok(_) => println!("SVG successfully written to sugiyama.svg"),
                Err(e) => println!("Error writing to file: {}", e),
            }
        },
        Err(e) => println!("Error creating file: {}", e),
    }
}

fn create_full_supply_chain() -> (Vec<TechNode>, Vec<TechEdge>) {
    // Create all the nodes from the mermaid diagram
    let nodes = vec![
        create_node(
            "advanced_circuitry_at_x_1_ad_75_d_44",
            "ADVANCED_CIRCUITRY",
            "X1-AD75-D44",
            "ADVANCED",
        ),
        create_node(
            "aluminum_at_x_1_ad_75_h_51",
            "ALUMINUM",
            "X1-AD75-H51",
            "REFINED",
        ),
        create_node(
            "aluminum_ore_at_x_1_ad_75_xd_5_a",
            "ALUMINUM_ORE",
            "X1-AD75-XD5A",
            "RAW_MATERIAL",
        ),
        create_node(
            "clothing_at_x_1_ad_75_k_81",
            "CLOTHING",
            "X1-AD75-K81",
            "CONSUMER",
        ),
        create_node(
            "copper_at_x_1_ad_75_h_51",
            "COPPER",
            "X1-AD75-H51",
            "REFINED",
        ),
        create_node(
            "copper_ore_at_x_1_ad_75_xd_5_a",
            "COPPER_ORE",
            "X1-AD75-XD5A",
            "RAW_MATERIAL",
        ),
        create_node(
            "electronics_at_x_1_ad_75_f_49",
            "ELECTRONICS",
            "X1-AD75-F49",
            "INDUSTRIAL",
        ),
        create_node(
            "equipment_at_x_1_ad_75_k_81",
            "EQUIPMENT",
            "X1-AD75-K81",
            "INDUSTRIAL",
        ),
        create_node(
            "fabrics_at_x_1_ad_75_e_46",
            "FABRICS",
            "X1-AD75-E46",
            "INDUSTRIAL",
        ),
        create_node(
            "fab_mats_at_x_1_ad_75_f_49",
            "FAB_MATS",
            "X1-AD75-F49",
            "INDUSTRIAL",
        ),
        create_node(
            "fertilizers_at_x_1_ad_75_g_50",
            "FERTILIZERS",
            "X1-AD75-G50",
            "INDUSTRIAL",
        ),
        create_node("iron_at_x_1_ad_75_h_51", "IRON", "X1-AD75-H51", "REFINED"),
        create_node(
            "iron_ore_at_x_1_ad_75_xd_5_a",
            "IRON_ORE",
            "X1-AD75-XD5A",
            "RAW_MATERIAL",
        ),
        create_node(
            "liquid_hydrogen_at_x_1_ad_75_c_40",
            "LIQUID_HYDROGEN",
            "X1-AD75-C40",
            "REFINED",
        ),
        create_node(
            "liquid_hydrogen_at_x_1_ad_75_c_41",
            "LIQUID_HYDROGEN",
            "X1-AD75-C41",
            "REFINED",
        ),
        create_node(
            "liquid_nitrogen_at_x_1_ad_75_c_40",
            "LIQUID_NITROGEN",
            "X1-AD75-C40",
            "REFINED",
        ),
        create_node(
            "liquid_nitrogen_at_x_1_ad_75_c_41",
            "LIQUID_NITROGEN",
            "X1-AD75-C41",
            "REFINED",
        ),
        create_node(
            "machinery_at_x_1_ad_75_e_46",
            "MACHINERY",
            "X1-AD75-E46",
            "INDUSTRIAL",
        ),
        create_node(
            "microprocessors_at_x_1_ad_75_a_3",
            "MICROPROCESSORS",
            "X1-AD75-A3",
            "ADVANCED",
        ),
        create_node(
            "plastics_at_x_1_ad_75_g_50",
            "PLASTICS",
            "X1-AD75-G50",
            "INDUSTRIAL",
        ),
        create_node(
            "quartz_sand_at_x_1_ad_75_h_53",
            "QUARTZ_SAND",
            "X1-AD75-H53",
            "REFINED",
        ),
        create_node(
            "quartz_sand_at_x_1_ad_75_xd_5_a",
            "QUARTZ_SAND",
            "X1-AD75-XD5A",
            "RAW_MATERIAL",
        ),
        create_node(
            "ship_parts_at_x_1_ad_75_d_43",
            "SHIP_PARTS",
            "X1-AD75-D43",
            "ADVANCED",
        ),
        create_node(
            "ship_plating_at_x_1_ad_75_d_44",
            "SHIP_PLATING",
            "X1-AD75-D44",
            "ADVANCED",
        ),
        create_node(
            "silicon_crystals_at_x_1_ad_75_h_53",
            "SILICON_CRYSTALS",
            "X1-AD75-H53",
            "REFINED",
        ),
        create_node(
            "silicon_crystals_at_x_1_ad_75_xd_5_a",
            "SILICON_CRYSTALS",
            "X1-AD75-XD5A",
            "RAW_MATERIAL",
        ),
    ];

    // Create all the edges from the mermaid diagram
    let edges = vec![
        create_edge("iron_at_x_1_ad_75_h_51", "machinery_at_x_1_ad_75_e_46"),
        create_edge(
            "liquid_hydrogen_at_x_1_ad_75_c_41",
            "plastics_at_x_1_ad_75_g_50",
        ),
        create_edge(
            "silicon_crystals_at_x_1_ad_75_h_53",
            "electronics_at_x_1_ad_75_f_49",
        ),
        create_edge("copper_at_x_1_ad_75_h_51", "electronics_at_x_1_ad_75_f_49"),
        create_edge("aluminum_at_x_1_ad_75_h_51", "equipment_at_x_1_ad_75_k_81"),
        create_edge("plastics_at_x_1_ad_75_g_50", "equipment_at_x_1_ad_75_k_81"),
        create_edge(
            "aluminum_at_x_1_ad_75_h_51",
            "ship_plating_at_x_1_ad_75_d_44",
        ),
        create_edge(
            "machinery_at_x_1_ad_75_e_46",
            "ship_plating_at_x_1_ad_75_d_44",
        ),
        create_edge("iron_at_x_1_ad_75_h_51", "fab_mats_at_x_1_ad_75_f_49"),
        create_edge(
            "quartz_sand_at_x_1_ad_75_h_53",
            "fab_mats_at_x_1_ad_75_f_49",
        ),
        create_edge(
            "silicon_crystals_at_x_1_ad_75_h_53",
            "microprocessors_at_x_1_ad_75_a_3",
        ),
        create_edge(
            "copper_at_x_1_ad_75_h_51",
            "microprocessors_at_x_1_ad_75_a_3",
        ),
        create_edge(
            "liquid_nitrogen_at_x_1_ad_75_c_41",
            "fertilizers_at_x_1_ad_75_g_50",
        ),
        create_edge(
            "equipment_at_x_1_ad_75_k_81",
            "ship_parts_at_x_1_ad_75_d_43",
        ),
        create_edge(
            "electronics_at_x_1_ad_75_f_49",
            "ship_parts_at_x_1_ad_75_d_43",
        ),
        create_edge("fertilizers_at_x_1_ad_75_g_50", "fabrics_at_x_1_ad_75_e_46"),
        create_edge(
            "electronics_at_x_1_ad_75_f_49",
            "advanced_circuitry_at_x_1_ad_75_d_44",
        ),
        create_edge(
            "microprocessors_at_x_1_ad_75_a_3",
            "advanced_circuitry_at_x_1_ad_75_d_44",
        ),
        create_edge("fabrics_at_x_1_ad_75_e_46", "clothing_at_x_1_ad_75_k_81"),
        create_edge(
            "quartz_sand_at_x_1_ad_75_xd_5_a",
            "quartz_sand_at_x_1_ad_75_h_53",
        ),
        create_edge(
            "liquid_nitrogen_at_x_1_ad_75_c_40",
            "liquid_nitrogen_at_x_1_ad_75_c_41",
        ),
        create_edge("copper_ore_at_x_1_ad_75_xd_5_a", "copper_at_x_1_ad_75_h_51"),
        create_edge(
            "liquid_hydrogen_at_x_1_ad_75_c_40",
            "liquid_hydrogen_at_x_1_ad_75_c_41",
        ),
        create_edge("iron_ore_at_x_1_ad_75_xd_5_a", "iron_at_x_1_ad_75_h_51"),
        create_edge(
            "aluminum_ore_at_x_1_ad_75_xd_5_a",
            "aluminum_at_x_1_ad_75_h_51",
        ),
        create_edge(
            "silicon_crystals_at_x_1_ad_75_xd_5_a",
            "silicon_crystals_at_x_1_ad_75_h_53",
        ),
    ];

    (nodes, edges)
}

// Helper function to create nodes with default values
fn create_node(id: &str, name: &str, waypoint: &str, node_type: &str) -> TechNode {
    TechNode {
        id: id.to_string(),
        name: name.to_string(),
        waypoint_symbol: waypoint.to_string(),
        waypoint_type: node_type.to_string(),
        supply: "MODERATE".to_string(),
        activity: "MODERATE".to_string(),
        cost: 100,
        volume: 10,
        width: 120.0,
        height: 70.0,
        x: None,
        y: None,
    }
}

// Helper function to create edges with default values
fn create_edge(source: &str, target: &str) -> TechEdge {
    TechEdge {
        source: source.to_string(),
        target: target.to_string(),
        cost: 50,
        activity: "MODERATE".to_string(),
        volume: 5,
        supply: "MODERATE".to_string(),
        points: None,
        curve_factor: None,
    }
}

// Function to build and layout the graph
fn build_supply_chain_layout(
    nodes: &[TechNode],
    edges: &[TechEdge],
) -> (Vec<TechNode>, Vec<TechEdge>) {
    // Create a new directed graph
    let mut graph: StableDiGraph<String, u32> = StableDiGraph::new();

    // Create a mapping from node ID to NodeIndex
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Add all nodes to the graph
    for node in nodes {
        let node_idx = graph.add_node(node.id.clone());
        node_indices.insert(node.id.clone(), node_idx);
    }

    // Add all edges to the graph
    for edge in edges {
        if let (Some(source_idx), Some(target_idx)) = (
            node_indices.get(&edge.source),
            node_indices.get(&edge.target),
        ) {
            graph.add_edge(*source_idx, *target_idx, edge.cost);
        }
    }

    // Configure the layout algorithm
    let config =  Config {
        minimum_length: 1,                        // Increase this from 0
        vertex_spacing: 150,
        dummy_vertices: true,                     // Enable dummy vertices
        dummy_size: 50.0,                         // Give them a size
        ranking_type: RankingType::MinimizeEdgeLength, // Change from Original
        c_minimization: CrossingMinimization::Median,
        transpose: false,
        // ..Default::default()
    };

    // Run the layout algorithm
    let layouts = from_graph(&graph).with_config(config);

    // Process the layout results
    let mut updated_nodes = nodes.to_vec();
    let mut updated_edges = edges.to_vec();

    // Create reverse lookup from NodeIndex to position in nodes array
    let mut node_positions: HashMap<String, usize> = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        node_positions.insert(node.id.clone(), i);
    }

    let built_layouts = layouts.build();

    // After building layouts, iterate through all and pick the best one
    let mut best_layout_index = 0;
    let mut best_layout_metric = 0.0; // Or some other appropriate initial value

    for (i, (layout, width, height)) in built_layouts.iter().enumerate() {
        // Define some metric to evaluate layout quality
        // For example, you might prefer layouts with more balanced width/height ratio
        let layout_metric = (*width as f64) / (*height as f64);

        // Compare with current best
        if layout_metric > 1.0 && layout_metric < best_layout_metric || best_layout_metric == 0.0 {
            best_layout_index = i;
            best_layout_metric = layout_metric;
        }
    }

    println!("{} layouts found", built_layouts.len());
    // Apply coordinates to nodes

    if let Some((layout, width, height)) = built_layouts.get(best_layout_index) {

        println!("Using layout #{}: width={}, height={}", best_layout_index, width, height);

        // Update node coordinates
        for (node_idx, (x, y)) in layout.iter() {
            let node_id = &graph[NodeIndex::from(*node_idx)];
            if let Some(&pos) = node_positions.get(node_id) {
                updated_nodes[pos].x = Some(*x as f64);
                updated_nodes[pos].y = Some(*y as f64);
            }
        }

        // Process edge routing
        for edge in &mut updated_edges {
            if let (Some(source_pos), Some(target_pos)) = (
                node_positions.get(&edge.source),
                node_positions.get(&edge.target),
            ) {
                let source_node = &updated_nodes[*source_pos];
                let target_node = &updated_nodes[*target_pos];

                if let (Some(sx), Some(sy), Some(tx), Some(ty)) =
                    (source_node.x, source_node.y, target_node.x, target_node.y)
                {
                    // For a straight line, we'd just use:
                    // edge.points = Some(vec![(sx, sy), (tx, ty)]);

                    // For curved edges (more visually appealing):
                    // Add a control point for the path
                    let mid_x = (sx + tx) / 2.0;
                    let mid_y = (sy + ty) / 2.0;

                    // Create a path with control points
                    edge.points = Some(vec![
                        (sx, sy),       // Start point
                        (mid_x, mid_y), // Control point
                        (tx, ty),       // End point
                    ]);

                    // Calculate curve factor based on distance
                    let distance = ((tx - sx).powi(2) + (ty - sy).powi(2)).sqrt();
                    edge.curve_factor = Some((distance / 500.0).min(0.5).max(0.1));
                }
            }
        }
    }

    (updated_nodes, updated_edges)
}

fn output_svg(nodes: &[TechNode], edges: &[TechEdge]) -> String {
    // Calculate SVG dimensions based on node positions
    let margin = 50.0;
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for node in nodes {
        if let (Some(x), Some(y)) = (node.x, node.y) {
            min_x = min_x.min(x - node.width / 2.0);
            min_y = min_y.min(y - node.height / 2.0);
            max_x = max_x.max(x + node.width / 2.0);
            max_y = max_y.max(y + node.height / 2.0);
        }
    }

    let svg_width = max_x - min_x + 2.0 * margin;
    let svg_height = (max_y - min_y).abs() + 2.0 * margin;

    // SVG header
    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        svg_width, svg_height
    );

    // Add a transformation that:
    // 1. Translates to account for margins and any negative coordinates
    // 2. Scales the y-axis by -1 to flip it (since Sugiyama outputs negative y values for top-to-bottom layout)
    // 3. Translates again to move everything back into the visible area after flipping
    svg.push_str(&format!(
        r#"<g transform="translate({},{}) scale(1,-1) translate(0,{})">"#,
        margin - min_x,
        margin - min_y,
        -max_y - min_y  // This value adjusts the vertical position after flipping
    ));

    // Draw edges
    for edge in edges {
        if let Some(ref points) = edge.points {
            if points.len() >= 2 {
                if points.len() == 2 {
                    // Simple straight line
                    svg.push_str(&format!(
                        r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="gray" stroke-width="2" />"#,
                        points[0].0, points[0].1, points[1].0, points[1].1
                    ));
                } else {
                    // Path with control points
                    svg.push_str(&format!(
                        r#"<path d="M{},{} Q{},{} {},{}" fill="none" stroke="gray" stroke-width="2" />"#,
                        points[0].0, points[0].1,
                        points[1].0, points[1].1,
                        points[2].0, points[2].1
                    ));

                    // Add an arrow at the end
                    svg.push_str(&format!(
                        r#"<circle cx="{}" cy="{}" r="4" fill="black" />"#,
                        points[2].0, points[2].1
                    ));
                }
            }
        }
    }

    // Draw nodes
    for node in nodes {
        if let (Some(x), Some(y)) = (node.x, node.y) {
            // Draw rectangle for the node
            let node_x = x - node.width / 2.0;
            let node_y = y - node.height / 2.0;

            // Determine color based on node type
            let color = match node.waypoint_type.as_str() {
                "RAW_MATERIAL" => "#d4f0fc",
                "REFINED" => "#b8e6f7",
                "INDUSTRIAL" => "#8dd6f0",
                "ADVANCED" => "#65c6ea",
                "CONSUMER" => "#42b6e3",
                _ => "#99ccff",
            };

            svg.push_str(&format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="5" ry="5" fill="{}" stroke="black" stroke-width="1" />"#,
                node_x, node_y, node.width, node.height, color
            ));

            // Add node name - need to flip the text back to be readable
            svg.push_str(&format!(
                r#"<text transform="scale(1,-1)" x="{}" y="{}" font-family="Arial" font-size="12" text-anchor="middle">{}</text>"#,
                x, -y, node.name
            ));
        }
    }

    // Close SVG
    svg.push_str("</g></svg>");

    svg
}
