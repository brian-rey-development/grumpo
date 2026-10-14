# Architecture

Grumpo is split along the one boundary that cannot be negotiated on macOS, which is that only root can write `/etc/hosts` while everything a person interacts with runs as that person. The design starts from that fact and pushes every decision about what is allowed into the privileged side, so that the user-facing side can be rich and fast without being trusted.

This document is the single source for the module map, the installed layout and the IPC contract. `PLAN.md` holds the scope, the product decisions, the conventions and the quality gates, and links here for everything structural.

## Processes and threads

The daemon `grumpod` runs as root under launchd with `KeepAlive`, owns the rules and the unlock state, and is the only writer of the marked block in `/etc/hosts`. The Tauri app runs as the logged-in user, samples the frontmost app and browser tab, stores usage in SQLite, and executes the enforcement actions that need user privileges, such as quitting an app or closing a tab. The webview inside the app renders the React UI and has no capability beyond the ones listed in the capability section below and a content security policy that allows no remote origins.

```
 root                                user
┌────────────────────┐   Unix     ┌───────────────────────────────────────────────┐
│ grumpod             │   socket   │ Grumpo.app (Tauri, Rust backend)              │
│  Service           │◀──────────▶│  StatusHub + DaemonClient  (poll every 5 s)   │
│  Reconciler (15 s) │  envelope  │  Sampler thread ─▶ SessionBuilder ─▶ Store    │
│  HostsFile         │  + JSON    │       │                                       │
│  state.json (0600) │            │       └─▶ Engine thread (policy ─▶ executor)  │
└─────────┬──────────┘            │  Tray worker, command tasks                   │
          │                       └───────────────┬───────────────────────────────┘
          ▼                                       │ Tauri IPC (camelCase DTOs)
     /etc/hosts                                   ▼
     marked block                      ┌────────────────────┐
                                       │ Webview (React/TS) │
                                       │ src/ipc only       │
                                       └────────────────────┘
```

The reason the daemon owns the rules, rather than merely applying them, is that a rule store the user process could edit freely would make every cooldown cosmetic. The app asks the daemon to change rules and the daemon decides, as described in [ADR 0002](adr/0002-privileged-daemon-owns-rules.md) and [ADR 0004](adr/0004-strictness-enforced-in-daemon.md).

The daemon is deliberately small in threads. The main thread accepts connections on the socket and serves them one at a time, each with a two second budget to send its request and another to take the response, which is enough because the app sends a status request every few seconds. A second thread named `reconciler` beats every 15 seconds, and it wakes just after each local midnight when that comes sooner, so exhausted limits clear promptly. Both threads share the service behind one lock that is held while a request is applied, including the state and hosts writes, and never during socket I/O or the resolver flush. A panic inside a beat is contained and logged, because nothing else would retry the work.

The app has more threads, each with one job, and the rule that ties them together is that nothing which has to stay on schedule may wait for something slow. Each worker is a `StoppableThread` from the `concurrency` module, and a failure to start one is handed up to `setup`, so the app never runs with tracking or enforcement silently missing.

| Thread | Started by | Job |
|---|---|---|
| `focus-sampler` | `setup` | Every 2 seconds reads the frontmost app, idle time, screen lock and media state, plus the browser tab at most every 4 seconds, builds sessions and writes them to SQLite. It runs retention once a day and hands every sample to a sink without blocking |
| `focus-enforcement` | `EngineSink::start` | Receives the newest sample through a one-slot mailbox, combines it with today's usage and the daemon status, and has the pure policy decide what to do. It performs the actions, which can take as long as an `osascript` call or a daemon exchange, and announces stored usage changes to the UI at most once per second |
| `focus-status-poller` | `StatusHub::start_polling` | Asks the daemon for its status every 5 seconds, or sooner when an unlock phase is about to change |
| `focus-tray` | `TrayHandle::build` | Redraws the menu bar total after bursts of usage events and the status line when it shows a countdown |
| Tauri runtime and blocking pool | Tauri | Runs the commands. Anything that touches SQLite, the socket or AppKit is moved onto the blocking pool by `commands::blocking`, so the UI thread never waits on it |

The sampler and the engine are separate threads because enforcement is slow and the sampler is not allowed to be. A sample is a short message posted to a mailbox that holds only the newest one, so an engine that is busy closing a tab loses stale samples, which is harmless because policy judges the present, and the sampler's two second cadence is never stretched. On exit the poller stops first and the status listeners are cut loose so that no worker can push an update into the tray or the webview while the app goes away, then the sampler stops and flushes its open session, then the engine, and finally the App Nap activity is released. The tray only ever queues its changes to the main thread and never waits for it, which is what lets the exit, itself running on the main thread, join every worker without a deadlock.

## Module map

### crates/focus-core

The crate is a pure library with no I/O apart from the blocking socket transport that both ends of the protocol share. Because it performs no other I/O it is tested exhaustively with plain unit tests, and both the daemon and the app depend on it, so the two sides cannot disagree about what a rule means.

