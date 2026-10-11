//! Celestia math core: geometry primitives and numeric helpers.
//!
//! Faithful Rust port of Celestia's `celmath` library (`mathlib.h`,
//! `frustum.h`/`.cpp`, `ray.h`, `sphere.h`, `ellipsoid.h`, `geomutil.h`,
//! `intersect.h`, `solve.h`, `randutils.h`, `distance.h`).
//!
//! Celestia's `celmath` is built entirely on Eigen (`Eigen::Matrix`,
//! `Eigen::Quaternion`, `Eigen::ParametrizedLine`, `Eigen::Hyperplane`).
//! This port maps Eigen to [`glam`]:
//!
//! | Eigen (C++)                          | Rust (glam)            |
//! |--------------------------------------|------------------------|
//! | `Eigen::Vector3<T>` / `Matrix<T,3,1>`| `Vec3` / `DVec3`       |
//! | `Eigen::Quaternion<T>`              | `Quat` / `DQuat`       |
//! | `Eigen::Matrix<T,3,3>` / `4,4`       | `Mat3`/`Dmat3`, `Mat4`/`DMat4` |
//! | `Eigen::ParametrizedLine<T,3>`      | [`Ray`]                 |
//! | `Eigen::Hyperplane<T,3>`            | [`Plane`]               |
//!
//! The port keeps Celestia's double-precision (`T = double`) and single-precision
//! (`T = float`) variants, which Celestia uses heavily (e.g. the frustum is
//! single precision, body positions are double precision).
//!
//! This module is intentionally free of any Bevy dependency so the geometry can be
//! unit-tested in isolation and reused by the engine and the render layer.
#![allow(missing_docs)]

use std::f64::consts::PI;

use glam::{
    DQuat, DVec3, DVec4, Mat3, Mat4, Quat, Vec3, Vec4,
};

// --- Scalar helpers (celmath/mathlib.h) --------------------------------------

/// `sin`/`cos` for `f32`, mirroring the `float` `sincos` template.
pub fn sincos_f32(angle: f32) -> (f32, f32) {
    let (s, c) = angle.sin_cos();
    (s, c)
}

/// `sin`/`cos` for `f64`, mirroring the `double` `sincos` template.
pub fn sincos_f64(angle: f64) -> (f64, f64) {
    let (s, c) = angle.sin_cos();
    (s, c)
}

/// Linear interpolation: `a + t * (b - a)`. Mirrors `celmath::lerp`.
pub const fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

/// Degrees to radians. Mirrors `celmath::degToRad`.
pub const fn deg_to_rad(d: f64) -> f64 {
    d / 180.0 * PI
}

/// Radians to degrees. Mirrors `celmath::radToDeg`.
pub const fn rad_to_deg(r: f64) -> f64 {
    r * 180.0 / PI
}

/// `x * x`. Mirrors `celmath::square`.
pub const fn square(x: f64) -> f64 {
    x * x
}

/// `x^3`. Mirrors `celmath::cube`.
pub const fn cube(x: f64) -> f64 {
    x * x * x
}

/// Sign of `x` (`-1`, `0`, or `1`). Mirrors `celmath::sign`.
pub const fn sign(x: f64) -> f64 {
    if x < 0.0 {
        -1.0
    } else if x > 0.0 {
        1.0
    } else {
        0.0
    }
}

/// A `fmod` that always returns a value in `[0, y)`. Mirrors
/// `celmath::pfmod`.
pub fn pfmod(x: f64, y: f64) -> f64 {
    let quotient = (x.abs() / y).floor();
    if x < 0.0 {
        x + (quotient + 1.0) * y
    } else {
        x - quotient * y
    }
}

/// Area of a circle of radius `r`. Mirrors `celmath::circleArea`.
pub const fn circle_area(r: f64) -> f64 {
    PI * r * r
}

/// Surface area of a sphere of radius `r`. Mirrors `celmath::sphereArea`.
pub const fn sphere_area(r: f64) -> f64 {
    4.0 * PI * r * r
}

// --- Sphere (celmath/sphere.h) ----------------------------------------------

/// A sphere in 3D, mirroring `celmath::Sphere<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sphere {
    pub center: DVec3,
    pub radius: f64,
}

