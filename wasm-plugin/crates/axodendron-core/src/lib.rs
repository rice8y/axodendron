//! Deterministic, format-independent neuronal arbor algorithms for Axodendron.
//!
//! The core crate contains no Typst or WebAssembly-specific code. External node
//! identifiers are kept separate from compact internal indices, and every
//! traversal uses stable input order.

mod analysis;
mod diagnostic;
mod geometry;
mod model;
#[doc(hidden)]
pub mod serde_number;
mod swc;
mod transform;

pub use analysis::{
    AnalysisBundle, AnalysisDomain, AnalysisOptions, ArborMetrics, BBox, DEFINITION_VERSION,
    NodeField, RadiusMetrics, Section, SectionBoundaryPolicy, SectionDecomposition, ShollBin,
    ShollDimension, ShollResult, SomaMetrics, Summary, Topology, TortuositySummary, TypeCount,
    TypeMetrics,
};
pub use diagnostic::{Diagnostic, Severity, ValidationProfile};
pub use geometry::{Projection, ProjectionError, Vec2, Vec3};
pub use model::{
    ModelError, Morphology, NONE_NODE, NodeId, NodeIx, SOMA_GEOMETRY_RELATIVE_TOLERANCE, SomaClass,
    SwcMetadata,
};
pub use swc::{MAX_NODE_COUNT, ParseResult, parse_swc};
pub use transform::{
    NodeLineage, NodeMapping, ResampleOptions, SimplifyOptions, SwcExport, TransformError,
    TransformReport, TransformResult,
};