| Module | Responsibility |
|---|---|
| `time/` | `Now { utc, local }` (`now.rs`), `Day` (`day.rs`), `TimeOfDay` as a minute of day (`time_of_day.rs`), and `units.rs`, the one home of the unit conversions (seconds per minute, minutes per day and the like) that every crate re-exports instead of redefining |
| `domain.rs`, `domain/address.rs` | `Domain` newtype. `parse` normalizes user input (`https://www.YouTube.com/x` becomes `youtube.com`), `covers` relates two domains, and `address.rs` reduces a browser address to a host. The accepted and rejected inputs are data in `tests/vectors/domain.json` and `whitespace.json`, so a parser in another language can be tested against the same cases |
| `app_ref.rs` | `AppRef { bundle_id, name }`, ordered and equal by `bundle_id` |
| `rules.rs` | `Rules { blocked_sites, blocked_apps, schedule, limits }`, `Limit { target, daily_minutes }` and `LimitTarget::{Site, App}` |
| `rules/schedule.rs`, `rules/week_mask.rs` | `Schedule { windows }`, `Window { days, start, end }`, `active_at(now)`, `active_until(now)` returning `Coverage`, and `WeekMask` (7 x 1440 coverage) |
| `rules/validate.rs` | `Rules::validate() -> Result<ValidatedRules, RulesError>` (bounds, `start < end`, no duplicate limits, no protected apps) and the named caps such as `MAX_SITES` and `MAX_LIMIT_MINUTES`. `ValidatedRules` is the only form persistence and enforcement accept |
| `rules/protected.rs` | `PROTECTED_BUNDLE_IDS` and `is_protected(bundle_id)` |
| `rules/strictness.rs` | `loosenings(old, new) -> Vec<Loosening>` (removed site or app, removed or raised limit, uncovered schedule minutes) and `Loosening::is_limit_change` |
| `unlock.rs`, `unlock/phase.rs` | The `UnlockState` machine with `request`, `confirm(phrase, duration)`, `cancel`, `relock`, `tick(now)` and the escalation, plus `UnlockPhase`, whose `advance` also undoes a clock set backwards. The policy constants `COOLDOWN_STEPS_MINUTES`, `UNLOCK_SENTENCE`, `UNLOCK_DURATIONS_MINUTES` and `CONFIRM_WINDOW_MINUTES` live in `unlock.rs` |
| `enforcement.rs`, `enforcement/exhausted.rs` | `EnforcementPlan::compute(rules, unlock, exhausted, now)` gives blocked sites, blocked apps, the reason and `active_until`, and `locked_loosenings` applies the loosening rule described below. `Exhausted` records the targets that ran out on one local day with the allowance each had |
| `hosts.rs`, `hosts/strip.rs` | `render_bytes(existing, sites)` rewrites a hosts file so it holds exactly one managed block, after `strip.rs` has removed the current and the legacy blocks, and `blocked_hosts` names the hostnames written for each site |
| `legacy.rs` | `rules(script)` parses the old script's `DOMAINS=( ... )` array and its fixed schedule into unvalidated `Rules` |
| `ipc/` | `PROTOCOL_VERSION` in `ipc.rs`, then `codec` (newline-delimited JSON framing, `read_frame`, size caps), `envelope`, `request` (`Request`, `Response`), `status` (`DaemonStatus`), `views` (`EnforcementView`, `UnlockView`), `error` (`IpcError`, `ErrorCode`), `socket` (`socket_path`, `DEFAULT_SOCKET_PATH` and the `GRUMPOD_SOCKET` override) and `transport`. `transport` is the blocking Unix socket layer used by the daemon and the app alike, with a `Deadline` stream whose reads and writes draw on one expiry, `read_message` sized by the message type, and `exchange`, which sends a request and reads the reply inside one shared budget |
| `bindings.rs` | Behind the `ts` feature, writes the Rust constants the UI needs (rule caps, limit bounds, unlock timings, domain lengths, hosts alias prefixes, time units) into `app/src/ipc/generated/constants.ts`, next to the ts-rs types. The app exports its own retention bounds through the same helper |
| `test_support.rs` | Behind the `test-support` feature, the builders for rules, instants, apps, sites and status that the tests of all three crates share, so no crate keeps its own copy |

### crates/grumpod

The daemon wraps the pure functions with the effects they need. Tests and development override the system locations with `GRUMPOD_STATE_DIR`, `GRUMPOD_HOSTS_FILE`, `GRUMPOD_SOCKET` and `GRUMPOD_LOG_FILE`, and the app's daemon client honors `GRUMPOD_SOCKET` too.

| Module | Responsibility |
|---|---|
| `main.rs`, `args.rs`, `cli.rs` | Argument parsing (`run --owner-uid <uid>`, `import-legacy <path>`, `strip-hosts` and `ping`), logging setup, and dispatch that ends in one `CliError` |
| `commands/` | One module per subcommand (`run`, `import`, `strip`, `ping`) |
| `paths.rs` | `Paths { state_dir, hosts_file, socket, log_file }` with the macOS defaults, overridable by environment variables for tests. A port to another system adds its defaults here |
| `state_dir.rs`, `instance_lock.rs` | `ensure_state_dir` makes the directory private and then takes the exclusive `grumpod.lock`, in that order, for every command that writes state. The lock uses `File::try_lock`, so it needs no `unsafe` |
| `state/` | `Snapshot` (validated rules, unlock, exhausted), `persisted` (the on-disk shape), `migrate` (the forward steps, with the current schema version derived from their list) and `file_store` (atomic load and save at `0600`, and the handling of unusable files) |
| `service/` | `Service` applies a `Request` to state and returns a `Response`. `requests.rs` holds the pure transitions, and I/O is injected |
| `hosts_writer.rs`, `atomic_write.rs`, `dns.rs` | Write the hosts file only when its content changes, atomically through the resolved symlink target, then flush DNS |
| `server.rs`, `server/connection.rs` | The Unix socket listener, the owner-uid check (`getpeereid`), one request per connection, a time budget per direction and the size limits |
| `reconciler.rs` | The 15 second tick plus a beat on every mutation and one just after local midnight, which ticks the unlock state, rolls over `exhausted` and applies the plan |
| `log_rotation.rs` | Reopens the log file after newsyslog rotates it |
| `shared.rs`, `clock.rs`, `sys.rs` | The shared service lock, the clock seam, and the one module that calls libc (peer credentials, the effective uid and `dup2`) |

### app/src-tauri/src

