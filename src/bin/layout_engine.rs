use itertools::Itertools;
use layout::backends::svg::SVGWriter;
use layout::core::utils::save_to_file;
use layout::gv;
use layout::gv::GraphBuilder;
use layout::std_shapes::shapes::{Element, ShapeKind};
use layout::topo::layout::VisualGraph;
use layout::topo::placer::Placer;

fn main() {
    let contents = include_str!("../../spacetraders.dot");
    let mut parser = gv::DotParser::new(&contents);

    match parser.process() {
        Ok(g) => {
            gv::dump_ast(&g);

            let mut gb = GraphBuilder::new();
            gb.visit_graph(&g);
            let mut graph = gb.get();


            generate_svg(&mut graph);

        },
        Err(err) => {
            parser.print_error();
            #[cfg(feature = "log")]
            log::error!("Error: {}", err);
        }
    }
}

fn generate_svg(graph: &mut VisualGraph) {
    let mut svg = SVGWriter::new();
    graph.do_it(
        false,
        false,
        false,
        &mut svg,
    );

    let mut positions = vec![];

    for node in graph.iter_nodes() {
        let pos = graph.pos(node);
        let element = graph.element(node);
        match &element.shape {
            ShapeKind::None => {}
            ShapeKind::Box(content) => {
                positions.push(pos.middle());
            }
            ShapeKind::Circle(_) => {}
            ShapeKind::DoubleCircle(_) => {}
            ShapeKind::Record(_) => {}
            ShapeKind::Connector(_) => {}
        }
        //println!("node: {:?}; element: {:?}", node, element);
    }

    for (col_id, column) in graph.dag.ranks().iter().enumerate() {
        for(row_id, &node) in column.iter().enumerate() {
            let pos = graph.pos(node);
            let element = graph.element(node);
            match &element.shape {
                ShapeKind::None => {}
                ShapeKind::Box(content) => {
                    println!("rank(rank idx: {}, inner_rank_idx: {}); (y: {}, x: {}); node: {}", row_id, col_id, pos.middle().y, pos.middle().x, content.replace("\n", " - "));
                }
                ShapeKind::Circle(_) => {}
                ShapeKind::DoubleCircle(_) => {}
                ShapeKind::Record(_) => {}
                ShapeKind::Connector(_) => {}
            }
        }
    }


    let x_positions_overview = positions.iter().map(|pos| (pos.x.round() as u32, pos.y.round() as u32)).into_group_map();
    let y_positions_overview = positions.iter().map(|pos| (pos.y.round() as u32, pos.x.round() as u32)).into_group_map();

    let x_positions = x_positions_overview.keys().into_iter().sorted().collect_vec();
    let y_positions = y_positions_overview.keys().into_iter().sorted().collect_vec();

    // dbg!(&x_positions_overview);
    // dbg!(&y_positions_overview);
    //
    // dbg!(x_positions);
    // dbg!(y_positions);

    let content = svg.finalize();

    let output_path = "test.svg";
    save_to_file(output_path, &content).expect("should be able to write svg");

    let res = save_to_file(output_path, &content);
    if let Err(err) = res {
        log::error!("Could not write the file {}", output_path);
        log::error!("Error {}", err);
        return;
    }
    log::info!("Wrote {}", output_path);
}
