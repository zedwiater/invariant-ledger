#!/usr/bin/env python3
"""invariant-ledger demo: measure Joules per LLM query.

Uses the M_0 ledger principle: E_in (RAPL) - E_out (attributed) = balance.
Runs a real query against Gemma on :8080, measures RAPL delta.

This is the Python proof-of-concept. The Rust daemon (invariant-ledger)
is the production version with cgroup attribution and Prometheus.
"""
import json
import time
import urllib.request

RAPL_PATH = "/sys/class/powercap/intel-rapl:0/energy_uj"
AMDGPU_PATH = "/sys/class/hwmon/hwmon1/power1_average"
GEMMA_URL = "http://localhost:8080/v1/chat/completions"


def read_rapl_uj():
    """Read RAPL energy in microjoules."""
    try:
        with open(RAPL_PATH) as f:
            return int(f.read().strip())
    except (FileNotFoundError, PermissionError) as e:
        return None


def read_amdgpu_uw():
    """Read AMD GPU power in microwatts (readable without root)."""
    try:
        with open(AMDGPU_PATH) as f:
            return int(f.read().strip())
    except (FileNotFoundError, PermissionError):
        return None


def query_gemma(prompt, max_tokens=50):
    """Run a query against Gemma, return response and timing."""
    data = json.dumps({
        "model": "gemma",
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": max_tokens,
        "stream": False,
    }).encode()
    req = urllib.request.Request(
        GEMMA_URL, data=data,
        headers={"Content-Type": "application/json"})
    t0 = time.time()
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            result = json.loads(resp.read())
        dt = time.time() - t0
        text = result["choices"][0]["message"]["content"]
        tokens = len(text.split())  # rough
        return text, tokens, dt
    except Exception as e:
        return None, 0, 0


def main():
    print("=== invariant-ledger demo: Joules per query ===", flush=True)

    # Try RAPL first, fall back to AMD GPU power integration
    use_rapl = read_rapl_uj() is not None
    use_amdgpu = read_amdgpu_uw() is not None

    if not use_rapl and not use_amdgpu:
        print(json.dumps({"error": "No telemetry available"}), flush=True)
        return

    telemetry = "RAPL" if use_rapl else "AMDGPU"
    print(f"Telemetry: {telemetry}", flush=True)

    # Test queries (increasing complexity)
    queries = [
        ("What is 2+2?", 10),
        ("Explain quantum entanglement in one paragraph.", 80),
        ("Write a Python function to sort a list and explain how it works.", 150),
    ]

    results = []
    for prompt, max_tok in queries:
        print(f"\nQuery: {prompt[:40]}...", flush=True)

        # Measure energy delta
        if use_rapl:
            e_before = read_rapl_uj()
        else:
            w_before = read_amdgpu_uw()
            t_before = time.time()

        text, tokens, dt = query_gemma(prompt, max_tok)

        if use_rapl:
            e_after = read_rapl_uj()
            duj = e_after - e_before
            if duj < 0:
                duj += 2**63
            joules = duj / 1e6
        else:
            w_after = read_amdgpu_uw()
            t_after = time.time()
            # Integrate: average power × time
            w_avg = ((w_before + w_after) / 2) / 1e6  # uW -> W
            joules = w_avg * (t_after - t_before)

        if text is None:
            print("  Gemma query failed, skipping", flush=True)
            continue
        j_per_token = joules / max(tokens, 1)

        r = {
            "prompt": prompt[:50],
            "tokens": tokens,
            "seconds": round(dt, 2),
            "joules": round(joules, 4),
            "joules_per_token": round(j_per_token, 6),
            "watts_avg": round(joules / max(dt, 0.01), 2),
        }
        results.append(r)
        print(f"  {tokens} tokens, {dt:.1f}s, {joules:.4f}J ({j_per_token:.6f} J/token)",
              flush=True)

    # Summary
    if results:
        total_j = sum(r["joules"] for r in results)
        total_tok = sum(r["tokens"] for r in results)
        print(f"\n=== M_0 Ledger Summary ===", flush=True)
        print(f"Total: {total_j:.4f}J for {total_tok} tokens", flush=True)
        print(f"Average: {total_j/max(total_tok,1):.6f} J/token", flush=True)

    print(json.dumps({"results": results}, indent=2), flush=True)
    print("Demo done", flush=True)


if __name__ == "__main__":
    main()
