use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::geometry::{Projection, Vec3};
use crate::model::{Morphology, NodeIx, SomaClass};

/// Version of the scientific definitions serialized in analysis results.
pub const DEFINITION_VERSION: &str = "axodendron-morphometrics-1";

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnalysisDomain {
    /// Analyze every node and edge exactly as encoded in the SWC file.
    Raw,
    /// Analyze non-soma arbors. Soma nodes and every soma-incident edge are excluded.
    #[default]
    Neurites,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SectionBoundaryPolicy {
    TopologyOnly,
    #[default]
    TopologyAndType,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisOptions {
    #[serde(default)]
    pub domain: AnalysisDomain,
    #[serde(default)]
    pub section_boundaries: SectionBoundaryPolicy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub min: Vec3,
    pub max: Vec3,
}

impl BBox {
    pub fn spans(self) -> Vec3 {
        self.max - self.min
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypeCount {
    pub kind: i32,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TypeMetrics {
    pub kind: i32,
    pub node_count: u32,
    pub cable_length: f64,
    pub surface_area: Option<f64>,
    pub volume: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArborMetrics {
    pub root_id: i64,
    pub kind: i32,
    pub node_count: u32,
    pub branch_point_count: u32,
    pub terminal_count: u32,
    pub cable_length: f64,
    pub max_path_length: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RadiusMetrics {
    pub neurite_surface_area: Option<f64>,
    pub neurite_volume: Option<f64>,
    pub invalid_radius_node_ids: Vec<i64>,
    pub segment_model: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SomaMetrics {
    pub model: SomaClass,
    pub equivalent_sphere_surface_area: Option<f64>,
    pub equivalent_sphere_volume: Option<f64>,
    pub encoded_cylinder_lateral_area: Option<f64>,
    pub encoded_cylinder_volume: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Summary {
    pub domain: AnalysisDomain,
    pub raw_node_count: u32,
    pub node_count: u32,
    pub edge_count: u32,
    pub root_count: u32,
    pub component_count: u32,
    pub branch_point_count: u32,
    pub terminal_count: u32,
    pub section_count: u32,
    pub total_cable_length: f64,
    pub max_root_path_length: f64,
    pub max_radial_distance: f64,
    pub bbox: Option<BBox>,
    pub type_counts: Vec<TypeCount>,
    pub type_metrics: Vec<TypeMetrics>,
    pub arbor_metrics: Vec<ArborMetrics>,
    pub radius_metrics: RadiusMetrics,
    pub soma_metrics: SomaMetrics,
    pub soma_class: SomaClass,
    pub units: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Topology {
    pub domain: AnalysisDomain,
    pub node_ids: Vec<i64>,
    pub parent_ids: Vec<Option<i64>>,
    pub root_ids: Vec<i64>,
    pub terminal_ids: Vec<i64>,
    pub branch_point_ids: Vec<i64>,
    pub component_ids: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeField {
    pub name: String,
    pub node_ids: Vec<i64>,
    pub values: Vec<f64>,
    pub units: String,
    pub fingerprint: String,
    pub domain: AnalysisDomain,
    pub definition_version: String,
}

impl NodeField {
    pub fn validate_for(&self, morphology: &Morphology) -> bool {
        self.fingerprint == morphology.fingerprint()
            && self.node_ids.len() == self.values.len()
            && self.values.iter().all(|value| value.is_finite())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub proximal_id: i64,
    pub distal_id: i64,
    pub node_ids: Vec<i64>,
    pub kind: i32,
    pub length: f64,
    pub endpoint_distance: f64,
    pub tortuosity: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionDecomposition {
    pub domain: AnalysisDomain,
    pub boundary_policy: SectionBoundaryPolicy,
    pub sections: Vec<Section>,
    pub units: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TortuositySummary {
    pub valid_sections: u32,
    pub excluded_zero_span: u32,
    pub arithmetic_mean: Option<f64>,
    pub length_weighted_mean: Option<f64>,
    pub median: Option<f64>,
    pub interquartile_range: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShollBin {
    pub radius: f64,
    pub intersections: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShollDimension {
    ThreeDimensional,
    TwoDimensional,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShollResult {
    pub center: Vec3,
    pub bins: Vec<ShollBin>,
    pub dimension: ShollDimension,
    pub domain: AnalysisDomain,
    pub endpoint_rule: String,
    pub units: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnalysisBundle {
    pub schema_version: u16,
    pub definition_version: String,
    pub fingerprint: String,
    pub domain: AnalysisDomain,
    pub summary: Summary,
    pub topology: Topology,
    pub sections: SectionDecomposition,
    pub tortuosity: TortuositySummary,
    pub root_path_length: NodeField,
    pub radial_distance: NodeField,
    pub branch_order: NodeField,
    pub strahler_order: NodeField,
}

struct AnalysisView<'a> {
    morphology: &'a Morphology,
    domain: AnalysisDomain,
}

impl<'a> AnalysisView<'a> {
    fn new(morphology: &'a Morphology, domain: AnalysisDomain) -> Self {
        Self { morphology, domain }
    }

    fn includes(&self, node: NodeIx) -> bool {
        self.domain == AnalysisDomain::Raw || self.morphology.kind(node) != 1
    }

    fn nodes(&self) -> impl Iterator<Item = NodeIx> + '_ {
        (0..self.morphology.len() as u32)
            .map(NodeIx)
            .filter(|node| self.includes(*node))
    }

    fn parent(&self, node: NodeIx) -> Option<NodeIx> {
        self.morphology
            .parent(node)
            .filter(|parent| self.includes(*parent))
    }

    fn children(&self, node: NodeIx) -> impl DoubleEndedIterator<Item = NodeIx> + '_ {
        self.morphology
            .children(node)
            .filter(|child| self.includes(*child))
    }

    fn child_count(&self, node: NodeIx) -> usize {
        self.children(node).count()
    }

    fn roots(&self) -> Vec<NodeIx> {
        self.nodes()
            .filter(|node| self.parent(*node).is_none())
            .collect()
    }

    fn edge_length(&self, child: NodeIx) -> Option<f64> {
        self.parent(child).map(|parent| {
            self.morphology
                .position(parent)
                .distance(self.morphology.position(child))
        })
    }
}

impl Morphology {
    pub fn bounding_box(&self) -> BBox {
        bbox_for_nodes(self, (0..self.len() as u32).map(NodeIx)).expect("morphology is non-empty")
    }

    pub fn soma_center(&self) -> Vec3 {
        if self.soma_class() == SomaClass::ThreePoint {
            if let Some(center) = central_soma_node(self) {
                return self.position(center);
            }
        }
        let soma: Vec<NodeIx> = (0..self.len() as u32)
            .map(NodeIx)
            .filter(|node| self.kind(*node) == 1)
            .collect();
        if !soma.is_empty() {
            let divisor = soma.len() as f64;
            return Vec3::new(
                compensated_sum(soma.iter().map(|node| self.position(*node).x)) / divisor,
                compensated_sum(soma.iter().map(|node| self.position(*node).y)) / divisor,
                compensated_sum(soma.iter().map(|node| self.position(*node).z)) / divisor,
            );
        }
        self.roots()
            .next()
            .map(|root| self.position(root))
            .unwrap_or_default()
    }

    pub fn analyze(&self) -> AnalysisBundle {
        self.analyze_with_options(AnalysisOptions::default())
    }

    pub fn analyze_raw(&self) -> AnalysisBundle {
        self.analyze_with_options(AnalysisOptions {
            domain: AnalysisDomain::Raw,
            ..AnalysisOptions::default()
        })
    }

    pub fn analyze_with_options(&self, options: AnalysisOptions) -> AnalysisBundle {
        let view = AnalysisView::new(self, options.domain);
        let paths = root_path_lengths(self, &view);
        let radial = radial_distances(self, &view);
        let sections = sections(self, &view, options.section_boundaries);
        let topology = topology(self, &view);
        let summary = summary(self, &view, &sections, &paths, &radial, &topology);
        let tortuosity = tortuosity_summary(&sections);
        AnalysisBundle {
            schema_version: 1,
            definition_version: DEFINITION_VERSION.to_owned(),
            fingerprint: self.fingerprint().to_owned(),
            domain: options.domain,
            summary,
            topology,
            sections,
            tortuosity,
            root_path_length: paths,
            radial_distance: radial,
            branch_order: branch_orders(self, &view),
            strahler_order: strahler_orders(self, &view),
        }
    }

    pub fn root_path_lengths(&self) -> NodeField {
        let view = AnalysisView::new(self, AnalysisDomain::Neurites);
        root_path_lengths(self, &view)
    }

    pub fn radial_distances(&self) -> NodeField {
        let view = AnalysisView::new(self, AnalysisDomain::Neurites);
        radial_distances(self, &view)
    }

    pub fn strahler_orders(&self) -> NodeField {
        let view = AnalysisView::new(self, AnalysisDomain::Neurites);
        strahler_orders(self, &view)
    }

    pub fn branch_orders(&self) -> NodeField {
        let view = AnalysisView::new(self, AnalysisDomain::Neurites);
        branch_orders(self, &view)
    }

    pub fn sections(&self) -> SectionDecomposition {
        let view = AnalysisView::new(self, AnalysisDomain::Neurites);
        sections(self, &view, SectionBoundaryPolicy::TopologyAndType)
    }

    pub fn tortuosity_summary(&self, sections: &SectionDecomposition) -> TortuositySummary {
        tortuosity_summary(sections)
    }

    pub fn topology(&self) -> Topology {
        topology(self, &AnalysisView::new(self, AnalysisDomain::Neurites))
    }

    pub fn sholl_3d(&self, center: Vec3, radii: &[f64]) -> ShollResult {
        self.sholl_3d_in_domain(center, radii, AnalysisDomain::Neurites)
    }

    pub fn sholl_3d_in_domain(
        &self,
        center: Vec3,
        radii: &[f64],
        domain: AnalysisDomain,
    ) -> ShollResult {
        let view = AnalysisView::new(self, domain);
        sholl_result(self, &view, center, radii, None)
    }

    pub fn sholl_2d(
        &self,
        center: Vec3,
        radii: &[f64],
        projection: Projection,
        domain: AnalysisDomain,
    ) -> ShollResult {
        let view = AnalysisView::new(self, domain);
        sholl_result(self, &view, center, radii, Some(projection))
    }
}

fn root_path_lengths(morphology: &Morphology, view: &AnalysisView<'_>) -> NodeField {
    let mut sums = vec![0.0; morphology.len()];
    let mut corrections = vec![0.0; morphology.len()];
    let mut stack: Vec<NodeIx> = view.roots().into_iter().rev().collect();
    while let Some(parent) = stack.pop() {
        for child in view.children(parent).rev() {
            let parent_ix = parent.0 as usize;
            let child_ix = child.0 as usize;
            let edge = morphology
                .position(parent)
                .distance(morphology.position(child));
            let next = sums[parent_ix] + edge;
            let correction = if sums[parent_ix].abs() >= edge.abs() {
                corrections[parent_ix] + (sums[parent_ix] - next) + edge
            } else {
                corrections[parent_ix] + (edge - next) + sums[parent_ix]
            };
            sums[child_ix] = next;
            corrections[child_ix] = correction;
            stack.push(child);
        }
    }
    let nodes: Vec<NodeIx> = view.nodes().collect();
    NodeField {
        name: "arbor-root-path-length".to_owned(),
        node_ids: nodes.iter().map(|node| morphology.id(*node).0).collect(),
        values: nodes
            .iter()
            .map(|node| sums[node.0 as usize] + corrections[node.0 as usize])
            .collect(),
        units: morphology.units().to_owned(),
        fingerprint: morphology.fingerprint().to_owned(),
        domain: view.domain,
        definition_version: DEFINITION_VERSION.to_owned(),
    }
}

fn radial_distances(morphology: &Morphology, view: &AnalysisView<'_>) -> NodeField {
    let mut distance = vec![0.0_f64; morphology.len()];
    let mut stack: Vec<(NodeIx, Vec3)> = view
        .roots()
        .into_iter()
        .rev()
        .map(|root| (root, morphology.position(root)))
        .collect();
    while let Some((node, root_position)) = stack.pop() {
        distance[node.0 as usize] = morphology.position(node).distance(root_position);
        for child in view.children(node).rev() {
            stack.push((child, root_position));
        }
    }
    let nodes: Vec<NodeIx> = view.nodes().collect();
    NodeField {
        name: "arbor-root-euclidean-distance".to_owned(),
        node_ids: nodes.iter().map(|node| morphology.id(*node).0).collect(),
        values: nodes.iter().map(|node| distance[node.0 as usize]).collect(),
        units: morphology.units().to_owned(),
        fingerprint: morphology.fingerprint().to_owned(),
        domain: view.domain,
        definition_version: DEFINITION_VERSION.to_owned(),
    }
}

fn strahler_orders(morphology: &Morphology, view: &AnalysisView<'_>) -> NodeField {
    let mut order = vec![0_u32; morphology.len()];
    let mut stack: Vec<(NodeIx, bool)> = view
        .roots()
        .into_iter()
        .rev()
        .map(|root| (root, false))
        .collect();
    while let Some((node, visited)) = stack.pop() {
        if !visited {
            stack.push((node, true));
            for child in view.children(node).rev() {
                stack.push((child, false));
            }
            continue;
        }
        let mut maximum = 0_u32;
        let mut maximum_count = 0_u32;
        for child in view.children(node) {
            let child_order = order[child.0 as usize];
            if child_order > maximum {
                maximum = child_order;
                maximum_count = 1;
            } else if child_order == maximum {
                maximum_count += 1;
            }
        }
        order[node.0 as usize] = if maximum == 0 {
            1
        } else {
            maximum + u32::from(maximum_count >= 2)
        };
    }
    let nodes: Vec<NodeIx> = view.nodes().collect();
    NodeField {
        name: "strahler-order".to_owned(),
        node_ids: nodes.iter().map(|node| morphology.id(*node).0).collect(),
        values: nodes
            .iter()
            .map(|node| f64::from(order[node.0 as usize]))
            .collect(),
        units: "1".to_owned(),
        fingerprint: morphology.fingerprint().to_owned(),
        domain: view.domain,
        definition_version: DEFINITION_VERSION.to_owned(),
    }
}

fn branch_orders(morphology: &Morphology, view: &AnalysisView<'_>) -> NodeField {
    let mut order = vec![0_u32; morphology.len()];
    let mut stack: Vec<NodeIx> = view.roots().into_iter().rev().collect();
    for root in view.roots() {
        order[root.0 as usize] = 1;
    }
    while let Some(parent) = stack.pop() {
        let increment = u32::from(view.child_count(parent) > 1);
        for child in view.children(parent).rev() {
            order[child.0 as usize] = order[parent.0 as usize] + increment;
            stack.push(child);
        }
    }
    let nodes: Vec<NodeIx> = view.nodes().collect();
    NodeField {
        name: "centrifugal-branch-order".to_owned(),
        node_ids: nodes.iter().map(|node| morphology.id(*node).0).collect(),
        values: nodes
            .iter()
            .map(|node| f64::from(order[node.0 as usize]))
            .collect(),
        units: "1".to_owned(),
        fingerprint: morphology.fingerprint().to_owned(),
        domain: view.domain,
        definition_version: DEFINITION_VERSION.to_owned(),
    }
}

fn sections(
    morphology: &Morphology,
    view: &AnalysisView<'_>,
    policy: SectionBoundaryPolicy,
) -> SectionDecomposition {
    let mut output = Vec::new();
    for start in view.nodes() {
        let begins_section = match view.parent(start) {
            None => true,
            Some(parent) => {
                view.child_count(start) != 1
                    || (policy == SectionBoundaryPolicy::TopologyAndType
                        && morphology.kind(parent) != morphology.kind(start))
            }
        };
        if !begins_section {
            continue;
        }
        for first_child in view.children(start) {
            let mut nodes = vec![start, first_child];
            let mut cursor = first_child;
            let first_child_is_type_boundary = policy == SectionBoundaryPolicy::TopologyAndType
                && morphology.kind(start) != morphology.kind(first_child);
            while !first_child_is_type_boundary && view.child_count(cursor) == 1 {
                let next = view.children(cursor).next().expect("one child was counted");
                nodes.push(next);
                let type_boundary = policy == SectionBoundaryPolicy::TopologyAndType
                    && morphology.kind(cursor) != morphology.kind(next);
                cursor = next;
                if type_boundary {
                    break;
                }
            }
            let length = compensated_sum(nodes.windows(2).map(|pair| {
                morphology
                    .position(pair[0])
                    .distance(morphology.position(pair[1]))
            }));
            let endpoint_distance = morphology
                .position(start)
                .distance(morphology.position(cursor));
            output.push(Section {
                proximal_id: morphology.id(start).0,
                distal_id: morphology.id(cursor).0,
                node_ids: nodes.iter().map(|node| morphology.id(*node).0).collect(),
                kind: morphology.kind(start),
                length,
                endpoint_distance,
                tortuosity: (endpoint_distance > 0.0).then_some(length / endpoint_distance),
            });
        }
    }
    SectionDecomposition {
        domain: view.domain,
        boundary_policy: policy,
        sections: output,
        units: morphology.units().to_owned(),
    }
}

fn topology(morphology: &Morphology, view: &AnalysisView<'_>) -> Topology {
    let nodes: Vec<NodeIx> = view.nodes().collect();
    let roots = view.roots();
    let mut component = vec![u32::MAX; morphology.len()];
    for (component_id, root) in roots.iter().copied().enumerate() {
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            component[node.0 as usize] = component_id as u32;
            for child in view.children(node).rev() {
                stack.push(child);
            }
        }
    }
    Topology {
        domain: view.domain,
        node_ids: nodes.iter().map(|node| morphology.id(*node).0).collect(),
        parent_ids: nodes
            .iter()
            .map(|node| view.parent(*node).map(|parent| morphology.id(parent).0))
            .collect(),
        root_ids: roots.iter().map(|root| morphology.id(*root).0).collect(),
        terminal_ids: nodes
            .iter()
            .filter(|node| view.child_count(**node) == 0)
            .map(|node| morphology.id(*node).0)
            .collect(),
        branch_point_ids: nodes
            .iter()
            .filter(|node| view.child_count(**node) > 1)
            .map(|node| morphology.id(*node).0)
            .collect(),
        component_ids: nodes
            .iter()
            .map(|node| component[node.0 as usize])
            .collect(),
    }
}

fn summary(
    morphology: &Morphology,
    view: &AnalysisView<'_>,
    sections: &SectionDecomposition,
    paths: &NodeField,
    radial: &NodeField,
    topology: &Topology,
) -> Summary {
    let nodes: Vec<NodeIx> = view.nodes().collect();
    let mut type_counts = BTreeMap::<i32, u32>::new();
    let mut type_accumulators = BTreeMap::<i32, MetricAccumulator>::new();
    for node in &nodes {
        *type_counts.entry(morphology.kind(*node)).or_default() += 1;
        type_accumulators
            .entry(morphology.kind(*node))
            .or_default()
            .node_count += 1;
    }
    let mut invalid_radius_nodes = Vec::new();
    let mut total_area = 0.0;
    let mut total_volume = 0.0;
    let mut radius_metrics_valid = true;
    for child in &nodes {
        let Some(parent) = view.parent(*child) else {
            continue;
        };
        let length = morphology
            .position(parent)
            .distance(morphology.position(*child));
        let accumulator = type_accumulators
            .entry(morphology.kind(*child))
            .or_default();
        accumulator.cable_length += length;
        let r0 = morphology.radius(parent);
        let r1 = morphology.radius(*child);
        if r0 <= 0.0 || r1 <= 0.0 {
            radius_metrics_valid = false;
            if r0 <= 0.0 {
                invalid_radius_nodes.push(morphology.id(parent).0);
            }
            if r1 <= 0.0 {
                invalid_radius_nodes.push(morphology.id(*child).0);
            }
            accumulator.radius_valid = false;
            continue;
        }
        let area = std::f64::consts::PI * (r0 + r1) * length.hypot(r0 - r1);
        let volume = std::f64::consts::PI * length * (r0 * r0 + r0 * r1 + r1 * r1) / 3.0;
        if !area.is_finite() || !volume.is_finite() {
            radius_metrics_valid = false;
            accumulator.radius_valid = false;
            continue;
        }
        total_area += area;
        total_volume += volume;
        accumulator.surface_area += area;
        accumulator.volume += volume;
    }
    invalid_radius_nodes.sort_unstable();
    invalid_radius_nodes.dedup();

    let type_metrics = type_accumulators
        .into_iter()
        .map(|(kind, value)| TypeMetrics {
            kind,
            node_count: value.node_count,
            cable_length: value.cable_length,
            surface_area: value.radius_valid.then_some(value.surface_area),
            volume: value.radius_valid.then_some(value.volume),
        })
        .collect();
    let arbor_metrics = arbor_metrics(morphology, view);
    Summary {
        domain: view.domain,
        raw_node_count: morphology.len() as u32,
        node_count: nodes.len() as u32,
        edge_count: nodes
            .iter()
            .filter(|node| view.parent(**node).is_some())
            .count() as u32,
        root_count: topology.root_ids.len() as u32,
        component_count: topology.root_ids.len() as u32,
        branch_point_count: topology.branch_point_ids.len() as u32,
        terminal_count: topology.terminal_ids.len() as u32,
        section_count: sections.sections.len() as u32,
        total_cable_length: compensated_sum(
            nodes.iter().filter_map(|node| view.edge_length(*node)),
        ),
        max_root_path_length: paths.values.iter().copied().fold(0.0, f64::max),
        max_radial_distance: radial.values.iter().copied().fold(0.0, f64::max),
        bbox: bbox_for_nodes(morphology, nodes.iter().copied()),
        type_counts: type_counts
            .into_iter()
            .map(|(kind, count)| TypeCount { kind, count })
            .collect(),
        type_metrics,
        arbor_metrics,
        radius_metrics: RadiusMetrics {
            neurite_surface_area: radius_metrics_valid.then_some(total_area),
            neurite_volume: radius_metrics_valid.then_some(total_volume),
            invalid_radius_node_ids: invalid_radius_nodes,
            segment_model: "uncapped-circular-frustum".to_owned(),
        },
        soma_metrics: soma_metrics(morphology),
        soma_class: morphology.soma_class(),
        units: morphology.units().to_owned(),
    }
}

struct MetricAccumulator {
    node_count: u32,
    cable_length: f64,
    surface_area: f64,
    volume: f64,
    radius_valid: bool,
}

impl Default for MetricAccumulator {
    fn default() -> Self {
        Self {
            node_count: 0,
            cable_length: 0.0,
            surface_area: 0.0,
            volume: 0.0,
            radius_valid: true,
        }
    }
}

fn arbor_metrics(morphology: &Morphology, view: &AnalysisView<'_>) -> Vec<ArborMetrics> {
    view.roots()
        .into_iter()
        .map(|root| {
            let mut stack = vec![(root, 0.0)];
            let mut node_count = 0_u32;
            let mut branch_count = 0_u32;
            let mut terminal_count = 0_u32;
            let mut lengths = Vec::new();
            let mut max_path = 0.0_f64;
            while let Some((node, path)) = stack.pop() {
                node_count += 1;
                let child_count = view.child_count(node);
                branch_count += u32::from(child_count > 1);
                terminal_count += u32::from(child_count == 0);
                max_path = max_path.max(path);
                for child in view.children(node).rev() {
                    let length = morphology
                        .position(node)
                        .distance(morphology.position(child));
                    lengths.push(length);
                    stack.push((child, path + length));
                }
            }
            ArborMetrics {
                root_id: morphology.id(root).0,
                kind: morphology.kind(root),
                node_count,
                branch_point_count: branch_count,
                terminal_count,
                cable_length: compensated_sum(lengths),
                max_path_length: max_path,
            }
        })
        .collect()
}

fn soma_metrics(morphology: &Morphology) -> SomaMetrics {
    let class = morphology.soma_class();
    let radius = match class {
        SomaClass::SinglePoint => (0..morphology.len() as u32)
            .map(NodeIx)
            .find(|node| morphology.kind(*node) == 1)
            .map(|node| morphology.radius(node)),
        SomaClass::ThreePoint => central_soma_node(morphology).map(|node| morphology.radius(node)),
        _ => None,
    }
    .filter(|radius| *radius > 0.0);
    let sphere_area = radius.map(|r| 4.0 * std::f64::consts::PI * r * r);
    let sphere_volume = radius.map(|r| 4.0 * std::f64::consts::PI * r * r * r / 3.0);
    let cylinder_area = (class == SomaClass::ThreePoint)
        .then(|| radius.map(|r| 4.0 * std::f64::consts::PI * r * r))
        .flatten();
    let cylinder_volume = (class == SomaClass::ThreePoint)
        .then(|| radius.map(|r| 2.0 * std::f64::consts::PI * r * r * r))
        .flatten();
    SomaMetrics {
        model: class,
        equivalent_sphere_surface_area: sphere_area,
        equivalent_sphere_volume: sphere_volume,
        encoded_cylinder_lateral_area: cylinder_area,
        encoded_cylinder_volume: cylinder_volume,
    }
}

fn central_soma_node(morphology: &Morphology) -> Option<NodeIx> {
    (0..morphology.len() as u32).map(NodeIx).find(|node| {
        morphology.kind(*node) == 1
            && morphology
                .parent(*node)
                .is_none_or(|parent| morphology.kind(parent) != 1)
    })
}

fn bbox_for_nodes(
    morphology: &Morphology,
    mut nodes: impl Iterator<Item = NodeIx>,
) -> Option<BBox> {
    let first = nodes.next()?;
    let mut min = morphology.position(first);
    let mut max = min;
    for node in nodes {
        let point = morphology.position(node);
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }
    Some(BBox { min, max })
}

fn tortuosity_summary(sections: &SectionDecomposition) -> TortuositySummary {
    let valid: Vec<(f64, f64)> = sections
        .sections
        .iter()
        .filter_map(|section| section.tortuosity.map(|ratio| (ratio, section.length)))
        .collect();
    let excluded = sections.sections.len() - valid.len();
    if valid.is_empty() {
        return TortuositySummary {
            valid_sections: 0,
            excluded_zero_span: excluded as u32,
            ..TortuositySummary::default()
        };
    }
    let arithmetic = compensated_sum(valid.iter().map(|(ratio, _)| *ratio)) / valid.len() as f64;
    let total_length = compensated_sum(valid.iter().map(|(_, length)| *length));
    let weighted = (total_length > 0.0).then(|| {
        compensated_sum(valid.iter().map(|(ratio, length)| ratio * length)) / total_length
    });
    let mut sorted: Vec<f64> = valid.iter().map(|(ratio, _)| *ratio).collect();
    sorted.sort_by(f64::total_cmp);
    let q1 = quantile(&sorted, 0.25);
    let median = quantile(&sorted, 0.5);
    let q3 = quantile(&sorted, 0.75);
    TortuositySummary {
        valid_sections: valid.len() as u32,
        excluded_zero_span: excluded as u32,
        arithmetic_mean: Some(arithmetic),
        length_weighted_mean: weighted,
        median: Some(median),
        interquartile_range: Some(q3 - q1),
    }
}

fn sholl_result(
    morphology: &Morphology,
    view: &AnalysisView<'_>,
    center: Vec3,
    radii: &[f64],
    projection: Option<Projection>,
) -> ShollResult {
    let projected_center = projection.map(|value| value.project(center).0);
    let bins = radii
        .iter()
        .copied()
        .map(|radius| {
            let intersections = if radius.is_finite() && radius >= 0.0 {
                view.nodes()
                    .filter_map(|child| view.parent(child).map(|parent| (parent, child)))
                    .map(|(parent, child)| match projection {
                        Some(projection) => {
                            let center = projected_center.expect("projection provides a center");
                            let a = projection.project(morphology.position(parent)).0;
                            let b = projection.project(morphology.position(child)).0;
                            segment_sphere_intersections(
                                Vec3::new(a.x - center.x, a.y - center.y, 0.0),
                                Vec3::new(b.x - center.x, b.y - center.y, 0.0),
                                radius,
                            )
                        }
                        None => segment_sphere_intersections(
                            morphology.position(parent) - center,
                            morphology.position(child) - center,
                            radius,
                        ),
                    })
                    .sum()
            } else {
                0
            };
            ShollBin {
                radius,
                intersections,
            }
        })
        .collect();
    ShollResult {
        center,
        bins,
        dimension: if projection.is_some() {
            ShollDimension::TwoDimensional
        } else {
            ShollDimension::ThreeDimensional
        },
        domain: view.domain,
        endpoint_rule: "count roots with edge parameter t in (0, 1]; tangencies count once"
            .to_owned(),
        units: morphology.units().to_owned(),
    }
}

fn compensated_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for value in values {
        let next = sum + value;
        if sum.abs() >= value.abs() {
            correction += (sum - next) + value;
        } else {
            correction += (value - next) + sum;
        }
        sum = next;
    }
    sum + correction
}

fn quantile(sorted: &[f64], probability: f64) -> f64 {
    let position = probability * (sorted.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    if lower == upper {
        sorted[lower]
    } else {
        sorted[lower] * (upper as f64 - position) + sorted[upper] * (position - lower as f64)
    }
}

fn segment_sphere_intersections(parent: Vec3, child: Vec3, radius: f64) -> u32 {
    let scale = [
        parent.x.abs(),
        parent.y.abs(),
        parent.z.abs(),
        child.x.abs(),
        child.y.abs(),
        child.z.abs(),
        radius.abs(),
    ]
    .into_iter()
    .fold(0.0_f64, f64::max);
    if scale == 0.0 {
        return 0;
    }
    let inverse_scale = 1.0 / scale;
    let parent = parent * inverse_scale;
    let child = child * inverse_scale;
    let radius = radius * inverse_scale;
    let delta = child - parent;
    let a = delta.norm_squared();
    if a == 0.0 {
        return 0;
    }
    let b = 2.0 * parent.dot(delta);
    let c = parent.norm_squared() - radius * radius;
    let child_value = child.norm_squared() - radius * radius;
    let endpoint_tolerance = 64.0
        * f64::EPSILON
        * parent
            .norm_squared()
            .max(child.norm_squared())
            .max(radius * radius)
            .max(f64::MIN_POSITIVE);
    let parent_on_sphere = c.abs() <= endpoint_tolerance;
    let child_on_sphere = child_value.abs() <= endpoint_tolerance;
    let interior = |t: f64| {
        let tolerance = 128.0 * f64::EPSILON * (1.0 + t.abs());
        t > tolerance && t < 1.0 - tolerance
    };
    if parent_on_sphere && child_on_sphere {
        return 1;
    }
    if parent_on_sphere {
        return u32::from(interior(-b / a));
    }
    if child_on_sphere {
        return 1 + u32::from(interior(c / a));
    }
    let discriminant = b * b - 4.0 * a * c;
    let tolerance = 64.0 * f64::EPSILON * (b * b).max((4.0 * a * c).abs()).max(f64::MIN_POSITIVE);
    if discriminant < -tolerance {
        return 0;
    }
    if discriminant.abs() <= tolerance {
        return u32::from(interior(-b / (2.0 * a)));
    }
    let square_root = discriminant.sqrt();
    let q = -0.5 * (b + square_root.copysign(b));
    let (t1, t2) = if q == 0.0 {
        (
            (-b - square_root) / (2.0 * a),
            (-b + square_root) / (2.0 * a),
        )
    } else {
        (q / a, c / q)
    };
    u32::from(interior(t1)) + u32::from(interior(t2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ValidationProfile, parse_swc};

    fn morphology(source: &str) -> Morphology {
        parse_swc(source, ValidationProfile::IncfStrict)
            .morphology
            .unwrap()
    }

    #[test]
    fn neurite_domain_excludes_soma_connectors_and_auxiliary_nodes() {
        let cell = morphology(
            "1 1 0 0 0 5 -1\n2 1 0 5 0 5 1\n3 1 0 -5 0 5 1\n4 3 1 0 0 1 1\n5 3 2 0 0 1 4\n",
        );
        assert_eq!(cell.soma_class(), SomaClass::ThreePoint);
        let neurites = cell.analyze();
        assert_eq!(neurites.summary.node_count, 2);
        assert_eq!(neurites.summary.edge_count, 1);
        assert_eq!(neurites.summary.terminal_count, 1);
        assert_eq!(neurites.summary.total_cable_length, 1.0);
        assert_eq!(neurites.topology.root_ids, vec![4]);
        assert_eq!(neurites.radial_distance.values, vec![0.0, 1.0]);
        assert_eq!(neurites.branch_order.values, vec![1.0, 1.0]);

        let raw = cell.analyze_raw();
        assert_eq!(raw.summary.node_count, 5);
        assert_eq!(raw.summary.edge_count, 4);
    }

    #[test]
    fn geometric_three_point_validation_rejects_near_misses() {
        let rounded =
            morphology("1 1 0 0 0 5 -1\n2 1 0 4.98 0 5 1\n3 1 0 -4.98 0 5 1\n4 3 1 0 0 1 1\n");
        assert_eq!(rounded.soma_class(), SomaClass::ThreePoint);

        let cell = morphology("1 1 0 0 0 5 -1\n2 1 0 6 0 5 1\n3 1 0 -4 0 5 1\n4 3 1 0 0 1 1\n");
        assert_eq!(cell.soma_class(), SomaClass::Ambiguous);
    }

    #[test]
    fn type_changes_split_sections_without_losing_edges() {
        let cell = morphology("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 2 2 0 0 1 2\n4 2 3 0 0 1 3\n");
        let analysis = cell.analyze();
        assert_eq!(analysis.sections.sections.len(), 2);
        assert_eq!(analysis.summary.total_cable_length, 3.0);
        assert_eq!(
            analysis
                .sections
                .sections
                .iter()
                .map(|section| section.length)
                .sum::<f64>(),
            3.0
        );

        let root_transition = morphology("1 3 0 0 0 1 -1\n2 2 1 0 0 1 1\n3 2 2 0 0 1 2\n");
        let sections = root_transition.analyze().sections.sections;
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].node_ids, vec![1, 2]);
        assert_eq!(sections[0].kind, 3);
        assert_eq!(sections[1].node_ids, vec![2, 3]);
        assert_eq!(sections[1].kind, 2);
        assert_eq!(
            sections.iter().map(|section| section.length).sum::<f64>(),
            2.0
        );
    }

    #[test]
    fn frustum_metrics_and_field_identity_are_explicit() {
        let cell = morphology("1 3 0 0 0 2 -1\n2 3 3 0 0 1 1\n");
        let analysis = cell.analyze();
        let expected_area = 3.0 * std::f64::consts::PI * 10.0_f64.sqrt();
        let expected_volume = 7.0 * std::f64::consts::PI;
        assert!(
            (analysis
                .summary
                .radius_metrics
                .neurite_surface_area
                .unwrap()
                - expected_area)
                .abs()
                < 1e-12
        );
        assert!(
            (analysis.summary.radius_metrics.neurite_volume.unwrap() - expected_volume).abs()
                < 1e-12
        );
        assert!(analysis.root_path_length.validate_for(&cell));
    }

    #[test]
    fn sholl_supports_physical_2d_and_3d_domains() {
        let cell = morphology("1 3 -2 0 4 1 -1\n2 3 2 0 4 1 1\n");
        assert_eq!(
            cell.sholl_3d(Vec3::default(), &[1.0]).bins[0].intersections,
            0
        );
        assert_eq!(
            cell.sholl_2d(
                Vec3::default(),
                &[1.0],
                Projection::xy(),
                AnalysisDomain::Neurites
            )
            .bins[0]
                .intersections,
            2
        );
    }

    #[test]
    fn forest_paths_and_strahler_are_independent() {
        let parsed = parse_swc(
            "10 3 0 0 0 1 -1\n20 3 3 0 0 1 10\n30 3 100 0 0 1 -1\n40 3 104 0 0 1 30\n",
            ValidationProfile::Permissive,
        );
        let analysis = parsed.morphology.unwrap().analyze();
        assert_eq!(analysis.root_path_length.values, vec![0.0, 3.0, 0.0, 4.0]);
        assert_eq!(analysis.topology.component_ids, vec![0, 0, 1, 1]);
        assert_eq!(analysis.summary.root_count, 2);
    }

    #[test]
    fn centrifugal_branch_order_increments_at_multifurcations() {
        let cell = morphology(
            "1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 1 0 1 2\n4 3 2 -1 0 1 2\n5 3 3 1 0 1 3\n",
        );
        assert_eq!(
            cell.analyze().branch_order.values,
            vec![1.0, 1.0, 2.0, 2.0, 2.0]
        );
    }
}
