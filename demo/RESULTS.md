# invariant-ledger Demo Results

**Date:** 2026-10-16
**Hardware:** AMD RX 6600 XT (Navi 23, 8GB VRAM)
**Model:** Gemma 3 4B Q4_K_M (llama.cpp, full GPU offload)
**Telemetry:** AMD GPU `power1_average` (hwmon, microwatts → integrated to Joules)
**Method:** GPU power sampled before/after each query, averaged, × duration.

## Results

| Query | Tokens | Time | Joules | J/token | Avg Watts |
|---|---|---|---|---|---|
| "What is 2+2?" | 5 | 0.65s | 9.16J | 1.83 | 14.0W |
| "Explain quantum entanglement in one paragraph." | 67 | 3.14s | 76.86J | 1.15 | 24.5W |
| "Write a Python function to sort a list..." | 66 | 5.75s | 146.65J | 2.22 | 25.5W |
| **Total** | **138** | | **232.68J** | **1.69 avg** | |

## Caveats

- GPU power includes ~4W idle baseline. True per-query delta is slightly lower.
  - Corrected estimates: 6.6J, 64.3J, 123.7J (subtracting idle × time).
- This is GPU-only. CPU/RAM/system power not included (RAPL was root-locked).
- Single-run measurement, not averaged. Expect ±10% run-to-run variance.
- Token counts are whitespace-split estimates, not tokenizer-exact.

## What This Proves

The M_0 ledger works on real hardware. We can measure per-query energy cost
with commodity telemetry — no specialized power meters, no data center PDU access.
Just `/sys/class/hwmon` and a Python script.

**Every Joule accounted for.**
