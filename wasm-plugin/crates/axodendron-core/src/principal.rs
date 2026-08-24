use serde::{Deserialize, Serialize};

use crate::geometry::{Projection, Vec3};
use crate::model::{Morphology, SomaClass};
use crate::query::{QueryError, SelectionQuery, SelectionView};

pub const PRINCIPAL_FRAME_DEFINITION_VERSION: u16 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipalWeighting {
    Nodes,
    #[default]
    CableLength,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FrameOrigin {
    #[default]
    Centroid,
    Soma,
    World,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalFrameOptions {
    #[serde(default)]
    pub selection: SelectionQuery,
    #[serde(default)]
    pub weighting: PrincipalWeighting,
    #[serde(default)]
    pub origin: FrameOrigin,
    #[serde(default = "default_relative_tolerance")]
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub relative_tolerance: f64,
    #[serde(default = "default_absolute_tolerance")]
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub absolute_tolerance: f64,
}

impl Default for PrincipalFrameOptions {
    fn default() -> Self {
        Self {
            selection: SelectionQuery::default(),
            weighting: PrincipalWeighting::default(),
            origin: FrameOrigin::default(),
            relative_tolerance: default_relative_tolerance(),
            absolute_tolerance: default_absolute_tolerance(),
        }
    }
}

const fn default_relative_tolerance() -> f64 {
    1e-10
}

const fn default_absolute_tolerance() -> f64 {
    1e-12
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrincipalPlane {
    Xy,
    Xz,
    Yz,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrincipalFrameProvenance {
    pub definition_version: u16,
    pub algorithm: String,
    pub covariance_model: String,
    pub sign_rule: String,
    pub handedness_rule: String,
    pub relative_tolerance: f64,
    pub absolute_tolerance: f64,
    pub selection_fingerprint: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PrincipalFrame {
    pub morphology_fingerprint: String,
    pub topology_fingerprint: String,
    pub selection: SelectionQuery,
    pub weighting: PrincipalWeighting,
    pub origin_mode: FrameOrigin,
    pub origin: Vec3,
    pub centroid: Vec3,
    /// Principal axes ordered by decreasing covariance eigenvalue.
    pub axes: [Vec3; 3],
    pub eigenvalues: Vec3,
    pub extent_min: Vec3,
    pub extent_max: Vec3,
    pub rank: u8,
    /// An axis is marked ambiguous when it belongs to a repeated-eigenvalue
    /// eigenspace under the configured absolute/relative tolerance.
    pub ambiguous_axes: [bool; 3],
    pub provenance: PrincipalFrameProvenance,
}

impl PrincipalFrame {
    pub fn projection(&self, plane: PrincipalPlane) -> Projection {
        let (right, up, forward) = match plane {
            PrincipalPlane::Xy => (self.axes[0], self.axes[1], self.axes[2]),
            PrincipalPlane::Xz => (self.axes[0], self.axes[2], self.axes[1] * -1.0),
            PrincipalPlane::Yz => (self.axes[1], self.axes[2], self.axes[0]),
        };
        Projection { right, up, forward }
    }

    pub fn extents(&self) -> Vec3 {
        self.extent_max - self.extent_min
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrincipalFrameError {
    Query(QueryError),
    InvalidTolerance,
    InsufficientGeometry,
    NoSoma,
    AmbiguousSoma,
}

impl std::fmt::Display for PrincipalFrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Query(error) => error.fmt(f),
            Self::InvalidTolerance => {
                f.write_str("PCA tolerances must be finite and non-negative")
            }
            Self::InsufficientGeometry => f.write_str(
                "principal-frame requires positive selected cable length or at least two distinct selected nodes",
            ),
            Self::NoSoma => f.write_str("soma-centered principal frame requires soma geometry"),
            Self::AmbiguousSoma => {
                f.write_str("soma-centered principal frame requires an unambiguous soma")
            }
        }
    }
}

impl std::error::Error for PrincipalFrameError {}

impl From<QueryError> for PrincipalFrameError {
    fn from(value: QueryError) -> Self {
        Self::Query(value)
    }
}

impl Morphology {
    /// Compute a deterministic principal coordinate frame.
    ///
    /// Cable-length weighting integrates the first and second spatial moments
    /// analytically along every selected segment. It is therefore invariant to
    /// inserting collinear samples, unlike a plain covariance of SWC rows.
    pub fn principal_frame(
        &self,
        options: &PrincipalFrameOptions,
    ) -> Result<PrincipalFrame, PrincipalFrameError> {
        if !options.relative_tolerance.is_finite()
            || !options.absolute_tolerance.is_finite()
            || options.relative_tolerance < 0.0
            || options.absolute_tolerance < 0.0
        {
            return Err(PrincipalFrameError::InvalidTolerance);
        }
        let view = SelectionView::new(self, &options.selection)?;
        let (centroid, covariance, covariance_model) = match options.weighting {
            PrincipalWeighting::Nodes => node_moments(&view)?,
            PrincipalWeighting::CableLength => cable_moments(&view)?,
        };
        let (eigenvalues, mut axes) = symmetric_eigen(covariance);
        orient_axes(self, &view, centroid, &mut axes);
        if axes[0].cross(axes[1]).dot(axes[2]) < 0.0 {
            axes[2] = axes[2] * -1.0;
        }

        let eigenvalues = Vec3::new(
            eigenvalues[0].max(0.0),
            eigenvalues[1].max(0.0),
            eigenvalues[2].max(0.0),
        );
        let values = eigenvalues.to_array();
        let scale = values[0].max(1.0);
        let zero_tolerance = options.absolute_tolerance + options.relative_tolerance * scale;
        let rank = values
            .iter()
            .filter(|value| **value > zero_tolerance)
            .count() as u8;
        let mut ambiguous_axes = [false; 3];
        for a in 0..3 {
            for b in a + 1..3 {
                let tolerance = options.absolute_tolerance
                    + options.relative_tolerance * values[a].max(values[b]).max(1.0);
                if (values[a] - values[b]).abs() <= tolerance {
                    ambiguous_axes[a] = true;
                    ambiguous_axes[b] = true;
                }
            }
        }
        let origin = match options.origin {
            FrameOrigin::Centroid => centroid,
            FrameOrigin::World => Vec3::default(),
            FrameOrigin::Soma => match self.soma_class() {
                SomaClass::Absent => return Err(PrincipalFrameError::NoSoma),
                SomaClass::Disconnected | SomaClass::Ambiguous => {
                    return Err(PrincipalFrameError::AmbiguousSoma);
                }
                _ => self.soma_center(),
            },
        };
        let (extent_min, extent_max) = principal_extents(&view, origin, axes);
        Ok(PrincipalFrame {
            morphology_fingerprint: self.fingerprint().to_owned(),
            topology_fingerprint: self.topology_fingerprint().to_owned(),
            selection: view.query.clone(),
            weighting: options.weighting,
            origin_mode: options.origin,
            origin,
            centroid,
            axes,
            eigenvalues,
            extent_min,
            extent_max,
            rank,
            ambiguous_axes,
            provenance: PrincipalFrameProvenance {
                definition_version: PRINCIPAL_FRAME_DEFINITION_VERSION,
                algorithm: "symmetric-jacobi-3x3-v1".to_owned(),
                covariance_model,
                sign_rule: "farthest-absolute-projection-then-lowest-node-id".to_owned(),
                handedness_rule: "axis-1-cross-axis-2-dot-axis-3-positive".to_owned(),
                relative_tolerance: options.relative_tolerance,
                absolute_tolerance: options.absolute_tolerance,
                selection_fingerprint: view.fingerprint().to_owned(),
            },
        })
    }
}

fn node_moments(
    view: &SelectionView<'_>,
) -> Result<(Vec3, [[f64; 3]; 3], String), PrincipalFrameError> {
    let nodes: Vec<_> = view.nodes().collect();
    if nodes.len() < 2 {
        return Err(PrincipalFrameError::InsufficientGeometry);
    }
    let weight = nodes.len() as f64;
    let centroid = nodes.iter().fold(Vec3::default(), |sum, node| {
        sum + view.morphology.position(*node)
    }) * (1.0 / weight);
    let mut covariance = [[0.0; 3]; 3];
    for node in nodes {
        add_outer(
            &mut covariance,
            view.morphology.position(node) - centroid,
            1.0 / weight,
        );
    }
    Ok((
        centroid,
        covariance,
        "selected-node-population-covariance".to_owned(),
    ))
}

fn cable_moments(
    view: &SelectionView<'_>,
) -> Result<(Vec3, [[f64; 3]; 3], String), PrincipalFrameError> {
    let segments: Vec<_> = view
        .nodes()
        .filter_map(|child| view.parent(child).map(|parent| (parent, child)))
        .filter_map(|(parent, child)| {
            let a = view.morphology.position(parent);
            let b = view.morphology.position(child);
            let length = a.distance(b);
            (length > 0.0).then_some((a, b, length))
        })
        .collect();
    let total_length: f64 = segments.iter().map(|item| item.2).sum();
    if !total_length.is_finite() || total_length <= 0.0 {
        return Err(PrincipalFrameError::InsufficientGeometry);
    }
    let centroid = segments
        .iter()
        .fold(Vec3::default(), |sum, (a, b, length)| {
            sum + (*a + *b) * (0.5 * *length)
        })
        * (1.0 / total_length);
    let mut second = [[0.0; 3]; 3];
    for (a, b, length) in segments {
        let aa = outer(a, a);
        let bb = outer(b, b);
        let ab = outer(a, b);
        let ba = outer(b, a);
        for i in 0..3 {
            for j in 0..3 {
                second[i][j] +=
                    length * ((aa[i][j] + bb[i][j]) / 3.0 + (ab[i][j] + ba[i][j]) / 6.0);
            }
        }
    }
    let cc = outer(centroid, centroid);
    for i in 0..3 {
        for j in 0..3 {
            second[i][j] = second[i][j] / total_length - cc[i][j];
        }
    }
    Ok((
        centroid,
        second,
        "exact-continuous-uniform-cable-covariance".to_owned(),
    ))
}

fn outer(a: Vec3, b: Vec3) -> [[f64; 3]; 3] {
    let a = a.to_array();
    let b = b.to_array();
    let mut result = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            result[i][j] = a[i] * b[j];
        }
    }
    result
}

fn add_outer(matrix: &mut [[f64; 3]; 3], value: Vec3, weight: f64) {
    let value = value.to_array();
    for i in 0..3 {
        for j in 0..3 {
            matrix[i][j] += value[i] * value[j] * weight;
        }
    }
}

fn symmetric_eigen(mut matrix: [[f64; 3]; 3]) -> ([f64; 3], [Vec3; 3]) {
    let mut vectors = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..64 {
        let mut p = 0;
        let mut q = 1;
        let mut maximum = matrix[0][1].abs();
        for (i, j) in [(0, 2), (1, 2)] {
            if matrix[i][j].abs() > maximum {
                maximum = matrix[i][j].abs();
                p = i;
                q = j;
            }
        }
        let diagonal_scale = matrix[0][0]
            .abs()
            .max(matrix[1][1].abs())
            .max(matrix[2][2].abs())
            .max(1.0);
        if maximum <= 8.0 * f64::EPSILON * diagonal_scale {
            break;
        }
        let angle = 0.5 * (2.0 * matrix[p][q]).atan2(matrix[q][q] - matrix[p][p]);
        let (sine, cosine) = angle.sin_cos();
        let app = matrix[p][p];
        let aqq = matrix[q][q];
        let apq = matrix[p][q];
        for k in [0_usize, 1, 2] {
            if k == p || k == q {
                continue;
            }
            let akp = matrix[k][p];
            let akq = matrix[k][q];
            matrix[k][p] = cosine * akp - sine * akq;
            matrix[p][k] = matrix[k][p];
            matrix[k][q] = sine * akp + cosine * akq;
            matrix[q][k] = matrix[k][q];
        }
        matrix[p][p] = cosine * cosine * app - 2.0 * sine * cosine * apq + sine * sine * aqq;
        matrix[q][q] = sine * sine * app + 2.0 * sine * cosine * apq + cosine * cosine * aqq;
        matrix[p][q] = 0.0;
        matrix[q][p] = 0.0;
        for row in &mut vectors {
            let vip = row[p];
            let viq = row[q];
            row[p] = cosine * vip - sine * viq;
            row[q] = sine * vip + cosine * viq;
        }
    }
    let mut order = [0, 1, 2];
    order.sort_by(|a, b| matrix[*b][*b].total_cmp(&matrix[*a][*a]));
    let values = order.map(|ix| matrix[ix][ix]);
    let axes = order.map(|column| {
        Vec3::new(vectors[0][column], vectors[1][column], vectors[2][column])
            .normalized()
            .unwrap_or_default()
    });
    (values, axes)
}

fn orient_axes(
    morphology: &Morphology,
    view: &SelectionView<'_>,
    centroid: Vec3,
    axes: &mut [Vec3; 3],
) {
    for axis in axes.iter_mut() {
        let mut best: Option<(f64, i64, f64)> = None;
        for node in view.nodes() {
            let projection = (morphology.position(node) - centroid).dot(*axis);
            let candidate = (projection.abs(), morphology.id(node).0, projection);
            if best.is_none_or(|current| {
                candidate.0 > current.0 || (candidate.0 == current.0 && candidate.1 < current.1)
            }) {
                best = Some(candidate);
            }
        }
        if best.is_some_and(|value| value.2 < 0.0) {
            *axis = *axis * -1.0;
        }
    }
}

fn principal_extents(view: &SelectionView<'_>, origin: Vec3, axes: [Vec3; 3]) -> (Vec3, Vec3) {
    let mut minimum = [f64::INFINITY; 3];
    let mut maximum = [f64::NEG_INFINITY; 3];
    for node in view.nodes() {
        let relative = view.morphology.position(node) - origin;
        for ix in 0..3 {
            let value = relative.dot(axes[ix]);
            minimum[ix] = minimum[ix].min(value);
            maximum[ix] = maximum[ix].max(value);
        }
    }
    (Vec3::from_array(minimum), Vec3::from_array(maximum))
}

#[cfg(test)]
mod tests {
    use crate::{ValidationProfile, parse_swc};

    use super::*;

    fn strict(source: &str) -> Morphology {
        parse_swc(source, ValidationProfile::IncfStrict)
            .morphology
            .unwrap()
    }

    #[test]
    fn cable_pca_is_invariant_to_collinear_resampling() {
        let sparse = strict("1 3 -2 0 0 1 -1\n2 3 2 0 0 1 1\n");
        let dense = strict(
            "1 3 -2 0 0 1 -1\n2 3 -1 0 0 1 1\n3 3 0 0 0 1 2\n4 3 1 0 0 1 3\n5 3 2 0 0 1 4\n",
        );
        let options = PrincipalFrameOptions {
            selection: SelectionQuery {
                domain: crate::AnalysisDomain::Raw,
                ..Default::default()
            },
            ..Default::default()
        };
        let a = sparse.principal_frame(&options).unwrap();
        let b = dense.principal_frame(&options).unwrap();
        assert!((a.eigenvalues.x - b.eigenvalues.x).abs() < 1e-14);
        assert!(a.axes[0].dot(b.axes[0]).abs() > 1.0 - 1e-14);
    }

    #[test]
    fn frame_reports_degenerate_axes_and_right_handedness() {
        let morphology =
            strict("1 3 -1 -1 0 1 -1\n2 3 1 -1 0 1 1\n3 3 1 1 0 1 2\n4 3 -1 1 0 1 3\n");
        let frame = morphology
            .principal_frame(&PrincipalFrameOptions {
                selection: SelectionQuery {
                    domain: crate::AnalysisDomain::Raw,
                    ..Default::default()
                },
                weighting: PrincipalWeighting::Nodes,
                ..Default::default()
            })
            .unwrap();
        assert!(frame.ambiguous_axes[0] && frame.ambiguous_axes[1]);
        assert!(frame.axes[0].cross(frame.axes[1]).dot(frame.axes[2]) > 0.999_999);
    }
}
