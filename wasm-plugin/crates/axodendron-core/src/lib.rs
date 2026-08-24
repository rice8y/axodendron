//! Deterministic, format-independent neuronal arbor algorithms for Axodendron.
//!
//! The core crate contains no Typst or WebAssembly-specific code. External node
//! identifiers are kept separate from compact internal indices, and every
//! traversal uses stable input order.

mod analysis;
mod diagnostic;
mod geometry;
mod metrics;
mod model;
mod population;
mod principal;
mod query;
#[doc(hidden)]
pub mod serde_number;
mod swc;
mod tmd;
mod transform;

pub use analysis::{
    AnalysisBundle, AnalysisDomain, AnalysisOptions, ArborMetrics, BBox, DEFINITION_VERSION,
    NodeField, RadiusMetrics, Section, SectionBoundaryPolicy, SectionDecomposition, ShollBin,
    ShollDimension, ShollResult, SomaMetrics, Summary, Topology, TortuositySummary, TypeCount,
    TypeMetrics,
};
pub use diagnostic::{Diagnostic, Severity, ValidationProfile};
pub use geometry::{Projection, ProjectionError, Vec2, Vec3};
pub use metrics::{
    BifurcationField, BifurcationKey, DiameterSampling, EntityKey, FieldConversionError,
    FieldPlacement, FieldReducer, FieldToNodesOptions, METRIC_RESULT_SCHEMA_VERSION,
    MeasureOptions, MetricData, MetricDefinition, MetricDescriptor, MetricError, MetricNodeField,
    MetricParameterDefinition, MetricParameters, MetricProvenance, MetricResult, MetricSource,
    MetricSpec, MetricValue, MissingReason, MissingValue, MorphologyMetric, MultifurcationPolicy,
    ParameterValue, SECTION_DEFINITION_VERSION, SectionField, SectionRef, SpatialPlane,
    TaperMethod, TaperQuantity, metric_registry,
};
pub use model::{
    ModelError, Morphology, NONE_NODE, NodeId, NodeIx, SOMA_GEOMETRY_RELATIVE_TOLERANCE, SomaClass,
    SwcMetadata,
};
pub use population::{
    FeatureCell, FeatureColumn, FeatureColumnSpec, FeatureComponent, FeatureRow, FeatureSummary,
    FeatureTable, FeatureTableOptions, FieldAggregate, FieldMissingPolicy, PopulationError,
    PopulationMorphology, feature_table, feature_table_csv,
};
pub use principal::{
    FrameOrigin, PRINCIPAL_FRAME_DEFINITION_VERSION, PrincipalFrame, PrincipalFrameError,
    PrincipalFrameOptions, PrincipalFrameProvenance, PrincipalPlane, PrincipalWeighting,
};
pub use query::{NodeSelection, QueryError, SelectionQuery, Selector};
pub use swc::{MAX_NODE_COUNT, ParseResult, parse_swc};
pub use tmd::{
    PersistencePair, TMD_DEFINITION_VERSION, TmdCenter, TmdError, TmdFiltration, TmdOptions,
    TmdProvenance, TmdResult,
};
pub use transform::{
    Affine3, AffineRadiusPolicy, GeometryTransformProvenance, NodeLineage, NodeMapping,
    ResampleOptions, SimplifyOptions, SwcExport, TransformError, TransformReport, TransformResult,
};
