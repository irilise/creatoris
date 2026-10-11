//! Celestia ephemeris core: orbits and rotation models.
//!
//! Faithful Rust port of Celestia's `celephem` library (`orbit.h`/`orbit.cpp`,
//! `rotation.h`). Orbits are parameterized by time as Julian dates (TDB) and
//! return positions in kilometers; velocities are in kilometers per day.
//!
//! This module is pure Rust (no Bevy) so it can be unit-tested in isolation.
#![allow(missing_docs)]

use glam::DVec3;

use super::astro;

/// The reference epoch used by most orbits (J2000).
pub const DEFAULT_EPOCH: f64 = astro::J2000;

/// Velocity is differentiated over one minute (1/1440 of a day) for orbits that
/// don't override `velocity_at_time`, matching Celestia's
/// `ORBITAL_VELOCITY_DIFF_DELTA`.
const ORBITAL_VELOCITY_DIFF_DELTA: f64 = 1.0 / 1440.0;

/// Minimum bounding radius for hyperbolic orbits: 1000 AU.
const HYPERBOLIC_MIN_BOUNDING_RADIUS: f64 = 1000.0 * astro::KM_PER_AU;

// --- Orbit sampling ----------------------------------------------------------

/// A callback that receives orbit samples, mirroring `OrbitSampleProc`.
pub trait OrbitSampleProc {
    fn sample(&mut self, t: f64, p: DVec3, v: DVec3);
}

/// A vector of sampled `(time, position, velocity)` tuples, the default
/// `OrbitSampleProc` target.
#[derive(Debug, Default, Clone)]
pub struct SampledOrbit {
    pub times: Vec<f64>,
    pub positions: Vec<DVec3>,
    pub velocities: Vec<DVec3>,
}

impl SampledOrbit {
    pub fn new() -> Self {
        Self::default()
    }
}

impl OrbitSampleProc for SampledOrbit {
    fn sample(&mut self, t: f64, p: DVec3, v: DVec3) {
        self.times.push(t);
        self.positions.push(p);
        self.velocities.push(v);
    }
}

// --- Orbit trait (celphem/orbit.h: Orbit) ------------------------------------

/// An orbit describes the position of a body in its reference frame as a
/// function of time (TDB, Julian date).
///
/// The `Send + Sync + Debug` bounds let orbits be stored inside Bevy resources
/// (which must be thread-safe and debug-printable), mirroring how Celestia
/// stores orbits inside `Body` objects held by the `Universe`.
pub trait Orbit: Send + Sync + std::fmt::Debug {
    /// Position in the orbit's reference frame at time `tdb` (Julian date), in
    /// kilometers.
    fn position_at_time(&self, tdb: f64) -> DVec3;

    /// Velocity in km/day at `tdb`. Defaults to a finite-difference of
    /// `position_at_time`.
    fn velocity_at_time(&self, tdb: f64) -> DVec3 {
        let p0 = self.position_at_time(tdb);
        let p1 = self.position_at_time(tdb + ORBITAL_VELOCITY_DIFF_DELTA);
        (p1 - p0) * (1.0 / ORBITAL_VELOCITY_DIFF_DELTA)
    }

    /// The orbital period in days (0 if aperiodic).
    fn get_period(&self) -> f64;

    /// A radius (km) that bounds the orbit; used for frustum culling.
    fn get_bounding_radius(&self) -> f64;

    /// Whether the orbit is periodic.
    fn is_periodic(&self) -> bool {
        true
    }

