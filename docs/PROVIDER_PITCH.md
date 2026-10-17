# Pitch: Energy-Aware API Throttling for LLM Providers

## To: OpenAI, Anthropic, Google DeepMind, Mistral, Together AI, Fireworks

---

**Subject: You're throttling by tokens. Throttle by Joules instead.**

Hi,

You're already doing rate limiting. Tokens per minute, requests per minute.
But tokens aren't your cost. Joules are.

A 100-token query on a MoE model burns 3x the energy of the same tokens on
a dense model. Your token-based throttling can't see this. You're leaving
money and sustainability claims on the table.

**We built the fix:** invariant-ledger — open-source (Apache 2.0) per-workload
Joule attribution. We measured 1.69 J/token on commodity hardware. Real numbers.

**What we're proposing:**

Replace token counters with Joule counters in your rate limiter. Same
infrastructure, honest unit. The smoothstep governor degrades gracefully —
smaller model, queue, not hard reject.

**What you get:**

1. **True cost control** — throttle the actual kWh driver, not a proxy
2. **"Green tier" pricing** — 16,900 J/day for $5/mo. A product you can sell tomorrow
3. **Auditable sustainability data** — CSRD-ready, no estimates, measured per query
4. **Model efficiency incentive** — efficient architectures get more headroom per Joule

**What we need from you:**

A 30-minute technical call. We'll show you the daemon running on your
infrastructure in under an hour. No commitment, just measurements.

**Proof, not promises:**
- Repo: github.com/zedwiater/invariant-ledger
- Demo: 138 tokens, 232.68 J, 1.69 J/token average (published with caveats)
- Spec: docs/PROVIDER_RATE_LIMITER.md (full architecture)

We're not another AI company. We're the measurement layer the industry is missing.

Best,
Zak
zedwiater/invariant-ledger

---

## Follow-up (if no response in 7 days)

**Subject: Re: Joule-based throttling — the numbers**

Quick follow-up with a concrete number:

If your average query is 500 tokens at 1.69 J/token, that's 845 J per query.
At $0.12/kWh, you're paying $0.000028 per query in electricity alone.

Multiply by 100M queries/day. That's $2,800/day in power you're not attributing
to individual users or models.

We can fix that in an afternoon.

---

## Target Contacts

| Provider | Contact | Angle |
|----------|---------|-------|
| OpenAI | Platform partnerships | Cost at GPT scale |
| Anthropic | Infrastructure team | Efficiency-first brand alignment |
| Google DeepMind | Cloud AI team | CSRD compliance + GCP integration |
| Mistral | Business development | EU green tech positioning |
| Together AI | Founders (direct) | Startup speed, no bureaucracy |
| Fireworks AI | Founders (direct) | Inference-optimized, energy matters |
