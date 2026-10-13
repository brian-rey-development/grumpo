# Development

This guide covers everything needed to build, run and change Grumpo on a Mac. The project is a Cargo workspace with three members (`crates/focus-core`, `crates/focusd` and `app/src-tauri`) plus a React and TypeScript front end in `app/`.

## Prerequisites

Development currently requires macOS 13 or newer, because the tracker, the tray and the privileged daemon are macOS only in v1. The toolchain pieces are the following.

- Rust through `rustup`, at the exact version pinned by `rust-toolchain.toml` (which the workspace `rust-version` matches), installed with `rustup toolchain install` from inside the repository, since that command reads the file and also fetches both Apple targets
- Node.js 22 or newer
- pnpm, pinned by the `packageManager` field in `app/package.json` (enable it with `corepack enable`)
- Xcode command line tools, which provide the linker and the macOS SDK
- `cargo-deny` for the supply chain check, installed with `cargo install cargo-deny --locked`

### Which cargo runs

The `cargo` that comes first on `PATH` may be the one from Homebrew. It ignores `rust-toolchain.toml`, so it builds with whatever version it ships, and it carries no standard library for the other Apple architecture, which a universal build needs. The rustup proxies in `$CARGO_HOME/bin` (by default `~/.cargo/bin`) read the file and pick the pinned toolchain, so `scripts/build-release.sh` puts that directory first on `PATH` when it exists, which also makes the Tauri CLI find the same `cargo`. When a Rust target is missing, `--universal` stops and prints the matching `rustup target add` command, and `rustup toolchain install` inside the repository adds both targets in one step. For day to day work the same advice holds, so make sure the proxies come before Homebrew's `cargo` in your shell, or run `rustup which cargo` to see the one that counts.

## Setup

Clone the repository, install the JavaScript dependencies and let Cargo fetch the Rust ones on the first build.

```sh
cd app
pnpm install --frozen-lockfile
cd ..
cargo build --workspace
```

## Running the app in development

The app is a client of the daemon, so a development session has two parts. A daemon that runs as you against a temporary root, described in the next section, and the app, started from the `app` directory with `pnpm tauri dev`. That command starts the Vite server on port 1420 and the Tauri backend together, with hot reload for the UI and a rebuild of the Rust side when it changes. Both processes must agree on the socket path, and the app reads it from the same `FOCUSD_SOCKET` variable as the daemon, so give it to both.

```sh
FOCUSD_SOCKET=/tmp/focusd-dev.sock pnpm tauri dev
```

Without a daemon at that path the app still starts, and it shows the Grumpo service as not installed or not responding, which is an expected state covered by the error views. Tracking, reports and the Settings screen work in that state, and nothing is blocked. A development build behaves differently from an installed one in two ways. It never registers itself as a login item, because its path is the build directory, and it logs to stderr as well as to `~/Library/Logs/Focus/`. Tracking needs the Accessibility and Automation permissions described in [privacy.md](privacy.md), and macOS attaches them to the terminal or the built binary that launched the app, so they may need to be granted again after a rebuild. Settings has a button for each one, and a grant that stays stuck is cleared with the `tccutil reset` commands in the signing section below.

## Running the daemon against a temporary root

The daemon normally runs as root and writes the real `/etc/hosts`, which is exactly what must not happen during development. Its `paths.rs` module therefore resolves its locations from environment variables when they are set, and falls back to the system defaults otherwise. Empty values count as unset.

| Variable | Meaning |
|---|---|
| `FOCUSD_STATE_DIR` | Directory holding the persisted state file |
| `FOCUSD_HOSTS_FILE` | File treated as the hosts file, and the DNS cache is flushed only when this is the real `/etc/hosts` |
| `FOCUSD_SOCKET` | Path of the Unix socket the daemon listens on, and the one the app connects to |
| `FOCUSD_LOG_FILE` | Log file that the daemon keeps stdout and stderr pointed at, unset by default |

The first three let the daemon run as an ordinary user. The command takes `--owner-uid` as a required argument, which names the one user besides root whose connections are accepted. It must be a real, non-root user id, so `0` is rejected at parse time, and passing your own uid with `"$(id -u)"` is the right choice here. When the daemon runs as root it also hands the socket to that uid, whereas as an ordinary user it leaves the socket owned by you, so no `sudo` is needed.