impl Sphere {
    /// A unit sphere centered at the origin.
    pub fn new(radius: f64) -> Self {
        Self {
            center: DVec3::ZERO,
            radius,
        }
    }

    /// A sphere with the given center and radius.
    pub fn with_center(center: DVec3, radius: f64) -> Self {
        Self { center, radius }
    }
}

// --- Ellipsoid (celmath/ellipsoid.h) ----------------------------------------

/// An ellipsoid in 3D, mirroring `celmath::Ellipsoid<T>`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ellipsoid {
    pub center: DVec3,
    /// Semi-axes along x, y, z.
    pub axes: DVec3,
}

impl Ellipsoid {
    /// A unit sphere (axes all 1) centered at the origin.
    pub fn new(axes: DVec3) -> Self {
        Self {
            center: DVec3::ZERO,
            axes,
        }
    }

    /// An ellipsoid with the given center and semi-axes.
    pub fn with_center(center: DVec3, axes: DVec3) -> Self {
        Self { center, axes }
    }

    /// Test whether `p` lies inside the ellipsoid. Mirrors `Ellipsoid::contains`.
    pub fn contains(&self, p: DVec3) -> bool {
        let v = (p - self.center) / self.axes;
        v.dot(v) <= 1.0
    }
}

// --- Ray (celmath/ray.h) -----------------------------------------------------

/// A ray (an origin and a direction), mirroring
/// `Eigen::ParametrizedLine<T, 3>`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
}

impl Ray {
    pub fn new(origin: DVec3, direction: DVec3) -> Self {
        Self {
            origin,
            direction,
        }
    }

    /// A point on the ray at parameter `t`: `origin + t * direction`.
    pub fn point_at(&self, t: f64) -> DVec3 {
        self.origin + self.direction * t
    }

    /// Transform the ray by a rotation `q`. Mirrors
    /// `celmath::transformRay(ray, quaternion)`.
    pub fn transform_by_quat(&self, q: DQuat) -> Ray {
        Ray {
            origin: q * self.origin,
            direction: q * self.direction,
        }
    }

    /// Transform the ray by a linear `3x3` matrix. Mirrors
    /// `celmath::transformRay(ray, Matrix3)`.
    pub fn transform_by_mat3(&self, m: glam::DMat3) -> Ray {
        Ray {
            origin: m * self.origin,
            direction: m * self.direction,
        }
    }

    /// Transform the ray by a full `4x4` matrix. Mirrors
    /// `celmath::transformRay(ray, Matrix4)`.
    pub fn transform_by_mat4(&self, m: glam::DMat4) -> Ray {
        let o = DVec4::new(self.origin.x, self.origin.y, self.origin.z, 1.0);
        let d = DVec4::new(self.direction.x, self.direction.y, self.direction.z, 0.0);
        let o = m * o;
        let d = m * d;
        Ray {
            origin: o.truncate(),
            direction: d.truncate(),
        }
    }
}

// --- Plane / Frustum (celmath/frustum.h / .cpp) ----------------------------

/// The six frustum planes, in Celestia's order. Mirrors
/// `celmath::FrustumPlane`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrustumPlane {
    Bottom = 0,
    Top = 1,
    Left = 2,
    Right = 3,
    Near = 4,
    Far = 5,
}

/// The result of a frustum containment test. Mirrors `celmath::FrustumAspect`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrustumAspect {
    /// The object is entirely outside the frustum.
    Outside = 0,
    /// The object is entirely inside the frustum.
    Inside = 1,
    /// The object intersects the frustum.
    Intersect = 2,
}

/// A 3D plane, mirroring `Eigen::Hyperplane<f32, 3>`.
///
/// The plane equation is `normal · p + constant = 0`; [`Plane::signed_distance`]
/// returns `normal · p + constant`. Celestia builds planes with a normalized
/// normal, so `signed_distance` is a true signed distance (up to sign).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub normal: Vec3,
    pub constant: f32,
}

impl Plane {
    /// A plane with the given (un-normalized) normal and constant.
    pub fn new(normal: Vec3, constant: f32) -> Self {
        Self {
            normal,
            constant,
        }
    }

