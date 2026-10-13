# 0006. TypeScript bindings are generated from Rust with ts-rs

Accepted on 2026-10-12.

## Context

The Tauri bridge serializes Rust structs to JSON that the React code consumes. Maintaining the matching TypeScript types by hand invites drift, where a renamed field compiles on both sides and fails at runtime, and a field added in Rust is forgotten in the UI. Because the UI is strict TypeScript without `any`, accurate types are the main line of defense on that boundary.

## Decision

Command DTOs in `app/src-tauri/src/dto/` derive `ts_rs::TS` behind the `ts` Cargo feature and are exported by `cargo test --all-features` into `app/src/ipc/generated`, with the directory set in `.cargo/config.toml`. The `focus-core` rule and status types that the UI renders (`Rules`, `Schedule`, `DaemonStatus`, `EnforcementView`, `UnlockView`, `IpcError` and `ErrorCode` among them) derive the same bindings through the `ts` feature of that crate. `scripts/gen-bindings.sh` regenerates them, and `scripts/check.sh` exports a fresh copy to a temporary directory and fails when it differs from the committed files. The Rust constants the UI needs, such as the rule caps, the limit bounds, the unlock timings and the retention bounds, are written by the same run as `constants.ts` and `retention.ts` and are checked for freshness the same way, so the UI never copies a number. Large integers are mapped to `number`, which is safe here because durations and timestamps stay far below 2^53. The daemon wire envelope types `Request`, `Response` and `Envelope` are not exported, since the UI never speaks the socket protocol. The status and error types inside them are exported, because the UI renders `DaemonStatus` as part of the `FocusStatus` DTO.

## Consequences

A change to a DTO that is not reflected in the committed bindings fails CI, so the Rust and TypeScript sides cannot diverge silently. The generated files are committed, which makes the diff visible in review, and they must never be edited by hand. The dependency on `ts-rs` adds a derive to each DTO and ties the export step to the test run, and a future move to a different generator would only touch the DTO layer.
