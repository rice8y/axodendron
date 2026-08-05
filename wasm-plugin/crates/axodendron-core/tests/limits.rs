use std::fmt::Write as _;

use axodendron_core::{
    MAX_NODE_COUNT, ResampleOptions, TransformError, ValidationProfile, parse_swc,
};

fn chain(node_count: usize) -> String {
    let mut source = String::with_capacity(node_count * 36);
    writeln!(source, "1 3 0 0 0 1 -1").unwrap();
    for id in 2..=node_count {
        writeln!(source, "{id} 3 {} 0 0 0.5 {}", id - 1, id - 1).unwrap();
    }
    source
}

#[test]
#[ignore = "exercises the full 250,000-node resource boundary; run from scripts/check.sh"]
fn parser_accepts_the_limit_and_rejects_the_next_node() {
    let mut source = chain(MAX_NODE_COUNT);
    let at_limit = parse_swc(&source, ValidationProfile::IncfStrict);
    assert!(at_limit.is_valid(), "{:?}", at_limit.diagnostics);
    writeln!(
        source,
        "{} 3 {} 0 0 0.5 {}",
        MAX_NODE_COUNT + 1,
        MAX_NODE_COUNT,
        MAX_NODE_COUNT
    )
    .unwrap();
    let over_limit = parse_swc(&source, ValidationProfile::IncfStrict);
    assert!(over_limit.morphology.is_none());
    assert!(
        over_limit
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "SWC_NODE_LIMIT")
    );
}

#[test]
#[ignore = "exercises bounded resampling expansion; run from scripts/check.sh"]
fn resampling_stops_at_the_global_node_limit() {
    let morphology = parse_swc(
        "1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n",
        ValidationProfile::IncfStrict,
    )
    .morphology
    .unwrap();
    let error = morphology
        .resample(&ResampleOptions {
            step: 1.0 / (MAX_NODE_COUNT as f64 + 1.0),
            protected_ids: Vec::new(),
        })
        .unwrap_err();
    assert_eq!(error, TransformError::NodeLimitExceeded);
}
