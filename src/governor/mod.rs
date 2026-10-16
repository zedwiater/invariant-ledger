//! Smoothstep thermal governor.
//!
//! From cavity viscosity: selective throttling that protects normal
//! operation and ablates thermal runaway. The smoothstep function
//! provides a continuous transition between "no throttle" and
//! "full throttle" — no discontinuities, no oscillation.
//!
//! Mirrors visc_mode="smoothstep" from cavity_orbits.py:
//!   W = smoothstep((g - g_lo) / (g_hi - g_lo))
//! where g is the thermal gradient signal.

/// Smoothstep: 0 below lo, 1 above hi, smooth S-curve between.
pub fn smoothstep(x: f64, lo: f64, hi: f64) -> f64 {
    if x <= lo {
        return 0.0;
    }
    if x >= hi {
        return 1.0;
    }
    let t = (x - lo) / (hi - lo);
    t * t * (3.0 - 2.0 * t)
}

/// Thermal governor: returns throttle factor 0.0 (none) to 1.0 (full).
///
/// - temp_c: current temperature in Celsius
/// - t_lo: temperature where throttling begins (e.g., 70°C)
/// - t_hi: temperature where full throttling applies (e.g., 90°C)
pub fn thermal_throttle(temp_c: f64, t_lo: f64, t_hi: f64) -> f64 {
    smoothstep(temp_c, t_lo, t_hi)
}

/// Relative gradient detector (from cavity viscosity).
/// Returns |dT/dt| / T — high values indicate thermal spike, not drift.
pub fn thermal_gradient(temp_now: f64, temp_prev: f64, dt_secs: f64) -> f64 {
    if dt_secs <= 0.0 || temp_prev <= 0.0 {
        return 0.0;
    }
    let dtemp = (temp_now - temp_prev).abs();
    dtemp / dt_secs / temp_prev
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothstep_boundaries() {
        assert_eq!(smoothstep(60.0, 70.0, 90.0), 0.0);
        assert_eq!(smoothstep(95.0, 70.0, 90.0), 1.0);
        let mid = smoothstep(80.0, 70.0, 90.0);
        assert!((mid - 0.5).abs() < 1e-9);
    }
}