    /// Sample the orbit over `[start_time, end_time]` into `proc`.
    fn sample(&self, start_time: f64, end_time: f64, proc: &mut dyn OrbitSampleProc) {
        let span = if self.is_periodic() {
            self.get_period()
        } else {
            end_time - start_time
        };
        let tolerance = 1.0; // km
        let start_step = span / 1.0e5;
        let max_step = span / 100.0;
        let min_step = span / 1.0e7;

        let mut t = start_time;
        let mut last_p = self.position_at_time(t);
        let mut last_v = self.velocity_at_time(t);
        proc.sample(t, last_p, last_v);

        while t < end_time {
            let mut dt = (max_step.min(start_step * 2.0)).min(end_time - t);
            let p1 = self.position_at_time(t + dt);
            let v1 = self.velocity_at_time(t + dt);
            let t_mid = t + dt / 2.0;
            let p_test = self.position_at_time(t_mid);
            let p_interp = cubic_interpolate(last_p, last_v * dt, p1, v1 * dt, 0.5);
            let position_error = (p_interp - p_test).length();

            if position_error > tolerance {
                while position_error > tolerance && dt > min_step {
                    dt /= 1.25;
                    let p1 = self.position_at_time(t + dt);
                    let v1 = self.velocity_at_time(t + dt);
                    let t_mid = t + dt / 2.0;
                    let p_test = self.position_at_time(t_mid);
                    let p_interp = cubic_interpolate(last_p, last_v * dt, p1, v1 * dt, 0.5);
                    if (p_interp - p_test).length() <= tolerance {
                        break;
                    }
                }
            }

            t += dt;
            last_p = self.position_at_time(t);
            last_v = self.velocity_at_time(t);
            proc.sample(t, last_p, last_v);
        }
    }
}

/// Cubic Hermite interpolation used by adaptive sampling.
fn cubic_interpolate(p0: DVec3, v0: DVec3, p1: DVec3, v1: DVec3, t: f64) -> DVec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    p0 + (((2.0 * (p0 - p1) + v1 + v0) * t3) + ((3.0 * (p1 - p0) - 2.0 * v0 + v1) * t2) + (v0 * t))
}

// --- EllipticalOrbit (celphem/orbit.cpp) -------------------------------------

/// A bounded, periodic orbit described by Keplerian elements.
#[derive(Debug, Clone)]
pub struct EllipticalOrbit {
    semi_major_axis: f64,
    semi_minor_axis: f64,
    eccentricity: f64,
    mean_anomaly_at_epoch: f64,
    period: f64,
    epoch: f64,
    // Rotation that takes the orbital plane to Celestia's internal frame:
    // Z(Ω) · X(i) · Z(ω), stored as the rotation of the in-plane coordinates.
    // Celestia applies this via `orbitPlaneRotation · (x, y, 0)` then maps
    // (x, y, z) -> (x, z, -y).
    orbit_plane_rotation: [f64; 9],
}

impl EllipticalOrbit {
    /// Build an elliptical orbit from Keplerian elements. Mirrors the C++
    /// constructor, including its invariants.
    #[allow(clippy::new_ret_no_self)]
    pub fn new(elements: &astro::KeplerElements, epoch: f64) -> Option<Self> {
        let e = elements.eccentricity;
        if !(e >= 0.0 && e < 1.0) {
            return None;
        }
        let a = elements.semi_major_axis;
        if !(a >= 0.0) {
            return None;
        }
        if elements.period == 0.0 {
            return None;
        }

        // orbitPlaneRotation = Z(Ω) · X(i) · Z(ω). Compose the three
        // rotations explicitly (row-major 3×3).
        let om = elements.long_ascending_node;
        let i = elements.inclination;
        let w = elements.arg_pericenter;

        let cO = om.cos();
        let sO = om.sin();
        let ci = i.cos();
        let si = i.sin();
        let cw = w.cos();
        let sw = w.sin();

        // R = Rz(Ω) · Rx(i) · Rz(ω)
        // = [ cO*cw - sO*sw*ci,   -cO*sw - sO*cw*ci,   sO*si;
        //     sO*cw + cO*sw*ci,   -sO*sw + cO*cw*ci,  -cO*si;
        //     sw*si,               cw*si,               ci        ]
        let m = [
            cO * cw - sO * sw * ci,
            -cO * sw - sO * cw * ci,
            sO * si, sO * cw + cO * sw * ci,
            -sO * sw + cO * cw * ci,
            -cO * si, sw * si,
            cw * si,
            ci,
        ];

        Some(Self {
            semi_major_axis: a,
            semi_minor_axis: a * (1.0 - e * e).sqrt(),
            eccentricity: e,
            mean_anomaly_at_epoch: elements.mean_anomaly,
            period: elements.period,
            epoch,
            orbit_plane_rotation: m,
        })
    }

