use std::collections::{BTreeMap, HashMap};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::geometry::Vec3;

pub const NONE_NODE: u32 = u32::MAX;
pub const SOMA_GEOMETRY_RELATIVE_TOLERANCE: f64 = 0.01;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwcMetadata {
    /// Comment lines without the leading `#`, retained in source order.
    pub comments: Vec<String>,
    /// Best-effort normalized header fields. Values are never applied implicitly.
    pub fields: BTreeMap<String, Vec<String>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeIx(pub(crate) u32);

impl NodeIx {
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SomaClass {
    Absent,
    SinglePoint,
    ThreePoint,
    MultiPointChain,
    Branched,
    Disconnected,
    Ambiguous,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModelError {
    Empty,
    TooManyNodes,
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    NonpositiveId(i64),
    DuplicateId(i64),
    NonfinitePosition(i64),
    NonfiniteRadius(i64),
    ParentOutOfRange(i64),
    SelfParent(i64),
    Cycle(i64),
    NoRoot,
    EmptyUnits,
    InvalidUnits,
    FingerprintMismatch,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => f.write_str("morphology must contain at least one node"),
            Self::TooManyNodes => f.write_str("morphology exceeds the u32 internal-index limit"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => {
                write!(f, "{field} has length {actual}; expected {expected}")
            }
            Self::NonpositiveId(id) => write!(f, "node id {id} is not positive"),
            Self::DuplicateId(id) => write!(f, "node id {id} is duplicated"),
            Self::NonfinitePosition(id) => write!(f, "node {id} has a non-finite position"),
            Self::NonfiniteRadius(id) => write!(f, "node {id} has a non-finite radius"),
            Self::ParentOutOfRange(id) => write!(f, "node {id} has an out-of-range parent index"),
            Self::SelfParent(id) => write!(f, "node {id} is its own parent"),
            Self::Cycle(id) => write!(f, "parent chain at node {id} contains a cycle"),
            Self::NoRoot => f.write_str("morphology has no root"),
            Self::EmptyUnits => f.write_str("morphology units must not be empty"),
            Self::InvalidUnits => {
                f.write_str("morphology units must be valid XML text no longer than 64 UTF-8 bytes")
            }
            Self::FingerprintMismatch => {
                f.write_str("morphology payload fingerprint does not match its content")
            }
        }
    }
}

impl std::error::Error for ModelError {}

/// Canonical neuronal morphology with compact derived topology.
///
/// Serialization intentionally includes only canonical arrays. CSR children,
/// roots, components, soma classification, and the semantic fingerprint are
/// rebuilt and validated during deserialization. This keeps the WASM payload
/// compact and prevents malformed payloads from creating unsafe indices.
#[derive(Clone, Debug, PartialEq)]
pub struct Morphology {
    ids: Vec<i64>,
    kinds: Vec<i32>,
    positions: Vec<Vec3>,
    radii: Vec<f64>,
    parents: Vec<u32>,
    child_offsets: Vec<u32>,
    child_indices: Vec<u32>,
    roots: Vec<u32>,
    components: Vec<u32>,
    id_to_index: HashMap<i64, u32>,
    source_lines: Vec<u32>,
    units: String,
    fingerprint: String,
    topology_fingerprint: String,
    source_fingerprint: Option<String>,
    soma_class: SomaClass,
    metadata: SwcMetadata,
}

#[derive(Serialize)]
struct MorphologyWireRef<'a> {
    ids: &'a [i64],
    kinds: &'a [i32],
    positions: Vec<[f64; 3]>,
    radii: &'a [f64],
    parents: &'a [u32],
    source_lines: &'a [u32],
    units: &'a str,
    fingerprint: &'a str,
    source_fingerprint: Option<&'a str>,
    metadata: &'a SwcMetadata,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MorphologyWireOwned {
    ids: Vec<i64>,
    kinds: Vec<i32>,
    positions: Vec<[f64; 3]>,
    radii: Vec<f64>,
    parents: Vec<u32>,
    source_lines: Vec<u32>,
    units: String,
    fingerprint: String,
    #[serde(default)]
    source_fingerprint: Option<String>,
    #[serde(default)]
    metadata: SwcMetadata,
}

impl Serialize for Morphology {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        MorphologyWireRef {
            ids: &self.ids,
            kinds: &self.kinds,
            positions: self.positions.iter().copied().map(Vec3::to_array).collect(),
            radii: &self.radii,
            parents: &self.parents,
            source_lines: &self.source_lines,
            units: &self.units,
            fingerprint: &self.fingerprint,
            source_fingerprint: self.source_fingerprint.as_deref(),
            metadata: &self.metadata,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Morphology {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = MorphologyWireOwned::deserialize(deserializer)?;
        Self::try_from_parts(
            wire.ids,
            wire.kinds,
            wire.positions.into_iter().map(Vec3::from_array).collect(),
            wire.radii,
            wire.parents,
            wire.source_lines,
            wire.units,
            wire.source_fingerprint,
            wire.metadata,
            Some(&wire.fingerprint),
        )
        .map_err(D::Error::custom)
    }
}

impl Morphology {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        ids: Vec<i64>,
        kinds: Vec<i32>,
        positions: Vec<Vec3>,
        radii: Vec<f64>,
        parents: Vec<u32>,
        source_lines: Vec<u32>,
        units: String,
        source_fingerprint: Option<String>,
        metadata: SwcMetadata,
    ) -> Self {
        Self::try_from_parts(
            ids,
            kinds,
            positions,
            radii,
            parents,
            source_lines,
            units,
            source_fingerprint,
            metadata,
            None,
        )
        .expect("internal morphology construction must preserve model invariants")
    }

    #[allow(clippy::too_many_arguments)]
    fn try_from_parts(
        ids: Vec<i64>,
        kinds: Vec<i32>,
        positions: Vec<Vec3>,
        radii: Vec<f64>,
        parents: Vec<u32>,
        source_lines: Vec<u32>,
        units: String,
        source_fingerprint: Option<String>,
        metadata: SwcMetadata,
        expected_fingerprint: Option<&str>,
    ) -> Result<Self, ModelError> {
        let n = ids.len();
        if n == 0 {
            return Err(ModelError::Empty);
        }
        if n > u32::MAX as usize {
            return Err(ModelError::TooManyNodes);
        }
        check_length("kinds", kinds.len(), n)?;
        check_length("positions", positions.len(), n)?;
        check_length("radii", radii.len(), n)?;
        check_length("parents", parents.len(), n)?;
        check_length("source_lines", source_lines.len(), n)?;
        if units.trim().is_empty() {
            return Err(ModelError::EmptyUnits);
        }
        if units.len() > 64
            || !units
                .chars()
                .all(|character| matches!(character, '\t' | '\n' | '\r') || character >= '\u{20}')
        {
            return Err(ModelError::InvalidUnits);
        }

        let mut id_to_index = HashMap::with_capacity(n);
        for (ix, id) in ids.iter().copied().enumerate() {
            if id <= 0 {
                return Err(ModelError::NonpositiveId(id));
            }
            if id_to_index.insert(id, ix as u32).is_some() {
                return Err(ModelError::DuplicateId(id));
            }
            let point = positions[ix];
            if !point.x.is_finite() || !point.y.is_finite() || !point.z.is_finite() {
                return Err(ModelError::NonfinitePosition(id));
            }
            if !radii[ix].is_finite() {
                return Err(ModelError::NonfiniteRadius(id));
            }
            let parent = parents[ix];
            if parent != NONE_NODE {
                if parent as usize >= n {
                    return Err(ModelError::ParentOutOfRange(id));
                }
                if parent as usize == ix {
                    return Err(ModelError::SelfParent(id));
                }
            }
        }
        validate_parent_chains(&ids, &parents)?;
        if !parents.contains(&NONE_NODE) {
            return Err(ModelError::NoRoot);
        }

        let fingerprint = semantic_fingerprint(&ids, &kinds, &positions, &radii, &parents, &units);
        let topology_fingerprint = topology_fingerprint(&ids, &kinds, &parents);
        if expected_fingerprint.is_some_and(|expected| expected != fingerprint) {
            return Err(ModelError::FingerprintMismatch);
        }
        let (child_offsets, child_indices, roots, components) = derived_topology(&parents);
        let soma_class = classify_soma(
            &kinds,
            &positions,
            &radii,
            &parents,
            &child_offsets,
            &child_indices,
        );
        Ok(Self {
            ids,
            kinds,
            positions,
            radii,
            parents,
            child_offsets,
            child_indices,
            roots,
            components,
            id_to_index,
            source_lines,
            units,
            fingerprint,
            topology_fingerprint,
            source_fingerprint,
            soma_class,
            metadata,
        })
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn ids(&self) -> &[i64] {
        &self.ids
    }

    pub fn kinds(&self) -> &[i32] {
        &self.kinds
    }

    pub fn positions(&self) -> &[Vec3] {
        &self.positions
    }

    pub fn radii(&self) -> &[f64] {
        &self.radii
    }

    pub fn parents_raw(&self) -> &[u32] {
        &self.parents
    }

    pub fn roots_raw(&self) -> &[u32] {
        &self.roots
    }

    pub fn source_lines(&self) -> &[u32] {
        &self.source_lines
    }

    pub fn units(&self) -> &str {
        &self.units
    }

    /// A deterministic hash of semantic morphology content, independent of SWC formatting.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// A deterministic hash of node identity, SWC kinds, and parent topology.
    ///
    /// Unlike [`Self::fingerprint`], this value is unchanged by coordinate-only
    /// transforms. It is suitable for transient section and bifurcation keys,
    /// but never proves that a geometry-dependent measurement is still valid.
    pub fn topology_fingerprint(&self) -> &str {
        &self.topology_fingerprint
    }

    /// Hash of the original SWC bytes, retained through pure transforms.
    pub fn source_fingerprint(&self) -> Option<&str> {
        self.source_fingerprint.as_deref()
    }

    pub fn metadata(&self) -> &SwcMetadata {
        &self.metadata
    }

    pub fn soma_class(&self) -> SomaClass {
        self.soma_class
    }

    pub fn id(&self, ix: NodeIx) -> NodeId {
        NodeId(self.ids[ix.0 as usize])
    }

    pub fn kind(&self, ix: NodeIx) -> i32 {
        self.kinds[ix.0 as usize]
    }

    pub fn position(&self, ix: NodeIx) -> Vec3 {
        self.positions[ix.0 as usize]
    }

    pub fn radius(&self, ix: NodeIx) -> f64 {
        self.radii[ix.0 as usize]
    }

    pub fn source_line(&self, ix: NodeIx) -> u32 {
        self.source_lines[ix.0 as usize]
    }

    pub fn parent(&self, ix: NodeIx) -> Option<NodeIx> {
        let parent = self.parents[ix.0 as usize];
        (parent != NONE_NODE).then_some(NodeIx(parent))
    }

    pub fn children(
        &self,
        ix: NodeIx,
    ) -> impl ExactSizeIterator<Item = NodeIx> + DoubleEndedIterator + '_ {
        let start = self.child_offsets[ix.0 as usize] as usize;
        let end = self.child_offsets[ix.0 as usize + 1] as usize;
        self.child_indices[start..end].iter().copied().map(NodeIx)
    }

    pub fn child_count(&self, ix: NodeIx) -> usize {
        let i = ix.0 as usize;
        (self.child_offsets[i + 1] - self.child_offsets[i]) as usize
    }

    pub fn roots(&self) -> impl ExactSizeIterator<Item = NodeIx> + DoubleEndedIterator + '_ {
        self.roots.iter().copied().map(NodeIx)
    }

    pub fn component(&self, ix: NodeIx) -> u32 {
        self.components[ix.0 as usize]
    }

    pub fn index_of(&self, id: NodeId) -> Option<NodeIx> {
        self.id_to_index.get(&id.0).copied().map(NodeIx)
    }

    pub(crate) fn rebuild_with_parents(&self, parents: Vec<u32>) -> Self {
        Self::from_parts(
            self.ids.clone(),
            self.kinds.clone(),
            self.positions.clone(),
            self.radii.clone(),
            parents,
            self.source_lines.clone(),
            self.units.clone(),
            self.source_fingerprint.clone(),
            self.metadata.clone(),
        )
    }

    pub(crate) fn subset(&self, keep: &[bool], collapse_removed_parents: bool) -> Self {
        debug_assert_eq!(keep.len(), self.len());
        let mut remap = vec![NONE_NODE; self.len()];
        let mut ids = Vec::new();
        let mut kinds = Vec::new();
        let mut positions = Vec::new();
        let mut radii = Vec::new();
        let mut lines = Vec::new();
        for (old, is_kept) in keep.iter().copied().enumerate() {
            if is_kept {
                remap[old] = ids.len() as u32;
                ids.push(self.ids[old]);
                kinds.push(self.kinds[old]);
                positions.push(self.positions[old]);
                radii.push(self.radii[old]);
                lines.push(self.source_lines[old]);
            }
        }

        let mut parents = Vec::with_capacity(ids.len());
        for old in 0..self.len() {
            if !keep[old] {
                continue;
            }
            let mut parent = self.parents[old];
            if collapse_removed_parents {
                while parent != NONE_NODE && !keep[parent as usize] {
                    parent = self.parents[parent as usize];
                }
            }
            parents.push(if parent == NONE_NODE {
                NONE_NODE
            } else {
                remap[parent as usize]
            });
        }

        Self::from_parts(
            ids,
            kinds,
            positions,
            radii,
            parents,
            lines,
            self.units.clone(),
            self.source_fingerprint.clone(),
            self.metadata.clone(),
        )
    }
}

