//! Deterministic, radius-aware orthographic SVG rendering for Axodendron.

use std::collections::HashMap;
use std::fmt::Write;

use axodendron_core::{Morphology, Projection, SimplifyOptions, SomaClass, Vec2, Vec3};
use serde::{Deserialize, Serialize};

pub const MAX_SVG_BYTES: usize = 64 * 1024 * 1024;
const MAX_STYLE_BYTES: usize = 128;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum View {
    #[default]
    Xy,
    Xz,
    Yz,
    Orthographic {
        direction: Vec3,
        up: Vec3,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GeometryMode {
    Skeleton,
    #[default]
    Tapered,
}

impl View {
    fn projection(&self) -> Result<Projection, RenderError> {
        match self {
            Self::Xy => Ok(Projection::xy()),
            Self::Xz => Ok(Projection::xz()),
            Self::Yz => Ok(Projection::yz()),
            Self::Orthographic { direction, up } => Projection::look(*direction, *up)
                .map_err(|error| RenderError::Projection(error.to_string())),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScalarField {
    pub node_ids: Vec<i64>,
    #[serde(deserialize_with = "axodendron_core::serde_number::vec_f64")]
    pub values: Vec<f64>,
    #[serde(
        default,
        deserialize_with = "axodendron_core::serde_number::option_f64"
    )]
    pub minimum: Option<f64>,
    #[serde(
        default,
        deserialize_with = "axodendron_core::serde_number::option_f64"
    )]
    pub maximum: Option<f64>,
    #[serde(default = "default_colormap")]
    pub colormap: String,
    #[serde(default)]
    pub fingerprint: Option<String>,
}

fn default_colormap() -> String {
    "viridis".to_owned()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ColorMode {
    #[default]
    ByType,
    Uniform {
        color: String,
    },
    Scalar(ScalarField),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RadiusMode {
    Physical,
    #[default]
    Readable,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SomaMode {
    #[default]
    EquivalentSphere,
    Encoded,
    RawPoints,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderOptions {
    #[serde(default = "default_width")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub width: f64,
    #[serde(default = "default_height")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub height: f64,
    #[serde(default = "default_padding")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub padding: f64,
    #[serde(default = "default_stroke")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub stroke_width: f64,
    #[serde(default)]
    pub geometry: GeometryMode,
    #[serde(default)]
    pub radius_mode: RadiusMode,
    #[serde(default)]
    pub soma_mode: SomaMode,
    #[serde(default = "default_minimum_radius")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub minimum_radius: f64,
    #[serde(default = "default_maximum_radius")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub maximum_radius: f64,
    #[serde(default = "default_maximum_soma_radius")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub maximum_soma_radius: f64,
    #[serde(default = "default_radius_scale")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub radius_scale: f64,
    #[serde(default = "default_radius_exponent")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub radius_exponent: f64,
    #[serde(default = "default_soma_scale")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub soma_scale: f64,
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub outline_color: Option<String>,
    #[serde(default = "default_outline_width")]
    #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
    pub outline_width: f64,
    #[serde(default)]
    pub view: View,
    #[serde(default)]
    pub color: ColorMode,
    #[serde(default)]
    #[serde(deserialize_with = "axodendron_core::serde_number::option_f64")]
    pub display_tolerance: Option<f64>,
    #[serde(default)]
    pub include_nodes: bool,
    #[serde(default)]
    pub overlay_node_ids: Vec<i64>,
    #[serde(default = "default_true")]
    pub strict_overlay_ids: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            width: default_width(),
            height: default_height(),
            padding: default_padding(),
            stroke_width: default_stroke(),
            geometry: GeometryMode::default(),
            radius_mode: RadiusMode::default(),
            soma_mode: SomaMode::default(),
            minimum_radius: default_minimum_radius(),
            maximum_radius: default_maximum_radius(),
            maximum_soma_radius: default_maximum_soma_radius(),
            radius_scale: default_radius_scale(),
            radius_exponent: default_radius_exponent(),
            soma_scale: default_soma_scale(),
            background: None,
            outline_color: None,
            outline_width: default_outline_width(),
            view: View::default(),
            color: ColorMode::default(),
            display_tolerance: None,
            include_nodes: false,
            overlay_node_ids: Vec::new(),
            strict_overlay_ids: true,
        }
    }
}

const fn default_width() -> f64 {
    800.0
}

const fn default_height() -> f64 {
    600.0
}

const fn default_padding() -> f64 {
    24.0
}

const fn default_stroke() -> f64 {
    2.0
}

const fn default_minimum_radius() -> f64 {
    1.0
}

const fn default_maximum_radius() -> f64 {
    18.0
}

const fn default_maximum_soma_radius() -> f64 {
    96.0
}

const fn default_radius_scale() -> f64 {
    1.0
}

const fn default_radius_exponent() -> f64 {
    0.5
}

const fn default_soma_scale() -> f64 {
    1.0
}

const fn default_outline_width() -> f64 {
    1.0
}

const fn default_true() -> bool {
    true
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectedBounds {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectedNode {
    pub node_id: i64,
    pub x: f64,
    pub y: f64,
    pub depth: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SvgDocument {
    pub svg: String,
    pub projected_bounds: ProjectedBounds,
    pub nodes: Vec<ProjectedNode>,
    pub pixels_per_unit: f64,
    pub rendered_node_count: u32,
    pub source_node_count: u32,
    pub report: RenderReport,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderReport {
    pub radius_mode: RadiusMode,
    pub soma_mode: SomaMode,
    pub floored_radius_count: u32,
    pub capped_radius_count: u32,
    pub simplified_node_count: u32,
    pub overlay_node_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RenderError {
    InvalidCanvas,
    InvalidDisplayTolerance,
    Projection(String),
    ScalarLengthMismatch,
    InvalidScalarRange,
    DuplicateScalarNode(i64),
    ScalarFingerprintMismatch,
    ScalarFieldTooLarge,
    UnknownScalarNode(i64),
    UnknownOverlayNode(i64),
    OverlayListTooLarge,
    UnknownColormap(String),
    InvalidStyle,
    OutputTooLarge,
    EmptyMorphology,
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCanvas => f.write_str(
                "canvas dimensions, padding, stroke, and radius controls must be finite and valid",
            ),
            Self::InvalidDisplayTolerance => {
                f.write_str("display tolerance must be finite and non-negative")
            }
            Self::Projection(message) => write!(f, "invalid projection: {message}"),
            Self::ScalarLengthMismatch => {
                f.write_str("scalar node_ids and values must have equal lengths")
            }
            Self::InvalidScalarRange => {
                f.write_str("scalar maximum must be greater than or equal to minimum")
            }
            Self::DuplicateScalarNode(id) => {
                write!(f, "scalar field contains duplicate node id {id}")
            }
            Self::ScalarFingerprintMismatch => {
                f.write_str("scalar field fingerprint does not match the morphology")
            }
            Self::ScalarFieldTooLarge => {
                f.write_str("scalar field contains more entries than the morphology")
            }
            Self::UnknownScalarNode(id) => write!(f, "scalar node id {id} does not exist"),
            Self::UnknownOverlayNode(id) => write!(f, "overlay node id {id} does not exist"),
            Self::OverlayListTooLarge => {
                f.write_str("overlay node list exceeds the morphology-dependent limit")
            }
            Self::UnknownColormap(name) => write!(f, "unknown colormap {name:?}"),
            Self::InvalidStyle => f.write_str(
                "SVG colors must use bounded safe CSS color syntax without external URLs",
            ),
            Self::OutputTooLarge => write!(
                f,
                "SVG output exceeds the {MAX_SVG_BYTES}-byte renderer limit"
            ),
            Self::EmptyMorphology => f.write_str("cannot render an empty morphology"),
        }
    }
}

impl std::error::Error for RenderError {}

pub fn render_svg(
    morphology: &Morphology,
    options: &RenderOptions,
) -> Result<SvgDocument, RenderError> {
    validate_options(options)?;
    if morphology.is_empty() {
        return Err(RenderError::EmptyMorphology);
    }
    let maximum_overlay_entries = morphology.len().saturating_mul(4).max(1024);
    if options.overlay_node_ids.len() > maximum_overlay_entries {
        return Err(RenderError::OverlayListTooLarge);
    }
    let mut overlay_ids = std::collections::HashSet::with_capacity(
        options.overlay_node_ids.len().min(morphology.len()),
    );
    let mut protected_overlay_ids = Vec::new();
    for id in &options.overlay_node_ids {
        if morphology.index_of(axodendron_core::NodeId(*id)).is_none() {
            if options.strict_overlay_ids {
                return Err(RenderError::UnknownOverlayNode(*id));
            }
            continue;
        }
        if overlay_ids.insert(*id) {
            protected_overlay_ids.push(*id);
        }
    }
    let scalar = scalar_colors(&options.color, morphology)?;
    let rendered = if let Some(tolerance) = options.display_tolerance {
        if !tolerance.is_finite() || tolerance < 0.0 {
            return Err(RenderError::InvalidDisplayTolerance);
        }
        morphology
            .simplify(&SimplifyOptions {
                tolerance,
                preserve_soma: true,
                preserve_type_changes: true,
                protected_ids: protected_overlay_ids,
            })
            .map_err(|_| RenderError::InvalidDisplayTolerance)?
    } else {
        morphology.clone()
    };
    let projection = options.view.projection()?;
    let projected: Vec<(Vec2, f64)> = rendered
        .positions()
        .iter()
        .copied()
        .map(|point| projection.project(point))
        .collect();
    let bounds = projected_bounds(&projected);
    let three_point_center = three_point_soma_center(&rendered, options.soma_mode);
    let equivalent_soma =
        equivalent_soma_display(&rendered, &projected, options.soma_mode, three_point_center);
    let (scale, screen) = fit_to_canvas(
        &rendered,
        &projected,
        bounds,
        options,
        equivalent_soma.as_ref(),
    );

    let mut svg = String::new();
    write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" role=\"img\" aria-label=\"Neuronal morphology\">",
        number(options.width),
        number(options.height),
        number(options.width),
        number(options.height)
    )
    .unwrap();
    svg.push_str("<metadata>Generated by Axodendron; geometry units: ");
    svg.push_str(&escape_xml(rendered.units()));
    svg.push_str("</metadata>");
    if let Some(background) = &options.background {
        write!(
            svg,
            "<rect width=\"100%\" height=\"100%\" fill=\"{}\"/>",
            escape_xml(background)
        )
        .unwrap();
    }
    let mut segments: Vec<Segment> = rendered
        .parents_raw()
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, parent)| *parent != axodendron_core::NONE_NODE)
        .map(|(child, parent)| Segment {
            child,
            parent: parent as usize,
            depth: (projected[parent as usize].1 + projected[child].1) * 0.5,
        })
        .collect();
    segments.sort_by(|a, b| {
        a.depth
            .total_cmp(&b.depth)
            .then_with(|| a.child.cmp(&b.child))
    });
    match options.geometry {
        GeometryMode::Skeleton => {
            render_skeleton_segments(&mut svg, &rendered, &screen, &segments, options, &scalar)?
        }
        GeometryMode::Tapered => render_tapered_segments(
            &mut svg, &rendered, &screen, &segments, scale, options, &scalar,
        )?,
    }
    render_soma(
        &mut svg,
        &rendered,
        &screen,
        scale,
        options,
        &scalar,
        three_point_center,
        equivalent_soma.as_ref(),
    )?;
    if options.include_nodes {
        render_node_markers(&mut svg, &rendered, &screen, options, &scalar)?;
    }
    svg.push_str("</svg>");
    ensure_svg_budget(&svg)?;

    let nodes = rendered
        .ids()
        .iter()
        .enumerate()
        .filter(|(_, id)| options.include_nodes || overlay_ids.contains(id))
        .map(|(ix, id)| ProjectedNode {
            node_id: *id,
            x: screen[ix].x,
            y: screen[ix].y,
            depth: projected[ix].1,
        })
        .collect();
    let (floored_radius_count, capped_radius_count) =
        radius_adjustment_counts(&rendered, scale, options);
    Ok(SvgDocument {
        svg,
        projected_bounds: bounds,
        nodes,
        pixels_per_unit: scale,
        rendered_node_count: rendered.len() as u32,
        source_node_count: morphology.len() as u32,
        report: RenderReport {
            radius_mode: options.radius_mode,
            soma_mode: options.soma_mode,
            floored_radius_count,
            capped_radius_count,
            simplified_node_count: (morphology.len() - rendered.len()) as u32,
            overlay_node_count: overlay_ids.len() as u32,
        },
    })
}

#[derive(Clone, Copy, Debug)]
struct Segment {
    child: usize,
    parent: usize,
    depth: f64,
}

#[derive(Clone, Copy, Debug)]
struct EquivalentSomaDisplay {
    projected_center: Vec2,
    representative: usize,
    radius_index: usize,
}

#[allow(clippy::too_many_arguments)]
fn render_skeleton_segments(
    svg: &mut String,
    morphology: &Morphology,
    screen: &[Vec2],
    segments: &[Segment],
    options: &RenderOptions,
    scalar: &Option<ScalarColors>,
) -> Result<(), RenderError> {
    svg.push_str("<g fill=\"none\" stroke-linecap=\"round\" stroke-linejoin=\"round\">");
    for segment in segments {
        if hide_soma_segment(morphology, segment, options) {
            continue;
        }
        let color = node_color(morphology, segment.child, &options.color, scalar);
        if let Some(outline) = outline(options) {
            write_line(
                svg,
                screen[segment.parent],
                screen[segment.child],
                outline,
                options.stroke_width + options.outline_width * 2.0,
                None,
            );
        }
        write_line(
            svg,
            screen[segment.parent],
            screen[segment.child],
            &color,
            options.stroke_width,
            Some(morphology.ids()[segment.child]),
        );
        ensure_svg_budget(svg)?;
    }
    svg.push_str("</g>");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_tapered_segments(
    svg: &mut String,
    morphology: &Morphology,
    screen: &[Vec2],
    segments: &[Segment],
    scale: f64,
    options: &RenderOptions,
    scalar: &Option<ScalarColors>,
) -> Result<(), RenderError> {
    svg.push_str("<g stroke-linejoin=\"round\">");
    for segment in segments {
        if hide_soma_segment(morphology, segment, options) {
            continue;
        }
        let color = node_color(morphology, segment.child, &options.color, scalar);
        let child_radius = display_radius(morphology, segment.child, scale, options, false);
        let mut parent_radius = display_radius(morphology, segment.parent, scale, options, false);
        if morphology.kinds()[segment.parent] == 1 && morphology.kinds()[segment.child] != 1 {
            parent_radius = child_radius;
        }
        write_tapered_shape(
            svg,
            screen[segment.parent],
            screen[segment.child],
            parent_radius,
            child_radius,
            &color,
            outline(options),
            options.outline_width,
            morphology.ids()[segment.child],
        );
        write_circle(
            svg,
            screen[segment.child],
            child_radius,
            &color,
            outline(options),
            options.outline_width,
            Some(morphology.ids()[segment.child]),
        );
        ensure_svg_budget(svg)?;
    }
    for (ix, parent) in morphology.parents_raw().iter().copied().enumerate() {
        if parent == axodendron_core::NONE_NODE && morphology.kinds()[ix] != 1 {
            let color = node_color(morphology, ix, &options.color, scalar);
            write_circle(
                svg,
                screen[ix],
                display_radius(morphology, ix, scale, options, false),
                &color,
                outline(options),
                options.outline_width,
                Some(morphology.ids()[ix]),
            );
            ensure_svg_budget(svg)?;
        }
    }
    svg.push_str("</g>");
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_soma(
    svg: &mut String,
    morphology: &Morphology,
    screen: &[Vec2],
    scale: f64,
    options: &RenderOptions,
    scalar: &Option<ScalarColors>,
    three_point_center: Option<usize>,
    equivalent_soma: Option<&EquivalentSomaDisplay>,
) -> Result<(), RenderError> {
    let all_soma_nodes: Vec<usize> = morphology
        .kinds()
        .iter()
        .enumerate()
        .filter_map(|(ix, kind)| (*kind == 1).then_some(ix))
        .collect();
    if let Some(equivalent_soma) = equivalent_soma {
        let divisor = all_soma_nodes.len() as f64;
        let center = Vec2::new(
            all_soma_nodes.iter().map(|ix| screen[*ix].x).sum::<f64>() / divisor,
            all_soma_nodes.iter().map(|ix| screen[*ix].y).sum::<f64>() / divisor,
        );
        let radius = soma_radius(morphology, equivalent_soma.radius_index, scale, options);
        let representative = equivalent_soma.representative;
        let color = node_color(morphology, representative, &options.color, scalar);
        write_circle(
            svg,
            center,
            radius,
            &color,
            outline(options),
            options.outline_width,
            Some(morphology.ids()[representative]),
        );
        ensure_svg_budget(svg)?;
        return Ok(());
    }
    if options.soma_mode == SomaMode::Encoded && morphology.soma_class() == SomaClass::ThreePoint {
        if let Some(center) = three_point_center {
            let center_node = morphology
                .index_of(axodendron_core::NodeId(morphology.ids()[center]))
                .expect("center index belongs to the morphology");
            let auxiliary: Vec<usize> = morphology
                .children(center_node)
                .filter(|child| morphology.kind(*child) == 1)
                .map(|child| child.get() as usize)
                .collect();
            if auxiliary.len() == 2 {
                let color = node_color(morphology, center, &options.color, scalar);
                let radius = soma_radius(morphology, center, scale, options);
                if let Some(outline) = outline(options) {
                    write_line(
                        svg,
                        screen[auxiliary[0]],
                        screen[auxiliary[1]],
                        outline,
                        radius * 2.0 + options.outline_width * 2.0,
                        None,
                    );
                }
                write_line(
                    svg,
                    screen[auxiliary[0]],
                    screen[auxiliary[1]],
                    &color,
                    radius * 2.0,
                    Some(morphology.ids()[center]),
                );
                ensure_svg_budget(svg)?;
                return Ok(());
            }
        }
    }
    let soma_nodes: Vec<usize> = if let Some(center) = three_point_center {
        vec![center]
    } else {
        all_soma_nodes
    };
    for ix in soma_nodes {
        let color = node_color(morphology, ix, &options.color, scalar);
        let radius = soma_radius(morphology, ix, scale, options);
        write_circle(
            svg,
            screen[ix],
            radius,
            &color,
            outline(options),
            options.outline_width,
            Some(morphology.ids()[ix]),
        );
        ensure_svg_budget(svg)?;
    }
    Ok(())
}

fn equivalent_soma_display(
    morphology: &Morphology,
    projected: &[(Vec2, f64)],
    soma_mode: SomaMode,
    three_point_center: Option<usize>,
) -> Option<EquivalentSomaDisplay> {
    if soma_mode != SomaMode::EquivalentSphere {
        return None;
    }
    let soma_nodes: Vec<usize> = morphology
        .kinds()
        .iter()
        .enumerate()
        .filter_map(|(ix, kind)| (*kind == 1).then_some(ix))
        .collect();
    if soma_nodes.is_empty() {
        return None;
    }
    let divisor = soma_nodes.len() as f64;
    let projected_center = Vec2::new(
        soma_nodes.iter().map(|ix| projected[*ix].0.x).sum::<f64>() / divisor,
        soma_nodes.iter().map(|ix| projected[*ix].0.y).sum::<f64>() / divisor,
    );
    let mut by_radius = soma_nodes.clone();
    by_radius.sort_by(|a, b| morphology.radii()[*a].total_cmp(&morphology.radii()[*b]));
    Some(EquivalentSomaDisplay {
        projected_center,
        representative: three_point_center.unwrap_or(soma_nodes[0]),
        radius_index: by_radius[by_radius.len() / 2],
    })
}

fn hide_soma_segment(morphology: &Morphology, segment: &Segment, options: &RenderOptions) -> bool {
    let soma_edge =
        morphology.kinds()[segment.parent] == 1 && morphology.kinds()[segment.child] == 1;
    soma_edge
        && (options.soma_mode == SomaMode::EquivalentSphere
            || (options.soma_mode == SomaMode::Encoded
                && morphology.soma_class() == SomaClass::ThreePoint))
}

fn render_node_markers(
    svg: &mut String,
    morphology: &Morphology,
    screen: &[Vec2],
    options: &RenderOptions,
    scalar: &Option<ScalarColors>,
) -> Result<(), RenderError> {
    for (ix, point) in screen.iter().copied().enumerate() {
        if morphology.kinds()[ix] == 1 {
            continue;
        }
        let color = node_color(morphology, ix, &options.color, scalar);
        write_circle(
            svg,
            point,
            options.stroke_width * 1.2,
            &color,
            None,
            0.0,
            Some(morphology.ids()[ix]),
        );
        ensure_svg_budget(svg)?;
    }
    Ok(())
}

fn three_point_soma_center(morphology: &Morphology, soma_mode: SomaMode) -> Option<usize> {
    if morphology.soma_class() != SomaClass::ThreePoint || soma_mode == SomaMode::RawPoints {
        return None;
    }
    morphology
        .kinds()
        .iter()
        .enumerate()
        .find(|(ix, kind)| {
            **kind == 1
                && (morphology.parents_raw()[*ix] == axodendron_core::NONE_NODE
                    || morphology.kinds()[morphology.parents_raw()[*ix] as usize] != 1)
        })
        .map(|(ix, _)| ix)
}

fn display_radius(
    morphology: &Morphology,
    ix: usize,
    scale: f64,
    options: &RenderOptions,
    soma: bool,
) -> f64 {
    let soma_scale = if soma { options.soma_scale } else { 1.0 };
    let physical = morphology.radii()[ix].max(0.0) * scale * options.radius_scale * soma_scale;
    let compressed = if soma
        || options.radius_mode == RadiusMode::Physical
        || physical <= options.minimum_radius
    {
        physical
    } else {
        options.minimum_radius * (physical / options.minimum_radius).powf(options.radius_exponent)
    };
    let maximum = if soma {
        options.maximum_soma_radius
    } else {
        options.maximum_radius
    };
    compressed.clamp(options.minimum_radius, maximum)
}

fn radius_adjustment_counts(
    morphology: &Morphology,
    scale: f64,
    options: &RenderOptions,
) -> (u32, u32) {
    let mut floored = 0_u32;
    let mut capped = 0_u32;
    for (ix, radius) in morphology.radii().iter().copied().enumerate() {
        let soma = morphology.kinds()[ix] == 1;
        let physical = radius.max(0.0)
            * scale
            * options.radius_scale
            * if soma { options.soma_scale } else { 1.0 };
        floored += u32::from(physical < options.minimum_radius);
        let maximum = if soma {
            options.maximum_soma_radius
        } else {
            options.maximum_radius
        };
        capped += u32::from(physical > maximum);
    }
    (floored, capped)
}

fn soma_radius(morphology: &Morphology, ix: usize, scale: f64, options: &RenderOptions) -> f64 {
    display_radius(morphology, ix, scale, options, true)
        .max(options.stroke_width * 2.25)
        .min(options.maximum_soma_radius)
}

fn fit_to_canvas(
    morphology: &Morphology,
    projected: &[(Vec2, f64)],
    bounds: ProjectedBounds,
    options: &RenderOptions,
    equivalent_soma: Option<&EquivalentSomaDisplay>,
) -> (f64, Vec<Vec2>) {
    let drawable_width = options.width - options.padding * 2.0;
    let drawable_height = options.height - options.padding * 2.0;
    let span_x = bounds.max.x - bounds.min.x;
    let span_y = bounds.max.y - bounds.min.y;
    let mut scale = if span_x <= 1e-12 && span_y <= 1e-12 {
        1.0
    } else {
        let x_scale = if span_x <= 1e-12 {
            f64::INFINITY
        } else {
            drawable_width / span_x
        };
        let y_scale = if span_y <= 1e-12 {
            f64::INFINITY
        } else {
            drawable_height / span_y
        };
        x_scale.min(y_scale)
    };

    // Radius floors and caps make the expanded bounds piecewise rather than
    // linearly proportional to scale. A short monotone iteration converges to
    // the largest scale whose actual painted geometry fits the padded canvas.
    for _ in 0..8 {
        let expanded =
            expanded_screen_bounds(morphology, projected, scale, options, equivalent_soma);
        let width = (expanded.max.x - expanded.min.x).max(1e-12);
        let height = (expanded.max.y - expanded.min.y).max(1e-12);
        let factor = (drawable_width / width)
            .min(drawable_height / height)
            .min(1.0);
        if factor >= 1.0 - 1e-10 {
            break;
        }
        scale *= factor;
    }

    let expanded = expanded_screen_bounds(morphology, projected, scale, options, equivalent_soma);
    let painted_width = expanded.max.x - expanded.min.x;
    let painted_height = expanded.max.y - expanded.min.y;
    let offset_x = options.padding + (drawable_width - painted_width) / 2.0 - expanded.min.x;
    let offset_y = options.padding + (drawable_height - painted_height) / 2.0 - expanded.min.y;
    let screen = projected
        .iter()
        .map(|(point, _)| Vec2::new(point.x * scale + offset_x, -point.y * scale + offset_y))
        .collect();
    (scale, screen)
}

fn expanded_screen_bounds(
    morphology: &Morphology,
    projected: &[(Vec2, f64)],
    scale: f64,
    options: &RenderOptions,
    equivalent_soma: Option<&EquivalentSomaDisplay>,
) -> ProjectedBounds {
    let mut min = Vec2::new(f64::INFINITY, f64::INFINITY);
    let mut max = Vec2::new(f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (ix, (point, _)) in projected.iter().enumerate() {
        let Some(radius) = visual_radius(morphology, ix, scale, options, equivalent_soma.is_some())
        else {
            continue;
        };
        let x = point.x * scale;
        let y = -point.y * scale;
        min.x = min.x.min(x - radius);
        min.y = min.y.min(y - radius);
        max.x = max.x.max(x + radius);
        max.y = max.y.max(y + radius);
    }
    if let Some(equivalent_soma) = equivalent_soma {
        let outline = if outline(options).is_some() {
            options.outline_width
        } else {
            0.0
        };
        let radius =
            soma_radius(morphology, equivalent_soma.radius_index, scale, options) + outline;
        let x = equivalent_soma.projected_center.x * scale;
        let y = -equivalent_soma.projected_center.y * scale;
        min.x = min.x.min(x - radius);
        min.y = min.y.min(y - radius);
        max.x = max.x.max(x + radius);
        max.y = max.y.max(y + radius);
    }
    // Every non-empty morphology has at least one visual root or soma. Keep a
    // defensive fallback so malformed future geometry modes cannot emit NaN.
    if !min.x.is_finite() {
        ProjectedBounds {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(0.0, 0.0),
        }
    } else {
        ProjectedBounds { min, max }
    }
}

fn visual_radius(
    morphology: &Morphology,
    ix: usize,
    scale: f64,
    options: &RenderOptions,
    hide_soma_nodes: bool,
) -> Option<f64> {
    if morphology.kinds()[ix] == 1 {
        if hide_soma_nodes {
            return None;
        }
        let outline = if outline(options).is_some() {
            options.outline_width
        } else {
            0.0
        };
        return Some(soma_radius(morphology, ix, scale, options) + outline);
    }

    let mut radius = match options.geometry {
        GeometryMode::Skeleton => options.stroke_width / 2.0,
        GeometryMode::Tapered => display_radius(morphology, ix, scale, options, false),
    };
    if outline(options).is_some() {
        radius += options.outline_width;
    }
    if options.include_nodes {
        radius = radius.max(options.stroke_width * 1.2);
    }
    Some(radius)
}

fn outline(options: &RenderOptions) -> Option<&str> {
    (options.outline_width > 0.0)
        .then_some(options.outline_color.as_deref())
        .flatten()
}

fn write_line(svg: &mut String, a: Vec2, b: Vec2, color: &str, width: f64, node_id: Option<i64>) {
    write!(
        svg,
        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"",
        number(a.x),
        number(a.y),
        number(b.x),
        number(b.y),
        escape_xml(color),
        number(width),
    )
    .unwrap();
    if let Some(node_id) = node_id {
        write!(svg, " data-node=\"{node_id}\"").unwrap();
    }
    svg.push_str("/>");
}

#[allow(clippy::too_many_arguments)]
fn write_tapered_shape(
    svg: &mut String,
    a: Vec2,
    b: Vec2,
    radius_a: f64,
    radius_b: f64,
    color: &str,
    outline: Option<&str>,
    outline_width: f64,
    node_id: i64,
) {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let length = dx.hypot(dy);
    if length <= 1e-12 {
        return;
    }
    let normal = Vec2::new(-dy / length, dx / length);
    let a_left = Vec2::new(a.x + normal.x * radius_a, a.y + normal.y * radius_a);
    let a_right = Vec2::new(a.x - normal.x * radius_a, a.y - normal.y * radius_a);
    let b_left = Vec2::new(b.x + normal.x * radius_b, b.y + normal.y * radius_b);
    let b_right = Vec2::new(b.x - normal.x * radius_b, b.y - normal.y * radius_b);
    write!(
        svg,
        "<path d=\"M {} {} L {} {} L {} {} L {} {} Z\" fill=\"{}\"",
        number(a_left.x),
        number(a_left.y),
        number(b_left.x),
        number(b_left.y),
        number(b_right.x),
        number(b_right.y),
        number(a_right.x),
        number(a_right.y),
        escape_xml(color),
    )
    .unwrap();
    if let Some(outline) = outline {
        write!(
            svg,
            " stroke=\"{}\" stroke-width=\"{}\"",
            escape_xml(outline),
            number(outline_width * 2.0),
        )
        .unwrap();
    }
    write!(svg, " data-node=\"{node_id}\"/>").unwrap();
}

#[allow(clippy::too_many_arguments)]
fn write_circle(
    svg: &mut String,
    point: Vec2,
    radius: f64,
    color: &str,
    outline: Option<&str>,
    outline_width: f64,
    node_id: Option<i64>,
) {
    write!(
        svg,
        "<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"",
        number(point.x),
        number(point.y),
        number(radius),
        escape_xml(color),
    )
    .unwrap();
    if let Some(outline) = outline {
        write!(
            svg,
            " stroke=\"{}\" stroke-width=\"{}\"",
            escape_xml(outline),
            number(outline_width * 2.0),
        )
        .unwrap();
    }
    if let Some(node_id) = node_id {
        write!(svg, " data-node=\"{node_id}\"").unwrap();
    }
    svg.push_str("/>");
}

fn validate_options(options: &RenderOptions) -> Result<(), RenderError> {
    let valid = options.width.is_finite()
        && options.height.is_finite()
        && options.padding.is_finite()
        && options.stroke_width.is_finite()
        && options.minimum_radius.is_finite()
        && options.maximum_radius.is_finite()
        && options.maximum_soma_radius.is_finite()
        && options.radius_scale.is_finite()
        && options.radius_exponent.is_finite()
        && options.soma_scale.is_finite()
        && options.outline_width.is_finite()
        && options.width > 0.0
        && options.height > 0.0
        && options.padding >= 0.0
        && options.stroke_width > 0.0
        && options.minimum_radius > 0.0
        && options.maximum_radius >= options.minimum_radius
        && options.maximum_soma_radius >= options.minimum_radius
        && options.radius_scale > 0.0
        && options.radius_exponent > 0.0
        && options.radius_exponent <= 1.0
        && options.soma_scale > 0.0
        && options.outline_width >= 0.0
        && options.width > options.padding * 2.0
        && options.height > options.padding * 2.0;
    if !valid {
        return Err(RenderError::InvalidCanvas);
    }
    let colors = [
        options.background.as_deref(),
        options.outline_color.as_deref(),
        match &options.color {
            ColorMode::Uniform { color } => Some(color.as_str()),
            _ => None,
        },
    ];
    if colors.into_iter().flatten().all(valid_style_text) {
        Ok(())
    } else {
        Err(RenderError::InvalidStyle)
    }
}

fn valid_style_text(value: &str) -> bool {
    if value.len() > MAX_STYLE_BYTES || value.is_empty() {
        return false;
    }
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        return matches!(hex.len(), 3 | 4 | 6 | 8)
            && hex.chars().all(|character| character.is_ascii_hexdigit());
    }
    if value
        .chars()
        .all(|character| character.is_ascii_alphabetic())
    {
        return true;
    }
    let lowercase = value.to_ascii_lowercase();
    for prefix in ["rgb(", "rgba(", "hsl(", "hsla("] {
        if let Some(body) = lowercase
            .strip_prefix(prefix)
            .and_then(|body| body.strip_suffix(')'))
        {
            return !body.is_empty()
                && body.chars().all(|character| {
                    character.is_ascii_digit()
                        || matches!(character, '.' | ',' | '%' | '+' | '-' | ' ' | '\t')
                });
        }
    }
    false
}

fn ensure_svg_budget(svg: &str) -> Result<(), RenderError> {
    if svg.len() <= MAX_SVG_BYTES {
        Ok(())
    } else {
        Err(RenderError::OutputTooLarge)
    }
}

fn projected_bounds(projected: &[(Vec2, f64)]) -> ProjectedBounds {
    let mut min = projected[0].0;
    let mut max = min;
    for (point, _) in &projected[1..] {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
    }
    ProjectedBounds { min, max }
}

struct ScalarColors {
    values: HashMap<i64, f64>,
    minimum: f64,
    maximum: f64,
    colormap: String,
}

fn scalar_colors(
    mode: &ColorMode,
    morphology: &Morphology,
) -> Result<Option<ScalarColors>, RenderError> {
    let ColorMode::Scalar(field) = mode else {
        return Ok(None);
    };
    if field
        .fingerprint
        .as_deref()
        .is_some_and(|fingerprint| fingerprint != morphology.fingerprint())
    {
        return Err(RenderError::ScalarFingerprintMismatch);
    }
    if field.node_ids.len() != field.values.len() {
        return Err(RenderError::ScalarLengthMismatch);
    }
    if field.node_ids.len() > morphology.len() {
        return Err(RenderError::ScalarFieldTooLarge);
    }
    if field.colormap != "viridis" && field.colormap != "magma" {
        return Err(RenderError::UnknownColormap(field.colormap.clone()));
    }
    let mut values = HashMap::with_capacity(field.node_ids.len());
    for (id, value) in field
        .node_ids
        .iter()
        .copied()
        .zip(field.values.iter().copied())
    {
        if morphology.index_of(axodendron_core::NodeId(id)).is_none() {
            return Err(RenderError::UnknownScalarNode(id));
        }
        if values.insert(id, value).is_some() {
            return Err(RenderError::DuplicateScalarNode(id));
        }
    }
    let finite: Vec<f64> = field
        .values
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    let minimum = field
        .minimum
        .unwrap_or_else(|| finite.iter().copied().fold(f64::INFINITY, f64::min));
    let maximum = field
        .maximum
        .unwrap_or_else(|| finite.iter().copied().fold(f64::NEG_INFINITY, f64::max));
    if minimum.is_finite() && maximum.is_finite() && maximum < minimum {
        return Err(RenderError::InvalidScalarRange);
    }
    Ok(Some(ScalarColors {
        values,
        minimum,
        maximum,
        colormap: field.colormap.clone(),
    }))
}

fn node_color(
    morphology: &Morphology,
    ix: usize,
    mode: &ColorMode,
    scalar: &Option<ScalarColors>,
) -> String {
    match mode {
        ColorMode::ByType => type_color(morphology.kinds()[ix]).to_owned(),
        ColorMode::Uniform { color } => color.clone(),
        ColorMode::Scalar(_) => {
            let scalar = scalar.as_ref().unwrap();
            let value = scalar
                .values
                .get(&morphology.ids()[ix])
                .copied()
                .unwrap_or(f64::NAN);
            if !value.is_finite() {
                return "#9ca3af".to_owned();
            }
            let span = scalar.maximum - scalar.minimum;
            let t = if span.is_finite() && span > 0.0 {
                ((value - scalar.minimum) / span).clamp(0.0, 1.0)
            } else {
                0.5
            };
            colormap(t, &scalar.colormap)
        }
    }
}

fn type_color(kind: i32) -> &'static str {
    match kind {
        1 => "#d62728",
        2 => "#0072b2",
        3 => "#009e73",
        4 => "#009e73",
        5 => "#e69f00",
        6 => "#56b4e9",
        7 => "#cc79a7",
        _ => "#4d4d4d",
    }
}

fn colormap(t: f64, name: &str) -> String {
    let stops: &[(f64, [u8; 3])] = match name {
        "magma" => &[
            (0.0, [0, 0, 4]),
            (0.25, [81, 18, 124]),
            (0.5, [183, 55, 121]),
            (0.75, [252, 137, 97]),
            (1.0, [252, 253, 191]),
        ],
        _ => &[
            (0.0, [68, 1, 84]),
            (0.25, [59, 82, 139]),
            (0.5, [33, 145, 140]),
            (0.75, [94, 201, 98]),
            (1.0, [253, 231, 37]),
        ],
    };
    let upper = stops
        .iter()
        .position(|(position, _)| *position >= t)
        .unwrap_or(stops.len() - 1);
    let lower = upper.saturating_sub(1);
    let (a_t, a) = stops[lower];
    let (b_t, b) = stops[upper];
    let local = if b_t > a_t {
        (t - a_t) / (b_t - a_t)
    } else {
        0.0
    };
    let blend = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * local).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        blend(a[0], b[0]),
        blend(a[1], b[1]),
        blend(a[2], b[2])
    )
}

fn number(value: f64) -> String {
    let mut text = format!("{value:.4}");
    while text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text == "-0" { "0".to_owned() } else { text }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use axodendron_core::{ValidationProfile, parse_swc};

    use super::*;

    fn morphology() -> Morphology {
        parse_swc(
            "1 1 0 0 0 1 -1\n2 2 10 0 2 1 1\n3 3 10 10 4 1 2\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap()
    }

    #[test]
    fn categorical_palette_follows_compartment_conventions() {
        assert_eq!(type_color(1), "#d62728");
        assert_eq!(type_color(2), "#0072b2");
        assert_eq!(type_color(3), "#009e73");
        assert_eq!(type_color(4), "#009e73");
        assert_eq!(type_color(5), "#e69f00");
        assert_eq!(type_color(6), "#56b4e9");
        assert_eq!(type_color(7), "#cc79a7");
        assert_eq!(type_color(0), "#4d4d4d");
        assert_eq!(type_color(42), "#4d4d4d");
    }

    #[test]
    fn renders_deterministic_svg_and_overlay_coordinates() {
        let morphology = morphology();
        let options = RenderOptions {
            include_nodes: true,
            ..Default::default()
        };
        let document = render_svg(&morphology, &options).unwrap();
        assert!(document.svg.starts_with("<svg"));
        assert!(document.svg.contains("fill=\"#0072b2\""));
        assert!(document.svg.contains("<path"));
        assert_eq!(document.nodes.len(), 3);
        assert_eq!(document.source_node_count, 3);
        assert_eq!(document, render_svg(&morphology, &options).unwrap());
    }

    #[test]
    fn display_simplification_does_not_change_source() {
        let morphology = parse_swc(
            "1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 0 0 1 2\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap();
        let options = RenderOptions {
            display_tolerance: Some(0.1),
            ..Default::default()
        };
        let document = render_svg(&morphology, &options).unwrap();
        assert_eq!(morphology.len(), 3);
        assert_eq!(document.rendered_node_count, 2);

        let protected = render_svg(
            &morphology,
            &RenderOptions {
                display_tolerance: Some(0.1),
                overlay_node_ids: vec![2, 2],
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(protected.rendered_node_count, 3);
        assert_eq!(protected.nodes.len(), 1);
        assert_eq!(protected.nodes[0].node_id, 2);
        assert_eq!(protected.report.overlay_node_count, 1);

        let lenient = render_svg(
            &morphology,
            &RenderOptions {
                overlay_node_ids: vec![2, 999],
                strict_overlay_ids: false,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(lenient.nodes.len(), 1);
        assert_eq!(lenient.nodes[0].node_id, 2);
    }

    #[test]
    fn validates_every_render_input() {
        let morphology = morphology();
        let invalid_canvases = [
            RenderOptions {
                width: 0.0,
                ..Default::default()
            },
            RenderOptions {
                height: f64::NAN,
                ..Default::default()
            },
            RenderOptions {
                padding: 500.0,
                ..Default::default()
            },
            RenderOptions {
                stroke_width: -1.0,
                ..Default::default()
            },
            RenderOptions {
                soma_scale: 0.0,
                ..Default::default()
            },
            RenderOptions {
                minimum_radius: 20.0,
                maximum_radius: 10.0,
                ..Default::default()
            },
            RenderOptions {
                maximum_soma_radius: 0.5,
                ..Default::default()
            },
            RenderOptions {
                radius_scale: 0.0,
                ..Default::default()
            },
            RenderOptions {
                radius_exponent: 1.1,
                ..Default::default()
            },
            RenderOptions {
                outline_width: -1.0,
                ..Default::default()
            },
        ];
        for options in invalid_canvases {
            assert_eq!(
                render_svg(&morphology, &options),
                Err(RenderError::InvalidCanvas)
            );
        }
        let invalid_view = RenderOptions {
            view: View::Orthographic {
                direction: Vec3::new(0.0, 0.0, 1.0),
                up: Vec3::new(0.0, 0.0, 2.0),
            },
            ..Default::default()
        };
        assert!(matches!(
            render_svg(&morphology, &invalid_view),
            Err(RenderError::Projection(_))
        ));
    }

    #[test]
    fn scalar_rendering_checks_shape_and_colormap() {
        let morphology = morphology();
        let mismatch = RenderOptions {
            color: ColorMode::Scalar(ScalarField {
                node_ids: vec![1, 2],
                values: vec![0.0],
                minimum: None,
                maximum: None,
                colormap: "viridis".to_owned(),
                fingerprint: None,
            }),
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology, &mismatch),
            Err(RenderError::ScalarLengthMismatch)
        );

        let unknown = RenderOptions {
            color: ColorMode::Scalar(ScalarField {
                node_ids: vec![1],
                values: vec![0.0],
                minimum: None,
                maximum: None,
                colormap: "rainbow".to_owned(),
                fingerprint: None,
            }),
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology, &unknown),
            Err(RenderError::UnknownColormap("rainbow".to_owned()))
        );

        let duplicate = RenderOptions {
            color: ColorMode::Scalar(ScalarField {
                node_ids: vec![1, 1],
                values: vec![0.0, 1.0],
                minimum: None,
                maximum: None,
                colormap: "viridis".to_owned(),
                fingerprint: None,
            }),
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology, &duplicate),
            Err(RenderError::DuplicateScalarNode(1))
        );

        let reversed_range = RenderOptions {
            color: ColorMode::Scalar(ScalarField {
                node_ids: vec![1],
                values: vec![0.0],
                minimum: Some(2.0),
                maximum: Some(1.0),
                colormap: "viridis".to_owned(),
                fingerprint: None,
            }),
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology, &reversed_range),
            Err(RenderError::InvalidScalarRange)
        );

        let wrong_cell = RenderOptions {
            color: ColorMode::Scalar(ScalarField {
                node_ids: vec![1],
                values: vec![0.0],
                minimum: None,
                maximum: None,
                colormap: "viridis".to_owned(),
                fingerprint: Some("fnv1a64:0000000000000000".to_owned()),
            }),
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology, &wrong_cell),
            Err(RenderError::ScalarFingerprintMismatch)
        );

        let unknown_node = RenderOptions {
            color: ColorMode::Scalar(ScalarField {
                node_ids: vec![999],
                values: vec![0.0],
                minimum: None,
                maximum: None,
                colormap: "viridis".to_owned(),
                fingerprint: Some(morphology.fingerprint().to_owned()),
            }),
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology, &unknown_node),
            Err(RenderError::UnknownScalarNode(999))
        );
    }

    #[test]
    fn scalar_rendering_maps_endpoints_and_missing_values_deterministically() {
        let options = RenderOptions {
            color: ColorMode::Scalar(ScalarField {
                node_ids: vec![2, 3],
                values: vec![0.0, 1.0],
                minimum: Some(0.0),
                maximum: Some(1.0),
                colormap: "viridis".to_owned(),
                fingerprint: None,
            }),
            include_nodes: true,
            ..Default::default()
        };
        let document = render_svg(&morphology(), &options).unwrap();
        assert!(document.svg.contains("fill=\"#440154\""));
        assert!(document.svg.contains("fill=\"#fde725\""));
        assert!(document.svg.contains("fill=\"#9ca3af\" data-node=\"1\""));
    }

    #[test]
    fn user_svg_attributes_are_xml_escaped() {
        let options = RenderOptions {
            background: Some("white\"/><script>alert(1)</script>".to_owned()),
            color: ColorMode::Uniform {
                color: "red\" onload=\"bad".to_owned(),
            },
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology(), &options),
            Err(RenderError::InvalidStyle)
        );
        assert_eq!(escape_xml("<&\"'>"), "&lt;&amp;&quot;&apos;&gt;".to_owned());

        let invalid = RenderOptions {
            color: ColorMode::Uniform {
                color: "x".repeat(MAX_STYLE_BYTES + 1),
            },
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology(), &invalid),
            Err(RenderError::InvalidStyle)
        );

        let external = RenderOptions {
            background: Some("url ( https://example.invalid/x )".to_owned()),
            ..Default::default()
        };
        assert_eq!(
            render_svg(&morphology(), &external),
            Err(RenderError::InvalidStyle)
        );
    }

    #[test]
    fn named_views_have_expected_depth_coordinates() {
        let morphology = morphology();
        let xy = render_svg(
            &morphology,
            &RenderOptions {
                include_nodes: true,
                ..Default::default()
            },
        )
        .unwrap();
        let xz = render_svg(
            &morphology,
            &RenderOptions {
                view: View::Xz,
                include_nodes: true,
                ..Default::default()
            },
        )
        .unwrap();
        let yz = render_svg(
            &morphology,
            &RenderOptions {
                view: View::Yz,
                include_nodes: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(
            xy.nodes.iter().map(|node| node.depth).collect::<Vec<_>>(),
            vec![0.0, 2.0, 4.0]
        );
        assert_eq!(
            xz.nodes.iter().map(|node| node.depth).collect::<Vec<_>>(),
            vec![0.0, 0.0, -10.0]
        );
        assert_eq!(
            yz.nodes.iter().map(|node| node.depth).collect::<Vec<_>>(),
            vec![0.0, 10.0, 10.0]
        );
    }

    #[test]
    fn degenerate_projection_extent_never_emits_nan_or_infinity() {
        let morphology = parse_swc(
            "1 1 2 2 2 1 -1\n2 3 2 2 2 1 1\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap();
        let document = render_svg(&morphology, &RenderOptions::default()).unwrap();
        assert!(!document.svg.contains("NaN"));
        assert!(!document.svg.contains("inf"));
        assert!(document.pixels_per_unit.is_finite());
    }

    #[test]
    fn three_point_soma_renders_as_one_body_without_auxiliary_segments() {
        let morphology = parse_swc(
            "1 1 0 0 0 5 -1\n2 1 0 -5 0 5 1\n3 1 0 5 0 5 1\n4 3 10 0 0 1 1\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap();
        let document = render_svg(&morphology, &RenderOptions::default()).unwrap();
        assert_eq!(document.svg.matches("fill=\"#d62728\"").count(), 1);
        assert!(!document.svg.contains("data-node=\"2\""));
        assert!(!document.svg.contains("data-node=\"3\""));
        assert!(document.svg.contains("data-node=\"1\""));
    }

    #[test]
    fn multipoint_soma_default_is_one_fitted_display_body() {
        let morphology = parse_swc(
            "1 1 0 0 0 6 -1\n2 1 0 20 0 4 1\n3 1 0 40 0 5 2\n4 3 80 0 0 1 3\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap();
        assert_eq!(morphology.soma_class(), SomaClass::MultiPointChain);
        let document = render_svg(&morphology, &RenderOptions::default()).unwrap();
        assert_eq!(document.svg.matches("fill=\"#d62728\"").count(), 1);
        assert!(!document.svg.contains("data-node=\"2\""));
        assert!(!document.svg.contains("data-node=\"3\""));

        let raw = render_svg(
            &morphology,
            &RenderOptions {
                soma_mode: SomaMode::RawPoints,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(raw.svg.matches("fill=\"#d62728\"").count() >= 3);
        assert!(raw.svg.contains("data-node=\"2\""));
        assert!(raw.svg.contains("data-node=\"3\""));
    }

    #[test]
    fn large_soma_uses_its_own_cap_and_is_fitted_inside_the_canvas() {
        let morphology = parse_swc(
            "1 1 0 0 0 50 -1\n2 1 0 -50 0 50 1\n3 1 0 50 0 50 1\n4 3 100 0 0 1 1\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap();
        let options = RenderOptions {
            width: 400.0,
            height: 240.0,
            padding: 20.0,
            maximum_radius: 8.0,
            maximum_soma_radius: 72.0,
            overlay_node_ids: vec![1],
            ..Default::default()
        };
        let document = render_svg(&morphology, &options).unwrap();
        let center = document
            .nodes
            .iter()
            .find(|node| node.node_id == 1)
            .unwrap();
        let radius = soma_radius(&morphology, 0, document.pixels_per_unit, &options);

        assert!(radius > options.maximum_radius);
        assert!(center.x - radius >= options.padding - 1e-6);
        assert!(center.y - radius >= options.padding - 1e-6);
        assert!(center.x + radius <= options.width - options.padding + 1e-6);
        assert!(center.y + radius <= options.height - options.padding + 1e-6);
    }

    #[test]
    fn skeleton_mode_and_optional_outline_remain_available() {
        let document = render_svg(
            &morphology(),
            &RenderOptions {
                geometry: GeometryMode::Skeleton,
                outline_color: Some("#ffffff".to_owned()),
                outline_width: 1.25,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(document.svg.contains("<line"));
        assert!(!document.svg.contains("<path"));
        assert!(document.svg.contains("stroke=\"#ffffff\""));
        assert!(document.svg.contains("stroke-width=\"4.5\""));
    }

    #[test]
    fn tapered_outline_is_applied_to_segment_shapes_and_round_caps() {
        let document = render_svg(
            &morphology(),
            &RenderOptions {
                outline_color: Some("#ffffff".to_owned()),
                outline_width: 1.25,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(document.svg.contains("<path"));
        assert!(document.svg.contains("<circle"));
        assert!(document.svg.contains("stroke=\"#ffffff\""));
        assert!(document.svg.contains("stroke-width=\"2.5\""));
    }
}