    /// Build from Kepler elements using the default J2000 epoch.
    pub fn from_elements(elements: &astro::KeplerElements) -> Option<Self> {
        Self::new(elements, DEFAULT_EPOCH)
    }

    /// Eccentric anomaly from mean anomaly. Selects the same three strategies
    /// Celestia uses, by eccentricity.
    fn eccentric_anomaly(&self, m: f64) -> f64 {
        if self.eccentricity == 0.0 {
            return m;
        }

        if self.eccentricity < 0.2 {
            // Standard Newton iteration: E <- E + (E - e·sin E - M)/(1 - e·cos E)
            let mut e_val = m;
            for _ in 0..5 {
                let err = e_val - self.eccentricity * e_val.sin() - m;
                let delta = err / (1.0 - self.eccentricity * e_val.cos());
                e_val -= delta;
            }
            return e_val;
        }

        if self.eccentricity < 0.9 {
            // Faster-converging iteration (Meeus 2nd ed, p.199).
            let mut e_val = m;
            for _ in 0..6 {
                let s = e_val.sin();
                let c = e_val.cos();
                let err = e_val + (m + self.eccentricity * s - e_val) / (1.0 - self.eccentricity * c);
                e_val = err;
            }
            return e_val;
        }

        // Laguerre-Conway method for very high eccentricity.
        let mut e_val = m + 0.85 * self.eccentricity * m.sin().signum();
        for _ in 0..8 {
            let s = self.eccentricity * e_val.sin();
            let c = self.eccentricity * e_val.cos();
            let f = e_val - s - m;
            let f1 = 1.0 - c;
            let f2 = s;
            let denom = f1 + f1.signum() * (16.0 * f1 * f1 - 20.0 * f * f2).abs().sqrt();
            e_val += -5.0 * f / denom;
        }
        e_val
    }

    /// Position in the orbit's reference frame at eccentric anomaly `E`.
    fn position_at_e(&self, e: f64) -> DVec3 {
        let x = self.semi_major_axis * (e.cos() - self.eccentricity);
        let y = self.semi_minor_axis * e.sin();
        let m = &self.orbit_plane_rotation;
        // Apply R · (x, y, 0).
        let px = m[0] * x + m[1] * y;
        let py = m[3] * x + m[4] * y;
        let pz = m[6] * x + m[7] * y;
        // Convert to Celestia's internal coordinate system: (x, y, z) -> (x, z, -y).
        DVec3::new(px, pz, -py)
    }

    /// Velocity at eccentric anomaly `E` with mean motion `n`.
    fn velocity_at_e(&self, e: f64, mean_motion: f64) -> DVec3 {
        let (sine, cos_e) = e.sin_cos();
        let edot = mean_motion / (1.0 - self.eccentricity * cos_e);
        let x = -self.semi_major_axis * sine * edot;
        let y = self.semi_minor_axis * cos_e * edot;
        let m = &self.orbit_plane_rotation;
        let px = m[0] * x + m[1] * y;
        let py = m[3] * x + m[4] * y;
        let pz = m[6] * x + m[7] * y;
        DVec3::new(px, pz, -py)
    }
}

impl Orbit for EllipticalOrbit {
    fn position_at_time(&self, tdb: f64) -> DVec3 {
        let mean_motion = 2.0 * std::f64::consts::PI / self.period;
        let m = self.mean_anomaly_at_epoch + mean_motion * (tdb - self.epoch);
        let e = self.eccentric_anomaly(m);
        self.position_at_e(e)
    }

