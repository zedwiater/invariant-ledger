//! Hardware telemetry sources.
//!
//! Each source implements the TelemetrySource trait, returning Joules.
//! RAPL: Intel Running Average Power Limit (MSR_PKG_ENERGY_STATUS)
//! AMD: hwmon power1_average (amdgpu)
//! NVIDIA: NVML power draw

use anyhow::{Context, Result};
use std::fs;
use std::time::{Duration, Instant};

pub trait TelemetrySource: Send {
    /// Read cumulative Joules since last call (delta).
    fn read_joules(&mut self) -> Result<f64>;
    /// Sampling resolution of this source.
    fn resolution(&self) -> Duration;
    fn name(&self) -> &'static str;
}

/// Intel RAPL via /sys/class/powercap/intel-rapl
pub struct RaplSource {
    path: String,
    last_uj: u64,
    last_read: Instant,
    max_uj: u64,
}

impl RaplSource {
    pub fn new() -> Result<Self> {
        // Find first RAPL package
        let base = "/sys/class/powercap/intel-rapl:0/energy_uj";
        let max_path = "/sys/class/powercap/intel-rapl:0/max_energy_range_uj";

        let uj: u64 = fs::read_to_string(base)
            .context("RAPL not available (need intel-rapl sysfs)")?
            .trim().parse()?;
        let max_uj: u64 = fs::read_to_string(max_path)
            .unwrap_or_else(|_| "1000000000000".to_string())
            .trim().parse().unwrap_or(1_000_000_000_000);

        Ok(Self {
            path: base.to_string(),
            last_uj: uj,
            last_read: Instant::now(),
            max_uj,
        })
    }
}

impl TelemetrySource for RaplSource {
    fn read_joules(&mut self) -> Result<f64> {
        let uj: u64 = fs::read_to_string(&self.path)?.trim().parse()?;
        // Handle wraparound
        let delta_uj = if uj >= self.last_uj {
            uj - self.last_uj
        } else {
            (self.max_uj - self.last_uj) + uj
        };
        self.last_uj = uj;
        self.last_read = Instant::now();
        Ok(delta_uj as f64 / 1e6) // microjoules -> joules
    }

    fn resolution(&self) -> Duration {
        Duration::from_millis(1)
    }

    fn name(&self) -> &'static str {
        "intel-rapl"
    }
}

/// AMD GPU via hwmon
pub struct AmdgpuSource {
    path: String,
    last_w: f64,
    last_read: Instant,
}

impl AmdgpuSource {
    pub fn new() -> Result<Self> {
        // Find first amdgpu hwmon with power1_average
        for entry in fs::read_dir("/sys/class/hwmon")? {
            let entry = entry?;
            let name_path = entry.path().join("name");
            if let Ok(name) = fs::read_to_string(&name_path) {
                if name.trim() == "amdgpu" {
                    let p = entry.path().join("power1_average");
                    if p.exists() {
                        let uw: u64 = fs::read_to_string(&p)?.trim().parse()?;
                        return Ok(Self {
                            path: p.to_string_lossy().to_string(),
                            last_w: uw as f64 / 1e6,
                            last_read: Instant::now(),
                        });
                    }
                }
            }
        }
        anyhow::bail!("No amdgpu hwmon found")
    }
}

impl TelemetrySource for AmdgpuSource {
    fn read_joules(&mut self) -> Result<f64> {
        let uw: u64 = fs::read_to_string(&self.path)?.trim().parse()?;
        let w = uw as f64 / 1e6;
        let dt = self.last_read.elapsed().as_secs_f64();
        let joules = w * dt; // P * dt (using current power as average)
        self.last_w = w;
        self.last_read = Instant::now();
        Ok(joules)
    }

    fn resolution(&self) -> Duration {
        Duration::from_millis(100)
    }

    fn name(&self) -> &'static str {
        "amdgpu-hwmon"
    }
}

/// Mock source for testing (no hardware needed)
pub struct MockSource {
    rate_w: f64,
    last: Instant,
}

impl MockSource {
    pub fn new(rate_w: f64) -> Self {
        Self { rate_w, last: Instant::now() }
    }
}

impl TelemetrySource for MockSource {
    fn read_joules(&mut self) -> Result<f64> {
        let dt = self.last.elapsed().as_secs_f64();
        self.last = Instant::now();
        Ok(self.rate_w * dt)
    }

    fn resolution(&self) -> Duration {
        Duration::from_millis(10)
    }

    fn name(&self) -> &'static str {
        "mock"
    }
}