pub(crate) fn fingerprint_bytes(bytes: &[u8]) -> String {
    let mut hash = Fnv1a64::new();
    hash.update(bytes);
    hash.finish()
}

fn semantic_fingerprint(
    ids: &[i64],
    kinds: &[i32],
    positions: &[Vec3],
    radii: &[f64],
    parents: &[u32],
    units: &str,
) -> String {
    let mut hash = Fnv1a64::new();
    hash.update(b"axodendron-morphology-v1\0");
    hash.update(&(ids.len() as u64).to_le_bytes());
    for ix in 0..ids.len() {
        hash.update(&ids[ix].to_le_bytes());
        hash.update(&kinds[ix].to_le_bytes());
        for coordinate in positions[ix].to_array() {
            hash.update(&normalized_float_bits(coordinate).to_le_bytes());
        }
        hash.update(&normalized_float_bits(radii[ix]).to_le_bytes());
        hash.update(&parents[ix].to_le_bytes());
    }
    hash.update(&(units.len() as u64).to_le_bytes());
    hash.update(units.as_bytes());
    hash.finish()
}

fn topology_fingerprint(ids: &[i64], kinds: &[i32], parents: &[u32]) -> String {
    let mut hash = Fnv1a64::new();
    hash.update(b"axodendron-topology-v1\0");
    hash.update(&(ids.len() as u64).to_le_bytes());
    for ix in 0..ids.len() {
        hash.update(&ids[ix].to_le_bytes());
        hash.update(&kinds[ix].to_le_bytes());
        hash.update(&parents[ix].to_le_bytes());
    }
    hash.finish()
}