    fn velocity_at_time(&self, tdb: f64) -> DVec3 {
        let mean_motion = 2.0 * std::f64::consts::PI / self.period;
        let m = self.mean_anomaly_at_epoch + mean_motion * (tdb - self.epoch);
        let e = self.eccentric_anomaly(m);
        self.velocity_at_e(e, mean_motion)
    }

    fn get_period(&self) -> f64 {
        self.period
    }

    fn get_bounding_radius(&self) -> f64 {
        self.semi_major_axis * (1.0 + self.eccentricity)
    }
}

// --- HyperbolicOrbit (celphem/orbit.cpp) -------------------------------------

/// A non-periodic orbit with eccentricity > 1.
#[derive(Debug, Clone)]
pub struct HyperbolicOrbit {
    semi_major_axis: f64, // negative
    semi_minor_axis: f64,
    eccentricity: f64,
    mean_anomaly_at_epoch: f64,
    period: f64,
    epoch: f64,
    orbit_plane_rotation: [f64; 9],
    // The time at which the body passes pericenter; the valid range is
    // [pericenter - margin, pericenter + margin].
    pericenter_time: f64,
    valid_range_start: f64,
    valid_range_end: f64,
}

impl HyperbolicOrbit {
    pub fn new(elements: &astro::KeplerElements, epoch: f64) -> Option<Self> {
        let e = elements.eccentricity;
        if !(e > 1.0) {
            return None;
        }
        let a = elements.semi_major_axis;
        if !(a < 0.0) {
            return None;
        }
        let period = elements.period;
        if period == 0.0 {
            return None;
        }

        let om = elements.long_ascending_node;
        let i = elements.inclination;
        let w = elements.arg_pericenter;
        let cO = om.cos();
        let sO = om.sin();
        let ci = i.cos();
        let si = i.sin();
        let cw = w.cos();
        let sw = w.sin();
        let m = [
            cO * cw - sO * sw * ci,
            -cO * sw - sO * cw * ci,
            sO * si, sO * cw + cO * sw * ci,
            -sO * sw + cO * cw * ci,
            -cO * si, sw * si,
            cw * si,
            ci,
        ];

        // Time of pericenter passage from the mean anomaly at epoch.
        // For a hyperbola, the hyperbolic mean anomaly M = e·sinh(H) - H.
        let hyperbolic_mean = elements.mean_anomaly;
        let h = solve_hyperbolic_anomaly(hyperbolic_mean, e);
        // M = e·sinh(H) - H, and mean motion n = 2π/period.
        // time since pericenter = M / n.
        let n = 2.0 * std::f64::consts::PI / period;
        let m_at_epoch = e * h.sinh() - h;
        let time_at_pericenter = epoch - m_at_epoch / n;

        // A reasonable validity window around pericenter.
        let margin = 3.0 * 365.25; // ±3 years
        Some(Self {
            semi_major_axis: a,
            semi_minor_axis: a * (e * e - 1.0).sqrt(),
            eccentricity: e,
            mean_anomaly_at_epoch: elements.mean_anomaly,
            period,
            epoch,
            orbit_plane_rotation: m,
            pericenter_time: time_at_pericenter,
            valid_range_start: time_at_pericenter - margin,
            valid_range_end: time_at_pericenter + margin,
        })
    }

    pub fn from_elements(elements: &astro::KeplerElements) -> Option<Self> {
        Self::new(elements, DEFAULT_EPOCH)
    }

    fn eccentric_anomaly(&self, tdb: f64) -> f64 {
        let mean_motion = 2.0 * std::f64::consts::PI / self.period;
        let m = self.mean_anomaly_at_epoch + mean_motion * (tdb - self.epoch);
        solve_hyperbolic_anomaly(m, self.eccentricity)
    }

    fn position_at_h(&self, h: f64) -> DVec3 {
        let x = self.semi_major_axis * (self.eccentricity * h.cosh() - 1.0);
        let y = self.semi_minor_axis * h.sinh();
        let m = &self.orbit_plane_rotation;
        let px = m[0] * x + m[1] * y;
        let py = m[3] * x + m[4] * y;
        let pz = m[6] * x + m[7] * y;
        DVec3::new(px, pz, -py)
    }