```sh
root="$(mktemp -d)"
printf '127.0.0.1 localhost\n' > "$root/hosts"
FOCUSD_STATE_DIR="$root/state" \
FOCUSD_HOSTS_FILE="$root/hosts" \
FOCUSD_SOCKET=/tmp/focusd-dev.sock \
cargo run -p focusd -- run --owner-uid "$(id -u)"
```

Keep the socket path short. A Unix socket path is limited to 104 bytes on macOS, and a longer one makes the daemon, or the app, fail with the error `path must be shorter than SUN_LEN`. That is why the examples use `/tmp/focusd-dev.sock` and not a file inside the `mktemp` directory, whose name under `$TMPDIR` is already long, and why a deep checkout or a long custom `TMPDIR` can break the other form without any hint about the cause.

`FOCUSD_LOG_FILE` exists for the installed daemon rather than for development, so leave it unset to read the logs in your terminal. The LaunchDaemon plist sets it to `/Library/Logs/Focus/focusd.log`, which is also the file launchd uses for stdout and stderr. When the variable is set, the daemon opens that file in append mode and uses `dup2` to point its stdout and stderr at it. Every reconciler beat (15 seconds) then compares the device and inode of the path with the descriptor it holds, and when newsyslog has renamed or removed the file, the daemon opens the new one and repoints the descriptors. Without this step the daemon would keep writing into the archived file that newsyslog compresses and eventually unlinks. If the file cannot be opened at startup, the daemon logs a warning and keeps writing to stderr. The log verbosity defaults to `info` and `RUST_LOG` overrides it.

The integration tests for the daemon use the same overrides against a fresh temporary directory, so they never touch system files.

### Daemon subcommands

The subcommands are defined in `args.rs` and dispatched from `cli.rs`. A missing or malformed argument prints the usage text and exits with status 2, and a command that fails exits with status 1.

| Command | Behavior |
|---|---|
| `focusd run --owner-uid <uid>` | Loads the state, takes the instance lock, reconciles the hosts block once, binds the socket and serves until stopped. This is the command launchd runs. A state file that is damaged is set aside as `state.json.corrupt-<unix ts>` and replaced by defaults, whereas a file that cannot be read or that a newer daemon wrote makes the command exit with status 1 and leave the file alone, so launchd retries and the existing hosts block stays |
| `focusd import-legacy <script-path>` | Reads the domain list and schedule from an old `focus-hosts` script and writes the initial state into `FOCUSD_STATE_DIR`. It does nothing when a state file already exists, because an existing state always wins, and it fails when the script has no domains |
| `focusd strip-hosts` | Removes the current and the legacy marked blocks from the file named by `FOCUSD_HOSTS_FILE` and reports whether anything changed. `scripts/uninstall.sh` uses it |
| `focusd ping` | Connects to `FOCUSD_SOCKET`, asks for the status and prints a line when the daemon answers with a valid protocol response, or fails with the reason. It takes no arguments and is left out of the usage text, because it is a support command for installer health checks and not something to run by hand |

The daemon holds an exclusive lock on `focusd.lock` inside the state directory for as long as it runs, so a second `run` fails before it touches the socket. `import-legacy` and `strip-hosts` take the same lock while they work, which means both refuse to run while a daemon is serving from the same state directory, and the error tells you to stop it first. The kernel releases the lock when the process dies, so a crash never leaves a stale one.

The two that edit files can be tried safely against the temporary root from above, and the legacy fixture that the importer tests use is `crates/focus-core/tests/fixtures/focus-hosts.sh`. Stop the daemon before running them, and use `ping` from another terminal while it is running.

```sh
FOCUSD_STATE_DIR="$root/state" cargo run -p focusd -- import-legacy crates/focus-core/tests/fixtures/focus-hosts.sh
FOCUSD_HOSTS_FILE="$root/hosts" FOCUSD_STATE_DIR="$root/state" cargo run -p focusd -- strip-hosts
FOCUSD_SOCKET=/tmp/focusd-dev.sock cargo run -p focusd -- ping
```

## Quality gates

`scripts/check.sh` runs every gate that CI runs, in this order, and stops at the first failure.