| Module | Responsibility |
|---|---|
| `lib.rs`, `main.rs` | The builder, the plugins, the login item label, the command registration and the run events. Nothing else |
| `setup.rs`, `setup/database.rs` | `setup` builds the long-lived parts once the runtime exists and starts the workers, and `database` opens the usage database, recovering from a damaged one |
| `state.rs` | `AppState` (store, status hub, platform handles, events, block screens, launch, paths) and `Workers`, which stops the threads in order on exit |
| `error.rs`, `error/rules_fault.rs` | `AppError` (thiserror), mapped once into the serializable `CommandError`, and the classification of a refused ruleset into a `RulesFault` |
| `dto/` | The contract with the UI, split by area (`apps`, `block_screen`, `clock`, `command_error`, `permissions`, `reports`, `service`, `settings`, `status`), with TypeScript bindings generated from it |
| `commands/` | Thin modules, one per area (`reports`, `status`, `rules`, `unlock`, `apps`, `settings`, `permissions`, `data`, `clock`, `service` and `block_screen`), that run blocking work through `commands::blocking` |
| `block_screen.rs`, `block_screen/tauri_surface.rs` | `BlockScreenHub`, which remembers what the block screen shows and replaces it when another block arrives, over the `BlockSurface` port, and `TauriBlockSurface`, which creates the `blocker` window on first use and queues every window change for the main thread |
| `daemon_client.rs` | The synchronous client for the grumpod socket, a thin layer over `focus_core::ipc::transport::exchange`, and the typed `DaemonError` |
| `status/` | `StatusHub` (latest status, ordered exchanges and deliveries), `exchange` (retries and the failure count), `poller` (every 5 seconds or at an unlock deadline), `ready_alert` (the unlock-ready notice) and `unavailable` (why the daemon is out of reach) |
| `time/` | The clock (`Clock`, `SystemClock`), the local calendar, `LocalDay` (the local day that reports and limits count in), the shared formatter for clock times and durations, which follows the system 12 or 24 hour setting, and a re-export of `focus_core::time::units` |
| `concurrency/` | `StoppableThread` and `StopSignal` for every worker, `LockExt::lock_or_recover` for locks that survive a poisoned mutex, and `Coalescer`, which throttles bursts of events |
| `tracking/activity.rs` | `Activity { app: AppRef, domain: Option<Domain>, browser: Option<Browser>, engaged: bool, private: bool, idle }`. Private browsing is enforced but never persisted |
| `tracking/` | `observer` reduces one probe read per sample to an `Activity`, `session_builder` merges activities into sessions, `session_writer` batches them into the store, `sampler` drives the loop and `pruner` applies retention daily. `tab_memory` paces tab reads, `health` records which browsers lack permission, `probe_errors` turns failures into a state per source and `timing` holds the cadence |
| `storage/` | `Store` over SQLite with `migrations` (`user_version`), `recovery` (opening a damaged database), `sessions`, `reports/` (hourly, timeline, top items and usage), `retention`, `erase`, `export`, `settings`, `apps` and `private_fs` |
| `enforcement/` | `policy` (with `policy/blocking.rs`) decides, `engine` gathers the inputs for one step and runs the periodic sweep (`engine/frame.rs`, `engine/sweep.rs`), `usage` and `memory` track what has been used and already done, `executor` performs the actions and sends a block to the block screen, `mailbox` and `sink` connect it to the sampler, and `notice` words the notifications and builds the content of the block screen |
| `notify.rs`, `tray/`, `window.rs`, `launch_at_login.rs`, `events.rs`, `icon_cache.rs`, `telemetry.rs` | The notification seam, the menu bar item (`menu`, `text`, `view` and the `worker`), the hide-on-close behavior, the login item and its typed error, the events that go to the webview, the bounded icon cache, and logging with the panic hook |
| `platform.rs`, `platform/ports.rs` | The OS ports `ActivityProbe`, `AppControl`, `IconProvider`, `AppLocator`, `PermissionProbe`, `HourCycleSource`, `Dialogs` and `ServiceInstaller`, `begin_tracking_activity()` and `platform::current()` |
| `platform/macos/` | The adapters, among them `frontmost`, `idle`, `screen_lock`, `media`, `browsers`, `tab_scripts`, `firefox`, `control`, `icons`, `locator`, `permissions`, `hour_cycle`, `dialogs`, `service_installer` and `app_nap`, over the AppleScript, Accessibility and process helpers |
| `platform/unsupported.rs` | Stub adapters that return `Unsupported` |

### app/src

```
main.tsx, App.tsx          providers (QueryClient, motion, modal stack, navigation, rules draft, unlock), shell, screen switch
ipc/                       generated/ (ts-rs types and constants), commands.ts (CommandMap and invokeCommand), events.ts (EventMap and subscribe), queries.ts and queryKeys.ts (TanStack Query hooks), errors.ts, useBackendEvent.ts, useBackendSync.ts
copy/                      the shared wording: common.ts (strings used in many places), failures.ts (describeFailure, keyed by CommandErrorCode) and service.ts (why the service is unavailable, keyed by DaemonUnavailableReason, and the install command)
design/                    tokens.css, global.css, motion.ts (shared springs and durations)
components/                Generic building blocks that take their labels as props, such as Button, Dialog, Toggle, Toast, Section, TimeField, BarList, ColumnChart, DayRibbon, ChartNavigation, AppIcon, Skeleton, EmptyState, ErrorState and LoadFailure (the one view for a failed load)
features/today/            TodayScreen, focus line, limits section, timeline and hourly sections
features/week/             WeekScreen
features/usage/            shared hero, usage lists, chart data, ribbon text and browser access hint for Today and Week
features/rules/            RulesScreen, SitesEditor, AppsEditor, ScheduleEditor, LimitsEditor, week preview, RulesDraftProvider and the save flow
features/unlock/           UnlockProvider and UnlockSheet (cooldown countdown, sentence typing, duration)
features/status/           SidebarStatus, StatusPill and the focus state shown in the sidebar
features/settings/         SettingsScreen (service, permissions, launch at login, retention, export, Delete history)
layout/                    Sidebar, Toolbar, AppShell, NavigationProvider, navigation (typed `Screen` union of today, week, rules and settings)
lib/                       format.ts (durations, times, with an injectable locale), color.ts, time.ts, weekDays.ts, limitTarget.ts, whitespace.ts, useNow.ts
test/                      ipc/ (fixtures and routeCommands for mocking commands), render.tsx, shell.tsx, setup.ts, and the contrast and token helpers
```

Feature folders own the wording of their screens in a `copy.ts`, and a string that several places share lives in `copy/`. The UI owns every user-facing error and unavailable-reason message, so the `message` that arrives with a Rust error is a diagnostic for logs and is never rendered.

### Module roots and tests

A module with children keeps its root in `name.rs` beside a `name/` directory, and `mod.rs` is never used, in any crate. Tests follow a fixed layout next to the module they cover. Fakes and fixtures go in `test_support.rs`, and the tests go in `tests.rs` or, when there are several files, a `tests/` directory. `focus-core` publishes the builders that several crates need as its `test_support` module behind the `test-support` feature, which the dev-dependencies of the other crates enable.

## Installed layout

On macOS everything below is owned by root unless noted.

| Item | Location |
|---|---|
| Binary | `/Library/PrivilegedHelperTools/grumpod` |
| LaunchDaemon | `/Library/LaunchDaemons/io.github.brian-rey-development.grumpod.plist` with `KeepAlive`, `RunAtLoad`, `GRUMPOD_LOG_FILE`, and `ProgramArguments` passing `run --owner-uid <uid>` |
| State | `/Library/Application Support/Grumpo/state.json` (`0600`, directory `0700`), with `grumpod.lock` beside it and any quarantined `state.json.corrupt-<unix ts>` files |
| Socket | `/var/run/grumpod.sock`, chowned to the owner uid with mode `0600` |
| Hosts | `/etc/hosts`, written through its resolved target `/private/etc/hosts` |
| Log | `/Library/Logs/Grumpo/grumpod.log`, rotated by `/etc/newsyslog.d/grumpod.conf` |
| App | `/Applications/Grumpo.app`, signed with the identifier `io.github.brian-rey-development.grumpo`, whose main executable is `Contents/MacOS/Grumpo` |
| Bundled daemon (not root-owned) | `Grumpo.app/Contents/MacOS/grumpod`, signed with the identifier `io.github.brian-rey-development.grumpod` and the hardened runtime |
| Bundled installer (not root-owned) | `Grumpo.app/Contents/Resources/installer/` with `scripts/install.sh`, `scripts/lib/` and `packaging/macos/`, the same relative layout as the repository |
| Login item (user) | `~/Library/LaunchAgents/io.github.brian-rey-development.grumpo.plist`, whose label is the bundle identifier |
| Usage database (user) | `~/Library/Application Support/io.github.brian-rey-development.grumpo/focus.db` |
| App logs (user) | `~/Library/Logs/Grumpo/` |