    fn velocity_at_h(&self, h: f64, h_dot: f64) -> DVec3 {
        let x = self.semi_major_axis * self.eccentricity * h.sinh() * h_dot;
        let y = self.semi_minor_axis * h.cosh() * h_dot;
        let m = &self.orbit_plane_rotation;
        let px = m[0] * x + m[1] * y;
        let py = m[3] * x + m[4] * y;
        let pz = m[6] * x + m[7] * y;
        DVec3::new(px, pz, -py)
    }
}

impl Orbit for HyperbolicOrbit {
    fn position_at_time(&self, tdb: f64) -> DVec3 {
        let h = self.eccentric_anomaly(tdb);
        self.position_at_h(h)
    }

    fn velocity_at_time(&self, tdb: f64) -> DVec3 {
        let h = self.eccentric_anomaly(tdb);
        let cosh_h = h.cosh();
        // M = e·sinh(H) - H => M' = e·cosh(H) - 1; H' = M' / (e·cosh(H) - 1).
        let mean_motion = 2.0 * std::f64::consts::PI / self.period;
        let h_dot = mean_motion / (self.eccentricity * cosh_h - 1.0);
        self.velocity_at_h(h, h_dot)
    }

    fn get_period(&self) -> f64 {
        self.period
    }

    fn get_bounding_radius(&self) -> f64 {
        HYPERBOLIC_MIN_BOUNDING_RADIUS
    }

    fn is_periodic(&self) -> bool {
        false
    }

    // Celestia's HyperbolicOrbit::getValidRange.
}

/// Newton iteration for the hyperbolic equation `M = e·sinh(H) - H`.
fn solve_hyperbolic_anomaly(m: f64, e: f64) -> f64 {
    let mut h = if m.abs() < 1.0 { m } else { (2.0 * m.abs()).ln() };
    for _ in 0..20 {
        let sinh_h = h.sinh();
        let cosh_h = h.cosh();
        let f = e * sinh_h - h - m;
        let f1 = e * cosh_h - 1.0;
        h -= f / f1;
    }
    h
}

// --- Rotation models (celphem/rotation.h) ------------------------------------

/// A rotation model describes the orientation of a body over time. The overall
/// orientation is `spin * equator`.
///
/// Like [`Orbit`], the `Send + Sync + Debug` bounds allow rotation models to be
/// held inside Bevy resources.
#[allow(missing_docs)]
pub trait RotationModel: Send + Sync + std::fmt::Debug {
    fn orientation_at_time(&self, tjd: f64) -> glam::DQuat;
    fn equator_orientation_at_time(&self, _tjd: f64) -> glam::DQuat {
        glam::DQuat::IDENTITY
    }
    fn spin(&self, tjd: f64) -> glam::DQuat;
    /// Rotation period in days (0 if aperiodic).
    fn get_period(&self) -> f64 {
        0.0
    }
}

/// A rotation model with a fixed orientation, mirroring `ConstantOrientation`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConstantOrientation {
    pub orientation: glam::DQuat,
}

impl ConstantOrientation {
    pub fn new(orientation: glam::DQuat) -> Self {
        Self { orientation }
    }
}

impl RotationModel for ConstantOrientation {
    fn orientation_at_time(&self, _tjd: f64) -> glam::DQuat {
        self.orientation
    }
    fn spin(&self, _tjd: f64) -> glam::DQuat {
        self.orientation
    }
}

/// A constant-period rotation about a fixed axis, mirroring
/// `FixedPeriodRotation` / `UniformRotationModel`.
#[derive(Debug, Clone, Copy)]
pub struct UniformRotation {
    /// Rotation period in days (0 => aperiodic / constant identity).
    pub period: f64,
    /// Rotation axis (unit vector in the reference frame).
    pub axis: glam::DVec3,
    /// Phase offset in radians.
    pub phase: f64,
}

