use std::collections::HashSet;

use axodendron_core::{
    AnalysisDomain, MeasureOptions, MetricData, MetricParameters, MetricSpec, Morphology,
    SectionBoundaryPolicy, SelectionQuery, ValidationProfile, metric_registry, parse_swc,
};

fn strict(source: &str) -> Morphology {
    parse_swc(source, ValidationProfile::IncfStrict)
        .morphology
        .unwrap()
}

fn field_values(morphology: &Morphology, metric: MetricSpec) -> Vec<f64> {
    let result = morphology
        .measure(&MeasureOptions {
            metrics: vec![metric],
            selection: SelectionQuery {
                domain: AnalysisDomain::Raw,
                ..Default::default()
            },
            section_boundaries: SectionBoundaryPolicy::TopologyOnly,
        })
        .unwrap()
        .remove(0);
    assert!(result.missing.is_empty(), "{:?}", result.missing);
    match result.data {
        MetricData::NodeField(field) => field.values,
        MetricData::SectionField(field) => field.values,
        MetricData::BifurcationField(field) => field.values,
        MetricData::MorphologyMetric(_) => panic!("expected an entity field"),
    }
}

#[test]
fn definition_difference_fixture_covers_every_cross_validated_metric() {
    let registry: HashSet<String> = metric_registry().into_iter().map(|item| item.id).collect();
    let fixture = include_str!("../../../test-data/metric-cross-validation.tsv");
    let mut covered = HashSet::new();
    for line in fixture.lines().filter(|line| !line.starts_with('#')) {
        let fields: Vec<&str> = line.split('\t').collect();
        assert_eq!(fields.len(), 12, "malformed fixture line: {line}");
        assert!(
            registry.contains(fields[0]),
            "unknown metric in fixture: {line}"
        );
        assert_eq!(fields[1], "1");
        assert!(fields[10].parse::<f64>().unwrap() > 0.0);
        assert!(
            !fields[11].is_empty(),
            "definition differences must be explicit"
        );
        covered.insert(fields[0]);
    }
    for required in [
        "local-bifurcation-angle",
        "remote-bifurcation-angle",
        "sibling-ratio",
        "partition-asymmetry-terminal",
        "taper-rate",
        "segment-meander-angle",
        "rall-ratio",
    ] {
        assert!(covered.contains(required));
    }
}

#[test]
fn analytic_fixtures_match_the_documented_upstream_definitions() {
    let branch = strict("1 3 -1 0 0 1.5 -1\n2 3 0 0 0 1 1\n3 3 1 0 0 0.5 2\n4 3 0 1 0 0.5 2\n");
    for id in ["local-bifurcation-angle", "remote-bifurcation-angle"] {
        let values = field_values(
            &branch,
            MetricSpec {
                id: id.to_owned(),
                parameters: MetricParameters::default(),
            },
        );
        assert_eq!(values.len(), 1);
        assert!((values[0] - 90.0).abs() < 1e-12, "{id}");
    }
    assert_eq!(
        field_values(
            &branch,
            MetricSpec {
                id: "sibling-ratio".to_owned(),
                parameters: MetricParameters::default(),
            }
        ),
        vec![1.0]
    );
    assert_eq!(
        field_values(
            &branch,
            MetricSpec {
                id: "partition-asymmetry-terminal".to_owned(),
                parameters: MetricParameters::default(),
            }
        ),
        vec![0.0]
    );
    let rall = field_values(
        &branch,
        MetricSpec {
            id: "rall-ratio".to_owned(),
            parameters: MetricParameters::default(),
        },
    );
    assert!((rall[0] - 2.0 / 2.0_f64.powf(1.5)).abs() < 1e-12);

    let taper = strict("1 3 0 0 0 2 -1\n2 3 1 0 0 1.5 1\n3 3 2 0 0 1 2\n");
    assert_eq!(
        field_values(
            &taper,
            MetricSpec {
                id: "taper-rate".to_owned(),
                parameters: MetricParameters::default(),
            }
        ),
        vec![-1.0]
    );

    let meander = strict("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 1 1 0 1 2\n4 3 1 2 0 1 3\n");
    let angles = field_values(
        &meander,
        MetricSpec {
            id: "segment-meander-angle".to_owned(),
            parameters: MetricParameters::default(),
        },
    );
    assert_eq!(angles, vec![90.0, 0.0]);
}