    /// `normal · p + constant`. Mirrors `Eigen::Hyperplane::signedDistance`.
    pub fn signed_distance(&self, p: Vec3) -> f32 {
        self.normal.dot(p) + self.constant
    }

    /// Signed distance to a double-precision point, computed at full precision.
    ///
    /// Celestia explicitly performs this at double precision and warns that a
    /// naive single-precision conversion is not an allowable optimization.
    pub fn signed_distance_d(&self, p: DVec3) -> f64 {
        f64::from(self.normal.x) * p.x
            + f64::from(self.normal.y) * p.y
            + f64::from(self.normal.z) * p.z
            + f64::from(self.constant)
    }

    fn normalized(&self) -> Plane {
        let n = self.normal.length();
        if n == 0.0 {
            return *self;
        }
        Plane {
            normal: self.normal / n,
            constant: self.constant / n,
        }
    }

    /// Transform the plane by a `3x3` matrix (isometry: rotate the normal, keep
    /// the constant). Mirrors `Frustum::transform(Matrix3f)`.
    pub fn transform_mat3(&self, m: Mat3) -> Plane {
        Plane {
            normal: m * self.normal,
            constant: self.constant,
        }
    }

    /// Transform the plane by a `4x4` matrix using the inverse-transpose of the
    /// linear part, then renormalize. Mirrors `Frustum::transform(Matrix4f)`.
    pub fn transform_mat4(&self, m: Mat4) -> Plane {
        // `inv_transpose * coeffs` where coeffs = (normal, constant).
        let inv_t = m.inverse().transpose();
        let coeffs = inv_t * Vec4::new(self.normal.x, self.normal.y, self.normal.z, self.constant);
        let n = coeffs.truncate().length();
        Plane {
            normal: (coeffs.truncate() / n).normalize_or_zero(),
            constant: coeffs.w / n,
        }
    }
}

/// A view frustum with six planes. Mirrors `celmath::Frustum`.
#[derive(Debug, Clone, Copy)]
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    /// Build a perspective frustum from a vertical field of view (radians), an
    /// aspect ratio, and near/far distances. Mirrors
    /// `Frustum(fov, aspect, near, far)`.
    pub fn new(fov: f32, aspect_ratio: f32, near_dist: f32, far_dist: f32) -> Self {
        let mut planes = [Plane::new(Vec3::ZERO, 0.0); 6];
        {
            let h = (fov * 0.5).tan();
            let w = h * aspect_ratio;
            init_planes(&mut planes, -w * near_dist, w * near_dist, h * near_dist, -h * near_dist, near_dist);
        }
        planes[FrustumPlane::Far as usize] = create_far_plane(far_dist);
        Self { planes }
    }

    /// Build a frustum from explicit left/right/top/bottom extents and a near
    /// distance. Mirrors `Frustum(left, right, top, bottom, near, far)`.
    pub fn from_extents(
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near_dist: f32,
        far_dist: f32,
    ) -> Self {
        let mut planes = [Plane::new(Vec3::ZERO, 0.0); 6];
        init_planes(&mut planes, left, right, top, bottom, near_dist);
        planes[FrustumPlane::Far as usize] = create_far_plane(far_dist);
        Self { planes }
    }

    /// The plane at index `which`. Mirrors `Frustum::plane(which)`.
    pub fn plane(&self, which: FrustumPlane) -> Plane {
        self.planes[which as usize]
    }

    /// Transform all six planes by a rotation. Mirrors `Frustum::transform(Matrix3f)`.
    pub fn transform_mat3(&mut self, m: Mat3) {
        for p in &mut self.planes {
            *p = p.transform_mat3(m);
        }
    }

    /// Transform all six planes by a full matrix. Mirrors `Frustum::transform(Matrix4f)`.
    pub fn transform_mat4(&mut self, m: Mat4) {
        for p in &mut self.planes {
            *p = p.transform_mat4(m);
        }
    }

    /// Test a point against all planes. Mirrors `Frustum::test(point)`.
    pub fn test(&self, point: Vec3) -> FrustumAspect {
        for plane in &self.planes {
            if plane.signed_distance(point) < 0.0 {
                return FrustumAspect::Outside;
            }
        }
        FrustumAspect::Inside
    }

    /// Test a sphere (single-precision center). Mirrors
    /// `Frustum::testSphere(Vector3f, float)`.
    pub fn test_sphere(&self, center: Vec3, radius: f32) -> FrustumAspect {
        let mut is_intersecting = false;
        for plane in &self.planes {
            let dist = plane.signed_distance(center);
            if dist < -radius {
                return FrustumAspect::Outside;
            }
            if dist <= radius {
                is_intersecting = true;
            }
        }
        if is_intersecting {
            FrustumAspect::Intersect
        } else {
            FrustumAspect::Inside
        }
    }

    /// Test a sphere (double-precision center). Celestia performs the
    /// distance calculation at double precision. Mirrors
    /// `Frustum::testSphere(Vector3d, double)`.
    pub fn test_sphere_d(&self, center: DVec3, radius: f64) -> FrustumAspect {
        let mut is_intersecting = false;
        for plane in &self.planes {
            let dist = plane.signed_distance_d(center);
            if dist < -radius {
                return FrustumAspect::Outside;
            }
            if dist <= radius {
                is_intersecting = true;
            }
        }
        if is_intersecting {
            FrustumAspect::Intersect
        } else {
            FrustumAspect::Inside
        }
    }
}

