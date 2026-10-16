# invariant-ledger v0.1.0 — "Every Joule Accounted For"

**Know exactly how many Joules every AI query and containerized workload costs — zero specialized hardware required.**

## What It Does

`invariant-ledger` is a lightweight Rust daemon that maps hardware energy telemetry (Intel RAPL, AMD GPU, NVIDIA) to a closed-loop M₀ energy ledger, attributing real-time Joules to cgroups and containers at 1-second resolution.

Data centers know total facility power. They don't know what each workload costs. This closes that gap.

## Measured, Not Promised

Real demo on commodity hardware (AMD RX 6600 XT, Gemma 3 4B):

| Query | Tokens | Joules | J/token |
|---|---|---|---|
| "What is 2+2?" | 5 | 9.16J | 1.83 |
| "Explain quantum entanglement" | 67 | 76.86J | 1.15 |
| "Write Python sort function" | 66 | 146.65J | 2.22 |

**138 tokens. 232.68 Joules. 1.69 J/token average.**

No power meters. No PDU access. Just `/sys/class/hwmon` and the M₀ ledger.

Full results with caveats: [`demo/RESULTS.md`](demo/RESULTS.md)

## What's Inside

- **M₀ Ledger** (`src/ledger.rs`): Closed-loop accounting. `E_in - E_out = balance`. Unattributed energy shows as leak, never silently zero.
- **Telemetry** (`src/telemetry/`): Intel RAPL, AMD GPU hwmon, mock (explicit `--mock` only — no silent fallback in production).
- **Attribution** (`src/attribution/`): cgroup v2 CPU-proportional Joule mapping.
- **Governor** (`src/governor/`): Smoothstep thermal throttle (from cavity viscosity operator).
- **Export** (`src/export/`): Prometheus-native metrics.

## Metrics

```
invariant_joules_total
invariant_joules_by_cgroup{cgroup="..."}
invariant_ledger_balance_joules
invariant_ledger_balance_percent
```

## Quickstart

```bash
cargo build --release
sudo cp target/release/invariant-ledger /usr/local/bin/
sudo systemctl enable --now invariant-ledger
curl localhost:9090/metrics | grep invariant_joules
```

## Design Principles

1. **Fail loud, never silent.** No telemetry? Hard error unless `--mock` or `INVARIANT_MOCK=1`. A phantom zero destroys trust.
2. **Leak > zero.** Unattributed energy is visible in the ledger balance. You see what you can't explain.
3. **Commodity hardware.** No specialized power meters. If Linux can see it, we can measure it.
4. **~600 lines.** One binary. No runtime dependencies beyond libc.

## Roadmap

- **v0.2**: High-frequency tracing hook for per-token LLM attribution
- **v0.3**: eBPF per-syscall attribution (socket → exact Joules, no proportional guessing)

## From Physics to Production

This daemon implements the M₀ energy ledger proven in 4D cavity wave simulations. The same invariant that balanced topological soliton energy now balances your server's power bill.

Software defines the geometry of what should happen. Hardware is the work of it actually happening. This daemon watches the work.

---

**License:** Apache 2.0
**Repo:** https://github.com/zedwiater/invariant-ledger