1. `shellcheck` over `scripts/*.sh` and `scripts/lib/*.sh`, skipped with a notice when it is not installed because CI runs it anyway
2. `cargo fmt --all --check`
3. `cargo clippy --workspace --all-targets --all-features -- -D warnings`, followed by a clippy pass over `focus-core` alone, which approximates the shipped daemon configuration
4. `cargo test --workspace --all-features`, which also exports the ts-rs bindings and the generated constants to a temporary directory
5. A diff between those fresh files and `app/src/ipc/generated`, which fails when the committed ones are stale
6. In `app/`, `pnpm install --frozen-lockfile`, then `typecheck`, `lint`, `format:check` and `test`

Dependency policy is checked separately with `cargo deny check`, which CI runs as its own job next to a shellcheck job. After the gates, the CI check job runs `scripts/build-release.sh`, which performs the production build of the app, signs it and verifies the signature. The Tauri CLI enables the `custom-protocol` feature for that build, which makes the binary embed `frontendDist` and serve it under the production content security policy, something a plain `cargo build` never exercises. A pull request is expected to pass all of these before review, and `scripts/build-release.sh` reproduces the last step locally.

## Building a release

`scripts/build-release.sh` produces the artifacts that the README tells a user to install. It builds `focusd` in release mode for this Mac's architecture, or for both Apple architectures with `--universal`, in which case it merges the two binaries with `lipo`, checks that the result holds both architectures and builds the app for `universal-apple-darwin`. It then runs `pnpm tauri build` in `app/`, signs the bundle, signs the collected `focusd`, verifies both signatures and copies `focusd`, `Grumpo.app` and the `.dmg` into `dist/release/`, which is ignored by git and is deleted and recreated by every build. If the disk image cannot be created, for instance without a GUI session, the script retries with the app bundle alone and says so. The disk image holds only the app, because the daemon needs root and is installed by `scripts/install.sh`. The shared pieces of the scripts, the paths of the installed layout, the step and failure helpers, the signing functions and the rollback logic, live in `scripts/lib/`, and the toolchain choice is explained under the prerequisites.

The two commands a person runs after a build are the following. The first appears in the README and in the service section of Settings, which shows it as `INSTALL_COMMAND` in `app/src/copy/service.ts`, and the second appears in the README and in the script output, so change them together.

```sh
scripts/build-release.sh && sudo scripts/install.sh --binary dist/release/focusd
rm -rf /Applications/Focus.app /Applications/Grumpo.app && ditto dist/release/Grumpo.app /Applications/Grumpo.app && open /Applications/Grumpo.app
```

The app goes to `/Applications` because `dist/release` disappears with the next build, and because the login item stores the path of the copy that registered it. The command removes the previous copy first, because `ditto` merges into an existing bundle and a file left over from an earlier build would break the seal of the new signature. `install.sh` accepts `--owner <user|uid>` to name the account whose connections the daemon accepts, which defaults to `SUDO_UID`, the user who ran `sudo`. It takes a lock in `/var/run/focusd-install.lock` first, so a second run fails at once instead of deleting the backups of the first, and it renders the plist straight into `/Library/LaunchDaemons` and never in a temporary folder. Before it replaces anything it copies the daemon binary and its property list to `.old` files, and it copies `state.json` the same way only after the previous daemon has stopped, so a save that daemon makes while it shuts down is not lost. When a step fails the rollback unloads the new daemon and waits until launchd reports it gone, restores the three files and restarts the previous daemon, so an upgrade that goes wrong leaves the earlier version running with the earlier rules. If there was no `state.json` before the install, the rollback deletes the one the new daemon or the legacy import wrote, so that an older binary never meets a file it rejects and imported rules do not linger, and it runs `strip-hosts` and warns when that fails. After a successful install the copies are discarded.

### Signing and permissions

macOS ties the Automation and Accessibility grants to the code signature of the app, so how the app is signed decides whether a rebuild keeps them. The script reads `FOCUS_SIGNING_IDENTITY`. When it names a keychain identity, the app and the daemon are signed with it, and a stable self-signed certificate therefore keeps the grants across rebuilds. When it is unset, the script signs both ad hoc with the identity `-`, warns about it, and the grants are lost with every build because an ad hoc signature changes each time. After an ad hoc rebuild Accessibility fails silently without showing a prompt, so run `tccutil reset Accessibility io.github.brian-rey-development.grumpo` and grant it again. To create a certificate, open Keychain Access, choose Certificate Assistant, Create a Certificate, and pick the type Code Signing, then build with `FOCUS_SIGNING_IDENTITY="<certificate name>" scripts/build-release.sh`.

