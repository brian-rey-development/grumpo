# Grumpo

Grumpo is a local-first, strict focus blocker and screen-time tracker, and its tagline is "Grumpo says no. You get things done." It replaces the `focus-hosts` bash LaunchDaemon with a Rust system that shows where time goes per app and per website, and enforces a weekly schedule, blocked sites and apps, and daily limits in a way that resists impulse bypassing. Nothing leaves the machine, so there are no network calls, no analytics and no remote favicons.

The product loop is the whole point, and every decision below serves it. You see your time, you set a rule, the rule holds, and breaking it costs a deliberate pause.

This file holds the scope, the product decisions, the conventions and the quality gates. The structure of the code, the installed layout and the IPC contract are described once, in [docs/architecture.md](docs/architecture.md), and the reasoning behind the major choices is in [docs/adr](docs/adr/README.md).

## Scope of v1

In scope on macOS, the product has the following parts.

- Tracking the frontmost app every 2 s, plus the active tab host in Chrome, Brave, Edge and Safari (AppleScript) and Firefox (Accessibility, best effort). Only `app id + app name + domain` is stored, never URLs or window titles.
- Pausing after 2 min without input unless a display-sleep assertion shows media is playing. Sleep and wake gaps close the open session.
- A weekly schedule of blocking windows. During a window the daemon writes blocked sites into a marked `/etc/hosts` block, the app quits blocked apps and shows the block screen, and it closes a blocked tab that is open in the frontmost window of Chrome, Brave, Edge or Safari. Blocked apps running behind other windows are found by a sweep that repeats every 30 s while blocking is active. Firefox tabs cannot be closed, so Firefox relies on the hosts block.
- Daily limits per site or app. One warning at 5 min left, then the target is blocked until local midnight.
- A block screen that appears when an app is quit or a tab is closed, naming the target, counting down to the end of the block and saying whether strict mode is on. A notification replaces it only when the window cannot be shown.
- Strictness, which means tightening rules always works and loosening while enforcement is active requires an unlock. Raising or removing a daily limit needs an unlock whatever the schedule says.
- Protected apps (Grumpo, Finder, the login window, System Settings and the Dock) that can never be blocked or limited.
- Private browsing in Chrome, Brave and Edge is never stored, and counts toward limits through an in-memory tally that a restart forgets.
- Unlock by escalating cooldown (5, 10, 20, 40 min, reset at local midnight), then retyping a sentence with paste disabled, then choosing 15, 30 or 60 min. "Relock now" is always available.
- Retention of 90 days by default (configurable), JSON export, Delete history.
- Migrating the legacy `focus-hosts` domain list and schedule, then removing it.
- A menu bar item with state and today's total, a main window that hides on close, and launch at login.

Out of scope for v1 are Linux and Windows implementations (the ports exist and return `Unsupported`), a Firefox extension, defense against a clock set forward or against a user with sudo, sync, and notarization or a Developer ID.

## Product decisions

These are the decisions that shape behavior and are easy to undo by accident. Each is implemented once and explained in the linked document.

