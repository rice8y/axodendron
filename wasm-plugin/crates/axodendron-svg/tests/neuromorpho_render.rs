use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use axodendron_core::{ValidationProfile, Vec3, parse_swc};
use axodendron_svg::{
    ColorMode, GeometryMode, RadiusMode, RenderOptions, SomaMode, View, render_svg,
};

#[derive(Debug)]
struct Case {
    id: String,
    node_count: usize,
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repository root")
}

fn fixture_dir(root: &Path) -> PathBuf {
    env::var_os("AXODENDRON_NEUROMORPHO_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/neuromorpho"))
}

fn cases(root: &Path) -> Vec<Case> {
    let manifest = fs::read_to_string(root.join("wasm-plugin/test-data/neuromorpho-cases.tsv"))
        .expect("read NeuroMorpho case manifest");
    manifest
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let columns: Vec<&str> = line.split('\t').collect();
            Case {
                id: columns[0].to_owned(),
                node_count: columns[4].parse().expect("node count"),
            }
        })
        .collect()
}

#[test]
#[ignore = "downloads private NeuroMorpho.Org fixtures; run the fetch script first"]
fn renders_every_case_across_views_and_styles() {
    let root = repository_root();
    let fixture_dir = fixture_dir(&root);
    for case in cases(&root) {
        let source = fs::read_to_string(fixture_dir.join(format!("{}.swc", case.id)))
            .unwrap_or_else(|error| panic!("missing {} fixture: {error}", case.id));
        let parsed = parse_swc(&source, ValidationProfile::IncfStrict);
        let morphology = parsed.morphology.unwrap_or_else(|| {
            panic!(
                "{} did not parse strictly: {:?}",
                case.id, parsed.diagnostics
            )
        });
        assert_eq!(morphology.len(), case.node_count, "{}", case.id);

        for view in [
            View::Xy,
            View::Xz,
            View::Yz,
            View::Orthographic {
                direction: Vec3::new(1.0, 1.0, 1.0),
                up: Vec3::new(0.0, 0.0, 1.0),
            },
        ] {
            for geometry in [GeometryMode::Tapered, GeometryMode::Skeleton] {
                for radius_mode in [RadiusMode::Readable, RadiusMode::Physical] {
                    for color in [
                        ColorMode::ByType,
                        ColorMode::Uniform {
                            color: "#111827".to_owned(),
                        },
                    ] {
                        let is_by_type = matches!(color, ColorMode::ByType);
                        let document = render_svg(
                            &morphology,
                            &RenderOptions {
                                width: 1200.0,
                                height: 1200.0,
                                padding: 36.0,
                                background: Some("#ffffff".to_owned()),
                                view: view.clone(),
                                geometry,
                                radius_mode,
                                color,
                                ..Default::default()
                            },
                        )
                        .unwrap_or_else(|error| panic!("{} render failed: {error}", case.id));

                        assert_eq!(document.rendered_node_count as usize, case.node_count);
                        assert!(document.pixels_per_unit.is_finite(), "{}", case.id);
                        assert!(!document.svg.contains("NaN"), "{}", case.id);
                        assert!(!document.svg.contains("inf"), "{}", case.id);
                        assert!(!document.svg.contains("<script"), "{}", case.id);
                        assert!(document.svg.ends_with("</svg>"), "{}", case.id);
                        if geometry == GeometryMode::Tapered {
                            assert!(document.svg.contains("<path"), "{}", case.id);
                        } else {
                            assert!(document.svg.contains("<line"), "{}", case.id);
                        }

                        if morphology.kinds().contains(&1) {
                            assert_eq!(
                                document.svg.matches("fill=\"#d62728\"").count(),
                                usize::from(is_by_type),
                                "{} must render the default equivalent soma as one body",
                                case.id
                            );
                        }
                    }
                }
            }
        }

        if morphology.kinds().contains(&1) {
            for soma_mode in [SomaMode::Encoded, SomaMode::RawPoints] {
                let document = render_svg(
                    &morphology,
                    &RenderOptions {
                        width: 1200.0,
                        height: 1200.0,
                        padding: 36.0,
                        view: View::Xy,
                        geometry: GeometryMode::Tapered,
                        soma_mode,
                        ..Default::default()
                    },
                )
                .unwrap_or_else(|error| panic!("{} soma render failed: {error}", case.id));
                assert!(!document.svg.contains("NaN"), "{}", case.id);
                assert!(document.svg.ends_with("</svg>"), "{}", case.id);
            }
        }
    }
}
