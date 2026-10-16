# invariant-ledger

**Know exactly how many Joules every AI query and containerized workload costs — zero specialized hardware required.**

A lightweight daemon that maps hardware telemetry (Intel RAPL, AMD GPU, NVIDIA) to a closed-loop M_0 energy ledger, attributing real-time Joules to cgroups and containers.

## Why

Data centers know total facility power but can't attribute Joules to individual workloads. `invariant-ledger` closes that gap:

- **Real-time** per-cgroup energy attribution (1-second resolution)
- **Closed-loop**: 100% of measured Joules attributed or flagged as leak
- **Zero-config**: auto-detects RAPL, amdgpu, NVML; falls back gracefully
- **Prometheus-native**: scrape `localhost:9090/metrics` out of the box
- **Low overhead**: <0.1% CPU (single-threaded, no polling storms)

## Quickstart

```bash
# Build (Rust 1.70+)
cargo build --release

# Install
sudo cp target/release/invariant-ledger /usr/local/bin/
sudo cp systemd/invariant-ledger.service /etc/systemd/system/
sudo systemctl enable --now invariant-ledger

# Watch the ledger
journalctl -u invariant-ledger -f | grep "M_0 audit"

# Prometheus
curl localhost:9090/metrics | grep invariant_joules
```

## How It Works

From wave-resonance physics: the ledger is the invariant.

```
Hardware Telemetry → M_0 Ledger → Per-Cgroup Attribution
     (RAPL)           (E_in)           (E_out by CPU share)
```

1. **Telemetry**: Read package energy from Intel RAPL (`/sys/class/powercap`), AMD GPU (`hwmon`), or NVIDIA (NVML).
2. **Ledger**: `M0Ledger` tracks `E_in` (total Joules) vs `E_out` (attributed). Balance should be ≈ 0.
3. **Attribution**: Apportion Joules to cgroups proportionally to CPU time delta (`cpu.stat`).
4. **Govern**: Smoothstep thermal throttling (from cavity viscosity operator) prevents runaway.

## Metrics

| Metric | Description |
|---|---|
| `invariant_joules_total` | Cumulative Joules consumed |
| `invariant_joules_by_cgroup` | Joules per cgroup (label: `cgroup`) |
| `invariant_ledger_balance_joules` | Unattributed Joules (should be ≈ 0) |
| `invariant_ledger_balance_percent` | Unattributed % (should be < 1%) |

## Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the M_0 ledger math and hardware mapping.

## From Physics to Production

This daemon implements the M_0 energy ledger proven in 4D cavity simulations ([Wave-resonance](https://github.com/zedwiater/Wave-resonance)). The same ledger that balanced topological soliton energy now balances your server's power bill.

Software defines the geometry of what should happen; hardware is the work of it actually happening. This daemon watches the work.

## License

Apache 2.0