## Data flow

Data moves through three loops that meet at the daemon status and the store.

The tracking loop begins with the sampler, which reads the frontmost app every 2 seconds through the `Observer`, along with the active tab host for supported browsers. The observer produces an `Activity` made of an app reference, an optional domain, the browser, an `engaged` flag and a `private` flag. A user is engaged unless they have been idle for two minutes or the screen is locked, and media playing under a display-sleep assertion held by the frontmost app keeps them engaged. The `SessionBuilder` merges consecutive engaged activities into sessions and closes a session when the app or domain changes, when the user stops being engaged, when the window is private, or when a gap of more than 6 seconds appears, which is how sleep and wake are handled without special cases. The `SessionWriter` writes the resulting sessions to SQLite, batching end-time updates so that the sampler does not write on every tick.

The enforcement loop starts from the same sample. The sampler hands it to a `SampleSink`, which the app implements as `EngineSink`, and the sink posts it to the engine's mailbox. For each sample the engine reads the latest `DaemonStatus` from the hub, computes today's usage per limit target, and gives the policy a plain-value picture of the world. The policy answers with a list of `Action` values, which are `QuitApp`, `CloseTab`, `ReportLimitReached` and `Notify`, and the executor performs them through the `AppControl` trait, the notifier and the daemon client, telling the memory how each one went.

```
probe ──Activity──▶ SessionBuilder ──Session──▶ Store ◀── commands ◀── UI
  │                                               │
  └──sample──▶ Engine(policy, usage, status, memory) ─▶ Actions ─▶ AppControl / Notifier / StatusHub ─▶ Daemon
                                    ▲
                      StatusHub ────┘ (status every 5 s or at an unlock deadline, rule edits, unlock)
```

The UI loop closes the circle. The webview reads reports and status through commands and refreshes when the backend emits `usage-changed`, which the engine coalesces to one per second, or `status-changed`, which carries the new `FocusStatus`. While the window is hidden the webview marks its queries stale without refetching them and refetches when it becomes visible again, so a hidden app does no work for a screen nobody sees. Mutations go back through commands to the daemon, so the daemon stays the single owner of rules.

## The IPC contract

The app and daemon exchange one request and one response per connection. Each is a single newline-terminated JSON frame, read through the bounded reader `read_frame`, with requests capped at 256 KiB and responses at 1 MiB, and the cap of each message type is the `MAX_BYTES` that `read_message` reads with. A test asserts that the largest valid ruleset still fits under the request cap, so the limit cannot silently become a functional ceiling. The daemon accepts connections only from the owner uid it was started with (which is never root) and from root, and gives each connection a two second budget for reading and writing, so a stuck client delays others by at most that long. The client in the app mirrors this with a single two second deadline that covers the write and the read together, because a plain socket timeout restarts on every call and a daemon that trickled bytes could otherwise hold the caller indefinitely. Both ends get that behavior from the same `focus_core::ipc::transport`, so the deadline logic exists once.

```
Envelope<T>  = { protocolVersion, body: T }    version is probed first, mismatch -> version_mismatch
Request      = Status | UpdateRules { rules } | RequestUnlock | ConfirmUnlock { phrase, minutes }
             | CancelUnlock | Relock | ReportLimitReached { target, dailyMinutes, day }
Response     = Ok { status: DaemonStatus } | Err { error: IpcError }
DaemonStatus = { rules, enforcement: { active, reason: None|Schedule|Limit|Both, activeUntil,
                 blockedSites, blockedApps }, exhausted: [LimitTarget],
                 unlock: { phase: Locked | CoolingDown{readyAt, minutes} | Ready{readyAt, expiresAt}
                 | Unlocked{until}, nextCooldownMinutes, sentence, durations } }
IpcError.code = version_mismatch | rules_invalid | loosening_locked | unlock_not_ready
              | unlock_not_requested | unlock_already_pending | unlock_already_active
              | phrase_mismatch | duration_invalid | protocol | internal
```

Every message travels in the envelope, and `PROTOCOL_VERSION` in `focus-core::ipc` is the one place that says which version is current. The receiver parses only the version first and compares it with its own, and a mismatch produces the typed error `version_mismatch` before the body is ever interpreted. This ordering matters because a newer app and an older daemon, which is the normal state during an upgrade, may disagree about the shape of the body, and parsing it would otherwise yield a confusing error. The app turns a mismatch into one of two reasons by comparing versions, so a daemon that speaks an older version is a service that needs updating, and a daemon that speaks a newer one means the app is the old side.

The protocol has changed since it was first released. Version 2 made a limit report carry the allowance that ran out, so the daemon can tell a report for a limit that has since been raised from one that still holds, and made the `CoolingDown` phase carry the `minutes` that were requested, so a cooldown that spans local midnight still reports its real length. Version 3 added the reporter's local `day` to `ReportLimitReached`, and the daemon ignores a report whose day is not its own current day, because a report that crosses midnight on its way to the daemon would otherwise block the whole new day. The daemon answers an ignored report with the current status like any other, so the reporter never sees a failure.

Rule payloads deserialize with `deny_unknown_fields`, so a field the receiver does not understand is rejected loudly. Errors carry a stable machine readable `code` plus a human message that is meant for logs. The UI reacts to the code, for instance by opening the unlock flow on `loosening_locked`, and words every failure itself. The app maps the daemon codes onto its own `CommandErrorCode` at the command boundary, and when the app itself refuses a ruleset it adds a `RulesFault` that names the section and the problem, so the UI can say which site or window to fix.

The wire envelope types `Request`, `Response` and `Envelope` are not exported to TypeScript, because the webview never speaks the socket protocol. The rule and status types are exported, because the UI renders them. `Rules`, `Schedule`, `DaemonStatus`, `EnforcementView`, `UnlockView`, `IpcError` and `ErrorCode` all derive the TypeScript bindings, and `DaemonStatus` reaches the webview inside the `FocusStatus` DTO, which wraps it in a `connected` variant or reports why the daemon is unavailable. The Rust constants the UI also needs, such as the rule caps, the limit bounds, the cooldown steps and the retention bounds, are generated beside the types as `constants.ts` and `retention.ts`, so the UI reads them and never copies a number. The decisions are recorded in [ADR 0006](adr/0006-typescript-bindings-generated-from-rust.md) and [ADR 0007](adr/0007-versioned-ipc-envelope-over-unix-socket.md).

### Tauri commands and events

