# 0010. Raising or removing a daily limit always needs an unlock

Accepted on 2026-10-12.

## Context

[ADR 0004](0004-strictness-enforced-in-daemon.md) made every loosening wait for an unlock while enforcement is active. That rule fits sites, apps and schedule windows, which are blocked or not according to the clock, but it fits daily limits badly. A limit is spent by use during the day, so a person who has used up an allowance is blocked by the exhausted record, and a person who is still under the allowance is not blocked by anything. In the second case enforcement is not active, the loosening rule did not apply, and the limit could be raised or removed for free just before it would have bitten. The same gap existed after hours, since a schedule that ended at 19:00 let anyone remove the limits that would have applied the next morning.

## Decision

The daemon treats a change that raises or removes a daily limit as locked unless an unlock is active, whatever the schedule says. `Loosening::is_limit_change` identifies those changes, and `locked_loosenings` in `focus-core::enforcement` applies the rule for every request, so the app has no other path. Adding a limit and lowering one are not loosenings and stay free, because they make the rules stricter. Every other loosening keeps the earlier rule and is locked only while the enforcement plan is active.

A locked change has to be reachable, so requesting an unlock works whenever a change is locked, including outside schedule windows, where enforcement is inactive and an unlock would otherwise have nothing to relax. The unlock flow is unchanged, with the same escalating cooldown, typed sentence and chosen duration.

## Consequences

A limit now carries the same cost to weaken as a block does, so the quiet hours before a limit is reached are no longer a window for removing it. The price is that a person who sincerely wants to raise a limit at a calm moment pays the cooldown too, which is the intended friction, and the interface has to explain that an unlock is needed even when nothing looks blocked. The rule lives in one function that the daemon calls and the interface only mirrors for its warnings, so a future loosening of another kind can opt in by extending the `is_limit_change` check or replacing it with a richer classification. A raise also frees a target that has already run out, so the same rule now protects the exhausted record as well.
