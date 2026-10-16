//! Prometheus metrics export.

use prometheus::{Encoder, IntCounter, Opts, Registry, TextEncoder};
use std::sync::Arc;

pub struct Metrics {
    pub registry: Registry,
    pub joules_total: prometheus::Counter,
    pub joules_by_cgroup: prometheus::CounterVec,
    pub ledger_balance: prometheus::Gauge,
    pub ledger_balance_pct: prometheus::Gauge,
}

impl Metrics {
    pub fn new() -> anyhow::Result<Arc<Self>> {
        let registry = Registry::new();

        let joules_total = prometheus::Counter::with_opts(
            Opts::new("invariant_joules_total", "Total Joules consumed"),
        )?;
        let joules_by_cgroup = prometheus::CounterVec::new(
            Opts::new("invariant_joules_by_cgroup", "Joules by cgroup"),
            &["cgroup"],
        )?;
        let ledger_balance = prometheus::Gauge::with_opts(
            Opts::new("invariant_ledger_balance_joules", "Unattributed Joules (should be ~0)"),
        )?;
        let ledger_balance_pct = prometheus::Gauge::with_opts(
            Opts::new("invariant_ledger_balance_percent", "Unattributed % (should be ~0)"),
        )?;

        registry.register(Box::new(joules_total.clone()))?;
        registry.register(Box::new(joules_by_cgroup.clone()))?;
        registry.register(Box::new(ledger_balance.clone()))?;
        registry.register(Box::new(ledger_balance_pct.clone()))?;

        Ok(Arc::new(Self {
            registry,
            joules_total,
            joules_by_cgroup,
            ledger_balance,
            ledger_balance_pct,
        }))
    }

    pub fn gather(&self) -> String {
        let encoder = TextEncoder::new();
        let mf = self.registry.gather();
        let mut buf = vec![];
        encoder.encode(&mf, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