Command names are snake_case and argument keys camelCase. Every DTO lives in `app/src-tauri/src/dto/` and its TypeScript twin is generated into `app/src/ipc/generated`. A failed command rejects with `CommandError { code: CommandErrorCode, message, rulesFault }`. The same list is held by `build.rs`, `capabilities/commands.json`, the handler list in `lib.rs` and the `CommandMap`, and a test fails when any of them disagrees.

```
get_day_report          { date: "YYYY-MM-DD" }            -> DayReport
get_week_report         { endDate: "YYYY-MM-DD" }         -> WeekReport
get_limit_usage                                           -> LimitUsage[]
get_focus_status                                          -> FocusStatus
save_rules              { rules: Rules }                  -> FocusStatus
request_unlock                                            -> FocusStatus
confirm_unlock          { phrase: string, minutes: number } -> FocusStatus
cancel_unlock                                             -> FocusStatus
relock                                                    -> FocusStatus
list_known_apps                                           -> KnownApp[]
get_app_icon            { bundleId: string }              -> string | null   (PNG data URL, 128 px, cached)
get_settings                                              -> Settings
update_settings         { settings: Settings }            -> Settings
get_permissions                                           -> Permissions
request_permission      { permission: PermissionKind }    -> Permissions
open_permission_settings { permission: PermissionKind }   -> null
export_data                                               -> string         (path of the written file)
delete_all_data                                           -> null           (Delete history in Settings)
get_clock_style                                           -> ClockStyle     (H12 or H23, read each time)
install_service                                           -> ServiceInstallOutcome (installed or cancelled)
get_onboarding_state                                      -> OnboardingState (completed: boolean)
complete_onboarding                                       -> OnboardingState (remembered, the welcome flow does not open again)
reset_onboarding                                          -> OnboardingState (Show welcome again in Settings)
get_block_screen                                          -> BlockScreen | null (what the block screen shows now)
dismiss_block_screen(id?: number)                         -> null           (hides the window and hands focus back, only if id is still the current block, or always without id)

Events
usage-changed           payload null                      (sessions written; refetch reports and limit usage)
status-changed          payload FocusStatus               (daemon status changed or became unavailable)
block-screen            payload BlockScreen               (the block screen content changed, sent to the blocker window only, before the window is shown)
block-screen-hidden     no payload                        (the window is about to hide, so the page resets for the next show)
```

## Persistence and migrations

The daemon persists `PersistedState { schemaVersion, rules, unlock, exhausted }` as a JSON file in its root-owned state directory, which is `0700`. The file carries a `schemaVersion`, and on load it is upgraded by forward migrations before its rules are validated. The current version is derived from the length of the migration list in `state/migrate.rs`, so adding a version means appending one function there and nothing else. The first step gives a pending cooldown the length it was requested with, which the earliest files never stored, by reading the saved count of unlocks, and it drops protected apps from the blocked apps and the limits, because later rules refuse them and one old entry would otherwise make the whole saved state unusable. The next step gives a running unlock the length it was granted with, using the longest menu entry as the most it can have been, so an unlock in force across the upgrade keeps its time and the clock tampering check has a figure to compare against. Writes go to a temporary file in the same directory with mode `0600` and are renamed over the target, so a crash leaves either the old or the new state and never a partial file.

A state file that cannot be loaded is handled by what is wrong with it. A file whose JSON is damaged, whose schema version is missing or whose rules no longer validate is unusable in itself, so the daemon moves it aside as `state.json.corrupt-<unix ts>`, logs the event with a summary that never quotes the file, and starts from safe defaults. Any other failure makes the daemon exit with a non-zero status and leave the file where it is. That covers a file that cannot be read, which is an `Io` error, and a file written by a newer daemon, which is `UnsupportedVersion`. Starting from defaults in those cases would switch enforcement off while the marked block in `/etc/hosts` still stands, whereas an exit lets launchd retry under `KeepAlive`, keeps the old hosts block in place meanwhile, and leaves the good file for the daemon that can read it. The installer backs up `state.json` once the old daemon has stopped, so a late save is kept, and puts it back if the install fails, which covers the case of a new binary that rewrote state before the failure. When there was no `state.json` before, the rollback deletes the one the new daemon wrote.

The app persists usage and its settings in one SQLite database named `focus.db`, inside the per-user data directory that Tauri derives from the bundle identifier, created with owner-only permissions. Schema changes are applied by forward migrations keyed on `PRAGMA user_version`, so opening an older database upgrades it step by step. The `settings` table holds the retention period and the flag that records that the launch at login default was applied. Retention deletes sessions older than the configured number of days, which is 90 by default and can be set between 30 and 365, bounds that are exported to the UI from `storage/retention.rs`.

The app cannot run without a database, so opening it has a recovery path in `storage/recovery.rs`. A file that is not a database is damaged, and the app moves `focus.db` together with its `-wal` and `-shm` files to `focus.db.corrupt-<unix ts>`, logs it, opens a fresh database and posts one notice that the history was reset. The old files stay on disk in case they can still be read. A database whose schema is newer than the app is not damaged, and running an old schema over newer data could lose it, so the app shows a native "Update Grumpo" dialog through the `Dialogs` port and quits without touching the file. Any other failure to open it also ends in a dialog. Because a worker thread that panics would otherwise leave nothing behind in the shipped app, which has no stderr, `telemetry::install_panic_hook` sends every panic with its location to the log through `tracing` before the default hook runs.

## Enforcement semantics

Enforcement exists only while the daemon reports `enforcement.active`, which is true when a schedule window is open, when a daily limit is spent, or both, and false while an unlock is running. The daemon computes that plan, writes the blocked sites into the hosts block, and publishes the blocked sites and apps in its status. The app acts on exactly what the status lists, so it never decides for itself what is blocked.

### The hosts block

A hosts file has no wildcards, so the daemon spells out every hostname it blocks. For each blocked registrable domain, such as `youtube.com` or `espn.com.ar`, it writes the domain with the aliases `www.` and `m.`, plus the aliases in `SITE_ALIASES` for the few sites that serve pages from others, such as `old.reddit.com`, each as an IPv4 sink line to `0.0.0.0` and an IPv6 sink line to `::`. A subdomain such as `m.youtube.com` is written as it is, since it has no aliases of its own. The strictness check works from the same `blocked_hosts` function, so a change that unblocks an alias counts as the loosening it is, and the app mirrors the rule in `features/rules/hosts.ts`, tested against the same `tests/vectors/hosts.json` ([ADR 0015](adr/0015-aliases-only-for-registrable-domains.md)). Only the marked block between the Grumpo markers is replaced, and any foreign line is kept byte for byte, whatever its encoding.

The hosts file applies to every account on the Mac, because `/etc/hosts` is shared, but only the owner uid can reach the daemon. Another account is therefore blocked from the listed sites without being able to change the rules or unlock, and its copy of the app reports the service as owned by another user.

### Tabs and apps

