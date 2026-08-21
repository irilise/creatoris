use bevy::math::DVec3;
use bevy::prelude::*;

pub struct OrbitalMathPlugin;

impl Plugin for OrbitalMathPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Standard Keplerian Orbital Elements
#[derive(Component, Debug, Clone)]
pub struct OrbitalElements {
    pub semi_major_axis: f64, // a (scaled units, 15,000 = 1 AU)
    pub eccentricity: f64,    // e (0 <= e < 1)
    pub inclination: f64,     // i (radians)
    pub long_asc_node: f64,   // Ω (radians)
    pub arg_periapsis: f64,   // ω (radians)
    pub mean_anomaly_0: f64,  // M0 at epoch (radians)
    pub mean_motion: f64,     // n (radians/second)
}

impl OrbitalElements {
    pub const AU_SCALE: f64 = 15_000.0;

    /// Create orbital elements with angles in degrees and period in Earth days.
    pub fn from_keplerian_degrees(
        semi_major_axis_au: f64,
        eccentricity: f64,
        inclination_deg: f64,
        long_asc_node_deg: f64,
        arg_periapsis_deg: f64,
        period_days: f64,
        mean_anomaly_0_deg: f64,
    ) -> Self {
        let period_seconds = period_days * 86_400.0;
        let mean_motion = std::f64::consts::TAU / period_seconds;
        Self {
            semi_major_axis: semi_major_axis_au * Self::AU_SCALE,
            eccentricity,
            inclination: inclination_deg.to_radians(),
            long_asc_node: long_asc_node_deg.to_radians(),
            arg_periapsis: arg_periapsis_deg.to_radians(),
            mean_anomaly_0: mean_anomaly_0_deg.to_radians(),
            mean_motion,
        }
    }

    /// Create relative orbital elements (e.g. for Moon around Earth) with raw distance units and period in days.
    pub fn from_relative_orbit(
        semi_major_axis_units: f64,
        eccentricity: f64,
        inclination_deg: f64,
        long_asc_node_deg: f64,
        arg_periapsis_deg: f64,
        period_days: f64,
        mean_anomaly_0_deg: f64,
    ) -> Self {
        let period_seconds = period_days * 86_400.0;
        let mean_motion = std::f64::consts::TAU / period_seconds;
        Self {
            semi_major_axis: semi_major_axis_units,
            eccentricity,
            inclination: inclination_deg.to_radians(),
            long_asc_node: long_asc_node_deg.to_radians(),
            arg_periapsis: arg_periapsis_deg.to_radians(),
            mean_anomaly_0: mean_anomaly_0_deg.to_radians(),
            mean_motion,
        }
    }

    /// Orbital period in Earth days.
    pub fn period_days(&self) -> f64 {
        (std::f64::consts::TAU / self.mean_motion) / 86_400.0
    }

    /// Orbital period in Earth years.
    pub fn period_years(&self) -> f64 {
        self.period_days() / 365.25
    }

    /// Semi-major axis in Astronomical Units (AU).
    pub fn semi_major_axis_au(&self) -> f64 {
        self.semi_major_axis / Self::AU_SCALE
    }

    /// Computes the 3D position of the body at a given simulation time.
    pub fn position_at_time(&self, time: f64) -> DVec3 {
        // Mean anomaly at time t
        let m = (self.mean_anomaly_0 + self.mean_motion * time).rem_euclid(std::f64::consts::TAU);
        
        // Solve Kepler's Equation for Eccentric Anomaly (E) using Newton-Raphson
        let mut e_ecc = m;
        for _ in 0..10 {
            let delta = (e_ecc - self.eccentricity * e_ecc.sin() - m) / (1.0 - self.eccentricity * e_ecc.cos());
            e_ecc -= delta;
            if delta.abs() < 1e-7 {
                break;
            }
        }

        // Position in the 2D orbital plane
        let x_orb = self.semi_major_axis * (e_ecc.cos() - self.eccentricity);
        let y_orb = self.semi_major_axis * (1.0 - self.eccentricity.powi(2)).sqrt() * e_ecc.sin();

        // Rotate by argument of periapsis (ω), inclination (i), and longitude of ascending node (Ω)
        let cw = self.arg_periapsis.cos();
        let sw = self.arg_periapsis.sin();
        let co = self.long_asc_node.cos();
        let so = self.long_asc_node.sin();
        let ci = self.inclination.cos();
        let si = self.inclination.sin();

        let x = (co * cw - so * sw * ci) * x_orb + (-co * sw - so * cw * ci) * y_orb;
        let y = (so * cw + co * sw * ci) * x_orb + (-so * sw + co * cw * ci) * y_orb;
        let z = (sw * si) * x_orb + (cw * si) * y_orb;

        // Map Z to Y to make the solar plane horizontal in Bevy
        DVec3::new(x, z, -y) 
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_earth_orbital_period() {
        let earth = OrbitalElements::from_keplerian_degrees(
            1.0, 0.0167, 0.0, 0.0, 102.9, 365.256, 0.0,
        );
        assert!((earth.period_days() - 365.256).abs() < 1e-3);
        assert!((earth.period_years() - 1.0).abs() < 1e-3);
        assert!((earth.semi_major_axis_au() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_orbital_period_repetition() {
        let earth = OrbitalElements::from_keplerian_degrees(
            1.0, 0.0167, 0.0, 0.0, 0.0, 365.25, 0.0,
        );
        let period_secs = 365.25 * 86_400.0;
        let pos_0 = earth.position_at_time(0.0);
        let pos_1_year = earth.position_at_time(period_secs);

        assert!((pos_0.x - pos_1_year.x).abs() < 1e-4);
        assert!((pos_0.y - pos_1_year.y).abs() < 1e-4);
        assert!((pos_0.z - pos_1_year.z).abs() < 1e-4);
    }
}