Tauri signs the bundle before it builds the disk image, so the identity is passed through the Tauri configuration for that run and `tauri.conf.json` keeps the ad hoc default. The build runs under `env -u APPLE_SIGNING_IDENTITY -u APPLE_CERTIFICATE -u APPLE_CERTIFICATE_PASSWORD`, so a variable in the environment cannot silently override the configured identity. The entitlements file `app/src-tauri/Entitlements.plist` grants `com.apple.security.automation.apple-events`, which a signed app needs before macOS will show the Automation prompt for a browser, and `Info.plist` holds the text of that prompt. After signing, the script runs `codesign --verify --deep --strict`, fails on any error, and checks that the identifier is the bundle identifier, that the signature is `Signature=adhoc` for an ad hoc build or carries `Authority=$FOCUS_SIGNING_IDENTITY` when an identity is set, and that the entitlement is present, so a build that would lose its permissions never reaches `dist/release`.

The daemon is signed after it is copied into `dist/release`, with the same identity, the identifier `io.github.brian-rey-development.focusd` and the hardened runtime, and the script then checks it with `codesign --verify --strict`, the identifier, the runtime flag and the same adhoc or authority assertion. The signature lives inside the Mach-O file, so `install.sh` keeps it when it copies the binary to `/Library/PrivilegedHelperTools`, and an install with the default `target/release/focusd` still works because that binary carries the linker's ad hoc signature.

When a grant gets stuck, and after every ad hoc rebuild for Accessibility, reset it and allow it again from Settings.

```sh
tccutil reset AppleEvents io.github.brian-rey-development.grumpo
tccutil reset Accessibility io.github.brian-rey-development.grumpo
```

A copy of the app that arrives by download or AirDrop is quarantined and is refused until it is allowed with Open Anyway in System Settings under Privacy & Security, whereas an app built locally opens normally.

## Regenerating TypeScript bindings

DTOs that cross the Tauri bridge derive `ts_rs::TS` behind the `ts` Cargo feature, and their TypeScript definitions are generated into `app/src/ipc/generated` by the test run, as explained in [ADR 0006](adr/0006-typescript-bindings-generated-from-rust.md). The same run writes `constants.ts` from `focus_core::bindings` and `retention.ts` from the app, which carry the Rust constants that the UI needs, so a number such as a limit cap is defined once in Rust and read in TypeScript. The same applies to the `focus-core` rule and status types that the UI renders, such as `Rules` and `DaemonStatus`, whereas the daemon wire envelope (`Request`, `Response` and `Envelope`) is deliberately not exported. The export directory is configured in `.cargo/config.toml`. After changing a DTO, run the following from the repository root and commit the result.

```sh
scripts/gen-bindings.sh
```

The script clears the generated directory (keeping `.gitkeep`) and reruns the workspace tests with all features enabled, which rewrites the files. Never edit generated files by hand, because the next regeneration discards the edit and the check in `scripts/check.sh` fails on any difference.

## Project conventions

The conventions that the reviews enforce are written out in `PLAN.md`, and the short version is below. The module map that they apply to is in [architecture.md](architecture.md).

- Rust uses edition 2021 with `clippy::pedantic` as warnings, `unsafe_code` denied outside `platform/macos` and the daemon's `sys.rs` (each block with a `// SAFETY:` note), and no `unwrap` or `expect` outside tests.
- A module with children keeps its root in `name.rs` beside a `name/` directory, and `mod.rs` is never used.
- Fakes and fixtures go in `test_support.rs` and tests in `tests.rs`, or in a `tests/` directory when there are several files, next to the module they cover. Builders that several crates share are in `focus_core::test_support`, behind the `test-support` feature that the dev-dependencies enable.
- Logging goes through `tracing` with structured fields, never `println!`, and domains and app names stay at `debug` level.
- Libraries use `thiserror`, errors are mapped once at the command boundary, and no error is swallowed silently.
- Functions are small, files stay near 200 lines, constants are named, and comments explain only a non-obvious reason. A unit of time, a limit or a timing that more than one place needs is defined once and imported, never copied.
- Wording follows the glossary in `PLAN.md`, and the UI owns all user-facing error text, keyed by code, so a Rust `message` is a diagnostic that is logged and never shown.
- TypeScript is strict with `noUncheckedIndexedAccess`, with no `any` and no non-null assertions, and only `src/ipc` may call `invoke`.
- Styling uses CSS Modules with the tokens from `design/tokens.css`, animation uses shared springs from `design/motion.ts`, and every data view has loading, empty and error states.
- pnpm is the only package manager, JavaScript direct dependencies use exact versions, and the bundle identifier `io.github.brian-rey-development.grumpo` and the daemon identifier `io.github.brian-rey-development.focusd` never change.

