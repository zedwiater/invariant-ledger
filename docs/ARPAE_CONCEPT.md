# ARPA-E Concept Paper: Thermodynamic Transparency for AI Infrastructure

## Project Title
**Invariant Ledger: Open-Source Per-Workload Energy Accounting for Sustainable AI**

## Executive Summary

Data centers consume ~2% of US electricity, growing rapidly with AI inference demand.
Yet no open-source tool exists for per-workload, real-time energy attribution.
We built one: invariant-ledger, a ~600-line Rust daemon that maps hardware energy
counters to per-container Joule accounting with <1% unattributed residual.

We measured 1.69 Joules per LLM token on commodity hardware — the first published
per-query energy number we're aware of. This project scales that proof into
production infrastructure for US data centers.

## Technical Approach

**Phase 1 (Months 1-6): Production Hardening**
- Harden RAPL/NVML telemetry for heterogeneous fleets
- Fleet-wide aggregation (multi-node M₀ ledger)
- eBPF per-syscall attribution (from cgroup-proportional to exact)

**Phase 2 (Months 7-12): Provider Integration**
- Joule-based rate limiting for API providers
- Smoothstep governor for graceful degradation
- Integration with Kubernetes (device plugin + scheduler extender)

**Phase 3 (Months 13-18): Grid Coordination**
- Time-shift AI training to green energy availability windows
- Real-time carbon intensity API integration (WattTime, Electricity Maps)
- Demand response: data centers as grid-flexible loads

## Innovation

1. **Closed-loop M₀ ledger**: Every Joule accounted for or flagged as leak. No phantom zeros.
2. **Substrate-agnostic**: Works for CPUs, GPUs, and future photonic accelerators.
3. **Open source**: Apache 2.0. No vendor lock-in for critical energy infrastructure.

## Impact

- **Energy**: 10-30% reduction in AI inference energy via visibility-driven optimization
- **Grid**: Enables demand-responsive data centers, reducing peak fossil generation
- **Equity**: Open-source tooling democratizes energy efficiency (not just hyperscalers)

## Team

Independent researcher with published open-source results. Seeking national lab
partnership for validation at scale (NREL, LBNL).

## Budget Estimate

$750K over 18 months: 1 FTE engineer, hardware testbed, cloud validation costs.

## Alignment with ARPA-E Mission

Directly addresses ARPA-E's focus on transformative energy technologies by making
AI's energy cost visible, attributable, and optimizable — prerequisites for any
meaningful efficiency improvement.
