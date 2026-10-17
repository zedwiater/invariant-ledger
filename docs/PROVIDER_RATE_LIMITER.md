# Provider-Tier Joule Rate Limiter

## Spec: Energy-Aware API Throttling

### Problem

API providers (OpenAI, Anthropic, Google) throttle by tokens/min and requests/min.
These are proxy metrics. The real cost is Joules. A 100-token query on a
mixture-of-experts model costs 3x the energy of the same tokens on a dense model.
Token-based throttling is blind to this.

### Solution

Replace token counters with Joule counters in the rate limiter.
Same infrastructure, honest unit.

```
BEFORE: user_123 → 10,000 tokens/min → 429 when exceeded
AFTER:  user_123 → 16,900 Joules/min → 429 when exceeded (at 1.69 J/token avg)
```

### Architecture

```
Inference Fleet → invariant-ledger (per-node) → Aggregator → Rate Limiter
     (RAPL/hwmon)      (Joules/user)           (sum)         (smoothstep)
```

1. **Per-node ledger**: Each inference server runs invariant-ledger, attributing
   Joules to user IDs (via request headers, not cgroups).
2. **Aggregator**: Sums per-user Joules across fleet in real time (Redis/streaming).
3. **Governor**: Smoothstep throttle — not binary allow/deny, but graceful degradation:
   - < 70% budget: full speed, best model
   - 70-90%: queue non-urgent, suggest smaller model
   - 90-100%: smallest model only, background queue
   - > 100%: 429 with `Retry-After` and Joule reset time

### Pricing Tiers (Example)

| Tier | Joules/day | ~Tokens/day | Price |
|------|-----------|-------------|-------|
| Free | 1,690 J | ~1,000 | $0 |
| Green | 16,900 J | ~10,000 | $5/mo |
| Pro | 169,000 J | ~100,000 | $20/mo |
| Enterprise | Unlimited* | — | Custom |

*Throttled by smoothstep, not hard cap. Never fully blocked.

### API Changes

```http
X-Joules-Used: 4523.7
X-Joules-Limit: 16900
X-Joules-Reset: 2026-10-17T00:00:00Z
```

429 response includes:
```json
{
  "error": "joule_budget_exceeded",
  "joules_used": 16900,
  "joules_limit": 16900,
  "reset_at": "2026-10-17T00:00:00Z",
  "suggested_model": "gpt-4o-mini"
}
```

### Sustainability Reporting

Each provider gets auditable per-user, per-model, per-day Joule logs.
CSRD/SEC climate disclosure ready. No estimates — measured.

### Implementation Phases

- **v0.2**: Request-header Joule attribution (bypass cgroup, use user_id from auth)
- **v0.3**: Redis aggregator for fleet-wide per-user sums
- **v0.4**: Smoothstep governor with model degradation ladder
- **v0.5**: Billing integration (Stripe per-Joule metering)

### Why Providers Want This

1. **Cost control**: They pay per kWh. Now they can throttle the actual cost driver.
2. **New revenue**: "Green tier" is a product differentiator.
3. **Compliance**: Sustainability reporting without building it themselves.
4. **Efficiency incentive**: Rewards efficient models with more headroom per Joule.