- **Limits are protected by their own rule.** Adding or lowering a daily limit is free. Raising or removing one needs an active unlock whatever the schedule says, and an unlock can be requested whenever a requested change is locked, including outside schedule windows. Every other loosening is locked only while the plan is active. See [ADR 0010](docs/adr/0010-raising-a-limit-always-needs-an-unlock.md).
- **Aliases are blocked with the site.** Each blocked domain is written to the hosts file for the prefixes `""`, `www.`, `m.`, `mobile.`, `old.`, `new.`, `music.`, `i.` and `amp.`, in IPv4 and IPv6. The list is the one constant `ALIAS_PREFIXES` in `focus-core`.
- **The clock only helps in one direction.** An unlock that holds more time than it was granted is relocked, a cooldown that holds more than its step restarts, and day counters reset only when the date moves forward.
- **Limit reports carry the day.** `ReportLimitReached` includes the reporter's local `day` (protocol version 3), and the daemon ignores a report for any other day.
- **A state file that cannot be used is quarantined, and one that cannot be read stops the daemon.** Only damaged JSON, a missing version and rules that no longer validate move the file aside. Read errors and a newer schema make the daemon exit non-zero, so launchd retries and the old hosts block stays.
- **A damaged app database is replaced, a newer one stops the app.** `focus.db` and its `-wal` and `-shm` files move to `focus.db.corrupt-<unix ts>`, the app starts fresh and posts one notice. A newer schema shows an "Update Grumpo" dialog and quits. A panic hook sends panics to the log.
- **Signing has two modes.** `scripts/build-release.sh` signs with `FOCUS_SIGNING_IDENTITY` when it is set, which keeps the macOS permissions across rebuilds, and ad hoc otherwise. It signs `focusd` the same way, with the identifier `io.github.brian-rey-development.focusd` and the hardened runtime, runs `codesign --verify --strict` on both, asserts `Signature=adhoc` or the configured `Authority`, and fails on error. The Tauri build runs without the `APPLE_*` signing variables so the environment cannot override the identity. The app carries the Apple Events entitlement. After an ad hoc rebuild, `tccutil reset Accessibility io.github.brian-rey-development.grumpo` is needed because Accessibility fails silently.
- **One install command everywhere.** `scripts/build-release.sh && sudo scripts/install.sh --binary dist/release/focusd`, followed by `rm -rf /Applications/Focus.app /Applications/Grumpo.app && ditto dist/release/Grumpo.app /Applications/Grumpo.app && open /Applications/Grumpo.app`. `install.sh` takes `--owner <user|uid>`, defaulting to `SUDO_UID`, and backs up `state.json` after the old daemon stops so a failed install restores it, or removes the new one when there was none.
- **The login item has its own label.** It is `io.github.brian-rey-development.grumpo`, the bundle identifier, and `uninstall.sh` also removes the two earlier names, `io.github.brian-rey-development.focus.plist` and `Focus.plist`. The first start of `Grumpo.app` enables the new item when it finds an old one and then deletes the old one.
- **The app identity is Grumpo, the daemon identity is not.** The bundle id is `io.github.brian-rey-development.grumpo` and the main executable is `Grumpo`, because System Settings showed the old name and icon under Accessibility and the old binary name under Background Activity. The first start migrates the app data folder with one rename and replaces the old login item. The daemon label and binary `focusd`, the socket, the state and log paths and the crate names keep their names, because the daemon is never shown and renaming it would strand its state. See [ADR 0012](docs/adr/0012-grumpo-rebrand-and-block-screen.md).
- **The block screen is the primary message, the notification is the fallback.** It is a reused `blocker` window, and a block that arrives while it is open replaces its content.
- **Quitting says what it costs.** While blocking is active the tray item reads "Quit Grumpo (stops app and tab blocking)".
- **Web-app wrappers are documented, not mapped.** A site installed as a web app is an app of its own, and the advice is to block the wrapper as an app.
- **The hosts block is machine wide, the control is not.** It applies to every account, and only the owner can reach the daemon.

## Architecture

The split follows the trust boundary, since the UI runs as the user and only root can write `/etc/hosts`.

```
crates/focus-core     pure library, no I/O apart from the shared socket transport. Rules, validation,
                      schedule math, strictness, unlock state machine, enforcement plan, hosts rendering,
                      IPC protocol, legacy import. Exhaustively unit tested.
crates/focusd         root LaunchDaemon. Owns rules and unlock state, persists them atomically,
                      reconciles /etc/hosts, serves a Unix socket restricted to the owner uid.
app/src-tauri         user process. Tracker, SQLite store, enforcement engine and executor, status hub
                      and daemon client, tray, notifications, Tauri commands. OS specifics live behind
                      traits in `platform/`.
app/src               React + TypeScript UI. Talks to the backend only through `src/ipc`.
scripts/              check.sh (all quality gates), gen-bindings.sh, build-release.sh, install.sh, uninstall.sh
```

