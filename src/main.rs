//! invariant-ledger: Every Joule accounted for.
//!
//! A lightweight daemon that maps hardware telemetry (RAPL, GPU)
//! to a closed-loop M_0 energy ledger, attributing Joules to
//! cgroups in real time.

mod attribution;
mod export;
mod governor;
mod ledger;
mod telemetry;

use anyhow::Result;
use ledger::M0Ledger;
use log::{info, warn};
use std::time::Duration;
use telemetry::{MockSource, TelemetrySource};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("invariant-ledger v0.1.0 starting");

    // Initialize telemetry (try hardware, fall back to mock)
    let mut sources: Vec<Box<dyn TelemetrySource>> = vec![];
    match telemetry::RaplSource::new() {
        Ok(s) => {
            info!("Using Intel RAPL telemetry");
            sources.push(Box::new(s));
        }
        Err(e) => warn!("RAPL unavailable: {}, using mock", e),
    }
    match telemetry::AmdgpuSource::new() {
        Ok(s) => {
            info!("Using AMD GPU telemetry");
            sources.push(Box::new(s));
        }
        Err(_) => {}
    }
    if sources.is_empty() {
        info!("No hardware telemetry, using mock (100W)");
        sources.push(Box::new(MockSource::new(100.0)));
    }

    let mut ledger = M0Ledger::new();
    let metrics = export::Metrics::new()?;
    let mut prev_stats = attribution::read_cgroup_stats().unwrap_or_default();

    info!("Entering main loop (1s interval)");
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Read total Joules from all sources
        let mut total_j = 0.0;
        for s in sources.iter_mut() {
            match s.read_joules() {
                Ok(j) => total_j += j,
                Err(e) => warn!("Telemetry {} failed: {}", s.name(), e),
            }
        }
        ledger.charge_in(total_j);
        metrics.joules_total.inc_by(total_j);

        // Attribute to cgroups
        if let Ok(curr_stats) = attribution::read_cgroup_stats() {
            let attributed = attribution::attribute_joules(total_j, &prev_stats, &curr_stats);
            for (cg, j) in &attributed {
                ledger.charge_out(*j, cg);
                metrics.joules_by_cgroup.with_label_values(&[cg]).inc_by(*j);
            }
            prev_stats = curr_stats;
        }

        // Update ledger metrics
        metrics.ledger_balance.set(ledger.balance());
        metrics.ledger_balance_pct.set(ledger.balance_fraction() * 100.0);

        // Log audit every 60s
        static mut TICKS: u32 = 0;
        unsafe {
            TICKS += 1;
            if TICKS % 60 == 0 {
                let audit = ledger.audit();
                info!(
                    "M_0 audit: in={:.1}J out={:.1}J balance={:.3}J ({:.2}%)",
                    audit.e_in, audit.e_out, audit.balance, audit.balance_pct
                );
            }
        }
    }
}
