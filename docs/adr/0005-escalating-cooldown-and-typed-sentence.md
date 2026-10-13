# 0005. Escalating cooldown and a typed sentence for unlocks

Accepted on 2026-10-12.

## Context

A hard lock that can never be released would be abandoned the first time it blocked something genuinely urgent, so the system needs an escape hatch. The hatch must still cost enough that an impulse fades before it succeeds, and it must resist being worn down by repetition, because a fixed five minute wait becomes routine after a few uses.

## Decision

Unlocking is a three stage process owned by the state machine in `focus-core::unlock`. A request starts a cooldown that escalates through 5, 10, 20 and 40 minutes within a day and resets at local midnight. When the cooldown ends, the person has a short confirmation window of 5 minutes to retype a fixed sentence with paste disabled in the UI, and then chooses an unlock of 15, 30 or 60 minutes. The constants `COOLDOWN_STEPS_MINUTES`, `UNLOCK_SENTENCE`, `UNLOCK_DURATIONS_MINUTES` and `CONFIRM_WINDOW_MINUTES` define the policy. Relocking is always available and immediate, because ending a loosening early never weakens the person's intent.

## Consequences

Each repeated unlock in a day is more expensive than the last, which targets the pattern of chipping away at a rule, and the typed sentence forces a moment of deliberate attention. Genuine emergencies remain possible, at the price of a wait of up to 40 minutes. Because paste prevention is only a UI measure, the sentence is a friction device and not a security control, and the daemon validates the phrase and duration itself. Tuning the policy is a matter of changing constants and tests in one file.
