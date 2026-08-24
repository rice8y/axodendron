use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::geometry::Vec3;
use crate::model::{Morphology, NONE_NODE, NodeId, NodeIx};
use crate::principal::PrincipalFrame;
use crate::swc::MAX_NODE_COUNT;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SimplifyOptions {
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub tolerance: f64,
    #[serde(default = "default_true")]
    pub preserve_type_changes: bool,
    #[serde(default = "default_true")]
    pub preserve_soma: bool,
    #[serde(default)]
    pub protected_ids: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResampleOptions {
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub step: f64,
    #[serde(default)]
    pub protected_ids: Vec<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeMapping {
    pub old_id: i64,
    pub new_id: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeLineage {
    pub new_id: i64,
    pub proximal_old_id: i64,
    pub distal_old_id: i64,
    pub distal_fraction: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformReport {
    pub operation: String,
    pub source_fingerprint: String,
    pub result_fingerprint: String,
    pub source_node_count: u32,
    pub result_node_count: u32,
    pub removed_node_ids: Vec<i64>,
    pub inserted_node_ids: Vec<i64>,
    pub source_cable_length: f64,
    pub result_cable_length: f64,
    pub cable_length_change: f64,
    pub guaranteed_max_deviation: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry: Option<GeometryTransformProvenance>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Affine3 {
    pub matrix: [[f64; 3]; 3],
    pub translation: Vec3,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AffineRadiusPolicy {
    #[default]
    Preserve,
    VolumeEquivalent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GeometryTransformProvenance {
    pub api_class: String,
    pub matrix: [[f64; 3]; 3],
    pub translation: Vec3,
    pub determinant: f64,
    pub radius_scale: f64,
    pub radius_policy: String,
    pub radius_representation_lossy: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransformResult {
    pub morphology: Morphology,
    pub mapping: Vec<NodeMapping>,
    pub lineage: Vec<NodeLineage>,
    pub report: TransformReport,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SwcExport {
    pub source: String,
    pub id_mapping: Vec<NodeMapping>,
}

impl Default for SimplifyOptions {
    fn default() -> Self {
        Self {
            tolerance: 0.0,
            preserve_type_changes: true,
            preserve_soma: true,
            protected_ids: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransformError {
    UnknownNode(i64),
    DifferentComponents(i64, i64),
    EmptyResult,
    InvalidTolerance,
    InvalidStep,
    IdSpaceExhausted,
    NodeLimitExceeded,
    InvalidGeometryTransform,
    InvalidRotationAxis,
    InvalidScale,
    NonFiniteResult,
    DegeneratePrincipalFrame,
}

impl std::fmt::Display for TransformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownNode(id) => write!(f, "node id {id} does not exist"),
            Self::DifferentComponents(a, b) => {
                write!(f, "nodes {a} and {b} are in different components")
            }
            Self::EmptyResult => f.write_str("transform would produce an empty morphology"),
            Self::InvalidTolerance => {
                f.write_str("simplification tolerance must be finite and non-negative")
            }
            Self::InvalidStep => f.write_str("resampling step must be finite and positive"),
            Self::IdSpaceExhausted => {
                f.write_str("resampling cannot allocate another positive node id")
            }
            Self::NodeLimitExceeded => write!(
                f,
                "resampling would exceed the {MAX_NODE_COUNT}-node morphology limit"
            ),
            Self::InvalidGeometryTransform => f.write_str(
                "affine matrix and translation must be finite and the matrix must be non-singular",
            ),
            Self::InvalidRotationAxis => {
                f.write_str("rotation or reflection axis must be finite and non-zero")
            }
            Self::InvalidScale => f.write_str("uniform scale must be finite and positive"),
            Self::NonFiniteResult => {
                f.write_str("geometry transform produced a non-finite coordinate or radius")
            }
            Self::DegeneratePrincipalFrame => f.write_str(
                "principal-frame alignment is ambiguous under the configured degeneracy tolerance",
            ),
        }
    }
}

impl std::error::Error for TransformError {}

impl Morphology {
    /// Apply a translation. This is an exactly SWC-compatible transform.
    pub fn translate_with_report(&self, offset: Vec3) -> Result<TransformResult, TransformError> {
        compatible_geometry_transform(
            self,
            Affine3 {
                matrix: identity_matrix(),
                translation: offset,
            },
            1.0,
            "translate",
        )
    }

    /// Apply a right-handed axis-angle rotation about `center`.
    pub fn rotate_with_report(
        &self,
        axis: Vec3,
        angle_radians: f64,
        center: Vec3,
    ) -> Result<TransformResult, TransformError> {
        let axis = axis
            .normalized()
            .ok_or(TransformError::InvalidRotationAxis)?;
        if !angle_radians.is_finite() || !finite_vec3(center) {
            return Err(TransformError::InvalidGeometryTransform);
        }
        let (s, c) = angle_radians.sin_cos();
        let k = 1.0 - c;
        let matrix = [
            [
                c + axis.x * axis.x * k,
                axis.x * axis.y * k - axis.z * s,
                axis.x * axis.z * k + axis.y * s,
            ],
            [
                axis.y * axis.x * k + axis.z * s,
                c + axis.y * axis.y * k,
                axis.y * axis.z * k - axis.x * s,
            ],
            [
                axis.z * axis.x * k - axis.y * s,
                axis.z * axis.y * k + axis.x * s,
                c + axis.z * axis.z * k,
            ],
        ];
        compatible_geometry_transform(
            self,
            Affine3 {
                matrix,
                translation: center - apply_matrix(matrix, center),
            },
            1.0,
            "rotate",
        )
    }

    /// Apply a uniform coordinate and radius scale about `center`.
    pub fn uniform_scale_with_report(
        &self,
        factor: f64,
        center: Vec3,
    ) -> Result<TransformResult, TransformError> {
        if !factor.is_finite() || factor <= 0.0 || !finite_vec3(center) {
            return Err(TransformError::InvalidScale);
        }
        let matrix = [[factor, 0.0, 0.0], [0.0, factor, 0.0], [0.0, 0.0, factor]];
        compatible_geometry_transform(
            self,
            Affine3 {
                matrix,
                translation: center - apply_matrix(matrix, center),
            },
            factor,
            "uniform-scale",
        )
    }

    /// Reflect across a plane passing through `point` with the given normal.
    pub fn reflect_with_report(
        &self,
        normal: Vec3,
        point: Vec3,
    ) -> Result<TransformResult, TransformError> {
        let n = normal
            .normalized()
            .ok_or(TransformError::InvalidRotationAxis)?;
        if !finite_vec3(point) {
            return Err(TransformError::InvalidGeometryTransform);
        }
        let matrix = [
            [1.0 - 2.0 * n.x * n.x, -2.0 * n.x * n.y, -2.0 * n.x * n.z],
            [-2.0 * n.y * n.x, 1.0 - 2.0 * n.y * n.y, -2.0 * n.y * n.z],
            [-2.0 * n.z * n.x, -2.0 * n.z * n.y, 1.0 - 2.0 * n.z * n.z],
        ];
        compatible_geometry_transform(
            self,
            Affine3 {
                matrix,
                translation: point - apply_matrix(matrix, point),
            },
            1.0,
            "reflect",
        )
    }

    /// Express coordinates in a previously computed principal frame.
    pub fn align_to_principal_frame_with_report(
        &self,
        frame: &PrincipalFrame,
        allow_degenerate: bool,
    ) -> Result<TransformResult, TransformError> {
        if frame.morphology_fingerprint != self.fingerprint() {
            return Err(TransformError::InvalidGeometryTransform);
        }
        if !allow_degenerate && frame.ambiguous_axes.iter().any(|value| *value) {
            return Err(TransformError::DegeneratePrincipalFrame);
        }
        let matrix = [
            frame.axes[0].to_array(),
            frame.axes[1].to_array(),
            frame.axes[2].to_array(),
        ];
        compatible_geometry_transform(
            self,
            Affine3 {
                matrix,
                translation: apply_matrix(matrix, frame.origin) * -1.0,
            },
            1.0,
            "principal-align",
        )
    }

    /// Apply a general invertible affine transform to SWC centerline points.
    ///
    /// A non-similarity affine map turns circular cross-sections into ellipses,
    /// which SWC cannot encode. The explicit radius policy therefore always
    /// records a lossy representation in the transform report.
    pub fn affine_with_report(
        &self,
        affine: Affine3,
        radius_policy: AffineRadiusPolicy,
    ) -> Result<TransformResult, TransformError> {
        validate_affine(affine)?;
        let determinant = determinant(affine.matrix);
        let radius_scale = match radius_policy {
            AffineRadiusPolicy::Preserve => 1.0,
            AffineRadiusPolicy::VolumeEquivalent => determinant.abs().cbrt(),
        };
        geometry_transform(
            self,
            affine,
            radius_scale,
            "general-affine",
            GeometryTransformProvenance {
                api_class: "general-affine".to_owned(),
                matrix: affine.matrix,
                translation: affine.translation,
                determinant,
                radius_scale,
                radius_policy: match radius_policy {
                    AffineRadiusPolicy::Preserve => "preserve".to_owned(),
                    AffineRadiusPolicy::VolumeEquivalent => "volume-equivalent".to_owned(),
                },
                radius_representation_lossy: true,
            },
        )
    }

    /// Select exactly the requested node IDs. Parent edges are retained only
    /// when both endpoints are selected; no synthetic bridging edge is added.
    pub fn select_nodes(&self, node_ids: &[i64]) -> Result<Self, TransformError> {
        let selected: HashSet<i64> = node_ids.iter().copied().collect();
        for id in &selected {
            if self.index_of(NodeId(*id)).is_none() {
                return Err(TransformError::UnknownNode(*id));
            }
        }
        let keep: Vec<bool> = self.ids().iter().map(|id| selected.contains(id)).collect();
        if !keep.iter().any(|value| *value) {
            return Err(TransformError::EmptyResult);
        }
        Ok(self.subset(&keep, false))
    }

    pub fn select_nodes_with_report(
        &self,
        node_ids: &[i64],
    ) -> Result<TransformResult, TransformError> {
        let morphology = self.select_nodes(node_ids)?;
        Ok(build_transform_result(
            self,
            morphology,
            "select-nodes",
            Vec::new(),
            Vec::new(),
            None,
        ))
    }

    /// Select nodes with one of the requested SWC kinds. The result is the
    /// induced forest over original parent edges.
    pub fn select_kinds(&self, kinds: &[i32]) -> Result<Self, TransformError> {
        let selected: HashSet<i32> = kinds.iter().copied().collect();
        let keep: Vec<bool> = self
            .kinds()
            .iter()
            .map(|kind| selected.contains(kind))
            .collect();
        if !keep.iter().any(|value| *value) {
            return Err(TransformError::EmptyResult);
        }
        Ok(self.subset(&keep, false))
    }

    pub fn select_kinds_with_report(
        &self,
        kinds: &[i32],
    ) -> Result<TransformResult, TransformError> {
        let morphology = self.select_kinds(kinds)?;
        Ok(build_transform_result(
            self,
            morphology,
            "select-kinds",
            Vec::new(),
            Vec::new(),
            None,
        ))
    }

    pub fn subtree(&self, root_id: NodeId) -> Result<Self, TransformError> {
        let root = self
            .index_of(root_id)
            .ok_or(TransformError::UnknownNode(root_id.0))?;
        let mut keep = vec![false; self.len()];
        let mut stack = vec![root];
        while let Some(node) = stack.pop() {
            keep[node.0 as usize] = true;
            for child in self.children(node).rev() {
                stack.push(child);
            }
        }
        Ok(self.subset(&keep, false))
    }

    pub fn subtree_with_report(&self, root_id: NodeId) -> Result<TransformResult, TransformError> {
        let morphology = self.subtree(root_id)?;
        Ok(build_transform_result(
            self,
            morphology,
            "subtree",
            Vec::new(),
            Vec::new(),
            None,
        ))
    }

    pub fn path_between(&self, a: NodeId, b: NodeId) -> Result<Self, TransformError> {
        let a_ix = self.index_of(a).ok_or(TransformError::UnknownNode(a.0))?;
        let b_ix = self.index_of(b).ok_or(TransformError::UnknownNode(b.0))?;
        if self.component(a_ix) != self.component(b_ix) {
            return Err(TransformError::DifferentComponents(a.0, b.0));
        }

        let mut ancestors = HashSet::<u32>::new();
        let mut cursor = Some(a_ix);
        while let Some(node) = cursor {
            ancestors.insert(node.0);
            cursor = self.parent(node);
        }
        let mut b_cursor = b_ix;
        while !ancestors.contains(&b_cursor.0) {
            b_cursor = self
                .parent(b_cursor)
                .expect("nodes in a rooted component have an LCA");
        }
        let lca = b_cursor;

        let mut keep = vec![false; self.len()];
        let mut cursor = a_ix;
        loop {
            keep[cursor.0 as usize] = true;
            if cursor == lca {
                break;
            }
            cursor = self.parent(cursor).unwrap();
        }
        let mut cursor = b_ix;
        loop {
            keep[cursor.0 as usize] = true;
            if cursor == lca {
                break;
            }
            cursor = self.parent(cursor).unwrap();
        }
        Ok(self.subset(&keep, false))
    }

    pub fn path_between_with_report(
        &self,
        a: NodeId,
        b: NodeId,
    ) -> Result<TransformResult, TransformError> {
        let morphology = self.path_between(a, b)?;
        Ok(build_transform_result(
            self,
            morphology,
            "path",
            Vec::new(),
            Vec::new(),
            None,
        ))
    }

    pub fn reroot(&self, new_root_id: NodeId) -> Result<Self, TransformError> {
        let new_root = self
            .index_of(new_root_id)
            .ok_or(TransformError::UnknownNode(new_root_id.0))?;
        let mut parents = self.parents_raw().to_vec();
        let mut child = new_root.0;
        let mut parent = parents[child as usize];
        parents[child as usize] = NONE_NODE;
        while parent != NONE_NODE {
            let grandparent = parents[parent as usize];
            parents[parent as usize] = child;
            child = parent;
            parent = grandparent;
        }
        Ok(self.rebuild_with_parents(parents))
    }

    pub fn reroot_with_report(
        &self,
        new_root_id: NodeId,
    ) -> Result<TransformResult, TransformError> {
        let morphology = self.reroot(new_root_id)?;
        Ok(build_transform_result(
            self,
            morphology,
            "reroot",
            Vec::new(),
            Vec::new(),
            None,
        ))
    }

    pub fn drop_kinds(&self, kinds: &[i32]) -> Result<Self, TransformError> {
        let dropped: HashSet<i32> = kinds.iter().copied().collect();
        let mut keep = vec![false; self.len()];
        let mut stack: Vec<(NodeIx, bool)> = self.roots().rev().map(|root| (root, false)).collect();
        while let Some((node, ancestor_dropped)) = stack.pop() {
            let is_dropped = ancestor_dropped || dropped.contains(&self.kind(node));
            keep[node.0 as usize] = !is_dropped;
            for child in self.children(node).rev() {
                stack.push((child, is_dropped));
            }
        }
        if !keep.iter().any(|value| *value) {
            return Err(TransformError::EmptyResult);
        }
        Ok(self.subset(&keep, false))
    }

    pub fn drop_kinds_with_report(&self, kinds: &[i32]) -> Result<TransformResult, TransformError> {
        let morphology = self.drop_kinds(kinds)?;
        Ok(build_transform_result(
            self,
            morphology,
            "drop-kinds",
            Vec::new(),
            Vec::new(),
            None,
        ))
    }

    pub fn simplify(&self, options: &SimplifyOptions) -> Result<Self, TransformError> {
        if !options.tolerance.is_finite() || options.tolerance < 0.0 {
            return Err(TransformError::InvalidTolerance);
        }
        for id in &options.protected_ids {
            if self.index_of(NodeId(*id)).is_none() {
                return Err(TransformError::UnknownNode(*id));
            }
        }
        if options.tolerance == 0.0 {
            return Ok(self.clone());
        }

        let protected: HashSet<i64> = options.protected_ids.iter().copied().collect();
        let mut keep = vec![false; self.len()];
        for raw in 0..self.len() as u32 {
            let node = NodeIx(raw);
            let topological = self.parent(node).is_none() || self.child_count(node) != 1;
            let type_change = options.preserve_type_changes
                && self
                    .parent(node)
                    .is_some_and(|parent| self.kind(parent) != self.kind(node));
            let soma = options.preserve_soma && self.kind(node) == 1;
            keep[raw as usize] =
                topological || type_change || soma || protected.contains(&self.id(node).0);
        }

        for start_raw in 0..self.len() as u32 {
            let start = NodeIx(start_raw);
            if self.parent(start).is_some() && self.child_count(start) == 1 {
                continue;
            }
            for first_child in self.children(start) {
                let mut section = vec![start, first_child];
                let mut cursor = first_child;
                while self.child_count(cursor) == 1 {
                    cursor = self.children(cursor).next().unwrap();
                    section.push(cursor);
                }

                let mandatory: Vec<usize> = section
                    .iter()
                    .enumerate()
                    .filter_map(|(position, node)| keep[node.0 as usize].then_some(position))
                    .collect();
                for pair in mandatory.windows(2) {
                    rdp_interval(
                        self,
                        &section,
                        pair[0],
                        pair[1],
                        options.tolerance,
                        &mut keep,
                    );
                }
            }
        }
        Ok(self.subset(&keep, true))
    }

    /// Resample every non-soma, same-type topological section at equal arc-length
    /// intervals. Roots, soma nodes, branches, terminals, type boundaries, and
    /// explicitly protected nodes retain their original IDs.
    pub fn resample(&self, options: &ResampleOptions) -> Result<Self, TransformError> {
        Ok(self.resample_with_report(options)?.morphology)
    }

    pub fn resample_with_report(
        &self,
        options: &ResampleOptions,
    ) -> Result<TransformResult, TransformError> {
        if !options.step.is_finite() || options.step <= 0.0 {
            return Err(TransformError::InvalidStep);
        }
        let protected: HashSet<i64> = options.protected_ids.iter().copied().collect();
        for id in &protected {
            if self.index_of(NodeId(*id)).is_none() {
                return Err(TransformError::UnknownNode(*id));
            }
        }

        let mut ids = Vec::new();
        let mut kinds = Vec::new();
        let mut positions = Vec::new();
        let mut radii = Vec::new();
        let mut parents = Vec::new();
        let mut lines = Vec::new();
        let mut old_to_new = HashMap::<u32, u32>::new();
        let mut lineage = Vec::new();
        let mut inserted_ids = Vec::new();
        let mut next_id = self.ids().iter().copied().max().unwrap_or(0);

        let append_old = |old: NodeIx,
                          parent: u32,
                          ids: &mut Vec<i64>,
                          kinds: &mut Vec<i32>,
                          positions: &mut Vec<Vec3>,
                          radii: &mut Vec<f64>,
                          parents: &mut Vec<u32>,
                          lines: &mut Vec<u32>,
                          old_to_new: &mut HashMap<u32, u32>| {
            let new = ids.len() as u32;
            ids.push(self.id(old).0);
            kinds.push(self.kind(old));
            positions.push(self.position(old));
            radii.push(self.radius(old));
            parents.push(parent);
            lines.push(self.source_line(old));
            old_to_new.insert(old.0, new);
            new
        };

        let roots: Vec<NodeIx> = self.roots().collect();
        for root in &roots {
            append_old(
                *root,
                NONE_NODE,
                &mut ids,
                &mut kinds,
                &mut positions,
                &mut radii,
                &mut parents,
                &mut lines,
                &mut old_to_new,
            );
        }
        let mut work: Vec<NodeIx> = roots.into_iter().rev().collect();

        while let Some(start) = work.pop() {
            let start_new = old_to_new[&start.0];
            let mut next_work = Vec::new();
            for first_child in self.children(start) {
                let mut path = vec![start, first_child];
                let mut cursor = first_child;
                let direct_unresampled_edge = self.kind(start) == 1
                    || self.kind(first_child) == 1
                    || self.kind(start) != self.kind(first_child);
                while !direct_unresampled_edge
                    && self.child_count(cursor) == 1
                    && !protected.contains(&self.id(cursor).0)
                    && self.kind(cursor) != 1
                {
                    let next = self.children(cursor).next().expect("one child was counted");
                    if self.kind(cursor) != self.kind(next) || self.kind(next) == 1 {
                        break;
                    }
                    path.push(next);
                    cursor = next;
                }

                if direct_unresampled_edge {
                    if !old_to_new.contains_key(&cursor.0) {
                        ensure_resample_capacity(ids.len())?;
                        append_old(
                            cursor,
                            start_new,
                            &mut ids,
                            &mut kinds,
                            &mut positions,
                            &mut radii,
                            &mut parents,
                            &mut lines,
                            &mut old_to_new,
                        );
                    }
                    next_work.push(cursor);
                    continue;
                }

                let cumulative = cumulative_lengths(self, &path);
                let total = *cumulative.last().unwrap_or(&0.0);
                let mut parent_new = start_new;
                let mut distance = options.step;
                while distance < total {
                    ensure_resample_capacity(ids.len())?;
                    let (segment, fraction) = locate_distance(&cumulative, distance);
                    let proximal = path[segment];
                    let distal = path[segment + 1];
                    let new_id = next_positive_id(&mut next_id)?;
                    let position =
                        lerp_vec3(self.position(proximal), self.position(distal), fraction);
                    let radius = lerp(self.radius(proximal), self.radius(distal), fraction);
                    let new_ix = ids.len() as u32;
                    ids.push(new_id);
                    kinds.push(self.kind(distal));
                    positions.push(position);
                    radii.push(radius);
                    parents.push(parent_new);
                    lines.push(0);
                    inserted_ids.push(new_id);
                    lineage.push(NodeLineage {
                        new_id,
                        proximal_old_id: self.id(proximal).0,
                        distal_old_id: self.id(distal).0,
                        distal_fraction: fraction,
                    });
                    parent_new = new_ix;
                    distance += options.step;
                }

                let endpoint = *path.last().expect("section has an endpoint");
                if let Some(existing) = old_to_new.get(&endpoint.0).copied() {
                    debug_assert_eq!(parents[existing as usize], parent_new);
                } else {
                    ensure_resample_capacity(ids.len())?;
                    append_old(
                        endpoint,
                        parent_new,
                        &mut ids,
                        &mut kinds,
                        &mut positions,
                        &mut radii,
                        &mut parents,
                        &mut lines,
                        &mut old_to_new,
                    );
                }
                next_work.push(endpoint);
            }
            for endpoint in next_work.into_iter().rev() {
                work.push(endpoint);
            }
        }

        let morphology = Morphology::from_parts(
            ids,
            kinds,
            positions,
            radii,
            parents,
            lines,
            self.units().to_owned(),
            self.source_fingerprint().map(str::to_owned),
            self.metadata().clone(),
        );
        Ok(build_transform_result(
            self,
            morphology,
            "resample",
            lineage,
            inserted_ids,
            None,
        ))
    }

    pub fn simplify_with_report(
        &self,
        options: &SimplifyOptions,
    ) -> Result<TransformResult, TransformError> {
        let morphology = self.simplify(options)?;
        Ok(build_transform_result(
            self,
            morphology,
            "simplify",
            Vec::new(),
            Vec::new(),
            Some(options.tolerance),
        ))
    }

    /// Export deterministic SWC with topological row order and IDs renumbered
    /// to `1..=N`. A single-root result satisfies the strict profile; forests
    /// retain every root and therefore require the permissive profile.
    pub fn to_canonical_swc(&self) -> SwcExport {
        let mut order = Vec::with_capacity(self.len());
        let mut stack: Vec<NodeIx> = self.roots().rev().collect();
        while let Some(node) = stack.pop() {
            order.push(node);
            for child in self.children(node).rev() {
                stack.push(child);
            }
        }
        let mut canonical_id = HashMap::<u32, i64>::new();
        for (position, node) in order.iter().copied().enumerate() {
            canonical_id.insert(node.0, position as i64 + 1);
        }
        let mut source = String::new();
        for comment in &self.metadata().comments {
            let clean = comment.replace(['\r', '\n'], " ");
            writeln!(source, "# {clean}").expect("writing to a String cannot fail");
        }
        writeln!(
            source,
            "# axodendron-canonical definition={}",
            crate::DEFINITION_VERSION
        )
        .expect("writing to a String cannot fail");
        for node in &order {
            let parent = self
                .parent(*node)
                .map_or(-1, |parent| canonical_id[&parent.0]);
            let point = self.position(*node);
            writeln!(
                source,
                "{} {} {} {} {} {} {}",
                canonical_id[&node.0],
                self.kind(*node),
                canonical_number(point.x),
                canonical_number(point.y),
                canonical_number(point.z),
                canonical_number(self.radius(*node)),
                parent
            )
            .expect("writing to a String cannot fail");
        }
        SwcExport {
            source,
            id_mapping: order
                .iter()
                .map(|node| NodeMapping {
                    old_id: self.id(*node).0,
                    new_id: Some(canonical_id[&node.0]),
                })
                .collect(),
        }
    }
}

fn build_transform_result(
    source: &Morphology,
    morphology: Morphology,
    operation: &str,
    lineage: Vec<NodeLineage>,
    inserted_node_ids: Vec<i64>,
    guaranteed_max_deviation: Option<f64>,
) -> TransformResult {
    let result_ids: HashSet<i64> = morphology.ids().iter().copied().collect();
    let mapping: Vec<NodeMapping> = source
        .ids()
        .iter()
        .copied()
        .map(|old_id| NodeMapping {
            old_id,
            new_id: result_ids.contains(&old_id).then_some(old_id),
        })
        .collect();
    let removed_node_ids = mapping
        .iter()
        .filter_map(|item| item.new_id.is_none().then_some(item.old_id))
        .collect();
    let source_cable_length = raw_cable_length(source);
    let result_cable_length = raw_cable_length(&morphology);
    TransformResult {
        report: TransformReport {
            operation: operation.to_owned(),
            source_fingerprint: source.fingerprint().to_owned(),
            result_fingerprint: morphology.fingerprint().to_owned(),
            source_node_count: source.len() as u32,
            result_node_count: morphology.len() as u32,
            removed_node_ids,
            inserted_node_ids,
            source_cable_length,
            result_cable_length,
            cable_length_change: result_cable_length - source_cable_length,
            guaranteed_max_deviation,
            geometry: None,
        },
        morphology,
        mapping,
        lineage,
    }
}

fn compatible_geometry_transform(
    morphology: &Morphology,
    affine: Affine3,
    radius_scale: f64,
    operation: &str,
) -> Result<TransformResult, TransformError> {
    validate_affine(affine)?;
    geometry_transform(
        morphology,
        affine,
        radius_scale,
        operation,
        GeometryTransformProvenance {
            api_class: "swc-compatible".to_owned(),
            matrix: affine.matrix,
            translation: affine.translation,
            determinant: determinant(affine.matrix),
            radius_scale,
            radius_policy: "exact-circular-radius".to_owned(),
            radius_representation_lossy: false,
        },
    )
}

fn geometry_transform(
    source: &Morphology,
    affine: Affine3,
    radius_scale: f64,
    operation: &str,
    provenance: GeometryTransformProvenance,
) -> Result<TransformResult, TransformError> {
    let positions: Vec<Vec3> = source
        .positions()
        .iter()
        .copied()
        .map(|point| apply_matrix(affine.matrix, point) + affine.translation)
        .collect();
    let radii: Vec<f64> = source
        .radii()
        .iter()
        .map(|radius| radius * radius_scale)
        .collect();
    if positions.iter().any(|point| !finite_vec3(*point))
        || radii.iter().any(|radius| !radius.is_finite())
    {
        return Err(TransformError::NonFiniteResult);
    }
    let morphology = Morphology::from_parts(
        source.ids().to_vec(),
        source.kinds().to_vec(),
        positions,
        radii,
        source.parents_raw().to_vec(),
        source.source_lines().to_vec(),
        source.units().to_owned(),
        source.source_fingerprint().map(str::to_owned),
        source.metadata().clone(),
    );
    let mut result =
        build_transform_result(source, morphology, operation, Vec::new(), Vec::new(), None);
    result.report.geometry = Some(provenance);
    Ok(result)
}

fn validate_affine(affine: Affine3) -> Result<(), TransformError> {
    if !affine
        .matrix
        .iter()
        .flatten()
        .all(|value| value.is_finite())
        || !finite_vec3(affine.translation)
        || !determinant(affine.matrix).is_finite()
        || determinant(affine.matrix).abs() <= 64.0 * f64::EPSILON
    {
        Err(TransformError::InvalidGeometryTransform)
    } else {
        Ok(())
    }
}

fn finite_vec3(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

fn identity_matrix() -> [[f64; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn apply_matrix(matrix: [[f64; 3]; 3], value: Vec3) -> Vec3 {
    Vec3::new(
        matrix[0][0] * value.x + matrix[0][1] * value.y + matrix[0][2] * value.z,
        matrix[1][0] * value.x + matrix[1][1] * value.y + matrix[1][2] * value.z,
        matrix[2][0] * value.x + matrix[2][1] * value.y + matrix[2][2] * value.z,
    )
}

fn determinant(matrix: [[f64; 3]; 3]) -> f64 {
    matrix[0][0] * (matrix[1][1] * matrix[2][2] - matrix[1][2] * matrix[2][1])
        - matrix[0][1] * (matrix[1][0] * matrix[2][2] - matrix[1][2] * matrix[2][0])
        + matrix[0][2] * (matrix[1][0] * matrix[2][1] - matrix[1][1] * matrix[2][0])
}

fn raw_cable_length(morphology: &Morphology) -> f64 {
    morphology
        .parents_raw()
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, parent)| *parent != NONE_NODE)
        .map(|(child, parent)| {
            morphology.positions()[child].distance(morphology.positions()[parent as usize])
        })
        .sum()
}

fn cumulative_lengths(morphology: &Morphology, path: &[NodeIx]) -> Vec<f64> {
    let mut cumulative = Vec::with_capacity(path.len());
    cumulative.push(0.0);
    for pair in path.windows(2) {
        let next = cumulative.last().copied().unwrap_or(0.0)
            + morphology
                .position(pair[0])
                .distance(morphology.position(pair[1]));
        cumulative.push(next);
    }
    cumulative
}

fn locate_distance(cumulative: &[f64], distance: f64) -> (usize, f64) {
    let upper = cumulative.partition_point(|value| *value < distance);
    let distal = upper.clamp(1, cumulative.len() - 1);
    let proximal = distal - 1;
    let span = cumulative[distal] - cumulative[proximal];
    let fraction = if span > 0.0 {
        (distance - cumulative[proximal]) / span
    } else {
        1.0
    };
    (proximal, fraction.clamp(0.0, 1.0))
}

fn next_positive_id(next: &mut i64) -> Result<i64, TransformError> {
    *next = next
        .checked_add(1)
        .ok_or(TransformError::IdSpaceExhausted)?;
    if *next <= 0 {
        return Err(TransformError::IdSpaceExhausted);
    }
    Ok(*next)
}

fn ensure_resample_capacity(current: usize) -> Result<(), TransformError> {
    if current < MAX_NODE_COUNT {
        Ok(())
    } else {
        Err(TransformError::NodeLimitExceeded)
    }
}

fn lerp(a: f64, b: f64, fraction: f64) -> f64 {
    a + (b - a) * fraction
}

fn lerp_vec3(a: Vec3, b: Vec3, fraction: f64) -> Vec3 {
    Vec3::new(
        lerp(a.x, b.x, fraction),
        lerp(a.y, b.y, fraction),
        lerp(a.z, b.z, fraction),
    )
}

fn canonical_number(value: f64) -> String {
    if value == 0.0 {
        "0".to_owned()
    } else {
        value.to_string()
    }
}

fn rdp_interval(
    morphology: &Morphology,
    nodes: &[NodeIx],
    first: usize,
    last: usize,
    tolerance: f64,
    keep: &mut [bool],
) {
    keep[nodes[first].0 as usize] = true;
    keep[nodes[last].0 as usize] = true;
    let mut stack = vec![(first, last)];
    while let Some((start, end)) = stack.pop() {
        if end <= start + 1 {
            continue;
        }
        let a = morphology.position(nodes[start]);
        let b = morphology.position(nodes[end]);
        let mut maximum = -1.0_f64;
        let mut maximum_ix = start + 1;
        for (candidate, node) in nodes.iter().enumerate().take(end).skip(start + 1) {
            let distance = point_segment_distance(morphology.position(*node), a, b);
            if distance > maximum {
                maximum = distance;
                maximum_ix = candidate;
            }
        }
        if maximum > tolerance {
            keep[nodes[maximum_ix].0 as usize] = true;
            stack.push((maximum_ix, end));
            stack.push((start, maximum_ix));
        }
    }
}

fn point_segment_distance(point: Vec3, a: Vec3, b: Vec3) -> f64 {
    let segment = b - a;
    let length = segment.norm();
    if length == 0.0 || !length.is_finite() {
        return point.distance(a);
    }
    let direction = segment * (1.0 / length);
    let distance_along = (point - a).dot(direction).clamp(0.0, length);
    point.distance(a + direction * distance_along)
}

#[cfg(test)]
mod tests {
    use crate::{
        Affine3, AffineRadiusPolicy, NodeId, SimplifyOptions, ValidationProfile, Vec3, parse_swc,
    };

    use super::{MAX_NODE_COUNT, ensure_resample_capacity};

    fn morphology() -> crate::Morphology {
        parse_swc(
            "1 1 0 0 0 1 -1\n2 3 1 0 0 1 1\n3 3 2 0 0 1 2\n4 3 3 0 0 1 3\n5 3 2 1 0 1 3\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap()
    }

    #[test]
    fn extracts_subtree_and_path() {
        let morphology = morphology();
        let selected = morphology.select_nodes(&[2, 4, 5]).unwrap();
        assert_eq!(selected.ids(), &[2, 4, 5]);
        assert_eq!(selected.roots().count(), 3);
        assert_eq!(
            morphology.select_nodes(&[999]),
            Err(crate::TransformError::UnknownNode(999))
        );
        assert_eq!(
            morphology.select_kinds(&[2]),
            Err(crate::TransformError::EmptyResult)
        );
        assert_eq!(morphology.select_kinds(&[3]).unwrap().ids(), &[2, 3, 4, 5]);
        assert_eq!(morphology.subtree(NodeId(3)).unwrap().len(), 3);
        let path = morphology.path_between(NodeId(4), NodeId(5)).unwrap();
        assert_eq!(path.ids(), &[3, 4, 5]);
    }

    #[test]
    fn reroot_reverses_parent_chain() {
        let rerooted = morphology().reroot(NodeId(4)).unwrap();
        let node_1 = rerooted.index_of(NodeId(1)).unwrap();
        assert_eq!(
            rerooted.parent(node_1).map(|node| rerooted.id(node)),
            Some(NodeId(2))
        );
    }

    #[test]
    fn simplification_keeps_branches_and_endpoints() {
        let simplified = morphology()
            .simplify(&SimplifyOptions {
                tolerance: 0.1,
                preserve_type_changes: false,
                preserve_soma: false,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(simplified.ids(), &[1, 3, 4, 5]);
    }

    #[test]
    fn simplification_rejects_unknown_protected_nodes_even_at_zero_tolerance() {
        let error = morphology()
            .simplify(&SimplifyOptions {
                tolerance: 0.0,
                protected_ids: vec![999],
                ..Default::default()
            })
            .unwrap_err();
        assert_eq!(error, crate::TransformError::UnknownNode(999));
    }

    #[test]
    fn resampling_is_arc_length_based_and_reports_lineage() {
        let source = parse_swc(
            "1 3 0 0 0 1 -1\n2 3 1 0 0 2 1\n3 3 3 0 0 4 2\n4 3 6 0 0 7 3\n",
            ValidationProfile::IncfStrict,
        )
        .morphology
        .unwrap();
        let result = source
            .resample_with_report(&crate::ResampleOptions {
                step: 2.0,
                protected_ids: Vec::new(),
            })
            .unwrap();
        assert_eq!(result.morphology.positions().len(), 4);
        assert_eq!(result.morphology.positions()[1].x, 2.0);
        assert_eq!(result.morphology.positions()[2].x, 4.0);
        assert_eq!(result.morphology.positions()[3].x, 6.0);
        assert_eq!(result.lineage.len(), 2);
        assert_eq!(result.report.removed_node_ids, vec![2, 3]);
        assert_eq!(result.report.cable_length_change, 0.0);
        assert_eq!(
            ensure_resample_capacity(MAX_NODE_COUNT),
            Err(crate::TransformError::NodeLimitExceeded)
        );
    }

    #[test]
    fn canonical_export_is_strict_and_roundtrips_metadata() {
        let source = parse_swc(
            "# CREATURE: mouse\n20 3 1 0 0 1 10\n10 1 0 0 0 2 -1\n",
            ValidationProfile::Permissive,
        )
        .morphology
        .unwrap();
        let exported = source.to_canonical_swc();
        assert!(exported.source.contains("# CREATURE: mouse"));
        let parsed = parse_swc(&exported.source, ValidationProfile::IncfStrict);
        assert!(parsed.is_valid(), "{:?}", parsed.diagnostics);
        assert_eq!(exported.id_mapping[0].old_id, 10);
        assert_eq!(exported.id_mapping[0].new_id, Some(1));

        let forest = parse_swc(
            "10 3 0 0 0 1 -1\n20 3 1 0 0 1 -1\n",
            ValidationProfile::Permissive,
        )
        .morphology
        .unwrap();
        let exported_forest = forest.to_canonical_swc();
        let permissive = parse_swc(&exported_forest.source, ValidationProfile::Permissive);
        assert!(permissive.is_valid(), "{:?}", permissive.diagnostics);
        assert_eq!(permissive.morphology.unwrap().roots().count(), 2);
        assert!(!parse_swc(&exported_forest.source, ValidationProfile::IncfStrict).is_valid());
    }

    #[test]
    fn compatible_geometry_transforms_preserve_topology_and_radius_semantics() {
        let source = morphology();
        let translated = source
            .translate_with_report(Vec3::new(10.0, -2.0, 3.0))
            .unwrap();
        assert_eq!(
            translated.morphology.topology_fingerprint(),
            source.topology_fingerprint()
        );
        assert_eq!(translated.morphology.radii(), source.radii());
        assert!(
            !translated
                .report
                .geometry
                .as_ref()
                .unwrap()
                .radius_representation_lossy
        );

        let scaled = source
            .uniform_scale_with_report(2.0, Vec3::default())
            .unwrap();
        assert_eq!(scaled.morphology.radii()[0], source.radii()[0] * 2.0);
        assert!(
            (scaled.report.result_cable_length - 2.0 * scaled.report.source_cable_length).abs()
                < 1e-12
        );
    }

    #[test]
    fn general_affine_requires_an_explicit_lossy_radius_policy() {
        let source = morphology();
        let transformed = source
            .affine_with_report(
                Affine3 {
                    matrix: [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 4.0]],
                    translation: Vec3::default(),
                },
                AffineRadiusPolicy::VolumeEquivalent,
            )
            .unwrap();
        let provenance = transformed.report.geometry.unwrap();
        assert!(provenance.radius_representation_lossy);
        assert!((provenance.radius_scale - 24.0_f64.cbrt()).abs() < 1e-12);
        assert_eq!(
            source.affine_with_report(
                Affine3 {
                    matrix: [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
                    translation: Vec3::default(),
                },
                AffineRadiusPolicy::Preserve,
            ),
            Err(crate::TransformError::InvalidGeometryTransform)
        );
    }
}
