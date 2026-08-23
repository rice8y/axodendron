//! Versioned CBOR boundary between Typst and Axodendron's pure Rust core.

use std::io::Cursor;

use axodendron_core::{
    Affine3, AffineRadiusPolicy, AnalysisDomain, AnalysisOptions, Diagnostic, FeatureTable,
    FeatureTableOptions, FieldToNodesOptions, MAX_NODE_COUNT, MeasureOptions, MetricResult,
    Morphology, NodeId, NodeSelection, ParseResult, PopulationMorphology, PrincipalFrame,
    PrincipalFrameOptions, PrincipalWeighting, Projection, ResampleOptions, SelectionQuery,
    Selector, Severity, SimplifyOptions, SomaClass, SwcMetadata, TmdError, TmdOptions,
    TransformError, TransformResult, ValidationProfile, Vec3, feature_table as build_feature_table,
    feature_table_csv as table_csv, metric_registry, parse_swc,
};
use axodendron_svg::{
    MAX_SVG_BYTES, RenderError, RenderOptions, SvgDocument, TreeRenderOptions, TreeSvgDocument,
    render_svg, render_tree_svg,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::wasm_func;

#[cfg(target_arch = "wasm32")]
wasm_minimal_protocol::initiate_protocol!();

pub const PROTOCOL_VERSION: u16 = 2;
pub const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PAYLOAD_SCHEMA_VERSION: u16 = 2;
const MAX_SOURCE_BYTES: usize = 64 * 1024 * 1024;
const MAX_REQUEST_BYTES: usize = 64 * 1024 * 1024;
const MAX_PAYLOAD_BYTES: usize = 128 * 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 128 * 1024 * 1024;
const MAX_SHOLL_RADII: usize = 10_000;
const MAX_KIND_SELECTIONS: usize = 4096;
const MAX_METRICS: usize = 256;
const MAX_POPULATION: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VersionedRequest<T> {
    pub protocol_version: u16,
    pub value: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WireResponse<T> {
    pub protocol_version: u16,
    pub package_version: String,
    pub ok: bool,
    pub value: Option<T>,
    pub error: Option<ApiError>,
}

impl<T> WireResponse<T> {
    fn success(value: T) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            package_version: PACKAGE_VERSION.to_owned(),
            ok: true,
            value: Some(value),
            error: None,
        }
    }

    fn failure(code: &str, message: impl Into<String>) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            package_version: PACKAGE_VERSION.to_owned(),
            ok: false,
            value: None,
            error: Some(ApiError {
                code: code.to_owned(),
                message: message.into(),
            }),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MorphologyPayload {
    pub schema_version: u16,
    pub morphology: Morphology,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParseOptions {
    #[serde(default)]
    pub profile: ValidationProfile,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            profile: ValidationProfile::Permissive,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParseOutput {
    pub valid: bool,
    pub payload: Option<MorphologyPayload>,
    pub diagnostics: Vec<Diagnostic>,
    pub fingerprint: Option<String>,
    pub source_fingerprint: Option<String>,
    pub node_count: u32,
    pub units: String,
    pub metadata: SwcMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShollRequest {
    #[serde(deserialize_with = "axodendron_core::serde_number::vec_f64")]
    pub radii: Vec<f64>,
    #[serde(default)]
    pub center: Option<Vec3>,
    #[serde(default)]
    pub center_node: Option<i64>,
    #[serde(default)]
    pub domain: AnalysisDomain,
    #[serde(default)]
    pub projection: Option<ShollProjection>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum ShollProjection {
    Xy,
    Xz,
    Yz,
    Orthographic { direction: Vec3, up: Vec3 },
}

impl ShollProjection {
    fn projection(&self) -> Result<Projection, String> {
        match self {
            Self::Xy => Ok(Projection::xy()),
            Self::Xz => Ok(Projection::xz()),
            Self::Yz => Ok(Projection::yz()),
            Self::Orthographic { direction, up } => {
                Projection::look(*direction, *up).map_err(|error| error.to_string())
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub enum TransformRequest {
    SelectNodes {
        node_ids: Vec<i64>,
    },
    SelectKinds {
        kinds: Vec<i32>,
    },
    Subtree {
        node_id: i64,
    },
    Path {
        from_id: i64,
        to_id: i64,
    },
    Reroot {
        node_id: i64,
    },
    DropKinds {
        kinds: Vec<i32>,
    },
    Simplify {
        options: SimplifyOptions,
    },
    Resample {
        options: ResampleOptions,
    },
    Translate {
        offset: Vec3,
    },
    Rotate {
        axis: Vec3,
        #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
        angle_radians: f64,
        center: Vec3,
    },
    UniformScale {
        #[serde(deserialize_with = "axodendron_core::serde_number::f64")]
        factor: f64,
        center: Vec3,
    },
    Reflect {
        normal: Vec3,
        point: Vec3,
    },
    PrincipalAlign {
        frame: Box<PrincipalFrame>,
        #[serde(default)]
        allow_degenerate: bool,
    },
    GeneralAffine {
        affine: Affine3,
        radius_policy: AffineRadiusPolicy,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryNodesRequest {
    #[serde(default)]
    pub query: SelectionQuery,
    pub selector: Selector,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldToNodesRequest {
    pub field: MetricResult,
    pub options: FieldToNodesOptions,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PopulationWireEntry {
    pub id: String,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureTableRequest {
    pub population: Vec<PopulationWireEntry>,
    pub options: FeatureTableOptions,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CenterKind {
    Soma,
    Centroid,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CenterPointRequest {
    pub center: CenterKind,
    #[serde(default)]
    pub selection: SelectionQuery,
    #[serde(default)]
    pub weighting: PrincipalWeighting,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransformOutput {
    pub payload: MorphologyPayload,
    pub mapping: Vec<axodendron_core::NodeMapping>,
    pub lineage: Vec<axodendron_core::NodeLineage>,
    pub report: axodendron_core::TransformReport,
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn version() -> Vec<u8> {
    PACKAGE_VERSION.as_bytes().to_vec()
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn parse(source: &[u8], request: &[u8]) -> Vec<u8> {
    let request = match decode_request::<ParseOptions>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    if source.len() > MAX_SOURCE_BYTES {
        return encode_response(WireResponse::<ParseOutput>::failure(
            "LIMIT_SOURCE_BYTES",
            format!("SWC source exceeds the {MAX_SOURCE_BYTES}-byte limit"),
        ));
    }
    let source = match std::str::from_utf8(source) {
        Ok(source) => source,
        Err(error) => {
            return encode_response(WireResponse::<ParseOutput>::failure(
                "SWC_INVALID_UTF8",
                error.to_string(),
            ));
        }
    };
    let ParseResult {
        morphology,
        diagnostics,
    } = parse_swc(source, request.profile);
    let valid = morphology.is_some()
        && !diagnostics
            .iter()
            .any(|item| item.severity == Severity::Error);
    let fingerprint = morphology
        .as_ref()
        .map(|value| value.fingerprint().to_owned());
    let source_fingerprint = morphology
        .as_ref()
        .and_then(|value| value.source_fingerprint().map(str::to_owned));
    let node_count = morphology.as_ref().map_or(0, |value| value.len() as u32);
    let units = morphology
        .as_ref()
        .map_or_else(|| "um".to_owned(), |value| value.units().to_owned());
    let metadata = morphology
        .as_ref()
        .map_or_else(SwcMetadata::default, |value| value.metadata().clone());
    let payload = morphology.map(|morphology| MorphologyPayload {
        schema_version: PAYLOAD_SCHEMA_VERSION,
        morphology,
    });
    encode_response(WireResponse::success(ParseOutput {
        valid,
        payload,
        diagnostics,
        fingerprint,
        source_fingerprint,
        node_count,
        units,
        metadata,
    }))
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn analyze(payload: &[u8]) -> Vec<u8> {
    with_payload(payload, |morphology| morphology.analyze())
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn analyze_with(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let options = match decode_request::<AnalysisOptions>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    with_payload(payload, |morphology| {
        morphology.analyze_with_options(options)
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn available_metrics() -> Vec<u8> {
    encode_response(WireResponse::success(metric_registry()))
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn measure(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let options = match decode_request::<MeasureOptions>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    if options.metrics.is_empty() || options.metrics.len() > MAX_METRICS {
        return encode_response(WireResponse::<Vec<MetricResult>>::failure(
            "MEASURE_INVALID_METRIC_COUNT",
            format!("measure requires between 1 and {MAX_METRICS} metrics"),
        ));
    }
    let payload = match decode_payload(payload) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match payload.morphology.measure(&options) {
        Ok(value) => encode_response(WireResponse::success(value)),
        Err(error) => encode_response(WireResponse::<Vec<MetricResult>>::failure(
            metric_error_code(&error),
            error.to_string(),
        )),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn principal_frame(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let options = match decode_request::<PrincipalFrameOptions>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let payload = match decode_payload(payload) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match payload.morphology.principal_frame(&options) {
        Ok(value) => encode_response(WireResponse::success(value)),
        Err(error) => encode_response(WireResponse::<PrincipalFrame>::failure(
            "PRINCIPAL_FRAME_INVALID",
            error.to_string(),
        )),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn center_point(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let request = match decode_request::<CenterPointRequest>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let payload = match decode_payload(payload) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let morphology = &payload.morphology;
    let value = match request.center {
        CenterKind::Soma => {
            if matches!(
                morphology.soma_class(),
                SomaClass::Absent | SomaClass::Disconnected | SomaClass::Ambiguous
            ) {
                return encode_response(WireResponse::<Vec3>::failure(
                    "CENTER_AMBIGUOUS_SOMA",
                    "soma centering requires unambiguous soma geometry",
                ));
            }
            morphology.soma_center()
        }
        CenterKind::Centroid if request.weighting == PrincipalWeighting::Nodes => {
            let selection = match morphology.query_nodes(&request.selection, Selector::All) {
                Ok(value) => value,
                Err(error) => {
                    return encode_response(WireResponse::<Vec3>::failure(
                        "CENTER_INVALID_SELECTION",
                        error.to_string(),
                    ));
                }
            };
            let sum = selection.node_ids.iter().fold(Vec3::default(), |sum, id| {
                sum + morphology.position(morphology.index_of(NodeId(*id)).unwrap())
            });
            sum * (1.0 / selection.node_ids.len() as f64)
        }
        CenterKind::Centroid => match morphology.principal_frame(&PrincipalFrameOptions {
            selection: request.selection,
            weighting: request.weighting,
            ..Default::default()
        }) {
            Ok(frame) => frame.centroid,
            Err(error) => {
                return encode_response(WireResponse::<Vec3>::failure(
                    "CENTER_INVALID_GEOMETRY",
                    error.to_string(),
                ));
            }
        },
    };
    encode_response(WireResponse::success(value))
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn query_nodes(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let request = match decode_request::<QueryNodesRequest>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let payload = match decode_payload(payload) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match payload
        .morphology
        .query_nodes(&request.query, request.selector)
    {
        Ok(value) => encode_response(WireResponse::success(value)),
        Err(error) => encode_response(WireResponse::<NodeSelection>::failure(
            "QUERY_INVALID",
            error.to_string(),
        )),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn field_to_nodes(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let request = match decode_request::<FieldToNodesRequest>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let payload = match decode_payload(payload) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match payload
        .morphology
        .field_to_nodes(&request.field, request.options)
    {
        Ok(value) => encode_response(WireResponse::success(value)),
        Err(error) => encode_response(WireResponse::<MetricResult>::failure(
            "FIELD_TO_NODES_INVALID",
            error.to_string(),
        )),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn tmd(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let options = match decode_request::<TmdOptions>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let payload = match decode_payload(payload) {
        Ok(value) => value,
        Err(response) => return response,
    };
    match payload.morphology.tmd(&options) {
        Ok(value) => encode_response(WireResponse::success(value)),
        Err(error) => encode_response(WireResponse::<axodendron_core::TmdResult>::failure(
            tmd_error_code(&error),
            error.to_string(),
        )),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn feature_table(request: &[u8]) -> Vec<u8> {
    let request = match decode_request::<FeatureTableRequest>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    if request.population.is_empty() || request.population.len() > MAX_POPULATION {
        return encode_response(WireResponse::<FeatureTable>::failure(
            "POPULATION_INVALID_SIZE",
            format!("population requires between 1 and {MAX_POPULATION} morphologies"),
        ));
    }
    if request.options.columns.is_empty() || request.options.columns.len() > MAX_METRICS {
        return encode_response(WireResponse::<FeatureTable>::failure(
            "POPULATION_INVALID_COLUMN_COUNT",
            format!("feature table requires between 1 and {MAX_METRICS} columns"),
        ));
    }
    let mut population = Vec::with_capacity(request.population.len());
    for item in request.population {
        let payload = match decode_payload(&item.payload) {
            Ok(value) => value,
            Err(response) => return response,
        };
        population.push(PopulationMorphology {
            id: item.id,
            morphology: payload.morphology,
        });
    }
    match build_feature_table(&population, &request.options) {
        Ok(value) => encode_response(WireResponse::success(value)),
        Err(error) => encode_response(WireResponse::<FeatureTable>::failure(
            "POPULATION_INVALID",
            error.to_string(),
        )),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn feature_table_csv(request: &[u8]) -> Vec<u8> {
    let table = match decode_request::<FeatureTable>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    encode_response(WireResponse::success(table_csv(&table)))
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn sholl(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let request = match decode_request::<ShollRequest>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let payload = match decode_payload(payload) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    if request.center.is_some() && request.center_node.is_some() {
        return encode_response(WireResponse::<axodendron_core::ShollResult>::failure(
            "SHOLL_AMBIGUOUS_CENTER",
            "provide either center or center-node, not both",
        ));
    }
    if request
        .radii
        .iter()
        .any(|radius| !radius.is_finite() || *radius < 0.0)
    {
        return encode_response(WireResponse::<axodendron_core::ShollResult>::failure(
            "SHOLL_INVALID_RADIUS",
            "Sholl radii must be finite and non-negative",
        ));
    }
    if request.radii.len() > MAX_SHOLL_RADII {
        return encode_response(WireResponse::<axodendron_core::ShollResult>::failure(
            "LIMIT_SHOLL_RADII",
            format!("Sholl request exceeds the {MAX_SHOLL_RADII}-radius limit"),
        ));
    }
    if request.center.is_some_and(|center| {
        !center.x.is_finite() || !center.y.is_finite() || !center.z.is_finite()
    }) {
        return encode_response(WireResponse::<axodendron_core::ShollResult>::failure(
            "SHOLL_INVALID_CENTER",
            "Sholl center coordinates must be finite",
        ));
    }
    let center = if let Some(node_id) = request.center_node {
        match payload.morphology.index_of(NodeId(node_id)) {
            Some(node) => payload.morphology.position(node),
            None => {
                return encode_response(WireResponse::<axodendron_core::ShollResult>::failure(
                    "SHOLL_UNKNOWN_CENTER_NODE",
                    format!("node id {node_id} does not exist"),
                ));
            }
        }
    } else if let Some(center) = request.center {
        center
    } else {
        let ambiguous = payload.morphology.soma_class() == SomaClass::Disconnected
            || (payload.morphology.soma_class() == SomaClass::Absent
                && payload.morphology.roots().len() != 1);
        if ambiguous {
            return encode_response(WireResponse::<axodendron_core::ShollResult>::failure(
                "SHOLL_AMBIGUOUS_DEFAULT_CENTER",
                "morphology has no unique default Sholl center; provide center or center-node",
            ));
        }
        payload.morphology.soma_center()
    };
    let result = match request.projection {
        Some(projection) => match projection.projection() {
            Ok(projection) => {
                payload
                    .morphology
                    .sholl_2d(center, &request.radii, projection, request.domain)
            }
            Err(message) => {
                return encode_response(WireResponse::<axodendron_core::ShollResult>::failure(
                    "SHOLL_INVALID_PROJECTION",
                    message,
                ));
            }
        },
        None => payload
            .morphology
            .sholl_3d_in_domain(center, &request.radii, request.domain),
    };
    encode_response(WireResponse::success(result))
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn transform(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let request = match decode_request::<TransformRequest>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let limit_error = match &request {
        TransformRequest::SelectNodes { node_ids } if node_ids.len() > MAX_NODE_COUNT => Some((
            "LIMIT_SELECTION_NODES",
            format!("node selection exceeds the {MAX_NODE_COUNT}-entry limit"),
        )),
        TransformRequest::SelectKinds { kinds } | TransformRequest::DropKinds { kinds }
            if kinds.len() > MAX_KIND_SELECTIONS =>
        {
            Some((
                "LIMIT_KIND_SELECTIONS",
                format!("kind selection exceeds the {MAX_KIND_SELECTIONS}-entry limit"),
            ))
        }
        TransformRequest::Simplify { options } if options.protected_ids.len() > MAX_NODE_COUNT => {
            Some((
                "LIMIT_PROTECTED_NODES",
                format!("protected-node list exceeds the {MAX_NODE_COUNT}-entry limit"),
            ))
        }
        TransformRequest::Resample { options } if options.protected_ids.len() > MAX_NODE_COUNT => {
            Some((
                "LIMIT_PROTECTED_NODES",
                format!("protected-node list exceeds the {MAX_NODE_COUNT}-entry limit"),
            ))
        }
        _ => None,
    };
    if let Some((code, message)) = limit_error {
        return encode_response(WireResponse::<TransformOutput>::failure(code, message));
    }
    let payload = match decode_payload(payload) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    let transformed: Result<TransformResult, TransformError> = match request {
        TransformRequest::SelectNodes { node_ids } => {
            payload.morphology.select_nodes_with_report(&node_ids)
        }
        TransformRequest::SelectKinds { kinds } => {
            payload.morphology.select_kinds_with_report(&kinds)
        }
        TransformRequest::Subtree { node_id } => {
            payload.morphology.subtree_with_report(NodeId(node_id))
        }
        TransformRequest::Path { from_id, to_id } => payload
            .morphology
            .path_between_with_report(NodeId(from_id), NodeId(to_id)),
        TransformRequest::Reroot { node_id } => {
            payload.morphology.reroot_with_report(NodeId(node_id))
        }
        TransformRequest::DropKinds { kinds } => payload.morphology.drop_kinds_with_report(&kinds),
        TransformRequest::Simplify { options } => payload.morphology.simplify_with_report(&options),
        TransformRequest::Resample { options } => payload.morphology.resample_with_report(&options),
        TransformRequest::Translate { offset } => payload.morphology.translate_with_report(offset),
        TransformRequest::Rotate {
            axis,
            angle_radians,
            center,
        } => payload
            .morphology
            .rotate_with_report(axis, angle_radians, center),
        TransformRequest::UniformScale { factor, center } => {
            payload.morphology.uniform_scale_with_report(factor, center)
        }
        TransformRequest::Reflect { normal, point } => {
            payload.morphology.reflect_with_report(normal, point)
        }
        TransformRequest::PrincipalAlign {
            frame,
            allow_degenerate,
        } => payload
            .morphology
            .align_to_principal_frame_with_report(&frame, allow_degenerate),
        TransformRequest::GeneralAffine {
            affine,
            radius_policy,
        } => payload.morphology.affine_with_report(affine, radius_policy),
    };
    match transformed {
        Ok(result) => encode_response(WireResponse::success(TransformOutput {
            payload: MorphologyPayload {
                schema_version: payload.schema_version,
                morphology: result.morphology,
            },
            mapping: result.mapping,
            lineage: result.lineage,
            report: result.report,
        })),
        Err(error) => {
            let code = match error {
                TransformError::UnknownNode(_) => "TRANSFORM_UNKNOWN_NODE",
                TransformError::DifferentComponents(_, _) => "TRANSFORM_DIFFERENT_COMPONENTS",
                TransformError::EmptyResult => "TRANSFORM_EMPTY_RESULT",
                TransformError::InvalidTolerance => "TRANSFORM_INVALID_TOLERANCE",
                TransformError::InvalidStep => "TRANSFORM_INVALID_STEP",
                TransformError::IdSpaceExhausted => "TRANSFORM_ID_SPACE_EXHAUSTED",
                TransformError::NodeLimitExceeded => "LIMIT_NODE_COUNT",
                TransformError::InvalidGeometryTransform => "TRANSFORM_INVALID_GEOMETRY",
                TransformError::InvalidRotationAxis => "TRANSFORM_INVALID_AXIS",
                TransformError::InvalidScale => "TRANSFORM_INVALID_SCALE",
                TransformError::NonFiniteResult => "TRANSFORM_NONFINITE_RESULT",
                TransformError::DegeneratePrincipalFrame => "TRANSFORM_DEGENERATE_FRAME",
            };
            encode_response(WireResponse::<TransformOutput>::failure(
                code,
                error.to_string(),
            ))
        }
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn export_swc(payload: &[u8]) -> Vec<u8> {
    with_payload(payload, Morphology::to_canonical_swc)
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn render(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let options = match decode_request::<RenderOptions>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let payload = match decode_payload(payload) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    match render_svg(&payload.morphology, &options) {
        Ok(document) if document.svg.len() <= MAX_SVG_BYTES => {
            encode_response(WireResponse::success(document))
        }
        Ok(_) => encode_response(WireResponse::<SvgDocument>::failure(
            "LIMIT_SVG_BYTES",
            format!("SVG output exceeds the {MAX_SVG_BYTES}-byte limit"),
        )),
        Err(error) => encode_response(WireResponse::<SvgDocument>::failure(
            render_error_code(&error),
            error.to_string(),
        )),
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn render_tree(payload: &[u8], request: &[u8]) -> Vec<u8> {
    let options = match decode_request::<TreeRenderOptions>(request) {
        Ok(value) => value,
        Err(response) => return response,
    };
    let payload = match decode_payload(payload) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    match render_tree_svg(&payload.morphology, &options) {
        Ok(document) if document.svg.len() <= MAX_SVG_BYTES => {
            encode_response(WireResponse::success(document))
        }
        Ok(_) => encode_response(WireResponse::<TreeSvgDocument>::failure(
            "LIMIT_SVG_BYTES",
            format!("SVG output exceeds the {MAX_SVG_BYTES}-byte limit"),
        )),
        Err(error) => encode_response(WireResponse::<TreeSvgDocument>::failure(
            render_error_code(&error),
            error.to_string(),
        )),
    }
}

fn with_payload<T: Serialize>(payload: &[u8], operation: impl FnOnce(&Morphology) -> T) -> Vec<u8> {
    let payload = match decode_payload(payload) {
        Ok(payload) => payload,
        Err(response) => return response,
    };
    encode_response(WireResponse::success(operation(&payload.morphology)))
}

fn decode_request<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Vec<u8>> {
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(encode_response(WireResponse::<()>::failure(
            "LIMIT_REQUEST_BYTES",
            format!("request exceeds the {MAX_REQUEST_BYTES}-byte limit"),
        )));
    }
    let request: VersionedRequest<T> = decode(bytes).map_err(|error| {
        encode_response(WireResponse::<()>::failure(
            "PROTOCOL_DECODE_REQUEST",
            error.to_string(),
        ))
    })?;
    if request.protocol_version != PROTOCOL_VERSION {
        return Err(encode_response(WireResponse::<()>::failure(
            "PROTOCOL_VERSION_MISMATCH",
            format!(
                "expected protocol version {PROTOCOL_VERSION}, found {}",
                request.protocol_version
            ),
        )));
    }
    Ok(request.value)
}

fn decode_payload(bytes: &[u8]) -> Result<MorphologyPayload, Vec<u8>> {
    if bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(encode_response(WireResponse::<MorphologyPayload>::failure(
            "LIMIT_PAYLOAD_BYTES",
            format!("morphology payload exceeds the {MAX_PAYLOAD_BYTES}-byte limit"),
        )));
    }
    let payload: MorphologyPayload = decode(bytes).map_err(|error| {
        encode_response(WireResponse::<MorphologyPayload>::failure(
            "PROTOCOL_DECODE_PAYLOAD",
            error.to_string(),
        ))
    })?;
    if payload.schema_version != PAYLOAD_SCHEMA_VERSION {
        return Err(encode_response(WireResponse::<MorphologyPayload>::failure(
            "PAYLOAD_VERSION_MISMATCH",
            format!(
                "expected payload schema {PAYLOAD_SCHEMA_VERSION}, found {}",
                payload.schema_version
            ),
        )));
    }
    if payload.morphology.len() > MAX_NODE_COUNT {
        return Err(encode_response(WireResponse::<MorphologyPayload>::failure(
            "LIMIT_NODE_COUNT",
            format!("morphology exceeds the {MAX_NODE_COUNT}-node limit"),
        )));
    }
    Ok(payload)
}

fn render_error_code(error: &RenderError) -> &'static str {
    match error {
        RenderError::InvalidCanvas => "RENDER_INVALID_CANVAS",
        RenderError::InvalidDisplayTolerance => "RENDER_INVALID_DISPLAY_TOLERANCE",
        RenderError::Projection(_) => "RENDER_INVALID_PROJECTION",
        RenderError::ScalarLengthMismatch => "RENDER_SCALAR_LENGTH_MISMATCH",
        RenderError::InvalidScalarRange => "RENDER_INVALID_SCALAR_RANGE",
        RenderError::DuplicateScalarNode(_) => "RENDER_DUPLICATE_SCALAR_NODE",
        RenderError::ScalarFingerprintMismatch => "RENDER_SCALAR_FINGERPRINT_MISMATCH",
        RenderError::ScalarFieldTooLarge => "RENDER_SCALAR_FIELD_TOO_LARGE",
        RenderError::UnknownScalarNode(_) => "RENDER_UNKNOWN_SCALAR_NODE",
        RenderError::UnknownOverlayNode(_) => "RENDER_UNKNOWN_OVERLAY_NODE",
        RenderError::OverlayListTooLarge => "LIMIT_OVERLAY_NODES",
        RenderError::UnknownColormap(_) => "RENDER_UNKNOWN_COLORMAP",
        RenderError::InvalidStyle => "RENDER_INVALID_STYLE",
        RenderError::OutputTooLarge => "LIMIT_SVG_BYTES",
        RenderError::EmptyMorphology => "RENDER_EMPTY_MORPHOLOGY",
        RenderError::InvalidSelection(_) => "RENDER_INVALID_SELECTION",
    }
}

fn metric_error_code(error: &axodendron_core::MetricError) -> &'static str {
    match error {
        axodendron_core::MetricError::Query(_) => "MEASURE_INVALID_SELECTION",
        axodendron_core::MetricError::UnknownMetric(_) => "MEASURE_UNKNOWN_METRIC",
        axodendron_core::MetricError::InvalidParameter { .. } => "MEASURE_INVALID_PARAMETER",
        axodendron_core::MetricError::PrincipalFrame(_) => "MEASURE_PRINCIPAL_FRAME",
    }
}

fn tmd_error_code(error: &TmdError) -> &'static str {
    match error {
        TmdError::Query(_) => "TMD_INVALID_SELECTION",
        TmdError::NoSoma => "TMD_SOMA_MISSING",
        TmdError::AmbiguousSoma => "TMD_SOMA_AMBIGUOUS",
        TmdError::CenterNotApplicable => "TMD_CENTER_NOT_APPLICABLE",
    }
}

fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let mut cursor = Cursor::new(bytes);
    let value = ciborium::from_reader(&mut cursor).map_err(|error| error.to_string())?;
    if cursor.position() != bytes.len() as u64 {
        return Err("trailing bytes after the CBOR value".to_owned());
    }
    Ok(value)
}

fn encode_response<T: Serialize>(response: WireResponse<T>) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::into_writer(&response, &mut bytes)
        .expect("serializing a response into memory cannot fail");
    if bytes.len() <= MAX_RESPONSE_BYTES {
        return bytes;
    }
    let mut fallback = Vec::new();
    ciborium::into_writer(
        &WireResponse::<()>::failure(
            "LIMIT_RESPONSE_BYTES",
            format!("response exceeds the {MAX_RESPONSE_BYTES}-byte limit"),
        ),
        &mut fallback,
    )
    .expect("serializing a bounded error response cannot fail");
    fallback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Serialize)]
    struct ForgedPayload {
        schema_version: u16,
        morphology: ForgedMorphology,
    }

    #[derive(Serialize)]
    struct LegacyVersionedRequest<T> {
        api_version: u16,
        value: T,
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    #[derive(Clone, Serialize)]
    struct ForgedMorphology {
        ids: Vec<i64>,
        kinds: Vec<i32>,
        positions: Vec<[f64; 3]>,
        radii: Vec<f64>,
        parents: Vec<u32>,
        source_lines: Vec<u32>,
        units: String,
        fingerprint: String,
        source_fingerprint: Option<String>,
    }

    fn request<T: Serialize>(value: T) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::into_writer(
            &VersionedRequest {
                protocol_version: PROTOCOL_VERSION,
                value,
            },
            &mut bytes,
        )
        .unwrap();
        bytes
    }

    fn parsed_payload() -> Vec<u8> {
        let response = parse(
            b"1 1 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 1 0 1 2\n",
            &request(ParseOptions::default()),
        );
        let response: WireResponse<ParseOutput> =
            ciborium::from_reader(response.as_slice()).unwrap();
        let mut payload = Vec::new();
        ciborium::into_writer(&response.value.unwrap().payload.unwrap(), &mut payload).unwrap();
        payload
    }

    fn forged_error(morphology: ForgedMorphology) -> ApiError {
        let mut bytes = Vec::new();
        ciborium::into_writer(
            &ForgedPayload {
                schema_version: PAYLOAD_SCHEMA_VERSION,
                morphology,
            },
            &mut bytes,
        )
        .unwrap();
        let response: WireResponse<axodendron_core::AnalysisBundle> =
            ciborium::from_reader(analyze(&bytes).as_slice()).unwrap();
        assert!(!response.ok);
        response.error.unwrap()
    }

    fn valid_shape_with_wrong_fingerprint() -> ForgedMorphology {
        ForgedMorphology {
            ids: vec![1, 2],
            kinds: vec![1, 3],
            positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            radii: vec![1.0, 1.0],
            parents: vec![u32::MAX, 0],
            source_lines: vec![1, 2],
            units: "um".to_owned(),
            fingerprint: "fnv1a64:0000000000000000".to_owned(),
            source_fingerprint: None,
        }
    }

    #[test]
    fn protocol_parse_analyze_render_roundtrip() {
        let payload = parsed_payload();
        let analysis: WireResponse<axodendron_core::AnalysisBundle> =
            ciborium::from_reader(analyze(&payload).as_slice()).unwrap();
        assert_eq!(analysis.value.unwrap().summary.node_count, 2);

        let raw: WireResponse<axodendron_core::AnalysisBundle> = ciborium::from_reader(
            analyze_with(
                &payload,
                &request(AnalysisOptions {
                    domain: AnalysisDomain::Raw,
                    ..AnalysisOptions::default()
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(raw.value.unwrap().summary.node_count, 3);

        let render_options = RenderOptions::default();
        let document: WireResponse<SvgDocument> =
            ciborium::from_reader(render(&payload, &request(render_options)).as_slice()).unwrap();
        assert!(document.value.unwrap().svg.contains("<svg"));

        let metrics: WireResponse<Vec<MetricResult>> = ciborium::from_reader(
            measure(
                &payload,
                &request(MeasureOptions {
                    metrics: vec![axodendron_core::MetricSpec {
                        id: "centroid".to_owned(),
                        parameters: axodendron_core::MetricParameters::default(),
                    }],
                    selection: SelectionQuery {
                        domain: AnalysisDomain::Raw,
                        ..Default::default()
                    },
                    section_boundaries: axodendron_core::SectionBoundaryPolicy::TopologyOnly,
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(metrics.value.unwrap()[0].metric.id, "centroid");

        let selection: WireResponse<NodeSelection> = ciborium::from_reader(
            query_nodes(
                &payload,
                &request(QueryNodesRequest {
                    query: SelectionQuery {
                        domain: AnalysisDomain::Raw,
                        ..Default::default()
                    },
                    selector: Selector::Terminals,
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(selection.value.unwrap().node_ids, vec![3]);

        let tree: WireResponse<TreeSvgDocument> = ciborium::from_reader(
            render_tree(&payload, &request(TreeRenderOptions::default())).as_slice(),
        )
        .unwrap();
        assert!(tree.value.unwrap().svg.contains("abstract topology layout"));
    }

    #[test]
    fn protocol_rejects_wrong_version() {
        let mut bytes = Vec::new();
        ciborium::into_writer(
            &VersionedRequest {
                protocol_version: 1,
                value: ParseOptions::default(),
            },
            &mut bytes,
        )
        .unwrap();
        let response: WireResponse<ParseOutput> =
            ciborium::from_reader(parse(b"", &bytes).as_slice()).unwrap();
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, "PROTOCOL_VERSION_MISMATCH");
    }

    #[test]
    fn protocol_v2_request_and_response_have_golden_cbor_shapes() {
        let request = request(ParseOptions::default());
        let response = encode_response(WireResponse::<()>::failure("GOLDEN", "fixed"));
        assert_eq!(
            hex(&request),
            "a27070726f746f636f6c5f76657273696f6e026576616c7565a16770726f66696c656a7065726d697373697665"
        );
        assert_eq!(
            hex(&response),
            "a57070726f746f636f6c5f76657273696f6e026f7061636b6167655f76657273696f6e65302e312e31626f6bf46576616c7565f6656572726f72a264636f646566474f4c44454e676d657373616765656669786564"
        );
    }

    #[test]
    fn protocol_rejects_the_legacy_api_version_field() {
        let mut bytes = Vec::new();
        ciborium::into_writer(
            &LegacyVersionedRequest {
                api_version: 1,
                value: ParseOptions::default(),
            },
            &mut bytes,
        )
        .unwrap();
        let response: WireResponse<ParseOutput> =
            ciborium::from_reader(parse(b"", &bytes).as_slice()).unwrap();
        assert_eq!(response.error.unwrap().code, "PROTOCOL_DECODE_REQUEST");
    }

    #[test]
    fn protocol_rejects_negative_sholl_radius() {
        let payload = parsed_payload();
        let response: WireResponse<axodendron_core::ShollResult> = ciborium::from_reader(
            sholl(
                &payload,
                &request(ShollRequest {
                    radii: vec![-1.0],
                    center: None,
                    center_node: None,
                    domain: AnalysisDomain::Neurites,
                    projection: None,
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, "SHOLL_INVALID_RADIUS");
    }

    #[test]
    fn protocol_rejects_a_center_for_root_path_tmd_with_a_stable_code() {
        let payload = parsed_payload();
        let response: WireResponse<axodendron_core::TmdResult> = ciborium::from_reader(
            tmd(
                &payload,
                &request(TmdOptions {
                    selection: SelectionQuery::default(),
                    filtration: axodendron_core::TmdFiltration::RootPathLength,
                    center: Some(axodendron_core::TmdCenter::Soma),
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, "TMD_CENTER_NOT_APPLICABLE");
    }

    #[test]
    fn protocol_rejects_ambiguous_or_non_finite_sholl_inputs() {
        let payload = parsed_payload();
        let response: WireResponse<axodendron_core::ShollResult> = ciborium::from_reader(
            sholl(
                &payload,
                &request(ShollRequest {
                    radii: vec![1.0],
                    center: Some(Vec3::new(0.0, 0.0, 0.0)),
                    center_node: Some(1),
                    domain: AnalysisDomain::Neurites,
                    projection: None,
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(response.error.unwrap().code, "SHOLL_AMBIGUOUS_CENTER");

        let response: WireResponse<axodendron_core::ShollResult> = ciborium::from_reader(
            sholl(
                &payload,
                &request(ShollRequest {
                    radii: vec![f64::NAN],
                    center: None,
                    center_node: None,
                    domain: AnalysisDomain::Neurites,
                    projection: None,
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(response.error.unwrap().code, "PROTOCOL_DECODE_REQUEST");

        let response: WireResponse<axodendron_core::ShollResult> = ciborium::from_reader(
            sholl(
                &payload,
                &request(ShollRequest {
                    radii: vec![1.0],
                    center: Some(Vec3::new(f64::INFINITY, 0.0, 0.0)),
                    center_node: None,
                    domain: AnalysisDomain::Neurites,
                    projection: None,
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(response.error.unwrap().code, "PROTOCOL_DECODE_REQUEST");

        let forest = parse(
            b"10 3 0 0 0 1 -1\n20 3 5 0 0 1 -1\n",
            &request(ParseOptions::default()),
        );
        let forest: WireResponse<ParseOutput> = ciborium::from_reader(forest.as_slice()).unwrap();
        let mut forest_payload = Vec::new();
        ciborium::into_writer(&forest.value.unwrap().payload.unwrap(), &mut forest_payload)
            .unwrap();
        let response: WireResponse<axodendron_core::ShollResult> = ciborium::from_reader(
            sholl(
                &forest_payload,
                &request(ShollRequest {
                    radii: vec![1.0],
                    center: None,
                    center_node: None,
                    domain: AnalysisDomain::Neurites,
                    projection: None,
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(
            response.error.unwrap().code,
            "SHOLL_AMBIGUOUS_DEFAULT_CENTER"
        );
    }

    #[test]
    fn protocol_rejects_malformed_requests_without_trapping() {
        let response: WireResponse<ParseOutput> =
            ciborium::from_reader(parse(b"", b"not cbor").as_slice()).unwrap();
        assert!(!response.ok);
        assert_eq!(response.error.unwrap().code, "PROTOCOL_DECODE_REQUEST");

        let response: WireResponse<ParseOutput> =
            ciborium::from_reader(parse(&[0xff], &request(ParseOptions::default())).as_slice())
                .unwrap();
        assert_eq!(response.error.unwrap().code, "SWC_INVALID_UTF8");

        let mut trailing_request = request(ParseOptions::default());
        trailing_request.push(0);
        let response: WireResponse<ParseOutput> =
            ciborium::from_reader(parse(b"", &trailing_request).as_slice()).unwrap();
        assert_eq!(response.error.unwrap().code, "PROTOCOL_DECODE_REQUEST");

        let mut trailing_payload = parsed_payload();
        trailing_payload.push(0);
        let response: WireResponse<axodendron_core::AnalysisBundle> =
            ciborium::from_reader(analyze(&trailing_payload).as_slice()).unwrap();
        assert_eq!(response.error.unwrap().code, "PROTOCOL_DECODE_PAYLOAD");
    }

    #[test]
    fn protocol_decoders_are_total_for_deterministic_arbitrary_bytes() {
        fn assert_cbor_response(bytes: Vec<u8>) {
            let _: ciborium::Value = ciborium::from_reader(bytes.as_slice())
                .expect("every plugin call must return a CBOR response");
        }

        let valid_parse_request = request(ParseOptions::default());
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        for case in 0..256_usize {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let length = (state as usize + case * 17) % 513;
            let mut bytes = Vec::with_capacity(length);
            for _ in 0..length {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                bytes.push(state as u8);
            }

            assert_cbor_response(parse(&bytes, &bytes));
            assert_cbor_response(analyze(&bytes));
            assert_cbor_response(analyze_with(&bytes, &bytes));
            assert_cbor_response(sholl(&bytes, &bytes));
            assert_cbor_response(measure(&bytes, &bytes));
            assert_cbor_response(principal_frame(&bytes, &bytes));
            assert_cbor_response(center_point(&bytes, &bytes));
            assert_cbor_response(query_nodes(&bytes, &bytes));
            assert_cbor_response(field_to_nodes(&bytes, &bytes));
            assert_cbor_response(tmd(&bytes, &bytes));
            assert_cbor_response(feature_table(&bytes));
            assert_cbor_response(feature_table_csv(&bytes));
            assert_cbor_response(transform(&bytes, &bytes));
            assert_cbor_response(render(&bytes, &bytes));
            assert_cbor_response(render_tree(&bytes, &bytes));
            assert_cbor_response(export_swc(&bytes));

            let ascii: Vec<u8> = bytes.iter().map(|byte| 32 + byte % 95).collect();
            assert_cbor_response(parse(&ascii, &valid_parse_request));
        }
    }

    #[test]
    fn protocol_rejects_unknown_payload_schema() {
        let payload: MorphologyPayload =
            ciborium::from_reader(parsed_payload().as_slice()).unwrap();
        let mut bytes = Vec::new();
        ciborium::into_writer(
            &MorphologyPayload {
                schema_version: 99,
                ..payload
            },
            &mut bytes,
        )
        .unwrap();
        let response: WireResponse<axodendron_core::AnalysisBundle> =
            ciborium::from_reader(analyze(&bytes).as_slice()).unwrap();
        assert_eq!(response.error.unwrap().code, "PAYLOAD_VERSION_MISMATCH");
    }

    #[test]
    fn protocol_rejects_forged_payload_indices_without_trapping() {
        let mut out_of_range = valid_shape_with_wrong_fingerprint();
        out_of_range.parents[1] = 99;
        let error = forged_error(out_of_range);
        assert_eq!(error.code, "PROTOCOL_DECODE_PAYLOAD");
        assert!(error.message.contains("out-of-range parent"));

        let mut wrong_length = valid_shape_with_wrong_fingerprint();
        wrong_length.kinds.pop();
        assert!(
            forged_error(wrong_length)
                .message
                .contains("kinds has length")
        );

        let mut duplicate = valid_shape_with_wrong_fingerprint();
        duplicate.ids[1] = 1;
        assert!(forged_error(duplicate).message.contains("duplicated"));

        let mut non_finite = valid_shape_with_wrong_fingerprint();
        non_finite.positions[1][0] = f64::NAN;
        assert!(
            forged_error(non_finite)
                .message
                .contains("non-finite position")
        );

        let mut self_parent = valid_shape_with_wrong_fingerprint();
        self_parent.parents[1] = 1;
        assert!(forged_error(self_parent).message.contains("its own parent"));

        let mut cycle = valid_shape_with_wrong_fingerprint();
        cycle.parents = vec![1, 0];
        assert!(forged_error(cycle).message.contains("cycle"));

        let mut empty_units = valid_shape_with_wrong_fingerprint();
        empty_units.units.clear();
        assert!(
            forged_error(empty_units)
                .message
                .contains("must not be empty")
        );

        let mut oversized_units = valid_shape_with_wrong_fingerprint();
        oversized_units.units = "u".repeat(65);
        assert!(
            forged_error(oversized_units)
                .message
                .contains("no longer than 64")
        );

        assert!(
            forged_error(valid_shape_with_wrong_fingerprint())
                .message
                .contains("fingerprint does not match")
        );
    }

    #[test]
    fn protocol_transform_returns_revalidated_canonical_payload() {
        let payload = parsed_payload();
        let response: WireResponse<TransformOutput> = ciborium::from_reader(
            transform(&payload, &request(TransformRequest::Reroot { node_id: 3 })).as_slice(),
        )
        .unwrap();
        let transformed = response.value.unwrap();
        assert_eq!(
            transformed
                .payload
                .morphology
                .roots()
                .next()
                .map(|node| transformed.payload.morphology.id(node)),
            Some(NodeId(3))
        );

        let mut roundtrip = Vec::new();
        ciborium::into_writer(&transformed.payload, &mut roundtrip).unwrap();
        let analysis: WireResponse<axodendron_core::AnalysisBundle> =
            ciborium::from_reader(analyze(&roundtrip).as_slice()).unwrap();
        assert!(analysis.ok);
    }

    #[test]
    fn protocol_transform_errors_have_stable_specific_codes() {
        let payload = parsed_payload();
        let response: WireResponse<TransformOutput> = ciborium::from_reader(
            transform(
                &payload,
                &request(TransformRequest::Subtree { node_id: 999 }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(response.error.unwrap().code, "TRANSFORM_UNKNOWN_NODE");

        let response: WireResponse<TransformOutput> = ciborium::from_reader(
            transform(
                &payload,
                &request(TransformRequest::Simplify {
                    options: SimplifyOptions {
                        tolerance: f64::INFINITY,
                        ..Default::default()
                    },
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(response.error.unwrap().code, "PROTOCOL_DECODE_REQUEST");

        let response: WireResponse<TransformOutput> = ciborium::from_reader(
            transform(
                &payload,
                &request(TransformRequest::SelectKinds {
                    kinds: vec![3; MAX_KIND_SELECTIONS + 1],
                }),
            )
            .as_slice(),
        )
        .unwrap();
        assert_eq!(response.error.unwrap().code, "LIMIT_KIND_SELECTIONS");
    }
}
