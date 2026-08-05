use std::fmt::Write as _;
use std::time::Instant;

use axodendron_core::{AnalysisDomain, AnalysisOptions, ValidationProfile, parse_swc};

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
    assert!((1..=axodendron_core::MAX_NODE_COUNT).contains(&node_count));

    let mut source = String::with_capacity(node_count * 36);
    writeln!(source, "1 3 0 0 0 1 -1").unwrap();
    for id in 2..=node_count {
        writeln!(source, "{id} 3 {} 0 0 0.5 {}", id - 1, id - 1).unwrap();
    }

    let started = Instant::now();
    let parsed = parse_swc(&source, ValidationProfile::IncfStrict);
    let morphology = parsed
        .morphology
        .unwrap_or_else(|| panic!("generated benchmark input failed: {:?}", parsed.diagnostics));
    let analysis = morphology.analyze_with_options(AnalysisOptions {
        domain: AnalysisDomain::Raw,
        ..AnalysisOptions::default()
    });
    let elapsed = started.elapsed();
    assert_eq!(analysis.summary.node_count as usize, node_count);
    assert_eq!(analysis.summary.edge_count as usize, node_count - 1);
    assert_eq!(analysis.sections.sections.len(), 1);
    assert_eq!(
        analysis.summary.max_root_path_length,
        (node_count - 1) as f64
    );
    println!(
        "core parse+analysis: {node_count} nodes in {:.1} ms ({:.0} nodes/s)",
        elapsed.as_secs_f64() * 1000.0,
        node_count as f64 / elapsed.as_secs_f64()
    );
    assert!(
        elapsed.as_millis() <= budget_ms,
        "core benchmark exceeded {budget_ms} ms: {:.1} ms",
        elapsed.as_secs_f64() * 1000.0
    );
}
