use std::fmt::Write as _;
use std::io::Cursor;

use axodendron_core::{
    AnalysisDomain, AnalysisOptions, Morphology, NodeId, SectionBoundaryPolicy, SimplifyOptions,
    TransformError, ValidationProfile, Vec3, parse_swc,
};
use swc_neuron::AnySwc;

fn strict(source: &str) -> Morphology {
    let parsed = parse_swc(source, ValidationProfile::IncfStrict);
    assert!(parsed.is_valid(), "{:?}", parsed.diagnostics);
    parsed.morphology.unwrap()
}

#[test]
fn reports_lexical_errors_with_exact_locations() {
    let cases = [
        ("1 1 0 nope 0 1 -1\n", "SWC_INVALID_NUMBER", 1, 7),
        ("1 x 0 0 0 1 -1\n", "SWC_INVALID_INTEGER", 1, 3),
        ("1 1 0 0 0 NaN -1\n", "SWC_NONFINITE_NUMBER", 1, 11),
        ("1 1 0 0 0 1\n", "SWC_COLUMN_COUNT", 1, 12),
        ("1 1 0 0 0 1 -1 extra\n", "SWC_COLUMN_COUNT", 1, 16),
        ("\u{2003}1 1 0 nope 0 1 -1\n", "SWC_INVALID_NUMBER", 1, 8),
    ];
    for (source, code, line, column) in cases {
        let parsed = parse_swc(source, ValidationProfile::Permissive);
        let diagnostic = parsed
            .diagnostics
            .iter()
            .find(|item| item.code == code)
            .unwrap();
        assert_eq!(diagnostic.line, Some(line), "{source:?}");
        assert_eq!(diagnostic.column, Some(column), "{source:?}");
    }
}

#[test]
fn strict_profile_rejects_each_standard_ordering_violation() {
    let cases = [
        ("2 1 0 0 0 1 -1\n", "SWC_STRICT_ID_SEQUENCE"),
        ("1 1 0 0 0 1 -2\n", "SWC_STRICT_ROOT_SENTINEL"),
        ("1 3 0 0 0 1 2\n2 1 1 0 0 1 -1\n", "SWC_STRICT_FIRST_ROOT"),
        ("1 1 0 0 0 1 -1\n2 3 1 0 0 1 -1\n", "SWC_MULTIPLE_ROOTS"),
    ];
    for (source, code) in cases {
        let parsed = parse_swc(source, ValidationProfile::IncfStrict);
        assert!(!parsed.is_valid(), "{source:?}");
        assert!(
            parsed.diagnostics.iter().any(|item| item.code == code),
            "{source:?}"
        );
    }
}

#[test]
fn permissive_profile_preserves_valid_nonstandard_forests() {
    let source = "20 9 2 0 0 1 10\n30 3 0 2 0 1 -7\n10 1 0 0 0 1 -1\n";
    let parsed = parse_swc(source, ValidationProfile::Permissive);
    assert!(parsed.is_valid(), "{:?}", parsed.diagnostics);
    let morphology = parsed.morphology.unwrap();
    assert_eq!(morphology.ids(), &[20, 30, 10]);
    assert_eq!(morphology.roots().count(), 2);
    assert_eq!(morphology.kinds(), &[9, 3, 1]);
    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|item| item.code == "SWC_CUSTOM_TYPE")
    );
    assert!(
        parsed
            .diagnostics
            .iter()
            .any(|item| item.code == "SWC_NONSTANDARD_ROOT_SENTINEL")
    );
}

#[test]
fn agrees_with_swc_neuron_on_standard_rows() {
    let source = "# header\n1 1 0 0 0 2 -1\n2 3 1.25 -2.5 3e-1 0.75 1\n3 42 2 0 0 0.5 2\n";
    let ours = strict(source);
    let reference = AnySwc::from_reader(Cursor::new(source.as_bytes())).unwrap();
    reference.validate(true).unwrap();
    assert_eq!(ours.len(), reference.samples.len());
    for (ix, sample) in reference.samples.iter().enumerate() {
        let structure: isize = sample.structure.into();
        assert_eq!(ours.ids()[ix], sample.sample_id as i64);
        assert_eq!(ours.kinds()[ix], structure as i32);
        assert_eq!(
            ours.positions()[ix],
            Vec3::new(sample.x, sample.y, sample.z)
        );
        assert_eq!(ours.radii()[ix], sample.radius);
    }
}

#[test]
fn semantic_fingerprint_ignores_text_formatting_but_tracks_content() {
    let compact = strict("1 1 0 0 0 1 -1\n2 3 1 0 0 1 1\n");
    let formatted =
        strict("# same morphology\r\n  1   1  -0.0 0 0 1.0 -1\r\n2 3 1e0 0 0 1 1 # node\r\n");
    assert_eq!(compact.fingerprint(), formatted.fingerprint());
    assert_ne!(compact.source_fingerprint(), formatted.source_fingerprint());

    let changed = strict("1 1 0 0 0 1 -1\n2 3 2 0 0 1 1\n");
    assert_ne!(compact.fingerprint(), changed.fingerprint());
}

