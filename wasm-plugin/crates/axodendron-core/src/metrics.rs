use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::analysis::{BBox, SectionBoundaryPolicy};
use crate::geometry::{Projection, Vec2, Vec3};
use crate::model::{Morphology, NodeIx};
use crate::principal::{
    FrameOrigin, PrincipalFrameError, PrincipalFrameOptions, PrincipalPlane, PrincipalWeighting,
};
use crate::query::{QueryError, SelectionQuery, SelectionView};

pub const METRIC_RESULT_SCHEMA_VERSION: u16 = 1;
pub const SECTION_DEFINITION_VERSION: u16 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    Number(f64),
    Integer(i64),
    Boolean(bool),
    Text(String),
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiameterSampling {
    #[default]
    FirstPoint,
    SectionMean,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaperQuantity {
    Radius,
    #[default]
    Diameter,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TaperMethod {
    Endpoint,
    #[default]
    LinearFit,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MultifurcationPolicy {
    Exclude,
    #[default]
    Pairwise,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpatialPlane {
    #[default]
    Xy,
    Xz,
    Yz,
    PrincipalXy,
    PrincipalXz,
    PrincipalYz,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricParameters {
    #[serde(default, deserialize_with = "crate::serde_number::option_f64")]
    pub p: Option<f64>,
    #[serde(default)]
    pub diameter_sampling: Option<DiameterSampling>,
    #[serde(default)]
    pub taper_quantity: Option<TaperQuantity>,
    #[serde(default)]
    pub taper_method: Option<TaperMethod>,
    #[serde(default)]
    pub multifurcation: Option<MultifurcationPolicy>,
    #[serde(default)]
    pub weighting: Option<PrincipalWeighting>,
    #[serde(default)]
    pub plane: Option<SpatialPlane>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSpec {
    pub id: String,
    #[serde(default)]
    pub parameters: MetricParameters,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasureOptions {
    pub metrics: Vec<MetricSpec>,
    #[serde(default)]
    pub selection: SelectionQuery,
    #[serde(default)]
    pub section_boundaries: SectionBoundaryPolicy,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricDescriptor {
    pub id: String,
    pub definition_version: u16,
    pub parameters: BTreeMap<String, ParameterValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricSource {
    pub morphology_fingerprint: String,
    pub topology_fingerprint: String,
    pub selection_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricProvenance {
    pub implementation: String,
    pub algorithm: String,
    pub notes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MissingReason {
    NotApplicable,
    InsufficientGeometry,
    NonBinaryBifurcation,
    ZeroLength,
    NonPositiveRadius,
    Degenerate,
    NonFiniteResult,
    Collision,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct SectionRef {
    pub topology_fingerprint: String,
    pub selection_fingerprint: String,
    pub section_definition_version: u16,
    pub boundary_policy: SectionBoundaryPolicy,
    pub proximal_node: i64,
    pub distal_node: i64,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BifurcationKey {
    pub branch_node: i64,
    pub child_sections: Vec<SectionRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum EntityKey {
    Morphology,
    Node { node_id: i64 },
    Section { section: SectionRef },
    Bifurcation { bifurcation: BifurcationKey },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissingValue {
    pub entity: EntityKey,
    pub reason: MissingReason,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "shape", content = "value", rename_all = "kebab-case")]
pub enum MetricValue {
    Scalar(f64),
    Vector3(Vec3),
    Box3(BBox),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MorphologyMetric {
    pub value: Option<MetricValue>,
    pub units: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricNodeField {
    pub node_ids: Vec<i64>,
    pub values: Vec<f64>,
    pub units: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SectionField {
    pub sections: Vec<SectionRef>,
    pub values: Vec<f64>,
    pub units: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BifurcationField {
    pub bifurcations: Vec<BifurcationKey>,
    pub values: Vec<f64>,
    pub units: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum MetricData {
    MorphologyMetric(MorphologyMetric),
    NodeField(MetricNodeField),
    SectionField(SectionField),
    BifurcationField(BifurcationField),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricResult {
    pub schema_version: u16,
    pub metric: MetricDescriptor,
    pub selection: SelectionQuery,
    pub source: MetricSource,
    pub provenance: MetricProvenance,
    pub data: MetricData,
    pub missing: Vec<MissingValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub id: String,
    pub definition_version: u16,
    pub entity: String,
    pub units: String,
    pub summary: String,
    pub parameters: Vec<MetricParameterDefinition>,
    pub reference: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricParameterDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: String,
    pub default: Option<ParameterValue>,
    pub choices: Vec<ParameterValue>,
    pub minimum: Option<f64>,
    pub exclusive_minimum: bool,
    pub applies_when: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetricError {
    Query(QueryError),
    UnknownMetric(String),
    InvalidParameter { metric: String, message: String },
    PrincipalFrame(PrincipalFrameError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldPlacement {
    SectionProximal,
    SectionDistal,
    SectionBroadcast,
    BifurcationBranch,
    BifurcationChildren,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldReducer {
    #[default]
    Error,
    Mean,
    Minimum,
    Maximum,
    Sum,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldToNodesOptions {
    pub placement: FieldPlacement,
    #[serde(default)]
    pub reducer: FieldReducer,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldConversionError {
    FingerprintMismatch,
    UnsupportedField,
    InvalidSectionReference,
    Collision(i64),
    Query(QueryError),
}

impl std::fmt::Display for FieldConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FingerprintMismatch => {
                f.write_str("field morphology fingerprint does not match the morphology")
            }
            Self::UnsupportedField => {
                f.write_str("field placement is incompatible with field kind")
            }
            Self::InvalidSectionReference => {
                f.write_str("field contains a stale or invalid section reference")
            }
            Self::Collision(id) => write!(
                f,
                "multiple field values map to node {id}; choose an explicit reducer"
            ),
            Self::Query(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for FieldConversionError {}

impl From<QueryError> for FieldConversionError {
    fn from(value: QueryError) -> Self {
        Self::Query(value)
    }
}

impl std::fmt::Display for MetricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query(error) => error.fmt(f),
            Self::UnknownMetric(id) => write!(f, "unknown metric {id:?}"),
            Self::InvalidParameter { metric, message } => {
                write!(f, "invalid parameters for {metric:?}: {message}")
            }
            Self::PrincipalFrame(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for MetricError {}

impl From<QueryError> for MetricError {
    fn from(value: QueryError) -> Self {
        Self::Query(value)
    }
}

impl From<PrincipalFrameError> for MetricError {
    fn from(value: PrincipalFrameError) -> Self {
        Self::PrincipalFrame(value)
    }
}

#[derive(Clone)]
struct MetricSection {
    key: SectionRef,
    nodes: Vec<NodeIx>,
}

impl Morphology {
    pub fn measure(&self, options: &MeasureOptions) -> Result<Vec<MetricResult>, MetricError> {
        let view = SelectionView::new(self, &options.selection)?;
        let sections = metric_sections(self, &view, options.section_boundaries);
        options
            .metrics
            .iter()
            .map(|spec| self.measure_one(spec, &view, &sections, options.section_boundaries))
            .collect()
    }

    fn measure_one(
        &self,
        spec: &MetricSpec,
        view: &SelectionView<'_>,
        sections: &[MetricSection],
        section_boundaries: SectionBoundaryPolicy,
    ) -> Result<MetricResult, MetricError> {
        let id = spec.id.as_str();
        if !is_known_metric(id) {
            return Err(MetricError::UnknownMetric(spec.id.clone()));
        }
        validate_metric_parameters(spec)?;
        let (data, missing, mut parameters, algorithm, notes) = match id {
            "local-bifurcation-angle" => bifurcation_angles(
                self,
                view,
                sections,
                false,
                spec.parameters.multifurcation.unwrap_or_default(),
            ),
            "remote-bifurcation-angle" => bifurcation_angles(
                self,
                view,
                sections,
                true,
                spec.parameters.multifurcation.unwrap_or_default(),
            ),
            "sibling-ratio" => sibling_ratios(
                self,
                view,
                sections,
                spec.parameters.diameter_sampling.unwrap_or_default(),
                spec.parameters.multifurcation.unwrap_or_default(),
            ),
            "partition-asymmetry-terminal" => partition_asymmetry(
                self,
                view,
                sections,
                spec.parameters.multifurcation.unwrap_or_default(),
            ),
            "diameter-power-ratio" | "rall-ratio" => {
                let p = spec.parameters.p.unwrap_or(1.5);
                if !p.is_finite() || p <= 0.0 {
                    return Err(MetricError::InvalidParameter {
                        metric: spec.id.clone(),
                        message: "p must be finite and positive".to_owned(),
                    });
                }
                diameter_power_ratios(
                    self,
                    view,
                    sections,
                    p,
                    spec.parameters.diameter_sampling.unwrap_or_default(),
                )
            }
            "taper-rate" => taper_rates(
                self,
                sections,
                spec.parameters.taper_quantity.unwrap_or_default(),
                spec.parameters.taper_method.unwrap_or_default(),
            ),
            "segment-meander-angle" => segment_meander_angles(self, sections),
            "root-path-length" => selection_node_field(
                self,
                view,
                view.root_path_lengths(),
                self.units(),
                "path-length-from-selected-induced-arbor-root",
            ),
            "radial-distance" => selection_node_field(
                self,
                view,
                view.radial_distances(),
                self.units(),
                "euclidean-distance-from-selected-induced-arbor-root",
            ),
            "branch-order" => selection_order_field(
                self,
                view,
                view.branch_orders(),
                "centrifugal-order-in-selected-induced-forest",
            ),
            "strahler-order" => selection_order_field(
                self,
                view,
                view.strahler_orders(),
                "standard-strahler-order-in-selected-induced-forest",
            ),
            "node-count" => count_metric(view.nodes().count(), "selected-node-count"),
            "branch-point-count" => count_metric(
                view.nodes()
                    .filter(|node| view.child_count(*node) > 1)
                    .count(),
                "selected-topological-branch-point-count",
            ),
            "terminal-count" => count_metric(
                view.nodes()
                    .filter(|node| view.child_count(*node) == 0)
                    .count(),
                "selected-topological-terminal-count",
            ),
            "section-count" => count_metric(sections.len(), "selected-section-count"),
            "total-cable-length" => total_cable_length_metric(self, view),
            "maximum-root-path-length" => maximum_root_path_metric(self, view),
            "neurite-surface-area" => selected_frustum_metric(self, view, false),
            "neurite-volume" => selected_frustum_metric(self, view, true),
            "section-length" => section_geometry_field(self, sections, false),
            "section-contraction" => section_geometry_field(self, sections, true),
            "centroid" => {
                morphology_centroid(self, view, spec.parameters.weighting.unwrap_or_default())
            }
            "bounding-box" => morphology_bbox(self, view),
            "principal-extents" => {
                principal_extents_metric(self, view, spec.parameters.weighting.unwrap_or_default())
            }
            "fractional-anisotropy" => {
                fractional_anisotropy(self, view, spec.parameters.weighting.unwrap_or_default())
            }
            "convex-hull-2d-area" => convex_hull_2d_metric(
                self,
                view,
                spec.parameters.plane.unwrap_or_default(),
                spec.parameters.weighting.unwrap_or_default(),
            ),
            "convex-hull-3d-surface-area" => convex_hull_3d_metric(self, view, false),
            "convex-hull-3d-volume" => convex_hull_3d_metric(self, view, true),
            "volume-density" => volume_density(self, view),
            _ => return Err(MetricError::UnknownMetric(spec.id.clone())),
        };
        if matches!(
            data,
            MetricData::SectionField(_) | MetricData::BifurcationField(_)
        ) || id == "segment-meander-angle"
        {
            parameters.insert(
                "section-boundaries".to_owned(),
                ParameterValue::Text(enum_text(section_boundaries)),
            );
        }
        Ok(MetricResult {
            schema_version: METRIC_RESULT_SCHEMA_VERSION,
            metric: MetricDescriptor {
                id: spec.id.clone(),
                definition_version: 1,
                parameters,
            },
            selection: view.query.clone(),
            source: MetricSource {
                morphology_fingerprint: self.fingerprint().to_owned(),
                topology_fingerprint: self.topology_fingerprint().to_owned(),
                selection_fingerprint: view.fingerprint().to_owned(),
            },
            provenance: MetricProvenance {
                implementation: format!("axodendron-core/{}", env!("CARGO_PKG_VERSION")),
                algorithm,
                notes,
            },
            data,
            missing,
        })
    }

    /// Explicitly project a section- or bifurcation-supported field onto nodes.
    /// Collisions are errors unless the caller selects a reducer.
    pub fn field_to_nodes(
        &self,
        field: &MetricResult,
        options: FieldToNodesOptions,
    ) -> Result<MetricResult, FieldConversionError> {
        if field.source.morphology_fingerprint != self.fingerprint() {
            return Err(FieldConversionError::FingerprintMismatch);
        }
        let view = SelectionView::new(self, &field.selection)?;
        if field.source.selection_fingerprint != view.fingerprint()
            || field.source.topology_fingerprint != self.topology_fingerprint()
        {
            return Err(FieldConversionError::FingerprintMismatch);
        }
        let policy = match field.metric.parameters.get("section-boundaries") {
            Some(ParameterValue::Text(value)) if value == "topology-only" => {
                SectionBoundaryPolicy::TopologyOnly
            }
            _ => SectionBoundaryPolicy::TopologyAndType,
        };
        let sections = metric_sections(self, &view, policy);
        let section_by_key: HashMap<SectionRef, &MetricSection> = sections
            .iter()
            .map(|section| (section.key.clone(), section))
            .collect();
        let mut mapped: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
        match (&field.data, options.placement) {
            (MetricData::NodeField(node), _) => {
                for (id, value) in node
                    .node_ids
                    .iter()
                    .copied()
                    .zip(node.values.iter().copied())
                {
                    mapped.entry(id).or_default().push(value);
                }
            }
            (
                MetricData::SectionField(section_field),
                placement @ (FieldPlacement::SectionProximal
                | FieldPlacement::SectionDistal
                | FieldPlacement::SectionBroadcast),
            ) => {
                if section_field.sections.len() != section_field.values.len() {
                    return Err(FieldConversionError::InvalidSectionReference);
                }
                for (key, value) in section_field.sections.iter().zip(&section_field.values) {
                    let section = section_by_key
                        .get(key)
                        .ok_or(FieldConversionError::InvalidSectionReference)?;
                    let nodes: Vec<i64> = match placement {
                        FieldPlacement::SectionProximal => vec![key.proximal_node],
                        FieldPlacement::SectionDistal => vec![key.distal_node],
                        FieldPlacement::SectionBroadcast => {
                            section.nodes.iter().map(|node| self.id(*node).0).collect()
                        }
                        _ => unreachable!(),
                    };
                    for id in nodes {
                        mapped.entry(id).or_default().push(*value);
                    }
                }
            }
            (
                MetricData::BifurcationField(bifurcation_field),
                placement @ (FieldPlacement::BifurcationBranch
                | FieldPlacement::BifurcationChildren),
            ) => {
                if bifurcation_field.bifurcations.len() != bifurcation_field.values.len() {
                    return Err(FieldConversionError::InvalidSectionReference);
                }
                for (key, value) in bifurcation_field
                    .bifurcations
                    .iter()
                    .zip(&bifurcation_field.values)
                {
                    let nodes = if placement == FieldPlacement::BifurcationBranch {
                        vec![key.branch_node]
                    } else {
                        let mut nodes = Vec::new();
                        for child in &key.child_sections {
                            let section = section_by_key
                                .get(child)
                                .ok_or(FieldConversionError::InvalidSectionReference)?;
                            let first = section
                                .nodes
                                .get(1)
                                .ok_or(FieldConversionError::InvalidSectionReference)?;
                            nodes.push(self.id(*first).0);
                        }
                        nodes
                    };
                    for id in nodes {
                        mapped.entry(id).or_default().push(*value);
                    }
                }
            }
            _ => return Err(FieldConversionError::UnsupportedField),
        }
        let mut node_ids = Vec::with_capacity(mapped.len());
        let mut values = Vec::with_capacity(mapped.len());
        for (id, entries) in mapped {
            if entries.len() > 1 && options.reducer == FieldReducer::Error {
                return Err(FieldConversionError::Collision(id));
            }
            let value = match options.reducer {
                FieldReducer::Error => entries[0],
                FieldReducer::Mean => entries.iter().sum::<f64>() / entries.len() as f64,
                FieldReducer::Minimum => entries.iter().copied().fold(f64::INFINITY, f64::min),
                FieldReducer::Maximum => entries.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                FieldReducer::Sum => entries.iter().sum(),
            };
            node_ids.push(id);
            values.push(value);
        }
        let units = match &field.data {
            MetricData::MorphologyMetric(value) => value.units.clone(),
            MetricData::NodeField(value) => value.units.clone(),
            MetricData::SectionField(value) => value.units.clone(),
            MetricData::BifurcationField(value) => value.units.clone(),
        };
        let mut result = field.clone();
        result.metric.parameters.insert(
            "node-placement".to_owned(),
            ParameterValue::Text(enum_text(options.placement)),
        );
        result.metric.parameters.insert(
            "node-reducer".to_owned(),
            ParameterValue::Text(enum_text(options.reducer)),
        );
        result.provenance.algorithm = format!(
            "explicit-field-to-node-projection({}; {})",
            enum_text(options.placement),
            enum_text(options.reducer)
        );
        result.data = MetricData::NodeField(MetricNodeField {
            node_ids,
            values,
            units,
        });
        Ok(result)
    }
}

type ComputedMetric = (
    MetricData,
    Vec<MissingValue>,
    BTreeMap<String, ParameterValue>,
    String,
    Vec<String>,
);

pub fn metric_registry() -> Vec<MetricDefinition> {
    let neurom_bifurcation = Some(
        "https://neurom.readthedocs.io/en/stable/_neurom_build/neurom.features.bifurcation.html"
            .to_owned(),
    );
    let neurom_section = Some(
        "https://neurom.readthedocs.io/en/stable/_neurom_build/neurom.features.section.html"
            .to_owned(),
    );
    let lmeasure = Some("https://doi.org/10.1038/nprot.2008.51".to_owned());
    vec![
        definition(
            "local-bifurcation-angle",
            "bifurcation-field",
            "deg",
            "Angle between the first non-zero outgoing segment vectors.",
            neurom_bifurcation.clone(),
        ),
        definition(
            "remote-bifurcation-angle",
            "bifurcation-field",
            "deg",
            "Angle between vectors from the branch node to outgoing section endpoints.",
            neurom_bifurcation.clone(),
        ),
        definition(
            "sibling-ratio",
            "bifurcation-field",
            "1",
            "Smaller divided by larger positive child diameter.",
            neurom_bifurcation.clone(),
        ),
        definition(
            "partition-asymmetry-terminal",
            "bifurcation-field",
            "1",
            "Uylings terminal-count partition asymmetry.",
            neurom_bifurcation.clone(),
        ),
        definition(
            "diameter-power-ratio",
            "bifurcation-field",
            "1",
            "Sum of child diameters to p divided by parent diameter to p.",
            lmeasure.clone(),
        ),
        definition(
            "rall-ratio",
            "bifurcation-field",
            "1",
            "Diameter power ratio with p=3/2 unless explicitly overridden.",
            lmeasure,
        ),
        definition(
            "taper-rate",
            "section-field",
            "1",
            "Signed radius or diameter change per path length, estimated by endpoint difference or least-squares fit.",
            neurom_section.clone(),
        ),
        definition(
            "segment-meander-angle",
            "node-field",
            "deg",
            "Turning angle between consecutive non-zero segments inside a section.",
            neurom_section,
        ),
        definition(
            "root-path-length",
            "node-field",
            "morphology-units",
            "Path length from each selected induced-arbor root.",
            None,
        ),
        definition(
            "radial-distance",
            "node-field",
            "morphology-units",
            "Euclidean distance from each selected induced-arbor root.",
            None,
        ),
        definition(
            "branch-order",
            "node-field",
            "1",
            "Centrifugal branch order, starting at one and increasing after selected multifurcations.",
            None,
        ),
        definition(
            "strahler-order",
            "node-field",
            "1",
            "Standard Strahler order in the selected induced forest.",
            None,
        ),
        definition(
            "node-count",
            "morphology-metric",
            "1",
            "Number of selected nodes.",
            None,
        ),
        definition(
            "branch-point-count",
            "morphology-metric",
            "1",
            "Number of selected nodes with more than one selected child.",
            None,
        ),
        definition(
            "terminal-count",
            "morphology-metric",
            "1",
            "Number of selected nodes without a selected child.",
            None,
        ),
        definition(
            "section-count",
            "morphology-metric",
            "1",
            "Number of sections under the requested boundary policy.",
            None,
        ),
        definition(
            "total-cable-length",
            "morphology-metric",
            "morphology-units",
            "Sum of selected parent-child segment lengths.",
            None,
        ),
        definition(
            "maximum-root-path-length",
            "morphology-metric",
            "morphology-units",
            "Maximum selected induced-arbor root path length.",
            None,
        ),
        definition(
            "neurite-surface-area",
            "morphology-metric",
            "morphology-units^2",
            "Uncapped circular-frustum lateral area over selected edges.",
            None,
        ),
        definition(
            "neurite-volume",
            "morphology-metric",
            "morphology-units^3",
            "Uncapped circular-frustum volume over selected edges.",
            None,
        ),
        definition(
            "section-length",
            "section-field",
            "morphology-units",
            "Path length of each selected section.",
            None,
        ),
        definition(
            "section-contraction",
            "section-field",
            "1",
            "Section endpoint distance divided by section path length.",
            None,
        ),
        definition(
            "centroid",
            "morphology-metric",
            "morphology-units",
            "Selected-node or exact cable-length weighted centroid.",
            None,
        ),
        definition(
            "bounding-box",
            "morphology-metric",
            "morphology-units",
            "Axis-aligned bounds of selected node centers; radii are excluded.",
            None,
        ),
        definition(
            "principal-extents",
            "morphology-metric",
            "morphology-units",
            "Full min-to-max spans along deterministic principal axes.",
            None,
        ),
        definition(
            "fractional-anisotropy",
            "morphology-metric",
            "1",
            "Fractional anisotropy of the selected spatial covariance eigenvalues.",
            None,
        ),
        definition(
            "convex-hull-2d-area",
            "morphology-metric",
            "morphology-units^2",
            "Area of selected node centers after the requested orthographic projection.",
            None,
        ),
        definition(
            "convex-hull-3d-surface-area",
            "morphology-metric",
            "morphology-units^2",
            "Surface area of the three-dimensional node-center convex hull.",
            None,
        ),
        definition(
            "convex-hull-3d-volume",
            "morphology-metric",
            "morphology-units^3",
            "Volume of the three-dimensional node-center convex hull.",
            None,
        ),
        definition(
            "volume-density",
            "morphology-metric",
            "1",
            "Selected circular-frustum volume divided by the node-center convex hull volume.",
            None,
        ),
    ]
}

fn definition(
    id: &str,
    entity: &str,
    units: &str,
    summary: &str,
    reference: Option<String>,
) -> MetricDefinition {
    MetricDefinition {
        id: id.to_owned(),
        definition_version: 1,
        entity: entity.to_owned(),
        units: units.to_owned(),
        summary: summary.to_owned(),
        parameters: parameter_definitions(id),
        reference,
    }
}

fn parameter_definitions(id: &str) -> Vec<MetricParameterDefinition> {
    let enum_parameter =
        |name: &str, default: &str, choices: &[&str], summary: &str| MetricParameterDefinition {
            name: name.to_owned(),
            value_type: "str".to_owned(),
            default: Some(ParameterValue::Text(default.to_owned())),
            choices: choices
                .iter()
                .map(|value| ParameterValue::Text((*value).to_owned()))
                .collect(),
            minimum: None,
            exclusive_minimum: false,
            applies_when: None,
            summary: summary.to_owned(),
        };
    let multifurcation = || {
        enum_parameter(
            "multifurcation",
            "pairwise",
            &["pairwise", "exclude"],
            "Policy for branch nodes with more than two selected children.",
        )
    };
    let diameter_sampling = || {
        enum_parameter(
            "diameter-sampling",
            "first-point",
            &["first-point", "section-mean"],
            "Child-section diameter sampling rule.",
        )
    };
    let weighting = || {
        enum_parameter(
            "weighting",
            "cable-length",
            &["cable-length", "nodes"],
            "Spatial moment weighting rule.",
        )
    };
    match id {
        "local-bifurcation-angle" | "remote-bifurcation-angle" | "partition-asymmetry-terminal" => {
            vec![multifurcation()]
        }
        "sibling-ratio" => vec![diameter_sampling(), multifurcation()],
        "diameter-power-ratio" | "rall-ratio" => vec![
            MetricParameterDefinition {
                name: "p".to_owned(),
                value_type: "number".to_owned(),
                default: Some(ParameterValue::Number(1.5)),
                choices: Vec::new(),
                minimum: Some(0.0),
                exclusive_minimum: true,
                applies_when: None,
                summary: "Positive diameter-power exponent.".to_owned(),
            },
            diameter_sampling(),
        ],
        "taper-rate" => vec![
            enum_parameter(
                "taper-quantity",
                "diameter",
                &["diameter", "radius"],
                "Geometric quantity whose signed slope is measured.",
            ),
            enum_parameter(
                "taper-method",
                "linear-fit",
                &["linear-fit", "endpoint"],
                "Slope estimator along section path distance.",
            ),
        ],
        "centroid" | "principal-extents" | "fractional-anisotropy" => vec![weighting()],
        "convex-hull-2d-area" => {
            let mut principal_weighting = weighting();
            principal_weighting.applies_when =
                Some("plane is principal-xy, principal-xz, or principal-yz".to_owned());
            vec![
                enum_parameter(
                    "plane",
                    "xy",
                    &[
                        "xy",
                        "xz",
                        "yz",
                        "principal-xy",
                        "principal-xz",
                        "principal-yz",
                    ],
                    "Projection plane used before computing area.",
                ),
                principal_weighting,
            ]
        }
        _ => Vec::new(),
    }
}

fn validate_metric_parameters(spec: &MetricSpec) -> Result<(), MetricError> {
    let allowed: &[&str] = match spec.id.as_str() {
        "local-bifurcation-angle" | "remote-bifurcation-angle" | "partition-asymmetry-terminal" => {
            &["multifurcation"]
        }
        "sibling-ratio" => &["diameter-sampling", "multifurcation"],
        "diameter-power-ratio" | "rall-ratio" => &["p", "diameter-sampling"],
        "taper-rate" => &["taper-quantity", "taper-method"],
        "centroid" | "principal-extents" | "fractional-anisotropy" => &["weighting"],
        "convex-hull-2d-area" => &["plane", "weighting"],
        _ => &[],
    };
    let supplied = [
        ("p", spec.parameters.p.is_some()),
        (
            "diameter-sampling",
            spec.parameters.diameter_sampling.is_some(),
        ),
        ("taper-quantity", spec.parameters.taper_quantity.is_some()),
        ("taper-method", spec.parameters.taper_method.is_some()),
        ("multifurcation", spec.parameters.multifurcation.is_some()),
        ("weighting", spec.parameters.weighting.is_some()),
        ("plane", spec.parameters.plane.is_some()),
    ];
    if let Some((name, _)) = supplied
        .iter()
        .find(|(name, present)| *present && !allowed.contains(name))
    {
        return Err(MetricError::InvalidParameter {
            metric: spec.id.clone(),
            message: format!("parameter {name:?} is not defined for this metric"),
        });
    }
    if spec.id == "convex-hull-2d-area"
        && spec.parameters.weighting.is_some()
        && matches!(
            spec.parameters.plane.unwrap_or_default(),
            SpatialPlane::Xy | SpatialPlane::Xz | SpatialPlane::Yz
        )
    {
        return Err(MetricError::InvalidParameter {
            metric: spec.id.clone(),
            message: "`weighting` is only applicable to a principal projection plane".to_owned(),
        });
    }
    Ok(())
}

fn is_known_metric(id: &str) -> bool {
    matches!(
        id,
        "local-bifurcation-angle"
            | "remote-bifurcation-angle"
            | "sibling-ratio"
            | "partition-asymmetry-terminal"
            | "diameter-power-ratio"
            | "rall-ratio"
            | "taper-rate"
            | "segment-meander-angle"
            | "root-path-length"
            | "radial-distance"
            | "branch-order"
            | "strahler-order"
            | "node-count"
            | "branch-point-count"
            | "terminal-count"
            | "section-count"
            | "total-cable-length"
            | "maximum-root-path-length"
            | "neurite-surface-area"
            | "neurite-volume"
            | "section-length"
            | "section-contraction"
            | "centroid"
            | "bounding-box"
            | "principal-extents"
            | "fractional-anisotropy"
            | "convex-hull-2d-area"
            | "convex-hull-3d-surface-area"
            | "convex-hull-3d-volume"
            | "volume-density"
    )
}

fn metric_sections(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    policy: SectionBoundaryPolicy,
) -> Vec<MetricSection> {
    let mut sections = Vec::new();
    for start in view.nodes() {
        let boundary = view.parent(start).is_none()
            || view.child_count(start) != 1
            || (policy == SectionBoundaryPolicy::TopologyAndType
                && view
                    .parent(start)
                    .is_some_and(|parent| morphology.kind(parent) != morphology.kind(start)));
        if !boundary {
            continue;
        }
        for first in view.children(start) {
            let mut nodes = vec![start, first];
            let mut cursor = first;
            let first_is_type_boundary = policy == SectionBoundaryPolicy::TopologyAndType
                && morphology.kind(start) != morphology.kind(first);
            while !first_is_type_boundary && view.child_count(cursor) == 1 {
                let next = view.children(cursor).next().expect("one child was counted");
                let type_boundary = policy == SectionBoundaryPolicy::TopologyAndType
                    && morphology.kind(cursor) != morphology.kind(next);
                nodes.push(next);
                cursor = next;
                if type_boundary {
                    break;
                }
            }
            sections.push(MetricSection {
                key: SectionRef {
                    topology_fingerprint: morphology.topology_fingerprint().to_owned(),
                    selection_fingerprint: view.fingerprint().to_owned(),
                    section_definition_version: SECTION_DEFINITION_VERSION,
                    boundary_policy: policy,
                    proximal_node: morphology.id(start).0,
                    distal_node: morphology.id(cursor).0,
                },
                nodes,
            });
        }
    }
    sections
}

fn outgoing_sections(sections: &[MetricSection]) -> HashMap<i64, Vec<&MetricSection>> {
    let mut result: HashMap<i64, Vec<&MetricSection>> = HashMap::new();
    for section in sections {
        result
            .entry(section.key.proximal_node)
            .or_default()
            .push(section);
    }
    for children in result.values_mut() {
        children.sort_by_key(|section| section.nodes.get(1).map(|node| node.0).unwrap_or(u32::MAX));
    }
    result
}

fn branch_pairs<'a>(
    children: &'a [&'a MetricSection],
    policy: MultifurcationPolicy,
) -> Vec<(&'a MetricSection, &'a MetricSection)> {
    if children.len() < 2 || (children.len() != 2 && policy == MultifurcationPolicy::Exclude) {
        return Vec::new();
    }
    let mut pairs = Vec::new();
    for a in 0..children.len() {
        for b in a + 1..children.len() {
            pairs.push((children[a], children[b]));
        }
    }
    pairs
}

fn pair_key(branch: i64, a: &MetricSection, b: &MetricSection) -> BifurcationKey {
    let mut children = vec![a.key.clone(), b.key.clone()];
    children.sort_by_key(|section| (section.proximal_node, section.distal_node));
    BifurcationKey {
        branch_node: branch,
        child_sections: children,
    }
}

fn all_children_key(branch: i64, children: &[&MetricSection]) -> BifurcationKey {
    let mut children: Vec<SectionRef> = children.iter().map(|item| item.key.clone()).collect();
    children.sort_by_key(|section| (section.proximal_node, section.distal_node));
    BifurcationKey {
        branch_node: branch,
        child_sections: children,
    }
}

fn bifurcation_angles(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    sections: &[MetricSection],
    remote: bool,
    policy: MultifurcationPolicy,
) -> ComputedMetric {
    let outgoing = outgoing_sections(sections);
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut missing = Vec::new();
    for node in view.nodes().filter(|node| view.child_count(*node) > 1) {
        let branch = morphology.id(node).0;
        let children = outgoing.get(&branch).cloned().unwrap_or_default();
        if children.len() != 2 && policy == MultifurcationPolicy::Exclude {
            missing.push(MissingValue {
                entity: EntityKey::Bifurcation {
                    bifurcation: all_children_key(branch, &children),
                },
                reason: MissingReason::NonBinaryBifurcation,
                detail: format!("branch has {} outgoing sections", children.len()),
            });
            continue;
        }
        for (a, b) in branch_pairs(&children, policy) {
            let key = pair_key(branch, a, b);
            let vector = |section: &MetricSection| {
                if remote {
                    let distal = *section.nodes.last()?;
                    let value = morphology.position(distal) - morphology.position(node);
                    value.normalized()
                } else {
                    section.nodes.windows(2).find_map(|pair| {
                        (morphology.position(pair[1]) - morphology.position(pair[0])).normalized()
                    })
                }
            };
            match (vector(a), vector(b)) {
                (Some(a), Some(b)) => {
                    keys.push(key);
                    values.push(a.dot(b).clamp(-1.0, 1.0).acos().to_degrees());
                }
                _ => missing.push(MissingValue {
                    entity: EntityKey::Bifurcation { bifurcation: key },
                    reason: MissingReason::ZeroLength,
                    detail: "one or both child direction vectors have zero length".to_owned(),
                }),
            }
        }
    }
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "multifurcation".to_owned(),
        ParameterValue::Text(enum_text(policy)),
    );
    (
        MetricData::BifurcationField(BifurcationField {
            bifurcations: keys,
            values,
            units: "deg".to_owned(),
        }),
        missing,
        parameters,
        if remote {
            "branch-to-distal-section-endpoint-vector-angle".to_owned()
        } else {
            "first-nonzero-outgoing-segment-vector-angle".to_owned()
        },
        vec!["Angles are in degrees on [0, 180].".to_owned()],
    )
}

fn child_diameter(
    morphology: &Morphology,
    section: &MetricSection,
    sampling: DiameterSampling,
) -> Option<f64> {
    let nodes = section.nodes.get(1..)?;
    match sampling {
        DiameterSampling::FirstPoint => nodes
            .first()
            .map(|node| 2.0 * morphology.radius(*node))
            .filter(|diameter| diameter.is_finite() && *diameter > 0.0),
        DiameterSampling::SectionMean => {
            let values: Vec<f64> = nodes
                .iter()
                .map(|node| 2.0 * morphology.radius(*node))
                .filter(|value| value.is_finite() && *value > 0.0)
                .collect();
            (values.len() == nodes.len() && !values.is_empty())
                .then(|| values.iter().sum::<f64>() / values.len() as f64)
        }
    }
}

fn sibling_ratios(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    sections: &[MetricSection],
    sampling: DiameterSampling,
    policy: MultifurcationPolicy,
) -> ComputedMetric {
    let outgoing = outgoing_sections(sections);
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut missing = Vec::new();
    for node in view.nodes().filter(|node| view.child_count(*node) > 1) {
        let branch = morphology.id(node).0;
        let children = outgoing.get(&branch).cloned().unwrap_or_default();
        if children.len() != 2 && policy == MultifurcationPolicy::Exclude {
            missing.push(MissingValue {
                entity: EntityKey::Bifurcation {
                    bifurcation: all_children_key(branch, &children),
                },
                reason: MissingReason::NonBinaryBifurcation,
                detail: format!("branch has {} outgoing sections", children.len()),
            });
            continue;
        }
        for (a, b) in branch_pairs(&children, policy) {
            let key = pair_key(branch, a, b);
            match (
                child_diameter(morphology, a, sampling),
                child_diameter(morphology, b, sampling),
            ) {
                (Some(a), Some(b)) => {
                    keys.push(key);
                    values.push(a.min(b) / a.max(b));
                }
                _ => missing.push(MissingValue {
                    entity: EntityKey::Bifurcation { bifurcation: key },
                    reason: MissingReason::NonPositiveRadius,
                    detail: "child diameter sampling encountered a non-positive radius".to_owned(),
                }),
            }
        }
    }
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "diameter-sampling".to_owned(),
        ParameterValue::Text(enum_text(sampling)),
    );
    parameters.insert(
        "multifurcation".to_owned(),
        ParameterValue::Text(enum_text(policy)),
    );
    (
        MetricData::BifurcationField(BifurcationField {
            bifurcations: keys,
            values,
            units: "1".to_owned(),
        }),
        missing,
        parameters,
        "minimum-child-diameter-divided-by-maximum-child-diameter".to_owned(),
        vec!["Valid values lie on [0, 1].".to_owned()],
    )
}

fn terminal_counts(view: &SelectionView<'_>) -> Vec<u32> {
    let mut counts = vec![0_u32; view.morphology.len()];
    let mut stack: Vec<(NodeIx, bool)> = view
        .roots()
        .into_iter()
        .rev()
        .map(|node| (node, false))
        .collect();
    while let Some((node, visited)) = stack.pop() {
        if !visited {
            stack.push((node, true));
            for child in view.children(node).rev() {
                stack.push((child, false));
            }
        } else if view.child_count(node) == 0 {
            counts[node.0 as usize] = 1;
        } else {
            counts[node.0 as usize] = view
                .children(node)
                .map(|child| counts[child.0 as usize])
                .sum();
        }
    }
    counts
}

fn partition_asymmetry(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    sections: &[MetricSection],
    policy: MultifurcationPolicy,
) -> ComputedMetric {
    let counts = terminal_counts(view);
    let outgoing = outgoing_sections(sections);
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut missing = Vec::new();
    for node in view.nodes().filter(|node| view.child_count(*node) > 1) {
        let branch = morphology.id(node).0;
        let children = outgoing.get(&branch).cloned().unwrap_or_default();
        if children.len() != 2 && policy == MultifurcationPolicy::Exclude {
            missing.push(MissingValue {
                entity: EntityKey::Bifurcation {
                    bifurcation: all_children_key(branch, &children),
                },
                reason: MissingReason::NonBinaryBifurcation,
                detail: format!("branch has {} outgoing sections", children.len()),
            });
            continue;
        }
        for (a, b) in branch_pairs(&children, policy) {
            let a_root = a.nodes[1];
            let b_root = b.nodes[1];
            let na = counts[a_root.0 as usize];
            let nb = counts[b_root.0 as usize];
            let denominator = na + nb - 2;
            let value = if denominator == 0 {
                0.0
            } else {
                (f64::from(na) - f64::from(nb)).abs() / f64::from(denominator)
            };
            keys.push(pair_key(branch, a, b));
            values.push(value);
        }
    }
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "multifurcation".to_owned(),
        ParameterValue::Text(enum_text(policy)),
    );
    parameters.insert(
        "terminal-count".to_owned(),
        ParameterValue::Text("selected-induced-subtree".to_owned()),
    );
    (
        MetricData::BifurcationField(BifurcationField {
            bifurcations: keys,
            values,
            units: "1".to_owned(),
        }),
        missing,
        parameters,
        "uylings-abs-n1-minus-n2-over-n1-plus-n2-minus-2".to_owned(),
        vec!["A bifurcation with two terminal children is defined as 0.".to_owned()],
    )
}

fn diameter_power_ratios(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    sections: &[MetricSection],
    p: f64,
    sampling: DiameterSampling,
) -> ComputedMetric {
    let outgoing = outgoing_sections(sections);
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut missing = Vec::new();
    for node in view.nodes().filter(|node| view.child_count(*node) > 1) {
        let branch = morphology.id(node).0;
        let children = outgoing.get(&branch).cloned().unwrap_or_default();
        let key = all_children_key(branch, &children);
        let parent = 2.0 * morphology.radius(node);
        let child_diameters: Option<Vec<f64>> = children
            .iter()
            .map(|section| child_diameter(morphology, section, sampling))
            .collect();
        match child_diameters.filter(|_| parent.is_finite() && parent > 0.0) {
            Some(children) => {
                keys.push(key);
                values.push(
                    children
                        .iter()
                        .map(|diameter| diameter.powf(p))
                        .sum::<f64>()
                        / parent.powf(p),
                );
            }
            None => missing.push(MissingValue {
                entity: EntityKey::Bifurcation { bifurcation: key },
                reason: MissingReason::NonPositiveRadius,
                detail: "parent or child diameter is non-positive".to_owned(),
            }),
        }
    }
    let mut parameters = BTreeMap::new();
    parameters.insert("p".to_owned(), ParameterValue::Number(p));
    parameters.insert(
        "diameter-sampling".to_owned(),
        ParameterValue::Text(enum_text(sampling)),
    );
    (
        MetricData::BifurcationField(BifurcationField {
            bifurcations: keys,
            values,
            units: "1".to_owned(),
        }),
        missing,
        parameters,
        "sum-child-diameter-powers-over-parent-diameter-power".to_owned(),
        vec!["A value of 1 satisfies the requested diameter power relation.".to_owned()],
    )
}

fn taper_rates(
    morphology: &Morphology,
    sections: &[MetricSection],
    quantity: TaperQuantity,
    method: TaperMethod,
) -> ComputedMetric {
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut missing = Vec::new();
    for section in sections {
        let mut distance = Vec::with_capacity(section.nodes.len());
        distance.push(0.0);
        for pair in section.nodes.windows(2) {
            distance.push(
                distance.last().copied().unwrap_or(0.0)
                    + morphology
                        .position(pair[0])
                        .distance(morphology.position(pair[1])),
            );
        }
        let measurements: Vec<f64> = section
            .nodes
            .iter()
            .map(|node| {
                morphology.radius(*node)
                    * if quantity == TaperQuantity::Diameter {
                        2.0
                    } else {
                        1.0
                    }
            })
            .collect();
        let entity = EntityKey::Section {
            section: section.key.clone(),
        };
        if measurements.iter().any(|value| *value <= 0.0) {
            missing.push(MissingValue {
                entity,
                reason: MissingReason::NonPositiveRadius,
                detail: "section contains a non-positive radius".to_owned(),
            });
            continue;
        }
        let total = *distance.last().unwrap_or(&0.0);
        if total <= 0.0 {
            missing.push(MissingValue {
                entity,
                reason: MissingReason::ZeroLength,
                detail: "section has zero path length".to_owned(),
            });
            continue;
        }
        let slope = match method {
            TaperMethod::Endpoint => {
                (measurements.last().unwrap() - measurements.first().unwrap()) / total
            }
            TaperMethod::LinearFit => linear_slope(&distance, &measurements).unwrap_or(0.0),
        };
        keys.push(section.key.clone());
        values.push(slope);
    }
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "taper-quantity".to_owned(),
        ParameterValue::Text(enum_text(quantity)),
    );
    parameters.insert(
        "taper-method".to_owned(),
        ParameterValue::Text(enum_text(method)),
    );
    (
        MetricData::SectionField(SectionField {
            sections: keys,
            values,
            units: "1".to_owned(),
        }),
        missing,
        parameters,
        match method {
            TaperMethod::Endpoint => "signed-endpoint-change-per-section-path-length".to_owned(),
            TaperMethod::LinearFit => {
                "ordinary-least-squares-slope-over-cumulative-path-distance".to_owned()
            }
        },
        vec!["Negative values indicate narrowing from root toward tip.".to_owned()],
    )
}

fn linear_slope(x: &[f64], y: &[f64]) -> Option<f64> {
    if x.len() != y.len() || x.len() < 2 {
        return None;
    }
    let count = x.len() as f64;
    let mean_x = x.iter().sum::<f64>() / count;
    let mean_y = y.iter().sum::<f64>() / count;
    let numerator = x
        .iter()
        .zip(y)
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum::<f64>();
    let denominator = x.iter().map(|x| (x - mean_x).powi(2)).sum::<f64>();
    (denominator > 0.0).then_some(numerator / denominator)
}

fn segment_meander_angles(morphology: &Morphology, sections: &[MetricSection]) -> ComputedMetric {
    let mut node_ids = Vec::new();
    let mut values = Vec::new();
    let mut missing = Vec::new();
    for section in sections {
        for triple in section.nodes.windows(3) {
            let node_id = morphology.id(triple[1]).0;
            let a = (morphology.position(triple[1]) - morphology.position(triple[0])).normalized();
            let b = (morphology.position(triple[2]) - morphology.position(triple[1])).normalized();
            match (a, b) {
                (Some(a), Some(b)) => {
                    node_ids.push(node_id);
                    values.push(a.dot(b).clamp(-1.0, 1.0).acos().to_degrees());
                }
                _ => missing.push(MissingValue {
                    entity: EntityKey::Node { node_id },
                    reason: MissingReason::ZeroLength,
                    detail: "one of the adjacent section segments has zero length".to_owned(),
                }),
            }
        }
    }
    (
        MetricData::NodeField(MetricNodeField {
            node_ids,
            values,
            units: "deg".to_owned(),
        }),
        missing,
        BTreeMap::new(),
        "angle-between-consecutive-outward-section-segment-vectors".to_owned(),
        vec!["Zero degrees is locally straight; section endpoints are not evaluated.".to_owned()],
    )
}

fn selection_node_field(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    indexed_values: Vec<f64>,
    units: &str,
    algorithm: &str,
) -> ComputedMetric {
    let nodes: Vec<NodeIx> = view.nodes().collect();
    (
        MetricData::NodeField(MetricNodeField {
            node_ids: nodes.iter().map(|node| morphology.id(*node).0).collect(),
            values: nodes
                .iter()
                .map(|node| indexed_values[node.0 as usize])
                .collect(),
            units: units.to_owned(),
        }),
        Vec::new(),
        BTreeMap::new(),
        algorithm.to_owned(),
        Vec::new(),
    )
}

fn selection_order_field(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    indexed_values: Vec<u32>,
    algorithm: &str,
) -> ComputedMetric {
    selection_node_field(
        morphology,
        view,
        indexed_values.into_iter().map(f64::from).collect(),
        "1",
        algorithm,
    )
}

fn count_metric(value: usize, algorithm: &str) -> ComputedMetric {
    scalar_or_structured(
        Some(MetricValue::Scalar(value as f64)),
        "1",
        BTreeMap::new(),
        algorithm.to_owned(),
        Vec::new(),
    )
}

fn total_cable_length_metric(morphology: &Morphology, view: &SelectionView<'_>) -> ComputedMetric {
    let lengths = view.nodes().filter_map(|child| {
        view.parent(child).map(|parent| {
            morphology
                .position(parent)
                .distance(morphology.position(child))
        })
    });
    scalar_or_structured(
        Some(MetricValue::Scalar(stable_sum(lengths))),
        morphology.units(),
        BTreeMap::new(),
        "compensated-sum-of-selected-edge-lengths".to_owned(),
        Vec::new(),
    )
}

fn maximum_root_path_metric(morphology: &Morphology, view: &SelectionView<'_>) -> ComputedMetric {
    let paths = view.root_path_lengths();
    let maximum = view
        .nodes()
        .map(|node| paths[node.0 as usize])
        .fold(0.0_f64, f64::max);
    scalar_or_structured(
        Some(MetricValue::Scalar(maximum)),
        morphology.units(),
        BTreeMap::new(),
        "maximum-selected-induced-arbor-root-path-length".to_owned(),
        Vec::new(),
    )
}

fn selected_frustum_metric(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    volume: bool,
) -> ComputedMetric {
    let mut invalid = BTreeSet::new();
    let mut values = Vec::new();
    for child in view.nodes() {
        let Some(parent) = view.parent(child) else {
            continue;
        };
        let r0 = morphology.radius(parent);
        let r1 = morphology.radius(child);
        if r0 <= 0.0 || r1 <= 0.0 {
            if r0 <= 0.0 {
                invalid.insert(morphology.id(parent).0);
            }
            if r1 <= 0.0 {
                invalid.insert(morphology.id(child).0);
            }
            continue;
        }
        let length = morphology
            .position(parent)
            .distance(morphology.position(child));
        let value = if volume {
            std::f64::consts::PI * length * (r0 * r0 + r0 * r1 + r1 * r1) / 3.0
        } else {
            std::f64::consts::PI * (r0 + r1) * length.hypot(r0 - r1)
        };
        if !value.is_finite() {
            return scalar_missing(
                &format!("{}^{}", morphology.units(), if volume { 3 } else { 2 }),
                BTreeMap::new(),
                MissingReason::NonFiniteResult,
                format!(
                    "frustum ending at node {} produced a non-finite result",
                    morphology.id(child).0
                ),
                "uncapped-circular-frustum",
            );
        }
        values.push(value);
    }
    let units = format!("{}^{}", morphology.units(), if volume { 3 } else { 2 });
    if !invalid.is_empty() {
        return scalar_missing(
            &units,
            BTreeMap::new(),
            MissingReason::NonPositiveRadius,
            format!(
                "selected edges contain non-positive radii at nodes {:?}",
                invalid.into_iter().collect::<Vec<_>>()
            ),
            "uncapped-circular-frustum",
        );
    }
    scalar_or_structured(
        Some(MetricValue::Scalar(stable_sum(values))),
        &units,
        BTreeMap::new(),
        if volume {
            "uncapped-circular-frustum-volume"
        } else {
            "uncapped-circular-frustum-lateral-area"
        }
        .to_owned(),
        Vec::new(),
    )
}

fn section_geometry_field(
    morphology: &Morphology,
    sections: &[MetricSection],
    contraction: bool,
) -> ComputedMetric {
    let mut keys = Vec::new();
    let mut values = Vec::new();
    let mut missing = Vec::new();
    for section in sections {
        let length = stable_sum(section.nodes.windows(2).map(|pair| {
            morphology
                .position(pair[0])
                .distance(morphology.position(pair[1]))
        }));
        if contraction && length <= 0.0 {
            missing.push(MissingValue {
                entity: EntityKey::Section {
                    section: section.key.clone(),
                },
                reason: MissingReason::ZeroLength,
                detail: "section contraction is undefined for zero path length".to_owned(),
            });
            continue;
        }
        let value = if contraction {
            let endpoint = morphology.position(section.nodes[0]).distance(
                morphology.position(*section.nodes.last().expect("section is non-empty")),
            );
            endpoint / length
        } else {
            length
        };
        keys.push(section.key.clone());
        values.push(value);
    }
    (
        MetricData::SectionField(SectionField {
            sections: keys,
            values,
            units: if contraction {
                "1".to_owned()
            } else {
                morphology.units().to_owned()
            },
        }),
        missing,
        BTreeMap::new(),
        if contraction {
            "section-endpoint-distance-over-path-length"
        } else {
            "compensated-sum-of-section-edge-lengths"
        }
        .to_owned(),
        vec![if contraction {
            "Contraction lies on [0, 1] for non-zero sections.".to_owned()
        } else {
            "Section decomposition follows the resolved boundary policy.".to_owned()
        }],
    )
}

fn stable_sum(values: impl IntoIterator<Item = f64>) -> f64 {
    let mut sum = 0.0_f64;
    let mut correction = 0.0_f64;
    for value in values {
        let next = sum + value;
        correction += if sum.abs() >= value.abs() {
            (sum - next) + value
        } else {
            (value - next) + sum
        };
        sum = next;
    }
    sum + correction
}

fn frame_options(view: &SelectionView<'_>, weighting: PrincipalWeighting) -> PrincipalFrameOptions {
    PrincipalFrameOptions {
        selection: view.query.clone(),
        weighting,
        origin: FrameOrigin::Centroid,
        ..Default::default()
    }
}

fn morphology_centroid(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    weighting: PrincipalWeighting,
) -> ComputedMetric {
    let result = morphology.principal_frame(&frame_options(view, weighting));
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "weighting".to_owned(),
        ParameterValue::Text(enum_text(weighting)),
    );
    match result {
        Ok(frame) => scalar_or_structured(
            Some(MetricValue::Vector3(frame.centroid)),
            morphology.units(),
            parameters,
            frame.provenance.covariance_model,
            Vec::new(),
        ),
        Err(error) => scalar_missing(
            morphology.units(),
            parameters,
            MissingReason::InsufficientGeometry,
            error.to_string(),
            "principal-frame-moment-integration",
        ),
    }
}

fn morphology_bbox(morphology: &Morphology, view: &SelectionView<'_>) -> ComputedMetric {
    let mut nodes = view.nodes();
    let first = nodes.next().expect("query is non-empty");
    let mut min = morphology.position(first);
    let mut max = min;
    for node in nodes {
        let value = morphology.position(node);
        min.x = min.x.min(value.x);
        min.y = min.y.min(value.y);
        min.z = min.z.min(value.z);
        max.x = max.x.max(value.x);
        max.y = max.y.max(value.y);
        max.z = max.z.max(value.z);
    }
    scalar_or_structured(
        Some(MetricValue::Box3(BBox { min, max })),
        morphology.units(),
        BTreeMap::new(),
        "axis-aligned-selected-node-center-bounds".to_owned(),
        vec!["SWC radii are intentionally excluded.".to_owned()],
    )
}

fn principal_extents_metric(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    weighting: PrincipalWeighting,
) -> ComputedMetric {
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "weighting".to_owned(),
        ParameterValue::Text(enum_text(weighting)),
    );
    match morphology.principal_frame(&frame_options(view, weighting)) {
        Ok(frame) => scalar_or_structured(
            Some(MetricValue::Vector3(frame.extents())),
            morphology.units(),
            parameters,
            "full-minimum-to-maximum-span-on-principal-axes".to_owned(),
            vec![format!(
                "principal-frame definition {}",
                frame.provenance.definition_version
            )],
        ),
        Err(error) => scalar_missing(
            morphology.units(),
            parameters,
            MissingReason::InsufficientGeometry,
            error.to_string(),
            "principal-frame-extents",
        ),
    }
}

fn fractional_anisotropy(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    weighting: PrincipalWeighting,
) -> ComputedMetric {
    let mut parameters = BTreeMap::new();
    parameters.insert(
        "weighting".to_owned(),
        ParameterValue::Text(enum_text(weighting)),
    );
    match morphology.principal_frame(&frame_options(view, weighting)) {
        Ok(frame) => {
            let values = frame.eigenvalues.to_array();
            let mean = values.iter().sum::<f64>() / 3.0;
            let denominator = values.iter().map(|value| value * value).sum::<f64>();
            if denominator <= 0.0 {
                scalar_missing(
                    "1",
                    parameters,
                    MissingReason::Degenerate,
                    "all covariance eigenvalues are zero".to_owned(),
                    "fractional-anisotropy",
                )
            } else {
                let numerator = values
                    .iter()
                    .map(|value| (value - mean).powi(2))
                    .sum::<f64>();
                scalar_or_structured(
                    Some(MetricValue::Scalar((1.5 * numerator / denominator).sqrt())),
                    "1",
                    parameters,
                    "sqrt-three-halves-times-eigenvalue-variance-over-squared-norm".to_owned(),
                    Vec::new(),
                )
            }
        }
        Err(error) => scalar_missing(
            "1",
            parameters,
            MissingReason::InsufficientGeometry,
            error.to_string(),
            "fractional-anisotropy",
        ),
    }
}

fn coordinate_projection(plane: SpatialPlane) -> Option<Projection> {
    match plane {
        SpatialPlane::Xy => Some(Projection::xy()),
        SpatialPlane::Xz => Some(Projection::xz()),
        SpatialPlane::Yz => Some(Projection::yz()),
        _ => None,
    }
}

fn convex_hull_2d_metric(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    plane: SpatialPlane,
    weighting: PrincipalWeighting,
) -> ComputedMetric {
    let mut parameters = BTreeMap::new();
    parameters.insert("plane".to_owned(), ParameterValue::Text(enum_text(plane)));
    if matches!(
        plane,
        SpatialPlane::PrincipalXy | SpatialPlane::PrincipalXz | SpatialPlane::PrincipalYz
    ) {
        parameters.insert(
            "weighting".to_owned(),
            ParameterValue::Text(enum_text(weighting)),
        );
    }
    let projection = if let Some(projection) = coordinate_projection(plane) {
        Ok(projection)
    } else {
        morphology
            .principal_frame(&frame_options(view, weighting))
            .map(|frame| match plane {
                SpatialPlane::PrincipalXy => frame.projection(PrincipalPlane::Xy),
                SpatialPlane::PrincipalXz => frame.projection(PrincipalPlane::Xz),
                SpatialPlane::PrincipalYz => frame.projection(PrincipalPlane::Yz),
                _ => unreachable!(),
            })
    };
    let projection = match projection {
        Ok(value) => value,
        Err(error) => {
            return scalar_missing(
                &format!("{}^2", morphology.units()),
                parameters,
                MissingReason::InsufficientGeometry,
                error.to_string(),
                "andrew-monotone-chain",
            );
        }
    };
    let points: Vec<Vec2> = view
        .nodes()
        .map(|node| projection.project(morphology.position(node)).0)
        .collect();
    match convex_hull_2d(&points) {
        Some(area) => scalar_or_structured(
            Some(MetricValue::Scalar(area)),
            &format!("{}^2", morphology.units()),
            parameters,
            "andrew-monotone-chain".to_owned(),
            vec!["The support is selected SWC node centers; radii are excluded.".to_owned()],
        ),
        None => scalar_missing(
            &format!("{}^2", morphology.units()),
            parameters,
            MissingReason::Degenerate,
            "fewer than three non-collinear projected node centers".to_owned(),
            "andrew-monotone-chain",
        ),
    }
}

fn convex_hull_3d_metric(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    volume: bool,
) -> ComputedMetric {
    let points: Vec<Vec3> = view.nodes().map(|node| morphology.position(node)).collect();
    let units = format!("{}^{}", morphology.units(), if volume { 3 } else { 2 });
    match convex_hull_3d(&points) {
        Some(hull) => scalar_or_structured(
            Some(MetricValue::Scalar(if volume {
                hull.volume
            } else {
                hull.area
            })),
            &units,
            BTreeMap::new(),
            "deterministic-incremental-convex-hull-with-scale-aware-predicate".to_owned(),
            vec![
                "The support is selected SWC node centers; radii are excluded.".to_owned(),
                format!("orientation tolerance: {}", hull.tolerance),
            ],
        ),
        None => scalar_missing(
            &units,
            BTreeMap::new(),
            MissingReason::Degenerate,
            "fewer than four non-coplanar node centers".to_owned(),
            "deterministic-incremental-convex-hull",
        ),
    }
}

fn volume_density(morphology: &Morphology, view: &SelectionView<'_>) -> ComputedMetric {
    let points: Vec<Vec3> = view.nodes().map(|node| morphology.position(node)).collect();
    let Some(hull) = convex_hull_3d(&points) else {
        return scalar_missing(
            "1",
            BTreeMap::new(),
            MissingReason::Degenerate,
            "three-dimensional convex hull volume is undefined".to_owned(),
            "frustum-volume-over-node-center-convex-hull-volume",
        );
    };
    if hull.volume <= 0.0 {
        return scalar_missing(
            "1",
            BTreeMap::new(),
            MissingReason::Degenerate,
            "three-dimensional convex hull volume is zero".to_owned(),
            "frustum-volume-over-node-center-convex-hull-volume",
        );
    }
    let mut volume = 0.0;
    for child in view.nodes() {
        let Some(parent) = view.parent(child) else {
            continue;
        };
        let r0 = morphology.radius(parent);
        let r1 = morphology.radius(child);
        if r0 <= 0.0 || r1 <= 0.0 {
            return scalar_missing(
                "1",
                BTreeMap::new(),
                MissingReason::NonPositiveRadius,
                format!(
                    "edge ending at node {} has a non-positive endpoint radius",
                    morphology.id(child).0
                ),
                "frustum-volume-over-node-center-convex-hull-volume",
            );
        }
        let length = morphology
            .position(parent)
            .distance(morphology.position(child));
        volume += std::f64::consts::PI * length * (r0 * r0 + r0 * r1 + r1 * r1) / 3.0;
    }
    scalar_or_structured(
        Some(MetricValue::Scalar(volume / hull.volume)),
        "1",
        BTreeMap::new(),
        "uncapped-circular-frustum-volume-over-node-center-convex-hull-volume".to_owned(),
        vec!["Soma inclusion follows the common selection query.".to_owned()],
    )
}

fn scalar_or_structured(
    value: Option<MetricValue>,
    units: &str,
    parameters: BTreeMap<String, ParameterValue>,
    algorithm: String,
    notes: Vec<String>,
) -> ComputedMetric {
    (
        MetricData::MorphologyMetric(MorphologyMetric {
            value,
            units: units.to_owned(),
        }),
        Vec::new(),
        parameters,
        algorithm,
        notes,
    )
}

fn scalar_missing(
    units: &str,
    parameters: BTreeMap<String, ParameterValue>,
    reason: MissingReason,
    detail: String,
    algorithm: &str,
) -> ComputedMetric {
    (
        MetricData::MorphologyMetric(MorphologyMetric {
            value: None,
            units: units.to_owned(),
        }),
        vec![MissingValue {
            entity: EntityKey::Morphology,
            reason,
            detail,
        }],
        parameters,
        algorithm.to_owned(),
        Vec::new(),
    )
}

fn enum_text<T: std::fmt::Debug>(value: T) -> String {
    // All enums passed here use kebab-case unit variants. CBOR would preserve
    // the same text, but this explicit mapping keeps the core dependency-free.
    let debug = format!("{value:?}");
    let mut text = String::new();
    for (ix, character) in debug.chars().enumerate() {
        if character.is_ascii_uppercase() && ix > 0 {
            text.push('-');
        }
        text.push(character.to_ascii_lowercase());
    }
    text
}

fn convex_hull_2d(points: &[Vec2]) -> Option<f64> {
    let mut points = points.to_vec();
    points.sort_by(|a, b| a.x.total_cmp(&b.x).then_with(|| a.y.total_cmp(&b.y)));
    points.dedup_by(|a, b| a.x == b.x && a.y == b.y);
    if points.len() < 3 {
        return None;
    }
    fn cross(o: Vec2, a: Vec2, b: Vec2) -> f64 {
        (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
    }
    let scale = points
        .iter()
        .fold(0.0_f64, |value, point| {
            value.max(point.x.abs()).max(point.y.abs())
        })
        .max(1.0);
    let tolerance = 64.0 * f64::EPSILON * scale * scale;
    let mut lower = Vec::new();
    for point in &points {
        while lower.len() >= 2
            && cross(lower[lower.len() - 2], lower[lower.len() - 1], *point) <= tolerance
        {
            lower.pop();
        }
        lower.push(*point);
    }
    let mut upper = Vec::new();
    for point in points.iter().rev() {
        while upper.len() >= 2
            && cross(upper[upper.len() - 2], upper[upper.len() - 1], *point) <= tolerance
        {
            upper.pop();
        }
        upper.push(*point);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    if lower.len() < 3 {
        return None;
    }
    let area = lower
        .iter()
        .zip(lower.iter().cycle().skip(1))
        .take(lower.len())
        .map(|(a, b)| a.x * b.y - a.y * b.x)
        .sum::<f64>()
        .abs()
        * 0.5;
    (area > tolerance).then_some(area)
}

#[derive(Clone, Copy)]
struct Face(usize, usize, usize);

struct Hull3 {
    area: f64,
    volume: f64,
    tolerance: f64,
}

fn convex_hull_3d(points: &[Vec3]) -> Option<Hull3> {
    let mut points = points.to_vec();
    points.sort_by(|a, b| {
        a.x.total_cmp(&b.x)
            .then_with(|| a.y.total_cmp(&b.y))
            .then_with(|| a.z.total_cmp(&b.z))
    });
    points.dedup_by(|a, b| a == b);
    if points.len() < 4 {
        return None;
    }
    let mut min = points[0];
    let mut max = points[0];
    for point in &points[1..] {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }
    let scale = (max - min).norm().max(1.0);
    let tolerance = 256.0 * f64::EPSILON * scale;
    let i0 = 0;
    let i1 = (1..points.len()).max_by(|a, b| {
        points[*a]
            .distance(points[i0])
            .total_cmp(&points[*b].distance(points[i0]))
    })?;
    let line = points[i1] - points[i0];
    let line_norm = line.norm();
    if line_norm <= tolerance {
        return None;
    }
    let i2 = (0..points.len())
        .filter(|ix| *ix != i0 && *ix != i1)
        .max_by(|a, b| {
            line.cross(points[*a] - points[i0])
                .norm()
                .total_cmp(&line.cross(points[*b] - points[i0]).norm())
        })?;
    if line.cross(points[i2] - points[i0]).norm() <= tolerance * line_norm {
        return None;
    }
    let normal = (points[i1] - points[i0]).cross(points[i2] - points[i0]);
    let i3 = (0..points.len())
        .filter(|ix| *ix != i0 && *ix != i1 && *ix != i2)
        .max_by(|a, b| {
            normal
                .dot(points[*a] - points[i0])
                .abs()
                .total_cmp(&normal.dot(points[*b] - points[i0]).abs())
        })?;
    if normal.dot(points[i3] - points[i0]).abs() <= tolerance * normal.norm() {
        return None;
    }
    let interior = (points[i0] + points[i1] + points[i2] + points[i3]) * 0.25;
    let mut faces = vec![
        oriented_face(i0, i1, i2, &points, interior),
        oriented_face(i0, i3, i1, &points, interior),
        oriented_face(i0, i2, i3, &points, interior),
        oriented_face(i1, i3, i2, &points, interior),
    ];
    let initial = [i0, i1, i2, i3];
    for point_ix in 0..points.len() {
        if initial.contains(&point_ix) {
            continue;
        }
        let visible: Vec<usize> = faces
            .iter()
            .enumerate()
            .filter_map(|(ix, face)| {
                (face_distance(*face, points[point_ix], &points) > tolerance).then_some(ix)
            })
            .collect();
        if visible.is_empty() {
            continue;
        }
        let visible_set: std::collections::HashSet<usize> = visible.iter().copied().collect();
        let mut directed = HashMap::<(usize, usize), u32>::new();
        for ix in &visible {
            let Face(a, b, c) = faces[*ix];
            for edge in [(a, b), (b, c), (c, a)] {
                *directed.entry(edge).or_default() += 1;
            }
        }
        let mut horizon: Vec<(usize, usize)> = directed
            .keys()
            .copied()
            .filter(|(a, b)| !directed.contains_key(&(*b, *a)))
            .collect();
        horizon.sort_unstable();
        faces = faces
            .into_iter()
            .enumerate()
            .filter_map(|(ix, face)| (!visible_set.contains(&ix)).then_some(face))
            .collect();
        for (a, b) in horizon {
            faces.push(oriented_face(a, b, point_ix, &points, interior));
        }
    }
    let mut area = 0.0;
    let mut signed_volume = 0.0;
    for Face(a, b, c) in faces {
        area += (points[b] - points[a]).cross(points[c] - points[a]).norm() * 0.5;
        signed_volume += points[a].dot(points[b].cross(points[c])) / 6.0;
    }
    let volume = signed_volume.abs();
    (area.is_finite() && volume.is_finite() && volume > tolerance.powi(3)).then_some(Hull3 {
        area,
        volume,
        tolerance,
    })
}

fn oriented_face(a: usize, b: usize, c: usize, points: &[Vec3], interior: Vec3) -> Face {
    let normal = (points[b] - points[a]).cross(points[c] - points[a]);
    if normal.dot(interior - points[a]) > 0.0 {
        Face(a, c, b)
    } else {
        Face(a, b, c)
    }
}

fn face_distance(face: Face, point: Vec3, points: &[Vec3]) -> f64 {
    let Face(a, b, c) = face;
    let normal = (points[b] - points[a]).cross(points[c] - points[a]);
    normal
        .normalized()
        .map_or(0.0, |normal| normal.dot(point - points[a]))
}

#[cfg(test)]
mod tests {
    use crate::{AnalysisDomain, ValidationProfile, parse_swc};

    use super::*;

    fn strict(source: &str) -> Morphology {
        parse_swc(source, ValidationProfile::IncfStrict)
            .morphology
            .unwrap()
    }

    fn metric(morphology: &Morphology, id: &str) -> MetricResult {
        morphology
            .measure(&MeasureOptions {
                metrics: vec![MetricSpec {
                    id: id.to_owned(),
                    parameters: MetricParameters::default(),
                }],
                selection: SelectionQuery {
                    domain: AnalysisDomain::Raw,
                    ..Default::default()
                },
                section_boundaries: SectionBoundaryPolicy::TopologyOnly,
            })
            .unwrap()
            .remove(0)
    }

    #[test]
    fn registry_and_dispatch_have_the_same_unique_metric_ids() {
        let registry = metric_registry();
        let ids: BTreeSet<&str> = registry
            .iter()
            .map(|definition| definition.id.as_str())
            .collect();
        assert_eq!(registry.len(), 30);
        assert_eq!(ids.len(), registry.len());
        assert!(ids.iter().all(|id| is_known_metric(id)));
    }

    #[test]
    fn binary_branch_metrics_have_exact_known_values() {
        let morphology = strict(
            "1 3 0 0 0 2 -1\n2 3 1 0 0 2 1\n3 3 2 1 0 1 2\n4 3 3 0 0 0.5 2\n5 3 4 0 0 0.5 4\n",
        );
        let local = metric(&morphology, "local-bifurcation-angle");
        let MetricData::BifurcationField(local) = local.data else {
            panic!()
        };
        assert!((local.values[0] - 45.0).abs() < 1e-12);

        let sibling = metric(&morphology, "sibling-ratio");
        let MetricData::BifurcationField(sibling) = sibling.data else {
            panic!()
        };
        assert!((sibling.values[0] - 0.5).abs() < 1e-12);

        let partition = metric(&morphology, "partition-asymmetry-terminal");
        let MetricData::BifurcationField(partition) = partition.data else {
            panic!()
        };
        assert_eq!(partition.values, vec![0.0]);
    }

    #[test]
    fn multifurcations_are_pairwise_and_keys_are_canonical() {
        let morphology = strict("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 0 1 0 1 1\n4 3 0 0 1 1 1\n");
        let result = metric(&morphology, "local-bifurcation-angle");
        let MetricData::BifurcationField(field) = result.data else {
            panic!()
        };
        assert_eq!(field.values.len(), 3);
        assert!(
            field
                .bifurcations
                .iter()
                .all(|key| key.child_sections.len() == 2)
        );
    }

    #[test]
    fn taper_definition_records_quantity_and_method() {
        let morphology = strict("1 3 0 0 0 2 -1\n2 3 1 0 0 1.5 1\n3 3 2 0 0 1 2\n");
        let result = metric(&morphology, "taper-rate");
        let MetricData::SectionField(field) = result.data else {
            panic!()
        };
        assert!((field.values[0] + 1.0).abs() < 1e-12);
        assert_eq!(
            result.metric.parameters.get("taper-quantity"),
            Some(&ParameterValue::Text("diameter".to_owned()))
        );
    }

    #[test]
    fn hull_metrics_are_exact_for_unit_tetrahedron() {
        let points = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        let hull = convex_hull_3d(&points).unwrap();
        assert!((hull.volume - 1.0 / 6.0).abs() < 1e-12);
        assert!((hull.area - (1.5 + 3.0_f64.sqrt() / 2.0)).abs() < 1e-12);
    }

    #[test]
    fn degenerate_hulls_are_explicitly_missing() {
        let morphology = strict("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 0 0 1 2\n4 3 3 0 0 1 3\n");
        let result = metric(&morphology, "convex-hull-3d-volume");
        assert_eq!(result.missing[0].reason, MissingReason::Degenerate);
    }

    fn scalar(result: MetricResult) -> f64 {
        let MetricData::MorphologyMetric(value) = result.data else {
            panic!("expected a morphology metric")
        };
        let Some(MetricValue::Scalar(value)) = value.value else {
            panic!("expected a scalar value")
        };
        value
    }

    #[test]
    fn rigid_motion_preserves_angles_and_uniform_scale_has_correct_hull_laws() {
        let morphology =
            strict("1 3 0 0 0 0.2 -1\n2 3 1 0 0 0.2 1\n3 3 0 1 0 0.2 1\n4 3 0 0 1 0.2 1\n");
        let rotated = morphology
            .rotate_with_report(Vec3::new(1.0, 2.0, 3.0), 0.731, Vec3::new(-2.0, 0.5, 4.0))
            .unwrap()
            .morphology;
        let MetricData::BifurcationField(original_angles) =
            metric(&morphology, "local-bifurcation-angle").data
        else {
            panic!()
        };
        let MetricData::BifurcationField(rotated_angles) =
            metric(&rotated, "local-bifurcation-angle").data
        else {
            panic!()
        };
        assert_eq!(original_angles.values.len(), rotated_angles.values.len());
        for (original, rotated) in original_angles.values.iter().zip(&rotated_angles.values) {
            assert!((original - rotated).abs() < 1e-10);
        }

        let original_area = scalar(metric(&morphology, "convex-hull-3d-surface-area"));
        let original_volume = scalar(metric(&morphology, "convex-hull-3d-volume"));
        let original_density = scalar(metric(&morphology, "volume-density"));
        let scaled = morphology
            .uniform_scale_with_report(2.0, Vec3::new(1.0, -3.0, 5.0))
            .unwrap()
            .morphology;
        assert!(
            (scalar(metric(&scaled, "convex-hull-3d-surface-area")) - 4.0 * original_area).abs()
                < 1e-10
        );
        assert!(
            (scalar(metric(&scaled, "convex-hull-3d-volume")) - 8.0 * original_volume).abs()
                < 1e-10
        );
        assert!((scalar(metric(&scaled, "volume-density")) - original_density).abs() < 1e-10);
    }

    #[test]
    fn bifurcation_to_node_projection_requires_an_explicit_collision_reducer() {
        let morphology = strict("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 0 1 0 1 1\n4 3 0 0 1 1 1\n");
        let field = metric(&morphology, "local-bifurcation-angle");
        assert_eq!(
            morphology.field_to_nodes(
                &field,
                FieldToNodesOptions {
                    placement: FieldPlacement::BifurcationBranch,
                    reducer: FieldReducer::Error,
                },
            ),
            Err(FieldConversionError::Collision(1))
        );
        let reduced = morphology
            .field_to_nodes(
                &field,
                FieldToNodesOptions {
                    placement: FieldPlacement::BifurcationBranch,
                    reducer: FieldReducer::Mean,
                },
            )
            .unwrap();
        let MetricData::NodeField(reduced) = reduced.data else {
            panic!()
        };
        assert_eq!(reduced.node_ids, vec![1]);
        assert!((reduced.values[0] - 90.0).abs() < 1e-12);
    }

    #[test]
    fn metric_parameters_are_discoverable_and_irrelevant_parameters_are_errors() {
        let registry = metric_registry();
        let rall = registry
            .iter()
            .find(|definition| definition.id == "rall-ratio")
            .unwrap();
        assert_eq!(rall.parameters.len(), 2);
        let p = rall
            .parameters
            .iter()
            .find(|parameter| parameter.name == "p")
            .unwrap();
        assert_eq!(p.default, Some(ParameterValue::Number(1.5)));
        assert_eq!(p.minimum, Some(0.0));
        assert!(p.exclusive_minimum);

        let morphology = strict("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n");
        let error = morphology
            .measure(&MeasureOptions {
                metrics: vec![MetricSpec {
                    id: "centroid".to_owned(),
                    parameters: MetricParameters {
                        p: Some(1.5),
                        ..Default::default()
                    },
                }],
                selection: SelectionQuery {
                    domain: AnalysisDomain::Raw,
                    ..Default::default()
                },
                section_boundaries: SectionBoundaryPolicy::TopologyOnly,
            })
            .unwrap_err();
        assert!(matches!(error, MetricError::InvalidParameter { .. }));

        let unknown = morphology
            .measure(&MeasureOptions {
                metrics: vec![MetricSpec {
                    id: "not-a-metric".to_owned(),
                    parameters: MetricParameters {
                        p: Some(1.5),
                        ..Default::default()
                    },
                }],
                selection: SelectionQuery {
                    domain: AnalysisDomain::Raw,
                    ..Default::default()
                },
                section_boundaries: SectionBoundaryPolicy::TopologyOnly,
            })
            .unwrap_err();
        assert!(matches!(unknown, MetricError::UnknownMetric(_)));
    }

    #[test]
    fn unified_basic_metrics_match_selected_topology_and_geometry() {
        let morphology = strict("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 1 1 0 1 2\n4 3 2 0 0 1 2\n");
        assert_eq!(scalar(metric(&morphology, "node-count")), 4.0);
        assert_eq!(scalar(metric(&morphology, "branch-point-count")), 1.0);
        assert_eq!(scalar(metric(&morphology, "terminal-count")), 2.0);
        assert_eq!(scalar(metric(&morphology, "section-count")), 3.0);
        assert!((scalar(metric(&morphology, "total-cable-length")) - 3.0).abs() < 1e-12);
        assert!((scalar(metric(&morphology, "maximum-root-path-length")) - 2.0).abs() < 1e-12);

        let MetricData::NodeField(paths) = metric(&morphology, "root-path-length").data else {
            panic!()
        };
        assert_eq!(paths.node_ids, vec![1, 2, 3, 4]);
        assert_eq!(paths.values, vec![0.0, 1.0, 2.0, 2.0]);

        let MetricData::NodeField(branch) = metric(&morphology, "branch-order").data else {
            panic!()
        };
        assert_eq!(branch.values, vec![1.0, 1.0, 2.0, 2.0]);
        let MetricData::NodeField(strahler) = metric(&morphology, "strahler-order").data else {
            panic!()
        };
        assert_eq!(strahler.values, vec![2.0, 2.0, 1.0, 1.0]);
    }

    #[test]
    fn section_length_and_contraction_are_explicit_fields() {
        let morphology = strict("1 3 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 1 1 0 1 2\n");
        let MetricData::SectionField(length) = metric(&morphology, "section-length").data else {
            panic!()
        };
        assert_eq!(length.values, vec![2.0]);
        let MetricData::SectionField(contraction) = metric(&morphology, "section-contraction").data
        else {
            panic!()
        };
        assert!((contraction.values[0] - 2.0_f64.sqrt() / 2.0).abs() < 1e-12);
    }
}