Data flows in one direction. The sampler thread produces `Activity`, the session builder turns it into sessions in SQLite, and the sampler also posts every sample to the enforcement thread, which combines it with today's usage and the daemon status into a list of `Action`s that the executor performs. The UI reads reports and status through commands and refreshes on backend events. [docs/architecture.md](docs/architecture.md) has the diagram, the module map for every crate and for the UI, the installed layout, and the IPC and command contracts.

## Repository layout

The root holds only workspace-level manifests and policy files. Everything else lives in a directory that names its purpose.

```
.github/workflows/ci.yml      macOS CI running scripts/check.sh and a signed release build, plus shellcheck and cargo-deny
.github/dependabot.yml        weekly cargo, pnpm and actions updates
.cargo/config.toml            ts-rs export dir
crates/                       Rust libraries and the daemon binary
app/                          Tauri app (React UI in app/src, Rust backend in app/src-tauri)
packaging/macos/              LaunchDaemon plist and newsyslog rotation config
scripts/                      check.sh, gen-bindings.sh, build-release.sh, install.sh, uninstall.sh and lib/ helpers
docs/                         architecture.md, privacy.md, threat-model.md, development.md, adr/
dist/                         release artifacts from build-release.sh (ignored by git)
Cargo.toml  Cargo.lock  rust-toolchain.toml  deny.toml  clippy.toml  .editorconfig  .gitignore
README.md  CHANGELOG.md  LICENSE  PLAN.md
```

## Production readiness

- **Versioned persistence.** The daemon state file carries `schemaVersion` with forward migrations, and SQLite uses `PRAGMA user_version` migrations. An unusable state file is set aside with a timestamp and the daemon starts from safe defaults, whereas any other load failure stops the daemon without touching the file.
- **Versioned protocol.** Every IPC message is checked against `PROTOCOL_VERSION`, a mismatch yields a typed error, and the app tells a service that needs updating apart from an app that needs updating by comparing the versions.
- **Structured logging.** Both binaries use `tracing`. The daemon logs to stderr, which launchd sends to `/Library/Logs/Focus/focusd.log` (rotated by newsyslog, with the daemon reopening the file after a rotation), and the app logs to `~/Library/Logs/Focus/` with daily rotation and 7 files kept, including panics. Logs never contain domains or app names above `debug`.
- **Resilience.** The daemon runs under launchd with `KeepAlive`, and the app keeps polling it, going from connected to unavailable only after two failed exchanges and recovering on the first success. Every external call (osascript, socket, SQLite) has a timeout and a typed error, and no failure path panics silently.
- **Supply chain.** `cargo-deny` checks advisories, licenses, bans and sources, JavaScript direct dependencies and the Tauri crates use exact versions, and lockfiles are committed.
- **Reproducible toolchain.** `rust-toolchain.toml` pins Rust, and the `packageManager` field pins pnpm.
- **Release.** `scripts/build-release.sh` builds `focusd` for the host architecture (or both with `--universal`), runs `pnpm tauri build`, signs and verifies the app, and copies `focusd`, `Grumpo.app` and the `.dmg` into `dist/release/`. `install.sh --binary dist/release/focusd` is idempotent, safe to re-run for upgrades, and rolls back on failure. See [docs/development.md](docs/development.md) for the details.

## Conventions