fn normalized_float_bits(value: f64) -> u64 {
    if value == 0.0 { 0 } else { value.to_bits() }
}

struct Fnv1a64(u64);

impl Fnv1a64 {
    const fn new() -> Self {
        Self(0xcbf29ce484222325)
    }

    fn update(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x100000001b3);
        }
    }

    fn finish(self) -> String {
        format!("fnv1a64:{:016x}", self.0)
    }
}

fn check_length(field: &'static str, actual: usize, expected: usize) -> Result<(), ModelError> {
    if actual == expected {
        Ok(())
    } else {
        Err(ModelError::LengthMismatch {
            field,
            expected,
            actual,
        })
    }
}

fn validate_parent_chains(ids: &[i64], parents: &[u32]) -> Result<(), ModelError> {
    let mut state = vec![0_u8; parents.len()];
    for start in 0..parents.len() {
        if state[start] == 2 {
            continue;
        }
        let mut path = Vec::new();
        let mut cursor = start as u32;
        while cursor != NONE_NODE && state[cursor as usize] == 0 {
            state[cursor as usize] = 1;
            path.push(cursor);
            cursor = parents[cursor as usize];
        }
        if cursor != NONE_NODE && state[cursor as usize] == 1 {
            return Err(ModelError::Cycle(ids[cursor as usize]));
        }
        for ix in path {
            state[ix as usize] = 2;
        }
    }
    Ok(())
}

