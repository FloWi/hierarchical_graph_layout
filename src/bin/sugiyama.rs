use std::borrow::Cow;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use rust_sugiyama::configure::{CrossingMinimization, RankingType};
use rust_sugiyama::{configure::Config, from_graph};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use rand::Rng;
use strum::{Display, EnumIter, IntoEnumIterator};

// Your existing types (assuming these are defined elsewhere)
type TradeGoodSymbol = String;
type WaypointSymbol = String;
type TradeGoodType = String;


#[derive(
    Serialize, Deserialize, Clone, Debug, Display, EnumIter, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SupplyLevel {
    Abundant,
    High,
    Moderate,
    Limited,
    Scarce,
}

#[derive(
    Serialize, Deserialize, Clone, Debug, Display, EnumIter, Eq, PartialEq, Hash, Ord, PartialOrd,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityLevel {
    Weak,
    Growing,
    Strong,
    Restricted,
}

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
    // New fields
    #[serde(skip_serializing_if = "Option::is_none")]
    distance: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    profit: Option<i32>,  // Can be negative
}

// ColorString newtype using Cow for efficiency
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ColorString(pub Cow<'static, str>);

impl ColorString {
    pub fn new(color: &str) -> Self {
        Self(Cow::Owned(color.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for ColorString {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl From<&'static str> for ColorString {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

// Display implementation for easy formatting
impl fmt::Display for ColorString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TechNode {
    pub(crate) fn supply_color(&self) -> ColorString {
        get_supply_color(&self.supply)
    }

    pub(crate) fn activity_color(&self) -> ColorString {
        get_activity_color(&self.activity)
    }
}

fn get_activity_color(activity: &ActivityLevel) -> ColorString {
    match activity {
        ActivityLevel::Strong => "#22c55e",     // green-500
        ActivityLevel::Growing => "#86efac",    // green-300
        ActivityLevel::Weak => "#eab308",       // yellow-500
        ActivityLevel::Restricted => "#ef4444", // red-500
    }
        .to_string().into()
}

fn get_supply_color(supply: &SupplyLevel) -> ColorString {
    match supply {
        SupplyLevel::Abundant => "#22c55e", // green-500
        SupplyLevel::High => "#86efac",     // green-300
        SupplyLevel::Moderate => "#fde047", // yellow-300
        SupplyLevel::Limited => "#f97316",  // orange-500
        SupplyLevel::Scarce => "#ef4444",   // red-500
    }
        .to_string().into()
}


impl TechEdge {
    pub(crate) fn supply_color(&self) -> ColorString {
        get_supply_color(&self.supply)
    }

    pub(crate) fn activity_color(&self) -> ColorString {
        get_activity_color(&self.activity)
    }
}

enum Orientation {
    TopDown,
    LeftRight,
}

fn main() {
    let (nodes, edges) = create_full_supply_chain();

    // Run the layout
    let orientation = Orientation::LeftRight;
    let x_scale = 1.5;
    let y_scale = 0.75;
    let (layout_nodes, layout_edges) = build_supply_chain_layout(&nodes, &edges, orientation, x_scale, y_scale);

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
        Ok(mut file) => match file.write_all(svg.as_bytes()) {
            Ok(_) => println!("SVG successfully written to sugiyama.svg"),
            Err(e) => println!("Error writing to file: {}", e),
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

// Helper function to create nodes with random values
fn create_node(id: &str, name: &str, waypoint: &str, node_type: &str) -> TechNode {
    let mut rng = rand::thread_rng();

    // Generate random supply level
    let supplies: Vec<SupplyLevel> = SupplyLevel::iter().collect();
    let random_supply = supplies[rng.random_range(0..supplies.len())].clone();

    // Generate random activity level
    let activities: Vec<ActivityLevel> = ActivityLevel::iter().collect();
    let random_activity = activities[rng.random_range(0..activities.len())].clone();

    // Random cost between 50 and 500
    let random_cost = rng.random_range(50..=500);

    // Random volume between 5 and 100
    let random_volume = rng.random_range(5..=100);

    TechNode {
        id: id.to_string(),
        name: name.to_string(),
        waypoint_symbol: waypoint.to_string(),
        waypoint_type: node_type.to_string(),
        supply: random_supply,
        activity: random_activity,
        cost: random_cost,
        volume: random_volume,
        width: 200.0,
        height: 165.0,
        x: None,
        y: None,
    }
}

// Helper function to create edges with random activity and supply levels
fn create_edge(source: &str, target: &str) -> TechEdge {
    let mut rng = rand::thread_rng();

    // Generate random activity level
    let activities: Vec<ActivityLevel> = ActivityLevel::iter().collect();
    let random_activity = activities[rng.gen_range(0..activities.len())].clone();

    // Generate random supply level
    let supplies: Vec<SupplyLevel> = SupplyLevel::iter().collect();
    let random_supply = supplies[rng.gen_range(0..supplies.len())].clone();

    // Random cost between 10 and 200
    let random_cost = rng.gen_range(10..=200);

    // Random volume between 1 and 50
    let random_volume = rng.gen_range(1..=50);

    // Random distance between 10 and 150
    let random_distance = rng.gen_range(10..=150);

    // Random profit between -50 and 250 (can be negative)
    let random_profit = rng.gen_range(-50..=250);



    TechEdge {
        source: source.to_string(),
        target: target.to_string(),
        cost: random_cost,
        activity: random_activity,
        volume: random_volume,
        supply: random_supply,
        points: None,
        curve_factor: None,
        distance: Some(random_distance),
        profit: Some(random_profit),
    }
}

// Function to build the supply chain layout with separate x and y scaling
fn build_supply_chain_layout(
    nodes: &[TechNode],
    edges: &[TechEdge],
    orientation: Orientation,
    x_scale: f64,  // Scaling factor for horizontal spacing
    y_scale: f64,  // Scaling factor for vertical spacing
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
    let config = Config {
        minimum_length: 1, // Increase this from 0
        vertex_spacing: 300,
        dummy_vertices: true,                          // Enable dummy vertices
        dummy_size: 150.0,                              // Give them a size
        ranking_type: RankingType::MinimizeEdgeLength, // Change from Original
        c_minimization: CrossingMinimization::Barycenter,
        transpose: true,
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

    // Apply coordinates to nodes
    if let Some((layout, width, height)) = built_layouts.first() {
        for (node_idx, (x, y)) in layout.iter() {
            let node_id = &graph[NodeIndex::from(*node_idx)];
            if let Some(&pos) = node_positions.get(node_id) {
                match orientation {
                    Orientation::LeftRight => {
                        // Update node coordinates and rotate 90 degrees (swap and invert as needed)
                        // Also apply scaling factors
                        updated_nodes[pos].x = Some(-*y as f64 * x_scale);
                        updated_nodes[pos].y = Some(*x as f64 * y_scale);
                    }
                    Orientation::TopDown => {
                        updated_nodes[pos].x = Some(*x as f64 * x_scale);
                        updated_nodes[pos].y = Some(*y as f64 * y_scale);
                    }
                }
            }
        }

        // Process edge routing with scaling
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
                    // For curved edges with control points
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
    let svg_height = max_y - min_y + 2.0 * margin;

    // SVG header
    let mut svg = format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
        svg_width, svg_height
    );

    // Transform to adjust for margins and any negative coordinates
    svg.push_str(&format!(
        r#"<g transform="translate({},{})">"#,
        margin - min_x,
        margin - min_y
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

    // Draw nodes using the new node generator
    for node in nodes {
        svg.push_str(&generate_node_svg(node));
    }

    // Add edge labels after nodes to ensure they're in the foreground
    // But only for target nodes as per your update
    for edge in edges {
        if let Some(ref points) = edge.points {
            if points.len() >= 2 {
                // Get target node
                let target_node = nodes.iter().find(|n| n.id == edge.target).unwrap();

                if let (Some(tx), Some(ty)) = (target_node.x, target_node.y) {
                    // For target label:
                    // Calculate target node border intersection
                    let (target_ix, target_iy) = calculate_node_border_intersection(
                        tx, ty, target_node.width, target_node.height,
                        points[points.len()-1].0, points[points.len()-1].1,
                        points[points.len()-2].0, points[points.len()-2].1
                    );

                    // Calculate direction vector - pointing from node to edge (outward)
                    let direction_x = points[points.len()-2].0 - tx;
                    let direction_y = points[points.len()-2].1 - ty;

                    // Add label with direction vector for proper positioning
                    svg.push_str(&generate_edge_label_svg(target_ix, target_iy, edge, direction_x, direction_y));
                }
            }
        }
    }
    // Close SVG
    svg.push_str("</g></svg>");

    svg
}

// A utility function to generate SVG multiline text with varying colors
// Now with support for a font size multiplier for the first line
fn generate_multiline_text_svg(
    x: f64,                              // X position (anchor point)
    y: f64,                              // Y position (top of first line)
    lines: &[(String, ColorString)],     // Text content and colors
    text_anchor: &str,                   // "start", "middle", or "end"
    font_family: &str,                   // Font family
    font_size: u32,                      // Base font size
    line_height: f64,                    // Space between lines
    dominant_baseline: Option<&str>,     // Optional baseline alignment
    first_line_size_multiplier: Option<f64>, // Optional font size multiplier for the first line
) -> String {
    let baseline_attr = if let Some(baseline) = dominant_baseline {
        format!(" dominant-baseline=\"{}\"", baseline)
    } else {
        String::new()
    };

    let mut svg = format!(
        r#"<text x="{}" y="{}" font-family="{}" font-size="{}"{} text-anchor="{}">"#,
        x, y, font_family, font_size, baseline_attr, text_anchor
    );

    for (i, (text, color)) in lines.iter().enumerate() {
        let dy = if i == 0 { "0".to_string() } else { format!("{}", line_height) };

        // Apply font size multiplier to first line if specified
        let font_size_attr = if i == 0 && first_line_size_multiplier.is_some() {
            let multiplier = first_line_size_multiplier.unwrap();
            let adjusted_size = (font_size as f64 * multiplier).round() as u32;
            format!(" font-size=\"{}\"", adjusted_size)
        } else {
            String::new()
        };

        svg.push_str(&format!(
            r#"<tspan x="{}" dy="{}"{} fill="{}">{}</tspan>"#,
            x, dy, font_size_attr, color.0, text
        ));
    }

    svg.push_str("</text>");
    svg
}

// Refactored node SVG generator with increased padding and first line font size multiplier
fn generate_node_svg(node: &TechNode) -> String {
    if let (Some(x), Some(y)) = (node.x, node.y) {
        // Colors
        let text_color = "#FFFFFF";
        let bold_text_color = ColorString::from("#FFFFFF");
        let normal_text_color = ColorString::from("#CCCCCC");

        // Get activity color for border
        let border_color = node.activity_color().0;

        // Get color based on node type
        let fill_color = match node.waypoint_type.as_str() {
            "RAW_MATERIAL" => "#091c26",
            "REFINED" => "#0a2533",
            "INDUSTRIAL" => "#0c3040",
            "ADVANCED" => "#0e3a4d",
            "CONSUMER" => "#10425a",
            _ => "#000000",
        };

        // Layout parameters
        let node_x = x - node.width / 2.0;
        let node_y = y - node.height / 2.0;
        let text_right_x = x + node.width / 2.0 - 16.0;  // Increased padding from 10px to 16px
        let line_height = 20.0;

        // Text styling
        let font_family = "Arial";
        let normal_font_size = 10;
        let title_font_size_multiplier = 1.3;  // Make first line 30% larger
        let border_width = 4;
        let corner_radius = 5;

        // Prepare text lines with their colors
        let text_lines = vec![
            // Name (bold, title font)
            (node.name.clone(), bold_text_color.clone()),
            // Waypoint symbol
            (node.waypoint_symbol.clone(), normal_text_color.clone()),
            // Waypoint type
            (node.waypoint_type.clone(), normal_text_color.clone()),
            // Activity
            (format!("A: {}", node.activity.to_string()), node.activity_color()),
            // Supply
            (format!("S: {}", node.supply.to_string()), node.supply_color()),
            // Volume
            (format!("v: {}", node.volume), normal_text_color.clone()),
            // Costs
            (format!("p: {}c", node.cost), normal_text_color.clone()),
        ];

        format!(
            r#"<g>
                <!-- Node background -->
                <rect
                    x="{node_x}"
                    y="{node_y}"
                    width="{}"
                    height="{}"
                    rx="{corner_radius}"
                    ry="{corner_radius}"
                    fill="{fill_color}"
                    stroke="{border_color}"
                    stroke-width="{border_width}"
                />

                <!-- Node text content (using multiline text) -->
                {}
            </g>"#,
            node.width,
            node.height,
            generate_multiline_text_svg(
                text_right_x,              // x position (right-aligned with increased padding)
                node_y + 30.0,             // y position (starting from top with padding)
                &text_lines,               // text content and colors
                "end",                     // right-aligned text
                font_family,               // font family
                normal_font_size,          // font size
                line_height,               // line spacing
                None,                      // no special baseline alignment
                Some(title_font_size_multiplier), // Increase size of first line
            )
        )
    } else {
        // Return empty string if node has no position
        String::new()
    }
}

// Refactored edge label SVG generator with increased padding
fn generate_edge_label_svg(x: f64, y: f64, edge: &TechEdge, direction_x: f64, direction_y: f64) -> String {
    // Label parameters
    let label_width = 105.0;
    let label_height = 60.0;  // Increased height from 55.0 to 60.0 for more padding
    let padding = 8.0;        // Increased padding from 5.0 to 8.0

    // Calculate offset distance to move label along direction vector
    // Normalize direction vector
    let direction_length = (direction_x * direction_x + direction_y * direction_y).sqrt();

    // Prevent division by zero
    if direction_length < 0.001 {
        return String::new(); // Return empty string if direction vector is too small
    }

    let norm_dir_x = direction_x / direction_length;
    let norm_dir_y = direction_y / direction_length;

    // Move label out from the intersection point along the direction vector
    let offset_distance = 30.0;
    let offset_x = norm_dir_x * offset_distance;
    let offset_y = norm_dir_y * offset_distance;

    // Apply offset to position
    let center_x = x + offset_x;
    let center_y = y + offset_y;

    // Calculate label corner position
    let label_x = center_x - label_width / 2.0;
    let label_y = center_y - label_height / 2.0;

    // Text styling
    let font_size = 10;
    let font_family = "Arial";
    let normal_text_color = ColorString::from("#eee");
    let line_height = 18.0;

    // Background styling
    let background_fill = "#666";
    let background_opacity = 1.0;
    let border_color = "gray";
    let border_width = 1;
    let corner_radius = 4;

    // Content from edge
    let cost = edge.cost;
    let volume = edge.volume;
    let activity = &edge.activity;
    let supply = &edge.supply;

    // New fields
    let distance = edge.distance.unwrap_or(0);
    let profit = edge.profit.unwrap_or(0);

    // Colors for activity and supply
    let activity_color = edge.activity_color();
    let supply_color = edge.supply_color();

    // Profit color (green for positive, red for negative)
    let profit_color = if profit >= 0 { "#22c55e" } else { "#ef4444" };

    // Prepare left and right text content
    let left_text_lines = vec![
        (format!("d: {}", distance), normal_text_color.clone()),
        (format!("v: {}", volume), normal_text_color.clone()),
        (format!("p: {}c", cost), normal_text_color.clone()),
    ];

    let right_text_lines = vec![
         (format!("A: {}", activity), activity_color),
         (format!("S: {}", supply), supply_color),
        (format!("{:+}", profit), ColorString::from(profit_color)),
    ];

    // Calculate vertical center position with adjustment for 3 lines of text
    // For perfect vertical centering, we position the middle line at the center
    // and adjust the first line position accordingly
    let total_text_height = line_height * 2.0; // Height of 3 lines (with 2 line-height spaces)
    let vertical_center = label_y + label_height / 2.0;
    let row1_y = vertical_center - total_text_height / 2.0;

    format!(
        r#"<g>
            <!-- Label background -->
            <rect
                x="{label_x}"
                y="{label_y}"
                width="{label_width}"
                height="{label_height}"
                rx="{corner_radius}"
                ry="{corner_radius}"
                fill="{background_fill}"
                fill-opacity="{background_opacity}"
                stroke="{border_color}"
                stroke-width="{border_width}"
            />

            <!-- Left-aligned text (using multiline text) -->
            {}

            <!-- Right-aligned text (using multiline text) -->
            {}
        </g>"#,
        generate_multiline_text_svg(
            label_x + padding,      // x position (left side with increased padding)
            row1_y,                 // y position (starting from top, adjusted for padding)
            &left_text_lines,       // text content and colors
            "start",                // left-aligned text
            font_family,            // font family
            font_size,              // font size
            line_height,            // line spacing
            Some("middle"),         // middle baseline alignment
            None,                   // no font size multiplier for first line
        ),
        generate_multiline_text_svg(
            label_x + label_width - padding,  // x position (right side with increased padding)
            row1_y,                           // y position (starting from top, adjusted for padding)
            &right_text_lines,                // text content and colors
            "end",                            // right-aligned text
            font_family,                      // font family
            font_size,                        // font size
            line_height,                      // line spacing
            Some("middle"),                   // middle baseline alignment
            None,                             // no font size multiplier for first line
        )
    )
}


// Helper function to calculate the intersection of a line with a node's rectangle border
fn calculate_node_border_intersection(
    node_x: f64,
    node_y: f64,
    node_width: f64,
    node_height: f64,
    line_x1: f64,
    line_y1: f64,
    line_x2: f64,
    line_y2: f64,
) -> (f64, f64) {
    // Calculate node rectangle boundaries
    let left = node_x - node_width / 2.0;
    let right = node_x + node_width / 2.0;
    let top = node_y - node_height / 2.0;
    let bottom = node_y + node_height / 2.0;

    // Direction vector of the line
    let dx = line_x2 - line_x1;
    let dy = line_y2 - line_y1;

    // Parameters for intersection with each edge
    let t_left = if dx != 0.0 {
        (left - line_x1) / dx
    } else {
        f64::INFINITY
    };
    let t_right = if dx != 0.0 {
        (right - line_x1) / dx
    } else {
        f64::INFINITY
    };
    let t_top = if dy != 0.0 {
        (top - line_y1) / dy
    } else {
        f64::INFINITY
    };
    let t_bottom = if dy != 0.0 {
        (bottom - line_y1) / dy
    } else {
        f64::INFINITY
    };

    // Find valid intersections (0 <= t <= 1)
    let mut valid_intersections = Vec::new();

    if t_left >= 0.0 && t_left <= 1.0 {
        let y = line_y1 + t_left * dy;
        if y >= top && y <= bottom {
            valid_intersections.push((t_left, left, y));
        }
    }

    if t_right >= 0.0 && t_right <= 1.0 {
        let y = line_y1 + t_right * dy;
        if y >= top && y <= bottom {
            valid_intersections.push((t_right, right, y));
        }
    }

    if t_top >= 0.0 && t_top <= 1.0 {
        let x = line_x1 + t_top * dx;
        if x >= left && x <= right {
            valid_intersections.push((t_top, x, top));
        }
    }

    if t_bottom >= 0.0 && t_bottom <= 1.0 {
        let x = line_x1 + t_bottom * dx;
        if x >= left && x <= right {
            valid_intersections.push((t_bottom, x, bottom));
        }
    }

    // Sort by parameter t and get the closest intersection
    valid_intersections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    if valid_intersections.is_empty() {
        // Fallback - if no intersection found, use the point on the node's center
        (node_x, node_y)
    } else {
        // Return the first valid intersection (closest to line_x1, line_y1)
        (valid_intersections[0].1, valid_intersections[0].2)
    }
}