- Rust edition 2021, `[workspace.lints]` with `clippy::pedantic` as warnings and `unsafe_code` denied except in `platform/macos` and the daemon's `sys.rs`, with a `// SAFETY:` note on each block. No `unwrap`/`expect` outside tests and provably infallible statics.
- A module with children keeps its root in `name.rs` beside a `name/` directory. `mod.rs` is never used.
- Fakes and fixtures go in `test_support.rs`, and tests go in `tests.rs` or a `tests/` directory next to the module they cover. Builders that several crates share live in `focus_core::test_support`, behind the `test-support` feature.
- Logging goes through `tracing` with structured fields, never `println!` or `log`.
- Errors use `thiserror` in libraries and are mapped once at the command boundary. Nothing is swallowed, and every ignored error is logged with context.
- Functions are small and single purpose. Files stay under about 200 lines, and modules own one concept.
- Named constants instead of magic values, and a value that more than one place needs is defined once. Rust constants that the UI reads are generated into `app/src/ipc/generated`, and the UI never copies them.
- No comments that restate code, only the non-obvious why.
- TypeScript strict with `noUncheckedIndexedAccess`, no `any`, no non-null assertions. The UI never calls `invoke` outside `src/ipc`.
- UI uses CSS Modules with tokens from `design/tokens.css`. Animations go through `motion` with shared springs from `design/motion.ts` and respect `prefers-reduced-motion`. Every data view has loading, empty and error states, and every control is keyboard reachable with a visible focus ring.
- pnpm only, pinned through `packageManager`. JS direct dependencies use exact versions; Rust relies on the committed `Cargo.lock`, and Tauri crates are pinned exactly because they must match the JS packages.
- The bundle identifier is `io.github.brian-rey-development.grumpo`, and it is permanent because TCC grants, data paths and the login item label derive from it. The earlier identifier `io.github.brian-rey-development.focus` was replaced by a migration, as [ADR 0012](docs/adr/0012-grumpo-rebrand-and-block-screen.md) records, and the daemon identifier `io.github.brian-rey-development.focusd` is unchanged.

### Copy glossary

The UI, the tray and the notifications use the same words for the same things. The UI owns every user-facing error and unavailable-reason message, keyed by `CommandErrorCode` and `DaemonUnavailableReason`, and the `message` strings that Rust produces are diagnostics that are logged and never rendered.

| Concept | Wording |
|---|---|
| No active block | "No block active" |
| Product | "Grumpo", and "the Grumpo service" for the background service. "Focus" stays as the concept, as in focus windows and "Focus on until {time}" |
| Active block | "Focus on until {time}", or "Focus on" when it has no end |
| Limit spent | "Daily limit reached" |
| Rules section | "Focus schedule", and its windows are "focus time" in prose |
| Unlock ready | "Unlock ready, confirm by {time}", and the title "Unlock ready" |
| Durations | "{n} min" in compact places and "{n} minutes" in sentences, each from one formatter per language layer |
| Empty data | "Nothing tracked yet" |
| Contractions | "can't"-style contractions throughout |
| Settings | The in-app screen is "Settings", so "Open Settings", and the macOS app is "Open System Settings" |
| Sites | "site" in titles and prose (for example "Blocked sites" and "Add site"), never "website" |
| Clock times | Follow the macOS 12 or 24 hour setting in the tray and notifications as well as the UI |
| Voice | Grumpo is "he", dry and short and on the person's side. Jokes only where the person is told no, which is the block screen, notifications, empty states and setup. Settings, forms and errors stay plain |
| Block screen | Headline "Nice try.", "Nope." or "Not today.". Site "You told Grumpo to block {domain}. He's taking his job very seriously.", app "You told Grumpo to keep {name} closed. So he closed it.", limit "That's all of today's {target}. Grumpo is guarding the rest.". Countdown "Focus ends in" or "Resets in", and "Strict mode on" with a lock when strict. Button "Back to work" and footer "No, Grumpo doesn't take bribes." |
| Prose | No colons and no em dashes in user-facing text |

## Quality gates

`scripts/check.sh` runs shellcheck when it is installed, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings` and a clippy pass over `focus-core` alone, `cargo test --workspace --all-features`, a check that the generated bindings and constants are current, then in `app/` `pnpm install --frozen-lockfile`, `pnpm typecheck`, `pnpm lint`, `pnpm format:check` and `pnpm test`. CI runs it, then builds and verifies the signed release bundle, and runs shellcheck and `cargo deny check` as separate jobs. Every change ends green.
