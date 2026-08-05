use std::fmt::Write as _;
use std::time::Instant;

use axodendron_core::{ValidationProfile, parse_swc};
use axodendron_svg::{RenderOptions, render_svg};

fn main() {
    let mut arguments = std::env::args().skip(1);
    let node_count: usize = arguments
        .next()
        .as_deref()
        .unwrap_or("100000")
        .parse()
        .expect("node count must be an integer");
    let budget_ms: u128 = arguments
        .next()
        .as_deref()
        .unwrap_or("2000")
        .parse()
        .expect("budget must be an integer number of milliseconds");
    assert!((2..=axodendron_core::MAX_NODE_COUNT).contains(&node_count));

    let mut source = String::with_capacity(node_count * 36);
    writeln!(source, "1 3 0 0 0 1 -1").unwrap();
    for id in 2..=node_count {
        writeln!(source, "{id} 3 {} 0 0 0.5 {}", id - 1, id - 1).unwrap();
    }
    let morphology = parse_swc(&source, ValidationProfile::IncfStrict)
        .morphology
        .expect("generated benchmark morphology must parse");

    let started = Instant::now();
    let document = render_svg(
        &morphology,
        &RenderOptions {
            display_tolerance: Some(0.01),
            ..RenderOptions::default()
        },
    )
    .expect("benchmark morphology must render");
    let elapsed = started.elapsed();
    assert_eq!(document.source_node_count as usize, node_count);
    assert_eq!(document.rendered_node_count, 2);
    assert!(document.svg.len() < 4096);
    println!(
        "display simplify+SVG: {node_count} nodes in {:.1} ms ({:.0} source nodes/s)",
        elapsed.as_secs_f64() * 1000.0,
        node_count as f64 / elapsed.as_secs_f64()
    );
    assert!(
        elapsed.as_millis() <= budget_ms,
        "SVG benchmark exceeded {budget_ms} ms: {:.1} ms",
        elapsed.as_secs_f64() * 1000.0
    );
}
