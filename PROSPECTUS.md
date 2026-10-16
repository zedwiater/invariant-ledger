# invariant-ledger: Pitch & Prospectus

## The One-Liner

**Your data center knows total power. It doesn't know what each AI query costs. We fix that.**

## The Problem ($)

- Data centers spend 40-60% of OpEx on power, but can't attribute Joules to individual workloads.
- AI inference is the fastest-growing power draw. Nobody can tell you what a single query costs.
- Sustainability reporting (CSRD, SEC climate disclosure) demands per-workload carbon accounting. Current tools give you facility-level estimates.
- Cloud GPU billing is per-hour, not per-Joule. You're flying blind on actual energy efficiency.

## The Solution

`invariant-ledger`: a ~600-line Rust daemon that reads hardware energy counters (already in your servers) and attributes Joules to cgroups/containers in real time.

- **Zero new hardware.** Uses Intel RAPL, AMD hwmon, NVIDIA NVML — already in your fleet.
- **1-second resolution.** Per-container, per-query energy attribution.
- **Closed-loop.** Every Joule is accounted for or flagged as leak. No phantom zeros.
- **Prometheus-native.** Drops into your existing Grafana stack.

## Proof (Not Promises)

Measured on a $200 GPU (RX 6600 XT) running Gemma 3 4B:

> **138 tokens cost 232.68 Joules. That's 1.69 J/token.**

No lab equipment. No PDU taps. Just Linux sysfs and our ledger.

## Who Needs This

| Role | Pain | invariant-ledger answers |
|---|---|---|
| **ML Platform Lead** | "Which model/config is most energy-efficient?" | J/token per model, per deployment |
| **Data Center Ops** | "What's driving our power bill?" | Per-container Joules, real-time |
| **Sustainability Officer** | "CSRD needs per-workload carbon data" | Auditable energy ledger with <1% unattributed |
| **FinOps** | "GPU hours don't reflect actual cost" | True per-Joule cost allocation |
| **CTO** | "Is our AI strategy sustainable?" | Energy per capability, trendable |

## Business Model (Open)

- **Open source core** (Apache 2.0): telemetry, ledger, Prometheus export
- **Enterprise**: eBPF per-syscall attribution, multi-node aggregation, compliance dashboards, SLA support
- **Consulting**: deployment, custom integrations, energy optimization audits

## Traction

- ✅ Working daemon (Rust, ~600 lines)
- ✅ Real measurements published (demo/RESULTS.md)
- ✅ Prometheus/Grafana compatible
- 🔄 eBPF Phase 2 (per-syscall precision)
- 🔄 Multi-node fleet aggregation

## The Wave Behind It

`invariant-ledger` implements the M₀ energy ledger from wave-resonance physics research — the same closed-loop invariant that balanced energy in 4D cavity simulations of topological solitons. When the math balances at the scale of the universe, it balances in your data center.

This isn't a pivot. It's the first production deployment of a deeper principle: **every Joule accounted for, at every scale.**

## Contact

- GitHub: https://github.com/zedwiater/invariant-ledger
- Research: https://github.com/zedwiater/Wave-resonance

---

*The wave has left the simulator. It's in your server room now.*