When the frontmost app is a blocked app, the policy emits a quit and a notification. When the frontmost app is a supported browser showing a blocked site, it emits a tab close and a notification. The close script is a request to the browser's front window and nothing else, so tabs are closed only in the frontmost window of Chrome, Brave, Edge and Safari. A blocked site in a background window is neither read nor closed, and it is handled the moment that window comes to the front, with the hosts block stopping new loads in the meantime. Firefox exposes no scripting for closing a tab, so a blocked site open in Firefox relies on the hosts block alone, although a blocked Firefox app is still quit like any other. The tab is read at most every 4 seconds, so a tab switch can be noticed that much late, and the close script closes the tab only if it still shows the address that was read.

A site installed as a web app, such as a Chrome or Edge app or a Safari web app in the Dock, runs as an app of its own with its own bundle id. Its time counts under that app and not under the site, and Grumpo cannot close it as a tab, so a site limit does not reach it. The way to cover a wrapper is to block it as an app, while keeping the site itself blocked so the hosts block still applies.

A blocked app that is running behind other windows is handled by the background sweep. When blocking is active, the engine lists the running regular applications and quits every one that is blocked. It does this when blocking starts, when the status changes, and periodically every 30 seconds while blocking stays active, so an app launched in the background is found within half a minute. The sweep sees applications that appear in the Dock, and it does not reach background-only agents. Quitting is a polite request to the app, so an app that shows an unsaved-changes dialog may decline, and the engine asks again a few seconds later while the app stays in front.

The policy never acts on a protected app. Grumpo, Finder, the login window, System Settings and the Dock are refused by the daemon's rule validation and left out of the app picker, and the policy checks `is_protected` as well, so even a rule that predates the protection could not make Grumpo quit them. [ADR 0008](adr/0008-protected-apps-can-never-be-blocked.md) records the reasoning.

Repeated actions are rate limited by a per-day memory. A quit or a tab close for the same target is retried after 3 seconds, a blocked notice for the same target is shown at most once a minute, and a limit warning is shown once per target per day. The memory is cleared when the local date changes.

### Limits

Limits work on live usage. For each limit target the engine reads stored usage from SQLite, refreshing it every 10 seconds or when new sessions were written, and tops it up with the seconds of the session still running, which the writer has not stored yet, capped at 30 seconds so a failed write cannot credit time that was never spent. When 5 minutes or less remain, the policy emits one warning. When none remain, it emits `ReportLimitReached` carrying the allowance and the local day, unless the daemon already lists the target as exhausted or the same allowance was already reported. The daemon records the report only while it is for the daemon's current day and the current rules still hold a limit for that target that is not higher than the reported allowance, and it answers a stale report with the current status so the reporter never fails. From then on the target is part of the daemon's plan until local midnight, and a raise of the limit frees it, whereas keeping or lowering the limit does not.

Private browsing is the one place where enforcement and storage see different things. A private window in Chrome, Brave or Edge is still read, so blocked sites are closed and limits keep counting, but the session builder drops it. The engine keeps an in-memory tally of the engaged private seconds per limit target for the current day and adds it to the stored usage. The tally is never persisted, so a restart or the next midnight forgets it, and [ADR 0009](adr/0009-private-browsing-counts-toward-limits-in-memory-only.md) explains why that is the compromise.

Because the app might have last heard from the daemon before midnight, the policy also guards against acting on a stale block. When the last confirmation predates the start of the local day, a block that rests only on a limit which ran out yesterday is not acted on, since the daemon frees such targets at midnight, until the next confirmation arrives.

### Changing rules while enforcing

The daemon decides every rule change itself. On `UpdateRules` it validates the rules, computes the loosenings against the saved rules with `loosenings`, and passes them through `locked_loosenings`, which keeps the ones that cannot be applied now. Adding or lowering a limit is not a loosening and always works. Raising or removing a daily limit is locked unless an unlock is active, whatever the schedule says, because spent time is the one thing a limit protects and the schedule has no bearing on it. Every other loosening, such as removing a site or an app or uncovering schedule minutes, keeps the earlier rule and is locked only while the plan is active. A locked change is refused with `loosening_locked`, and requesting an unlock works whenever a change is locked, including outside schedule windows, so a person who is not currently blocked can still earn the right to raise a limit. The reasoning is in [ADR 0010](adr/0010-raising-a-limit-always-needs-an-unlock.md).

### Clock tampering

Time comes from the machine, so the daemon cannot stop a clock set forward, but it makes sure that setting the clock back gains nothing. The unlock state compares what every phase has left with what it was granted. An unlock whose remaining time exceeds the minutes it was granted is relocked, a cooldown whose remaining time exceeds its step restarts with the full step, and a confirmation window that exceeds its length is dropped. Day counters, which are the unlocks used today and the record of exhausted limits, reset only when the local date moves forward, so a date in the past neither hands out a fresh count nor frees a spent limit, and a record dated after today still blocks. The reconciler adopts the present time on every beat and every request, and the result is saved with the next write instead of being a reason to fail.

## Status delivery

The `StatusHub` is the app's view of the daemon. It owns the latest `FocusStatus`, which is either `Connected` with the full `DaemonStatus` or `Unavailable` with a reason, and every exchange with the daemon goes through it, so state and exchanges stay in the same order. An exchange lock is held across the daemon call and the storing of its result, so a slow poll can never overwrite the answer to a newer mutation. A request that is safe to repeat, namely status, rule updates, relock and limit reports, is tried a second time after 150 milliseconds when it failed in a way that can be a blip, and unlock requests and confirmations are never repeated because the second attempt would fail or change meaning. A connected status survives one failed exchange and becomes unavailable on the second, because a single miss is far more often a busy daemon than an outage. A refused request leaves the status alone and triggers a refresh instead, since a refusal often means the app's picture was stale.

Listeners hear about a status only when it changed, and deliveries are serialized under their own lock so that a later delivery always reads a status at least as new and a stale one can never land last. Two listeners are attached at startup. One emits the `status-changed` event to the webview and updates the tray, and the other watches for a cooldown that ended and posts the notification described below. Commands read and mutate the same hub, so they see exactly what the listeners see.

The poller asks for the status every 5 seconds, and it shortens that wait to the next unlock phase deadline, which is when a cooldown becomes ready, a confirmation window lapses or an unlock expires, so those changes show up within about a second. A deadline that has already passed waits a quarter of a second instead of spinning. The `get_focus_status` command always asks the daemon rather than reading the cache, because a cached answer could show a phase that ended seconds ago.

## Notifications

