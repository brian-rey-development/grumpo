# 0004. Strictness is enforced in the daemon with ValidatedRules

Accepted on 2026-10-12.

## Context

The rule that tightening always works but loosening during enforcement requires an unlock is the core of the product. If the UI enforced it, a modified app or a script speaking the socket protocol could skip it, and if the daemon accepted rule payloads without a single validation point, each handler would need to remember the checks, which tends to fail over time.

## Decision

The daemon computes strictness itself. On every `UpdateRules` request, `focus-core` compares the old and new rules with `loosenings(old, new)`, which reports removed sites or apps, removed or raised limits and schedule minutes that are no longer covered. While enforcement is active and the unlock state is locked, any non-empty result is rejected with `loosening_locked`, except that raising or removing a daily limit is rejected whenever no unlock is active, as [ADR 0010](0010-raising-a-limit-always-needs-an-unlock.md) explains. Rules are first converted into a `ValidatedRules` value that proves bounds, ordering and uniqueness, and only that type is accepted by persistence and enforcement code, so unvalidated data cannot reach them by construction.

## Consequences

The UI cannot weaken the guarantee, and it merely reacts to `loosening_locked` by opening the unlock flow. The pure functions are easy to test exhaustively in `focus-core`, including edge cases such as partially covered schedule windows. The cost is that the UI may let a person edit a draft that the daemon will then refuse, so the interface has to explain rejections clearly, and every new rule field must extend both validation and the loosening comparison.