fn derived_topology(parents: &[u32]) -> (Vec<u32>, Vec<u32>, Vec<u32>, Vec<u32>) {
    let n = parents.len();
    let mut counts = vec![0_u32; n];
    let mut roots = Vec::new();
    for (ix, parent) in parents.iter().copied().enumerate() {
        if parent == NONE_NODE {
            roots.push(ix as u32);
        } else {
            counts[parent as usize] += 1;
        }
    }

    let mut offsets = vec![0_u32; n + 1];
    for ix in 0..n {
        offsets[ix + 1] = offsets[ix] + counts[ix];
    }
    let mut cursor = offsets[..n].to_vec();
    let mut children = vec![0_u32; offsets[n] as usize];
    for (child, parent) in parents.iter().copied().enumerate() {
        if parent != NONE_NODE {
            let slot = &mut cursor[parent as usize];
            children[*slot as usize] = child as u32;
            *slot += 1;
        }
    }

    let mut components = vec![NONE_NODE; n];
    for (component, root) in roots.iter().copied().enumerate() {
        let mut stack = vec![root];
        while let Some(ix) = stack.pop() {
            components[ix as usize] = component as u32;
            let start = offsets[ix as usize] as usize;
            let end = offsets[ix as usize + 1] as usize;
            for child in children[start..end].iter().rev() {
                stack.push(*child);
            }
        }
    }
    (offsets, children, roots, components)
}