/// Initialize the four side planes + the near plane of a frustum, matching
/// `celmath::frustum.cpp`'s anonymous `init`.
fn init_planes(planes: &mut [Plane; 6], l: f32, r: f32, t: f32, b: f32, n: f32) {
    let normals = [
        Vec3::new(0.0, n, b), // Bottom
        Vec3::new(0.0, -n, -t), // Top
        Vec3::new(n, 0.0, l), // Left
        Vec3::new(-n, 0.0, -r), // Right
    ];
    for i in 0..4 {
        planes[i] = Plane::new(normals[i].normalize_or_zero(), 0.0);
    }
    planes[FrustumPlane::Near as usize] = Plane::new(Vec3::new(0.0, 0.0, -1.0), -n);
}

fn create_far_plane(f: f32) -> Plane {
    Plane::new(Vec3::new(0.0, 0.0, 1.0), f)
}

// --- Geometric helpers (celmath/geomutil.h) ---------------------------------

/// `XRot90` quaternion (a +90° rotation about the x-axis). Mirrors
/// `celmath::XRot90`.
pub fn x_rot_90() -> Quat {
    Quat::from_axis_angle(Vec3::X, 90_f32.to_radians())
}

/// `XRot180` quaternion. Mirrors `celmath::XRot180`.
pub fn x_rot_180() -> Quat {
    Quat::from_axis_angle(Vec3::X, 180_f32.to_radians())
}

/// `YRot180` quaternion. Mirrors `celmath::YRot180`.
pub fn y_rot_180() -> Quat {
    Quat::from_axis_angle(Vec3::Y, 180_f32.to_radians())
}

/// A `+radians` rotation about the x-axis, mirroring `celmath::XRotation`.
pub fn x_rotation(radians: f32) -> Quat {
    let (s, c) = sincos_f32(radians * 0.5);
    Quat::from_xyzw(s, 0.0, 0.0, c)
}

/// A `+radians` rotation about the y-axis, mirroring `celmath::YRotation`.
pub fn y_rotation(radians: f32) -> Quat {
    let (s, c) = sincos_f32(radians * 0.5);
    Quat::from_xyzw(0.0, s, 0.0, c)
}

/// A `+radians` rotation about the z-axis, mirroring `celmath::ZRotation`.
pub fn z_rotation(radians: f32) -> Quat {
    let (s, c) = sincos_f32(radians * 0.5);
    Quat::from_xyzw(0.0, 0.0, s, c)
}

