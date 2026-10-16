//! M_0 Ledger: closed-loop energy accounting.
//!
//! Every Joule in must equal every Joule out plus stored.
//! M_0 = E_in - E_out - E_stored ≡ 0 (within measurement error).
//!
//! From the wave-resonance cavity simulations: the ledger is the invariant.
//! ΔE = pump_in − accounted. Condensed matter is counted, never deleted.

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// The universal energy ledger.
#[derive(Debug, Clone)]
pub struct M0Ledger {
    /// Total Joules consumed (from hardware telemetry)
    pub e_in: f64,
    /// Joules attributed to cgroups/workloads
    pub e_out: f64,
    /// Timestamp of ledger creation
    pub started: Instant,
    /// Per-cgroup attribution
    pub by_cgroup: HashMap<String, f64>,
}

impl M0Ledger {
    pub fn new() -> Self {
        Self {
            e_in: 0.0,
            e_out: 0.0,
            started: Instant::now(),
            by_cgroup: HashMap::new(),
        }
    }

    /// Record energy consumed (from telemetry source).
    pub fn charge_in(&mut self, joules: f64) {
        self.e_in += joules;
    }

    /// Attribute energy to a cgroup/workload.
    pub fn charge_out(&mut self, joules: f64, cgroup: &str) {
        self.e_out += joules;
        *self.by_cgroup.entry(cgroup.to_string()).or_insert(0.0) += joules;
    }

    /// Ledger balance: should be ≈ 0. Positive = unattributed (leak).
    /// Negative = over-attributed (double-counting).
    pub fn balance(&self) -> f64 {
        self.e_in - self.e_out
    }

    /// Balance as fraction of total in. < 1% is good.
    pub fn balance_fraction(&self) -> f64 {
        if self.e_in == 0.0 {
            0.0
        } else {
            self.balance().abs() / self.e_in
        }
    }

    pub fn audit(&self) -> AuditReport {
        AuditReport {
            e_in: self.e_in,
            e_out: self.e_out,
            balance: self.balance(),
            balance_pct: self.balance_fraction() * 100.0,
            uptime: self.started.elapsed(),
            n_cgroups: self.by_cgroup.len(),
            top_consumers: self.top_consumers(10),
        }
    }

    fn top_consumers(&self, n: usize) -> Vec<(String, f64)> {
        let mut v: Vec<_> = self.by_cgroup.iter().collect();
        v.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        v.into_iter().take(n).map(|(k, v)| (k.clone(), *v)).collect()
    }
}

#[derive(Debug, serde::Serialize)]
pub struct AuditReport {
    pub e_in: f64,
    pub e_out: f64,
    pub balance: f64,
    pub balance_pct: f64,
    pub uptime: Duration,
    pub n_cgroups: usize,
    pub top_consumers: Vec<(String, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_balances() {
        let mut l = M0Ledger::new();
        l.charge_in(100.0);
        l.charge_out(60.0, "web");
        l.charge_out(40.0, "db");
        assert!((l.balance()).abs() < 1e-9);
    }

    #[test]
    fn detects_leak() {
        let mut l = M0Ledger::new();
        l.charge_in(100.0);
        l.charge_out(90.0, "web");
        assert!((l.balance() - 10.0).abs() < 1e-9); // 10J unattributed
    }
}