#[test]
fn morphology_cbor_roundtrip_rebuilds_derived_topology() {
    let morphology = strict("1 1 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 1 0 1 2\n4 3 2 -1 0 1 2\n");
    let mut bytes = Vec::new();
    ciborium::into_writer(&morphology, &mut bytes).unwrap();
    let decoded: Morphology = ciborium::from_reader(bytes.as_slice()).unwrap();
    assert_eq!(decoded, morphology);
    assert_eq!(
        decoded
            .children(decoded.index_of(NodeId(2)).unwrap())
            .count(),
        2
    );
    assert!(
        bytes.len() < 500,
        "canonical payload unexpectedly large: {} bytes",
        bytes.len()
    );
}

#[test]
fn generated_trees_preserve_all_analysis_invariants() {
    let mut random = Lcg::new(0x5eed_1234_cafe_babe);
    for case in 0..96 {
        let node_count = 2 + (random.next_u32() as usize % 126);
        let mut source = String::new();
        writeln!(source, "1 1 0 0 0 1 -1").unwrap();
        for id in 2..=node_count {
            let parent = 1 + random.next_u32() as usize % (id - 1);
            let kind = 2 + random.next_u32() % 4;
            let x = id as f64 * 0.25;
            let y = (random.next_u32() % 1000) as f64 / 37.0;
            let z = (random.next_u32() % 1000) as f64 / 53.0;
            writeln!(source, "{id} {kind} {x} {y} {z} 0.5 {parent}").unwrap();
        }
        let morphology = strict(&source);
        let analysis = morphology.analyze_with_options(AnalysisOptions {
            domain: AnalysisDomain::Raw,
            section_boundaries: SectionBoundaryPolicy::TopologyOnly,
        });
        assert_eq!(
            analysis.summary.node_count as usize, node_count,
            "case {case}"
        );
        assert_eq!(
            analysis.summary.edge_count as usize,
            node_count - 1,
            "case {case}"
        );
        assert_eq!(analysis.topology.root_ids, vec![1], "case {case}");

        for ix in 0..node_count {
            let parent = morphology.parents_raw()[ix];
            if parent == axodendron_core::NONE_NODE {
                assert_eq!(analysis.root_path_length.values[ix], 0.0);
            } else {
                let expected = analysis.root_path_length.values[parent as usize]
                    + morphology.positions()[ix].distance(morphology.positions()[parent as usize]);
                assert!((analysis.root_path_length.values[ix] - expected).abs() < 1e-12);
                assert!(
                    analysis.strahler_order.values[parent as usize]
                        >= analysis.strahler_order.values[ix]
                );
            }
        }

        let expected_sections: usize = morphology
            .roots()
            .chain(
                (0..node_count)
                    .filter_map(|ix| morphology.index_of(NodeId(ix as i64 + 1)))
                    .filter(|node| {
                        morphology.parent(*node).is_some() && morphology.child_count(*node) > 1
                    }),
            )
            .map(|node| morphology.child_count(node))
            .sum();
        assert_eq!(
            analysis.sections.sections.len(),
            expected_sections,
            "case {case}"
        );
    }
}

#[test]
fn transforms_are_pure_and_fingerprints_describe_results() {
    let original =
        strict("1 1 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 0 0 1 2\n4 3 3 0 0 1 3\n5 2 2 1 0 1 3\n");
    let snapshot = original.clone();
    let rerooted = original.reroot(NodeId(4)).unwrap();
    assert_eq!(original, snapshot);
    assert_ne!(original.fingerprint(), rerooted.fingerprint());
    assert_eq!(
        original.fingerprint(),
        rerooted.reroot(NodeId(1)).unwrap().fingerprint()
    );

    let subtree = original.subtree(NodeId(3)).unwrap();
    let path = original.path_between(NodeId(4), NodeId(5)).unwrap();
    assert_eq!(subtree.fingerprint(), path.fingerprint());
    let different_path = original.path_between(NodeId(1), NodeId(4)).unwrap();
    assert_ne!(subtree.fingerprint(), different_path.fingerprint());
    assert_eq!(original.source_fingerprint(), subtree.source_fingerprint());
}

#[test]
fn transform_errors_are_typed_and_non_destructive() {
    let forest = parse_swc(
        "10 1 0 0 0 1 -1\n20 3 1 0 0 1 -1\n",
        ValidationProfile::Permissive,
    )
    .morphology
    .unwrap();
    assert_eq!(
        forest.subtree(NodeId(999)),
        Err(TransformError::UnknownNode(999))
    );
    assert_eq!(
        forest.path_between(NodeId(10), NodeId(20)),
        Err(TransformError::DifferentComponents(10, 20))
    );
    assert_eq!(
        forest.simplify(&SimplifyOptions {
            tolerance: f64::NAN,
            ..Default::default()
        }),
        Err(TransformError::InvalidTolerance)
    );
}