/// Determine an orientation that makes the negative z-axis point from `from` to
/// `to`, with the y-axis pointing in the direction of the component of `up`
/// orthogonal to the z-axis. Mirrors `celmath::LookAt`.
pub fn look_at(from: DVec3, to: DVec3, up: DVec3) -> DQuat {
    let mut n = to - from;
    let n_len = n.length();
    if n_len > 0.0 {
        n = n / n_len;
    }
    let v = n.cross(up).normalize_or_zero();
    let u = v.cross(n);

    // Celestia builds a matrix with columns (v, u, -n) and returns the
    // conjugate of the resulting quaternion. Build the matrix as a flat
    // row-major array from the orthonormal basis and extract its rotation
    // quaternion. The matrix is orthonormal (v, u orthogonal unit vectors, n
    // their cross), so the rotation quaternion is well-defined.
    let neg_n = -n;
    // columns: col0 = v, col1 = u, col2 = -n; element (r,c) = col_c[r].
    let m = [
        v.x, u.x, neg_n.x, // row 0
        v.y, u.y, neg_n.y, // row 1
        v.z, u.z, neg_n.z, // row 2
    ];
    rotation_quat(&m)
}

/// Extract the rotation quaternion from an orthonormal `3x3` matrix stored in a
/// flat row-major array `m[r * 3 + c]`. A simplified equivalent of
/// `Eigen::Quaternion(M)`, which returns the rotation the matrix represents.
fn rotation_quat(m: &[f64; 9]) -> DQuat {
    let (xx, yy, zz) = (m[0], m[4], m[8]);
    let trace = xx + yy + zz;
    if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        DQuat::from_xyzw(
            (m[5] - m[7]) / s,
            (m[6] - m[1]) / s,
            (m[1] - m[5]) / s,
            0.25 * s,
        )
    } else if xx > yy && xx > zz {
        let s = (1.0 + xx - yy - zz).sqrt() * 2.0;
        DQuat::from_xyzw(
            0.25 * s,
            (m[1] + m[3]) / s,
            (m[6] + m[2]) / s,
            (m[5] - m[7]) / s,
        )
    } else if yy > zz {
        let s = (1.0 + yy - xx - zz).sqrt() * 2.0;
        DQuat::from_xyzw(
            (m[6] + m[2]) / s,
            0.25 * s,
            (m[1] + m[3]) / s,
            (m[5] + m[7]) / s,
        )
    } else {
        let s = (1.0 + zz - xx - yy).sqrt() * 2.0;
        DQuat::from_xyzw(
            (m[1] + m[3]) / s,
            (m[6] + m[2]) / s,
            0.25 * s,
            (m[5] - m[7]) / s,
        )
    }
}

// --- Intersections (celmath/intersect.h) ------------------------------------

/// Test whether a ray intersects a sphere, returning the distance to the
/// intersection. Mirrors `celmath::testIntersection(ray, Sphere, distance)`.
pub fn test_intersection_sphere(ray: Ray, sphere: Sphere, distance: &mut Option<f64>) -> bool {
    let diff = ray.origin - sphere.center;
    let s = 1.0 / square(sphere.radius);
    let a = ray.direction.dot(ray.direction) * s;
    let b = ray.direction.dot(diff) * s;
    let c = diff.dot(diff) * s - 1.0;
    let disc = b * b - a * c;
    if disc < 0.0 {
        return false;
    }

    let disc = disc.sqrt();
    let sol0 = (-b + disc) / a;
    let sol1 = (-b - disc) / a;

    if sol0 > 0.0 {
        if sol0 < sol1 || sol1 < 0.0 {
            *distance = Some(sol0);
        } else {
            *distance = Some(sol1);
        }
        true
    } else if sol1 > 0.0 {
        *distance = Some(sol1);
        true
    } else {
        false
    }
}

/// Test whether a ray intersects an ellipsoid, returning the distance to the
/// intersection. Mirrors `celmath::testIntersection(ray, Ellipsoid, distance)`.
pub fn test_intersection_ellipsoid(ray: Ray, e: Ellipsoid, distance: &mut Option<f64>) -> bool {
    let diff = ray.origin - e.center;
    let s = DVec3::new(
        1.0 / (e.axes.x * e.axes.x),
        1.0 / (e.axes.y * e.axes.y),
        1.0 / (e.axes.z * e.axes.z),
    );
    let sdir = ray.direction * s;
    let sdiff = diff * s;

    let a = ray.direction.dot(sdir);
    let b = ray.direction.dot(sdiff);
    let c = diff.dot(sdiff) - 1.0;
    let disc = b * b - a * c;
    if disc < 0.0 {
        return false;
    }

    let disc = disc.sqrt();
    let sol0 = (-b + disc) / a;
    let sol1 = (-b - disc) / a;

    if sol0 > 0.0 {
        if sol0 < sol1 || sol1 < 0.0 {
            *distance = Some(sol0);
        } else {
            *distance = Some(sol1);
        }
        true
    } else if sol1 > 0.0 {
        *distance = Some(sol1);
        true
    } else {
        false
    }
}