fn classify_soma(
    kinds: &[i32],
    positions: &[Vec3],
    radii: &[f64],
    parents: &[u32],
    offsets: &[u32],
    children: &[u32],
) -> SomaClass {
    let soma_nodes: Vec<usize> = kinds
        .iter()
        .enumerate()
        .filter_map(|(ix, kind)| (*kind == 1).then_some(ix))
        .collect();
    if soma_nodes.is_empty() {
        return SomaClass::Absent;
    }
    if soma_nodes.len() == 1 {
        return SomaClass::SinglePoint;
    }

    let mut soma_roots = 0;
    let mut branched = false;
    let mut soma_root = None;
    for &ix in &soma_nodes {
        if parents[ix] == NONE_NODE || kinds[parents[ix] as usize] != 1 {
            soma_roots += 1;
            soma_root = Some(ix);
        }
        let start = offsets[ix] as usize;
        let end = offsets[ix + 1] as usize;
        let soma_children = children[start..end]
            .iter()
            .filter(|child| kinds[**child as usize] == 1)
            .count();
        branched |= soma_children > 1;
    }
    if soma_roots > 1 {
        return SomaClass::Disconnected;
    }
    if soma_nodes.len() == 3 {
        let root = soma_root.expect("a connected soma subgraph has one root");
        let start = offsets[root] as usize;
        let end = offsets[root + 1] as usize;
        let soma_children: Vec<usize> = children[start..end]
            .iter()
            .filter_map(|child| (kinds[*child as usize] == 1).then_some(*child as usize))
            .collect();
        if soma_children.len() == 2 {
            let center = positions[root];
            let a = positions[soma_children[0]] - center;
            let b = positions[soma_children[1]] - center;
            let radius = radii[root];
            let scale = radius
                .abs()
                .max(a.norm())
                .max(b.norm())
                .max(f64::MIN_POSITIVE);
            let tolerance = SOMA_GEOMETRY_RELATIVE_TOLERANCE * scale;
            let radii_match = soma_nodes
                .iter()
                .all(|ix| (radii[*ix] - radius).abs() <= tolerance);
            let endpoint_distances_match =
                (a.norm() - radius).abs() <= tolerance && (b.norm() - radius).abs() <= tolerance;
            let endpoints_are_opposite = (a + b).norm() <= tolerance;
            if radius > 0.0 && radii_match && endpoint_distances_match && endpoints_are_opposite {
                return SomaClass::ThreePoint;
            }
            return SomaClass::Ambiguous;
        }
    }
    if branched {
        SomaClass::Branched
    } else {
        SomaClass::MultiPointChain
    }
}
