use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub x: f64,
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec3 {
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub x: f64,
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub y: f64,
    #[serde(deserialize_with = "crate::serde_number::f64")]
    pub z: f64,
}

impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn from_array(value: [f64; 3]) -> Self {
        Self::new(value[0], value[1], value[2])
    }

    pub fn to_array(self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }

    pub fn norm_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn norm(self) -> f64 {
        self.x.hypot(self.y).hypot(self.z)
    }

    pub fn normalized(self) -> Option<Self> {
        let norm = self.norm();
        (norm > 0.0 && norm.is_finite()).then(|| self * (1.0 / norm))
    }

    pub fn distance(self, rhs: Self) -> f64 {
        (self - rhs).norm()
    }
}

impl std::ops::Add for Vec3 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Mul<f64> for Vec3 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Projection {
    /// Screen-horizontal basis vector.
    pub right: Vec3,
    /// Screen-vertical basis vector.
    pub up: Vec3,
    /// Depth direction.
    pub forward: Vec3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionError {
    ZeroDirection,
    ZeroUp,
    CollinearDirectionAndUp,
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::ZeroDirection => "projection direction must be non-zero",
            Self::ZeroUp => "projection up vector must be non-zero",
            Self::CollinearDirectionAndUp => {
                "projection direction and up vectors must not be collinear"
            }
        };
        f.write_str(text)
    }
}

impl std::error::Error for ProjectionError {}

impl Projection {
    /// Builds an orthonormal camera frame using a deterministic Gram-Schmidt step.
    pub fn look(direction: Vec3, up_hint: Vec3) -> Result<Self, ProjectionError> {
        let forward = direction
            .normalized()
            .ok_or(ProjectionError::ZeroDirection)?;
        let up_hint = up_hint.normalized().ok_or(ProjectionError::ZeroUp)?;
        let right = up_hint.cross(forward);
        if right.norm() <= 64.0 * f64::EPSILON {
            return Err(ProjectionError::CollinearDirectionAndUp);
        }
        let right = right
            .normalized()
            .ok_or(ProjectionError::CollinearDirectionAndUp)?;
        let up = forward.cross(right);
        Ok(Self { right, up, forward })
    }

    pub fn xy() -> Self {
        Self::look(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)).unwrap()
    }

    pub fn xz() -> Self {
        Self::look(Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 0.0, 1.0)).unwrap()
    }

    pub fn yz() -> Self {
        Self::look(Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)).unwrap()
    }

    pub fn project(self, point: Vec3) -> (Vec2, f64) {
        (
            Vec2::new(point.dot(self.right), point.dot(self.up)),
            point.dot(self.forward),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vector_algebra_is_consistent() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = Vec3::new(0.0, 0.0, 1.0);
        assert_eq!(x.cross(y), z);
        assert_eq!(x.dot(y), 0.0);
        assert_eq!((x + y).norm_squared(), 2.0);
        assert_eq!(x.distance(y), 2.0_f64.sqrt());
    }

    #[test]
    fn look_builds_a_right_handed_orthonormal_frame() {
        let projection =
            Projection::look(Vec3::new(1.0, 2.0, 3.0), Vec3::new(0.0, 0.0, 1.0)).unwrap();
        assert!((projection.right.norm() - 1.0).abs() < 1e-14);
        assert!((projection.up.norm() - 1.0).abs() < 1e-14);
        assert!((projection.forward.norm() - 1.0).abs() < 1e-14);
        assert!(projection.right.dot(projection.up).abs() < 1e-14);
        assert!(projection.right.dot(projection.forward).abs() < 1e-14);
        assert!(projection.up.dot(projection.forward).abs() < 1e-14);
        assert!(
            projection
                .right
                .cross(projection.up)
                .dot(projection.forward)
                > 0.999_999
        );
    }

    #[test]
    fn named_projection_axes_are_stable() {
        let point = Vec3::new(2.0, 3.0, 5.0);
        assert_eq!(Projection::xy().project(point), (Vec2::new(2.0, 3.0), 5.0));
        assert_eq!(Projection::xz().project(point), (Vec2::new(2.0, 5.0), -3.0));
        assert_eq!(Projection::yz().project(point), (Vec2::new(3.0, 5.0), 2.0));
    }

    #[test]
    fn invalid_projection_frames_are_rejected() {
        assert_eq!(
            Projection::look(Vec3::default(), Vec3::new(0.0, 1.0, 0.0)),
            Err(ProjectionError::ZeroDirection)
        );
        assert_eq!(
            Projection::look(Vec3::new(0.0, 0.0, 1.0), Vec3::default()),
            Err(ProjectionError::ZeroUp)
        );
        assert_eq!(
            Projection::look(Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -2.0)),
            Err(ProjectionError::CollinearDirectionAndUp)
        );
        assert!(Projection::look(Vec3::new(1e-200, 0.0, 0.0), Vec3::new(0.0, 1e-200, 0.0)).is_ok());
    }
}