// --- Numeric solvers (celmath/solve.h) --------------------------------------

/// Solve a function using the bisection method. Returns `(solution, error)`.
/// Mirrors `celmath::solve_bisection`.
pub fn solve_bisection<F>(mut f: F, lower: f64, upper: f64, err: f64, max_iter: usize) -> (f64, f64)
where
    F: FnMut(f64) -> f64,
{
    let mut x = 0.0;
    let mut lo = lower;
    let mut hi = upper;
    for _ in 0..max_iter {
        x = (lo + hi) * 0.5;
        if hi - lo < 2.0 * err {
            break;
        }
        let y = f(x);
        if y < 0.0 {
            lo = x;
        } else {
            hi = x;
        }
    }
    (x, (hi - lo) / 2.0)
}

/// Solve using fixed-point iteration; terminate when the step is below `err` or
/// the maximum iteration count is reached. Returns `(solution, error)`. Mirrors
/// `celmath::solve_iteration`.
pub fn solve_iteration<F>(mut f: F, x0: f64, err: f64, max_iter: usize) -> (f64, f64)
where
    F: FnMut(f64) -> f64,
{
    let mut x = 0.0;
    let mut x2 = x0;
    for _ in 0..max_iter {
        x = x2;
        x2 = f(x);
        if (x2 - x).abs() < err {
            return (x2, x2 - x);
        }
    }
    (x2, x2 - x)
}

/// Solve using fixed-point iteration for a fixed number of steps. Returns
/// `(solution, error)`. Mirrors `celmath::solve_iteration_fixed`.
pub fn solve_iteration_fixed<F>(mut f: F, x0: f64, max_iter: usize) -> (f64, f64)
where
    F: FnMut(f64) -> f64,
{
    let mut x = 0.0;
    let mut x2 = x0;
    for _ in 0..max_iter {
        x = x2;
        x2 = f(x);
    }
    (x2, x2 - x)
}

// --- Distance (celmath/distance.h) ------------------------------------------

/// Distance from a point `p` to a ray `r`. Mirrors `celmath::distance(p, r)`.
pub fn distance_to_ray(p: DVec3, r: Ray) -> f64 {
    let t = (p - r.origin).dot(r.direction) / r.direction.dot(r.direction);
    if t <= 0.0 {
        (p - r.origin).length()
    } else {
        (p - r.point_at(t)).length()
    }
}

// --- Random utilities (celmath/randutils.h) ---------------------------------

/// A point uniformly distributed on the unit circle. Mirrors
/// `celmath::randomOnCircle`. `next` is a closure yielding a uniform value in
/// `[0, 1)`, matching Celestia's `std::mt19937`-backed distributions without a
/// hard dependency on a random crate.
pub fn random_on_circle(mut next: impl FnMut() -> f64) -> glam::DVec2 {
    use std::f64::consts::PI as PI64;
    // Uniform in [-pi, pi).
    let phi = next() * (2.0 * PI64) - PI64;
    glam::DVec2::new(phi.cos(), phi.sin())
}