Notifications go through a `Notifier` trait that the Tauri notification plugin implements, so enforcement never depends on Tauri and delivery is best effort. Enforcement posts a notice for a blocked app, a blocked site and a limit warning, each worded in `enforcement/notice.rs` with the time the block ends. A blocked app or site goes to the block screen described below, and its notification is only the fallback for a window that cannot be shown, so the limit warning is the one notice that is always a notification. A further notice titled "Unlock ready" comes from the status layer. It fires once when a status delivery moves the unlock phase from cooling down to ready, it remembers the last phase across stretches where the service is unreachable so that a cooldown that ended out of reach still alerts, and it stays quiet for a ready state that the app merely started up in. The app also posts one notice when it had to start a new usage database. Clock times in notifications and in the tray follow the macOS 12 or 24 hour setting through the `HourCycleSource` port, which is read each time so a change applies at once, and the UI reads the same setting through `get_clock_style`. Notification text may name an app or a site, which is allowed because it is shown to the person on their own Mac and is not logged.

## App Nap

A hidden window makes the app a candidate for App Nap, which stretches timers by seconds or minutes and would make the 2 second sampler, the status poller and enforcement unreliable. At startup the app begins an `NSProcessInfo` activity with the user-initiated option that still allows idle system sleep, so the Mac sleeps normally while the app keeps its schedule, and the activity ends when the workers stop.

## Window, tray and launch at login

Closing the main window hides it, so tracking continues, and the tray menu or a click on the Dock icon brings it back. The tray offers the status line, Open Grumpo, Relock now, which is enabled only while an unlock is running, and a quit item. Quitting the app stops it from quitting apps and closing tabs, although the daemon keeps the hosts block, so while blocking is active the quit item reads "Quit Grumpo (stops app and tab blocking)" and reads "Quit Grumpo" otherwise.

The login item is installed through the autostart plugin with the argument `--autostarted`, under the label `io.github.brian-rey-development.grumpo`, which is also the name of its property list in `~/Library/LaunchAgents` and what System Settings shows. An app that finds the argument leaves its window hidden so it comes up quietly in the menu bar. On the first run of an installed copy the item is enabled and the fact is recorded, so a later opt-out in Settings sticks, and on every later start an enabled item is rewritten with the current path so a moved or updated app does not leave it pointing nowhere. The label changed with the bundle identifier when the product became Grumpo, so the first start of `Grumpo.app` looks for the property lists that earlier releases left, which are named `io.github.brian-rey-development.focus.plist` and `Focus.plist`. When one exists the person had the item on, so the item under the new label is enabled first and the old lists are deleted after, which makes a crash between the two steps harmless, and a person who had turned it off has no old list and gets no item. Nothing is registered from a disk image, from a translocated copy or from a debug build, whose path would not survive. `uninstall.sh` removes the current name and both earlier ones.

The app data folder is named after the bundle identifier, so the rename to `io.github.brian-rey-development.grumpo` would have orphaned the usage database. `legacy_data::adopt_legacy_data` runs when the paths are resolved, before the database opens, and renames the folder of the earlier identifier to the new one when only the earlier one exists or when the new one is empty. A rename is applied completely or not at all, so a crash leaves the data in exactly one place, and when both folders hold data the new one wins and the old one is left alone. If the rename is refused the app opens the database in the old folder for that run and retries at the next start. Whether the welcome flow has been completed is the setting `onboarding_completed` in the same table that holds the retention period, and it moves with the database.

## Signing and permissions

`scripts/build-release.sh` signs the daemon first, in a staging copy, and bundles it into `Grumpo.app` together with the installer, through a Tauri configuration overlay that only this script passes. Signing the daemon before the app is sealed keeps its own identifier and hardened runtime, and the overlay keeps `cargo test`, `tauri dev` and `scripts/check.sh` working when the bundle inputs do not exist. The script then checks that the finished app holds `Contents/MacOS/grumpod` and `Contents/Resources/installer/`, and verifies the bundled daemon signature as well as the app. It signs `Grumpo.app` and `grumpod` with the keychain identity named by `FOCUS_SIGNING_IDENTITY` when it is set, and ad hoc otherwise. The identity reaches the Tauri build through its configuration, because Tauri signs the bundle before it builds the disk image, and the build runs without `APPLE_SIGNING_IDENTITY`, `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` so that the environment cannot override it. The script then fails unless `codesign --verify --deep --strict` passes, the signature identifier equals the bundle identifier, the signature is ad hoc or carries the configured `Authority`, and the Apple Events entitlement is present. The daemon is signed with the identifier `io.github.brian-rey-development.grumpod` and the hardened runtime, and checked with `codesign --verify --strict`. That entitlement, `com.apple.security.automation.apple-events` in `app/src-tauri/Entitlements.plist`, is what lets the signed app ask to automate browsers, and `Info.plist` carries the usage description macOS shows with the request.

The mode matters because macOS stores the Automation and Accessibility grants against the code signature. A stable self-signed certificate keeps the grants across rebuilds, whereas an ad hoc signature changes with every build and macOS asks again for Automation, while Accessibility fails silently without a prompt. After an ad hoc rebuild the person runs `tccutil reset Accessibility io.github.brian-rey-development.grumpo` and grants it again, and a stuck Automation grant is cleared with `tccutil reset AppleEvents io.github.brian-rey-development.grumpo`, after which the person allows Grumpo again from Settings. A copy that arrives by download is quarantined, and macOS refuses it until the person chooses Open Anyway in System Settings under Privacy & Security.

## Installing the service from the app

The `install_service` command runs the installer bundled in `Grumpo.app` as root, through `/usr/bin/osascript` and `do shell script ... with administrator privileges`, so macOS shows its standard password prompt, and [ADR 0011](adr/0011-app-installs-the-service-through-the-admin-prompt.md) records why. The command takes no arguments and calls the `ServiceInstaller` port. The macOS adapter finds the installer from `current_exe()`, which must sit in `Contents/MacOS` of a bundle that holds both `Contents/Resources/installer/scripts/install.sh` and `Contents/MacOS/grumpod`, and otherwise reports `serviceInstallUnavailable`, which is what a development build or another system gets. It passes `env -i` with a fixed `PATH` and `HOME`, the shell with a fixed bootstrap, the installer folder, the daemon and the uid of the app process as arguments of a fixed AppleScript that applies `quoted form of` to each one, so no path reaches the script source. The bootstrap copies the installer folder and the daemon into a root-owned folder under `/private/var/tmp`, checks the copied daemon with `codesign --verify --strict` and runs `install.sh` from the copy. A second request while an install runs, or after a timeout while the installer still holds `/var/run/grumpod-install.lock`, returns `serviceInstallBusy`. A dismissed prompt (error `-128`) returns `cancelled`, any other failure returns `serviceInstallFailed` after logging a fixed sentence at warn level and the tail of the installer output at debug level with home folders blanked, and a success returns `installed` and refreshes the status hub at once. The installer sets its own `PATH` and needs no `SUDO_USER` or terminal, so the same script serves this flow and the Terminal path.