impl UniformRotation {
    pub fn new(period: f64, axis: glam::DVec3) -> Self {
        Self {
            period,
            axis: axis.normalize(),
            phase: 0.0,
        }
    }
}

impl RotationModel for UniformRotation {
    fn orientation_at_time(&self, tjd: f64) -> glam::DQuat {
        self.spin(tjd)
    }

    fn spin(&self, tjd: f64) -> glam::DQuat {
        if self.period == 0.0 {
            return glam::DQuat::IDENTITY;
        }
        let angle = self.phase + 2.0 * std::f64::consts::PI * tjd / self.period;
        glam::DQuat::from_axis_angle(self.axis, angle)
    }

    fn get_period(&self) -> f64 {
        self.period
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earth_elements() -> astro::KeplerElements {
        astro::KeplerElements::from_planet_units(1.0, 0.0167, 0.0, 0.0, 102.9, 358.617, 1.0)
    }

    #[test]
    fn elliptical_orbit_period() {
        let orbit = EllipticalOrbit::from_elements(&earth_elements()).unwrap();
        assert!((orbit.get_period() - astro::DAYS_PER_YEAR).abs() < 1e-6);
        assert!(orbit.is_periodic());
    }

    #[test]
    fn elliptical_orbit_repeats() {
        let orbit = EllipticalOrbit::from_elements(&earth_elements()).unwrap();
        let p0 = orbit.position_at_time(astro::J2000);
        let p1 = orbit.position_at_time(astro::J2000 + orbit.get_period());
        assert!(
            (p0 - p1).length() < 1e-3,
            "periodic orbit should repeat: {p0} vs {p1}"
        );
    }

    #[test]
    fn elliptical_orbit_at_pericenter() {
        // Circular orbit: distance is constant = semi-major axis.
        let circular = astro::KeplerElements::from_planet_units(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let orbit = EllipticalOrbit::from_elements(&circular).unwrap();
        let p = orbit.position_at_time(astro::J2000);
        assert!((p.length() - astro::KM_PER_AU).abs() < 1e-6);
    }

    #[test]
    fn hyperbolic_orbit_not_periodic() {
        let hyper = astro::KeplerElements::from_planet_units(-1.0, 1.5, 0.0, 0.0, 0.0, 0.0, 1000.0);
        let orbit = HyperbolicOrbit::from_elements(&hyper).unwrap();
        assert!(!orbit.is_periodic());
        // Should produce a finite position.
        let p = orbit.position_at_time(astro::J2000);
        assert!(p.is_finite());
    }

    #[test]
    fn sampling_produces_points() {
        let orbit = EllipticalOrbit::from_elements(&earth_elements()).unwrap();
        let mut sampler = SampledOrbit::new();
        orbit.sample(astro::J2000, astro::J2000 + orbit.get_period(), &mut sampler);
        assert!(!sampler.positions.is_empty());
        assert!(sampler.positions.len() >= 2);
    }

    #[test]
    fn uniform_rotation_periodic() {
        let rot = UniformRotation::new(1.0, glam::DVec3::Z);
        assert!((rot.get_period() - 1.0).abs() < 1e-12);
        // A full 2π rotation about Z returns to the identity orientation
        // (up to the q / -q sign ambiguity of quaternions).
        let q = rot.spin(1.0);
        let identity = glam::DQuat::IDENTITY;
        let is_identity = (q.x - identity.x).abs() < 1e-9
            && (q.y - identity.y).abs() < 1e-9
            && (q.z - identity.z).abs() < 1e-9
            && (q.w - identity.w).abs() < 1e-9;
        let is_neg_identity = (q.x + identity.x).abs() < 1e-9
            && (q.y + identity.y).abs() < 1e-9
            && (q.z + identity.z).abs() < 1e-9
            && (q.w + identity.w).abs() < 1e-9;
        assert!(is_identity || is_neg_identity, "q = {q:?}");
    }
}
