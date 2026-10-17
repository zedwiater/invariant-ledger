# Mobile Governor Spec (On-Device Joule Throttling)

## Status: DEFERRED (provider-level is the priority)

This spec is retained for future mobile integration. The provider-tier
rate limiter (docs/PROVIDER_RATE_LIMITER.md) is the primary target.

## Concept

Same smoothstep governor from invariant-ledger, running on-device.
Counts Joules (via BatteryStats + NPU power models), not tokens.

## Consumer Joule Budget

| Metric | Value |
|--------|-------|
| Phone battery | 54,000 J (15 Wh) |
| Mobile NPU efficiency | ~0.3-0.5 J/token (estimated) |
| Typical daily AI use | 5,000 tokens → 1,500-2,500 J |
| % of battery | 3-5% (invisible to user) |
| Complaint threshold | >10% (>5,400 J/day) |

## Governor Behavior

- < 5% battery for AI: unrestricted
- 5-8%: defer background inference, prefer on-device small models
- 8-10%: queue non-urgent, notify user
- > 10%: hard throttle background, foreground only

## Integration Points

- Android: BatteryStats + NNAPI power hints
- iOS: ProcessInfo.thermalState + CoreML compute units

## Why Deferred

Provider-level throttling captures the value with 1/Nth the integration effort.
One provider integration = millions of users governed. Revisit mobile when
on-device models dominate (2027+).