#[test]
fn deep_chain_is_iterative_and_simplifies_to_endpoints() {
    const NODES: usize = 50_000;
    let mut source = String::with_capacity(NODES * 32);
    writeln!(source, "1 1 0 0 0 1 -1").unwrap();
    for id in 2..=NODES {
        writeln!(source, "{id} 3 {} 0 0 1 {}", id - 1, id - 1).unwrap();
    }
    let morphology = strict(&source);
    let analysis = morphology.analyze_with_options(AnalysisOptions {
        domain: AnalysisDomain::Raw,
        section_boundaries: SectionBoundaryPolicy::TopologyOnly,
    });
    assert_eq!(analysis.summary.node_count, NODES as u32);
    assert_eq!(analysis.summary.max_root_path_length, (NODES - 1) as f64);
    assert_eq!(analysis.sections.sections.len(), 1);
    let simplified = morphology
        .simplify(&SimplifyOptions {
            tolerance: 0.01,
            preserve_type_changes: false,
            preserve_soma: false,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(simplified.ids(), &[1, NODES as i64]);
}

#[test]
fn wide_multifurcation_has_standard_strahler_behavior() {
    const LEAVES: usize = 10_000;
    let mut source = String::with_capacity(LEAVES * 32);
    writeln!(source, "1 1 0 0 0 1 -1").unwrap();
    for id in 2..=LEAVES + 1 {
        writeln!(source, "{id} 3 {id} 0 0 1 1").unwrap();
    }
    let morphology = strict(&source);
    let analysis = morphology.analyze_with_options(AnalysisOptions {
        domain: AnalysisDomain::Raw,
        section_boundaries: SectionBoundaryPolicy::TopologyOnly,
    });
    assert_eq!(analysis.summary.branch_point_count, 1);
    assert_eq!(analysis.summary.terminal_count, LEAVES as u32);
    assert_eq!(analysis.summary.section_count, LEAVES as u32);
    assert_eq!(analysis.strahler_order.values[0], 2.0);
}

#[test]
fn intrinsic_metrics_obey_rigid_motion_and_scaling_invariants() {
    let base =
        strict("1 3 0 0 0 2 -1\n2 3 3 0 0 1 1\n3 3 3 4 0 0.5 2\n4 3 3 4 12 0.25 3\n").analyze_raw();
    let rigid = strict("1 3 10 -7 5 2 -1\n2 3 10 -4 5 1 1\n3 3 6 -4 5 0.5 2\n4 3 6 -4 17 0.25 3\n")
        .analyze_raw();
    assert!((base.summary.total_cable_length - rigid.summary.total_cable_length).abs() < 1e-12);
    assert!(
        (base.summary.radius_metrics.neurite_surface_area.unwrap()
            - rigid.summary.radius_metrics.neurite_surface_area.unwrap())
        .abs()
            < 1e-12
    );
    assert!(
        (base.summary.radius_metrics.neurite_volume.unwrap()
            - rigid.summary.radius_metrics.neurite_volume.unwrap())
        .abs()
            < 1e-12
    );

    let scaled =
        strict("1 3 0 0 0 4 -1\n2 3 6 0 0 2 1\n3 3 6 8 0 1 2\n4 3 6 8 24 0.5 3\n").analyze_raw();
    assert!(
        (scaled.summary.total_cable_length - 2.0 * base.summary.total_cable_length).abs() < 1e-12
    );
    assert!(
        (scaled.summary.radius_metrics.neurite_surface_area.unwrap()
            - 4.0 * base.summary.radius_metrics.neurite_surface_area.unwrap())
        .abs()
            < 1e-10
    );
    assert!(
        (scaled.summary.radius_metrics.neurite_volume.unwrap()
            - 8.0 * base.summary.radius_metrics.neurite_volume.unwrap())
        .abs()
            < 1e-10
    );
}

#[test]
fn sholl_endpoint_and_tangency_rules_are_unambiguous() {
    let radial = strict("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 0 0 1 2\n");
    let result = radial.sholl_3d(Vec3::default(), &[0.0, 1.0, 2.0, 3.0]);
    assert_eq!(
        result
            .bins
            .iter()
            .map(|bin| bin.intersections)
            .collect::<Vec<_>>(),
        vec![0, 1, 1, 0]
    );

    let tangent = strict("1 3 -1 1 0 1 -1\n2 3 1 1 0 1 1\n");
    assert_eq!(
        tangent.sholl_3d(Vec3::default(), &[1.0]).bins[0].intersections,
        1
    );
}

struct Lcg(u64);

impl Lcg {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u32(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 32) as u32
    }
}