/// A point uniformly distributed on the unit sphere. Mirrors
/// `celmath::randomOnSphere`. `next` is a closure yielding a uniform value in
/// `[0, 1)`.
pub fn random_on_sphere(mut next: impl FnMut() -> f64) -> DVec3 {
    use std::f64::consts::PI as PI64;
    let phi = next() * (2.0 * PI64) - PI64;
    // Uniform in [-1, 1) for cos(theta).
    let cos_theta = next() * 2.0 - 1.0;
    let xy_scale = (1.0 - square(cos_theta)).sqrt();
    DVec3::new(xy_scale * phi.cos(), xy_scale * phi.sin(), cos_theta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_helpers() {
        assert!((deg_to_rad(180.0) - PI).abs() < 1e-12);
        assert!((rad_to_deg(PI) - 180.0).abs() < 1e-9);
        assert_eq!(square(3.0), 9.0);
        assert_eq!(cube(2.0), 8.0);
        assert_eq!(sign(-1.5), -1.0);
        assert_eq!(sign(0.0), 0.0);
        assert_eq!(sign(1.5), 1.0);
        // pfmod is always in [0, y): -1 mod 3 == 2, and 7 mod 3 == 1.
        assert!((pfmod(-1.0, 3.0) - 2.0).abs() < 1e-9);
        assert!((pfmod(7.0, 3.0) - 1.0).abs() < 1e-9);
        assert!((circle_area(1.0) - PI).abs() < 1e-12);
        assert!((sphere_area(1.0) - 4.0 * PI).abs() < 1e-12);
    }

    #[test]
    fn ellipsoid_contains() {
        let e = Ellipsoid::with_center(DVec3::ZERO, DVec3::new(2.0, 2.0, 2.0));
        assert!(e.contains(DVec3::new(1.0, 1.0, 1.0)));
        assert!(!e.contains(DVec3::new(3.0, 0.0, 0.0)));
    }

    #[test]
    fn ray_sphere_intersection() {
        // A ray from the origin along +x hits a sphere centered at (5,0,0) r=1.
        let ray = Ray::new(DVec3::ZERO, DVec3::new(1.0, 0.0, 0.0));
        let sphere = Sphere::with_center(DVec3::new(5.0, 0.0, 0.0), 1.0);
        let mut dist = None;
        let hit = test_intersection_sphere(ray, sphere, &mut dist);
        assert!(hit);
        // Nearest intersection is at x = 4.
        assert!((dist.unwrap() - 4.0).abs() < 1e-9);
    }

    #[test]
    fn ray_misses_sphere() {
        let ray = Ray::new(DVec3::ZERO, DVec3::new(0.0, 0.0, 1.0));
        let sphere = Sphere::with_center(DVec3::new(5.0, 0.0, 0.0), 1.0);
        let mut dist = None;
        assert!(!test_intersection_sphere(ray, sphere, &mut dist));
    }

    #[test]
    fn frustum_contains_origin() {
        // A forward-looking frustum (camera at origin looking down -z) has its
        // near plane 1 unit in front of the origin, so the origin itself is
        // *behind* the near plane and therefore OUTSIDE. A point 10 units ahead
        // on the axis is inside.
        let frustum = Frustum::new(60_f32.to_radians(), 16.0 / 9.0, 1.0, 1000.0);
        assert!(matches!(frustum.test(Vec3::ZERO), FrustumAspect::Outside));
        assert!(matches!(frustum.test(Vec3::new(0.0, 0.0, -10.0)), FrustumAspect::Inside));
        // A far-away point is also inside (it is in front of the far plane at
        // 1000 units).
        assert!(matches!(frustum.test(Vec3::new(0.0, 0.0, -500.0)), FrustumAspect::Inside));
    }

    #[test]
    fn solve_bisection_finds_root() {
        // Find x such that x^2 - 2 = 0 in [0, 2]; expect sqrt(2).
        let (x, err) = solve_bisection(|x| x * x - 2.0, 0.0, 2.0, 1e-9, 200);
        assert!(err < 1e-7);
        assert!((x - 2.0_f64.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn solve_iteration_fixed_point() {
        // Converge to the fixed point of x -> (x + 2/x)/2 (sqrt(2)).
        let (x, _) = solve_iteration(|x| 0.5 * (x + 2.0 / x), 1.0, 1e-12, 100);
        assert!((x - 2.0_f64.sqrt()).abs() < 1e-8);
    }

    #[test]
    fn test_distance_to_ray() {
        let ray = Ray::new(DVec3::ZERO, DVec3::new(1.0, 0.0, 0.0));
        // Point (5, 3, 0): perpendicular distance to the x-axis ray is 3.
        assert!((distance_to_ray(DVec3::new(5.0, 3.0, 0.0), ray) - 3.0).abs() < 1e-9);
        // Point behind the origin: distance is to the origin, sqrt(4 + 9) = sqrt(13).
        let behind = distance_to_ray(DVec3::new(-2.0, 3.0, 0.0), ray);
        assert!((behind - 13.0_f64.sqrt()).abs() < 1e-9);
    }
}