## Failure behavior

The app is built to be useful and honest when the daemon is not. If the daemon is not installed, not answering, owned by another user, older than the app or newer than the app, the hub reports the matching reason, and the UI shows it in the sidebar status, on the Today screen and in the Grumpo service section of Settings, the tray shows it in its status line, and commands that need the daemon fail with a `daemonUnavailable` or `versionMismatch` error that the UI words. Tracking and reports keep working, because they depend only on the local database.

Enforcement from the app stops, because without a status the policy knows nothing about the rules and does nothing, and it reports nothing. This is the safe direction, since the app would otherwise have to guess. The hosts block does not depend on the app at all, because launchd keeps the daemon running and it enforces the schedule and the exhausted limits by itself. Usage keeps accumulating in the store, so a limit passed while the daemon was away is reported once it is back. When the version is the problem, a service that is too old is fixed by Update service in Settings, which runs the bundled installer again, and an app that is too old by updating the app, and neither side tries to talk past the mismatch. `delete_all_data` treats an unreachable daemon as if limits existed and keeps today's usage, since it cannot know that no limit depends on it.

## The block screen

When enforcement closes a blocked tab or quits a blocked app, the person sees the block screen, a window of about 420 by 560 points with the label `blocker` that loads `blocker.html`, a Vite entry of its own so the main bundle is not loaded for it. The executor turns the block into a `BlockScreen { target, reason, until, strict }` through `block_screen` in `enforcement/notice.rs` and hands it to the `BlockScreenHub`, which stores it as the current content and asks the surface to show it. The window is created on first use and reused afterwards, it is centered, cannot be resized, floats above other windows, is visible on every Space and takes focus. A block that arrives while it is open replaces its content under a new id and emits `block-screen` before the window is ordered front, so windows never stack, and a page that has only just started reads the content with `get_block_screen` because the event may have come before it listened. `dismiss_block_screen` (with the id of the block being dismissed, which a stale call cannot use to hide a newer block) and the close button both clear the content and hide the window, after a `block-screen-hidden` event, and when the main window is not open the app hides itself too, so macOS gives focus back to the app the person was in.

The reason and `strict` come from what the policy already knows. A target held only by a spent limit is `limit`, a scheduled target is `schedule`, and a scheduled target whose window never ends is `always` and has no `until`. When a schedule and a spent limit hold the same target the block is described as a schedule block, because that is what lifts it. `strict` is true while enforcement is active and no unlock is running, which is the state in which the daemon refuses to loosen rules, so leaving needs the cooldown unlock. The surface never waits for the main thread, because enforcement runs on its own worker and the main thread joins that worker on exit, so building or showing the window is queued, and a failure there, including a queue the main thread refuses, runs the notification fallback exactly once. [ADR 0012](adr/0012-grumpo-rebrand-and-block-screen.md) records the decision.

## Capabilities and the webview

The webview is granted permissions through capability files in `app/src-tauri/capabilities/`. `blocker.json` is scoped to the `blocker` window and grants it only `allow-get-block-screen`, `allow-dismiss-block-screen`, and the event listen and unlisten permissions, so the block screen cannot call any other command and cannot move itself. The two files below are scoped to the `main` window. `default.json` holds the generic Tauri permissions the UI needs, which are listening for and unlistening from events, starting a window drag from the title bar and the toggle-maximize permission that Tauri's drag region uses when the title bar is double clicked. `commands.json` holds one `allow-<command>` permission for each app command. The commands are declared in an app manifest in `build.rs`, which makes Tauri generate those permissions under `permissions/autogenerated/` and refuse any command call that a capability does not grant. The notification and autostart plugins are used by the Rust side only, and no capability gives the webview access to them. The content security policy sets `default-src 'none'` and permits only the local IPC bridge, so a bug in the UI cannot reach the network either.

## The platform ports pattern

Everything that touches the operating system sits behind a trait in `app/src-tauri/src/platform/ports.rs`, namely `ActivityProbe`, `AppControl`, `IconProvider`, `AppLocator`, `PermissionProbe`, `HourCycleSource`, `Dialogs` and `ServiceInstaller`. The function `platform::current()` returns a `Platform` that bundles the adapters for the compiled target. The rest of the backend, including the observer, the session builder, the policy, the formatter for clock times and the commands, depends only on the traits, which is why it can be unit tested with fakes and why it contains no `cfg(target_os)` branches. The `HourCycleSource` port answers whether the system uses a 12 or 24 hour clock, the `Dialogs` port shows the native alert for failures that stop the app before its window exists, and the `ServiceInstaller` port installs or updates the privileged service and is unavailable on systems without a bundled installer. The App Nap assertion follows the same idea through `begin_tracking_activity`, which does nothing on systems without such throttling.

The macOS adapters live in `platform/macos/`, where `unsafe` is permitted and each block carries a `// SAFETY:` note. The daemon has the same arrangement in a single module, `sys.rs`, which wraps the libc calls for peer credentials, the effective uid and `dup2`. Other targets resolve to `platform/unsupported.rs`, whose adapters return an `Unsupported` error, so the app compiles and runs everywhere but reports that tracking is unavailable.

Adding a Linux or Windows implementation therefore means writing a new module such as `platform/linux/` that implements the ports, wiring it into `platform::current()` behind the matching `cfg`, and removing the target from the stub. On the daemon side, `paths.rs` currently holds the macOS defaults, so a port also adds the equivalent service definition (systemd or a Windows service), the hosts file location, and a socket replacement such as a named pipe on Windows, and it adds the new targets to `[graph]` in `deny.toml`. The pure crate `focus-core` needs no change apart from its Unix-only transport, because its hosts renderer and enforcement plan are platform independent.

## Observability

Both binaries use `tracing` with structured fields. The app writes to `~/Library/Logs/Grumpo/` with daily rotation and seven files retained, and debug builds also write to stderr.

The daemon writes to stderr, and under launchd the plist points both stdout and stderr at `/Library/Logs/Grumpo/grumpod.log`. That file is rotated by newsyslog at 1024 KB with five compressed archives kept, and the configuration deliberately sends no signal to the process. A daemon that simply inherited launchd's descriptor would then keep writing into the archived file and, once that is compressed and unlinked, into nothing. The plist therefore also sets `GRUMPOD_LOG_FILE` to the same path. When the variable is set, the daemon opens the file in append mode and uses `dup2` to point its stdout and stderr at it. On every reconciler beat, which is every 15 seconds, it compares the device and inode of the path with the file it holds, and if newsyslog has renamed or removed the file it reopens the path and repoints both descriptors, logging that it did so. If the file cannot be opened at startup, the daemon warns and keeps stderr as it is. Domains and app names are never logged above `debug` level, as detailed in [privacy.md](privacy.md).