## Adding a Tauri command end to end

A command touches several layers, and following the same order each time keeps them consistent. Command names are snake_case on both sides of the bridge, whereas argument keys are camelCase, so a Rust parameter `end_date` is passed from TypeScript as `endDate`. The example is a hypothetical `get_streak` command that returns a number of days.

1. Define the logic in the layer that owns it. A pure calculation belongs in `focus-core` with unit tests, and a query over stored sessions belongs in `app/src-tauri/src/storage/`, where reports live in the `reports/` directory.
2. Add the DTO to the module of its area under `app/src-tauri/src/dto/` and re-export it from `dto.rs`, with `#[derive(Serialize)]`, `#[serde(rename_all = "camelCase")]` and `#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]`, then run `scripts/gen-bindings.sh` so the type appears in `app/src/ipc/generated`. A command that returns a primitive needs no DTO.
3. Write the command in the module of its area under `app/src-tauri/src/commands/`, or in a new module that you declare in `commands.rs`. It is an async function annotated with `#[tauri::command]` that takes `State<'_, AppState>`, clones what it needs and runs its blocking work through `blocking(move || ...)`, which moves it onto the blocking pool and maps any `AppError` to the serializable `CommandError` in one place. If the command can change what the daemon enforces, route it through `state.status.send(...)` rather than storing anything locally, so that the daemon stays the single owner of rules.
4. Register it in the `tauri::generate_handler!` list in `lib.rs`, which does nothing else.
5. Add its name to the `COMMANDS` list in `app/src-tauri/build.rs`. The list builds the app manifest, which is what makes Tauri require an explicit permission for every app command.
6. Grant it to the main window by adding `allow-get-streak` to `app/src-tauri/capabilities/commands.json`, where the permission name is the command name with underscores turned into dashes. The build regenerates the matching file under `permissions/autogenerated/`.
7. Add the command name and its argument and result types to the `CommandMap` in `app/src/ipc/commands.ts`, which wraps `invokeCommand`. Arguments are `null` for a command that takes none.
8. Add a TanStack Query hook, or a mutation for a command that changes something, in `app/src/ipc/queries.ts`, with its key in `queryKeys.ts`, because that is what components use. A status-returning mutation goes through `useStatusMutation`, which writes the new status into the cache. Components never import `invoke`, and only the `src/ipc` directory may call it. A command that can fail in a new way also needs its wording, which goes in `app/src/copy/failures.ts` and is chosen from the error code.
9. Test it. Add the name to `COMMAND_NAMES` in `app/src-tauri/src/commands/tests.rs`. That file holds the command-list consistency test, which fails unless the same set of names appears in `build.rs`, in `capabilities/commands.json`, in the handler list in `lib.rs` and in the `CommandMap`, so a command that is added in only some of those places is caught before it reaches a user. On the UI side, mock `@/ipc/commands` in the test and answer the command with `routeCommands` from `@/test/ipc`, which already holds fixtures for rules, status, usage and settings. A read that every screen makes belongs in `defaultHandlers.ts`, so that tests state only what they check, and a command without a handler fails loudly.

Finish by running `scripts/check.sh`, which fails if the bindings or the types are out of step.

### Adding an event

Events flow from the backend to the webview, and the webview needs no new permission for them because `default.json` already grants listening. Add a name constant and an emit to `app/src-tauri/src/events.rs`, which is where the existing `usage-changed` and `status-changed` are emitted. On the TypeScript side, add the name and its payload type to the `EventMap` in `app/src/ipc/events.ts`, where `subscribe` is typed from it, and consume it with `useBackendEvent` from a component or from `useBackendSync`, which invalidates the relevant query keys or writes the payload straight into the cache.
