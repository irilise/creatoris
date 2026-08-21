use bevy::math::DVec3;
use bevy::prelude::*;

pub struct OrbitalMathPlugin;

impl Plugin for OrbitalMathPlugin {
    fn build(&self, _app: &mut App) {}
}

/// Standard Keplerian Orbital Elements
#[derive(Component, Debug, Clone)]
pub struct OrbitalElements {
    pub semi_major_axis: f64, // a (km)
    pub eccentricity: f64,    // e (0 <= e < 1)
    pub inclination: f64,     // i (radians)
    pub long_asc_node: f64,   // Ω (radians)
    pub arg_periapsis: f64,   // ω (radians)
    pub mean_anomaly_0: f64,  // M0 at epoch (radians)
    pub mean_motion: f64,     // n (radians/second)
}

impl OrbitalElements {
    /// Computes the 3D position of the body at a given simulation time.
    pub fn position_at_time(&self, time: f64) -> DVec3 {
        // Mean anomaly at time t
        let m = (self.mean_anomaly_0 + self.mean_motion * time) % std::f64::consts::TAU;
        
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
        // Using standard Z-X-Z Euler rotations inverted for coordinate space.
        let cw = self.arg_periapsis.cos();
        let sw = self.arg_periapsis.sin();
        let co = self.long_asc_node.cos();
        let so = self.long_asc_node.sin();
        let ci = self.inclination.cos();
        let si = self.inclination.sin();

        let x = (co * cw - so * sw * ci) * x_orb + (-co * sw - so * cw * ci) * y_orb;
        let y = (so * cw + co * sw * ci) * x_orb + (-so * sw + co * cw * ci) * y_orb;
        let z = (sw * si) * x_orb + (cw * si) * y_orb;

        // Note: Z is up/down in many aerospace applications, map to Bevy's Y-up if preferred.
        // We'll map Z to Y to make the solar plane horizontal in Bevy.
        DVec3::new(x, z, -y) 
    }
}