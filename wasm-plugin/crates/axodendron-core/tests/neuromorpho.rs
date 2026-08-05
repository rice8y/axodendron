use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use axodendron_core::{Morphology, Severity, SomaClass, ValidationProfile, parse_swc};
use swc_neuron::AnySwc;

#[derive(Debug)]
struct Case {
    id: String,
    node_count: usize,
    reference_max_path: f64,
    reference_cable_length: f64,
    compare_cable: bool,
}

#[test]
#[ignore = "requires ./scripts/fetch-neuromorpho-fixtures.sh"]
fn standardized_files_pass_strict_validation_and_independent_parsing() {
    for case in cases() {
        let source = source(&case);
        let parsed = parse_swc(&source, ValidationProfile::IncfStrict);
        let errors: Vec<_> = parsed
            .diagnostics
            .iter()
            .filter(|item| item.severity == Severity::Error)
            .collect();
        assert!(errors.is_empty(), "{}: {errors:?}", case.id);
        let morphology = parsed.morphology.unwrap();
        assert_eq!(morphology.len(), case.node_count, "{}", case.id);
        let expected_soma = match case.id.as_str() {
            "NMO_150000" | "NMO_200000" | "NMO_40000" => SomaClass::Ambiguous,
            "NMO_100000" | "NMO_12500" | "NMO_160000" | "NMO_20000" | "NMO_25000"
            | "NMO_267500" | "NMO_275000" | "NMO_285000" | "NMO_300000" | "NMO_87500" => {
                SomaClass::Absent
            }
            _ => SomaClass::ThreePoint,
        };
        assert_eq!(morphology.soma_class(), expected_soma, "{}", case.id);

        let reference = AnySwc::from_reader(Cursor::new(source.as_bytes())).unwrap();
        reference.validate(true).unwrap();
        assert_eq!(reference.samples.len(), morphology.len(), "{}", case.id);
        for (ix, sample) in reference.samples.iter().enumerate() {
            assert_eq!(
                sample.sample_id as i64,
                morphology.ids()[ix],
                "{} row {ix}",
                case.id
            );
            assert_eq!(
                sample.parent_id.map(|id| id as i64),
                morphology
                    .parent(axodendron_node(&morphology, ix))
                    .map(|parent| morphology.id(parent).0),
                "{} row {ix}",
                case.id
            );
        }
    }
}

#[test]
#[ignore = "requires ./scripts/fetch-neuromorpho-fixtures.sh"]
fn official_path_and_compatible_cable_measurements_agree() {
    for case in cases() {
        let morphology = morphology(&case);
        // NeuroMorpho.Org's archived path/cable values include the encoded SWC
        // graph convention. Compare them to Axodendron's explicit raw domain.
        let summary = morphology.analyze_raw().summary;
        assert!(
            (summary.max_root_path_length - case.reference_max_path).abs() <= 0.25,
            "{}: Axodendron {}, NeuroMorpho {}",
            case.id,
            summary.max_root_path_length,
            case.reference_max_path
        );
        if case.compare_cable {
            assert!(
                (summary.total_cable_length - case.reference_cable_length).abs() <= 0.1,
                "{}: Axodendron {}, NeuroMorpho {}",
                case.id,
                summary.total_cable_length,
                case.reference_cable_length
            );
        }
    }
}

#[test]
#[ignore = "requires ./scripts/fetch-neuromorpho-fixtures.sh"]
fn real_morphologies_preserve_topology_and_cbor_invariants() {
    for case in cases() {
        let morphology = morphology(&case);
        let raw = morphology.analyze_raw();
        let analysis = morphology.analyze();
        assert_eq!(raw.topology.node_ids.len(), morphology.len(), "{}", case.id);
        assert_eq!(
            raw.topology.parent_ids.len(),
            morphology.len(),
            "{}",
            case.id
        );
        assert_eq!(
            raw.topology.component_ids.len(),
            morphology.len(),
            "{}",
            case.id
        );
        assert_eq!(
            raw.summary.edge_count + raw.summary.root_count,
            raw.summary.node_count
        );
        assert_eq!(
            analysis.summary.node_count as usize,
            morphology.kinds().iter().filter(|kind| **kind != 1).count(),
            "{}",
            case.id
        );
        assert!(
            analysis.topology.terminal_ids.iter().all(|id| morphology
                .kind(morphology.index_of(axodendron_core::NodeId(*id)).unwrap())
                != 1),
            "{}",
            case.id
        );
        assert!(analysis.summary.total_cable_length.is_finite());
        assert!(analysis.summary.max_root_path_length.is_finite());
        let section_length: f64 = analysis
            .sections
            .sections
            .iter()
            .map(|section| section.length)
            .sum();
        assert!(
            (section_length - analysis.summary.total_cable_length).abs()
                <= 1e-9 * analysis.summary.total_cable_length.max(1.0),
            "{}: sections {section_length}, cable {}",
            case.id,
            analysis.summary.total_cable_length
        );
        assert!(
            analysis
                .root_path_length
                .values
                .iter()
                .all(|value| value.is_finite())
        );
        assert!(
            analysis
                .strahler_order
                .values
                .iter()
                .all(|value| *value >= 1.0)
        );

        let mut bytes = Vec::new();
        ciborium::into_writer(&morphology, &mut bytes).unwrap();
        let decoded: Morphology = ciborium::from_reader(bytes.as_slice()).unwrap();
        assert_eq!(
            decoded.fingerprint(),
            morphology.fingerprint(),
            "{}",
            case.id
        );
        assert_eq!(decoded, morphology, "{}", case.id);
    }
}

fn axodendron_node(morphology: &Morphology, ix: usize) -> axodendron_core::NodeIx {
    morphology
        .index_of(axodendron_core::NodeId(morphology.ids()[ix]))
        .unwrap()
}

fn morphology(case: &Case) -> Morphology {
    let parsed = parse_swc(&source(case), ValidationProfile::IncfStrict);
    assert!(parsed.is_valid(), "{}: {:?}", case.id, parsed.diagnostics);
    parsed.morphology.unwrap()
}

fn source(case: &Case) -> String {
    fs::read_to_string(cache_dir().join(format!("{}.swc", case.id))).unwrap_or_else(|error| {
        panic!(
            "{}: {error}; run ./scripts/fetch-neuromorpho-fixtures.sh",
            case.id
        )
    })
}

fn cache_dir() -> PathBuf {
    std::env::var_os("AXODENDRON_NEUROMORPHO_DIR").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../target/neuromorpho"),
        PathBuf::from,
    )
}

fn cases() -> Vec<Case> {
    let manifest = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-data/neuromorpho-cases.tsv"),
    )
    .unwrap();
    manifest
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(fields.len(), 8, "malformed fixture manifest row: {line}");
            Case {
                id: fields[0].to_owned(),
                node_count: fields[4].parse().unwrap(),
                reference_max_path: fields[5].parse().unwrap(),
                reference_cable_length: fields[6].parse().unwrap(),
                compare_cable: fields[7].parse().unwrap(),
            }
        })
        .collect()
}
