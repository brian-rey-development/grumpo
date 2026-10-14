# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Grumpo reads the site you're on in Firefox. Firefox builds the interface tree that Accessibility reads only after an app asks for its role, which Grumpo never did, so every Firefox window looked empty, Firefox time was counted per app only and blocked sites in Firefox were left to the hosts file alone.
- Closing tabs and counting daily limits follow the hosts file. A rule for youtube.com closes youtube.com, www.youtube.com and m.youtube.com, and leaves music.youtube.com open and outside the YouTube limit, where before the app closed any subdomain and counted it toward the limit.

### Changed

- A blocked site in Firefox brings up Grumpo's block screen, as in the other browsers. The Firefox tab stays open, because Firefox offers no reliable way for another app to close or redirect a tab, and the hosts block keeps the site from loading. Before, Firefox showed its no connection page, or a saved copy of the site, and Grumpo did nothing.
- The block screen centers Grumpo and groups the time left and the strict mode note in one card, with the button as wide as the card.

## [0.1.1] - 2026-10-14

### Added

- Grumpo updates itself. Two minutes after it starts and then once a day it checks the latest GitHub release, installs a newer version after verifying its signature, and restarts into it in the background once no Grumpo window is open. Settings has an Update automatically switch, on by default.
- Releases are signed with a self-signed certificate, so Accessibility and Automation grants carry over to every update. `scripts/create-release-keys.sh` creates the certificate and the updater key, and the release workflow signs with them, attaches the update archive and `latest.json`, and can run by hand as a dry run.
- The website shows the install steps, including Open Anyway, right after the download starts, and offers a Terminal install command that skips the macOS warning.
- Permission rows check again every two seconds while something is not allowed yet, so a switch turned on in System Settings shows up at once.

### Changed

- The website, the README and the privacy document now say that the only network request is the daily update check.
- The background service is now called `grumpod` everywhere a person can see it, including its label, binary, socket, its folders under `/Library/Application Support/Grumpo` and `/Library/Logs/Grumpo`, its environment overrides `GRUMPOD_*` and the `grumpo: managed block` markers in `/etc/hosts`, and the app logs to `~/Library/Logs/Grumpo`. Installing the service migrates a `focusd` install, carrying its rules over and replacing its hosts block, and the app shows Update service while `focusd` still runs. [ADR 0014](docs/adr/0014-the-daemon-is-renamed-grumpod.md) records the decision.
- The hosts file lists each site with only the aliases it really serves. A registrable domain such as `youtube.com` gets `www.` and `m.`, plus `old.`, `new.` and `i.` for Reddit and `mobile.` for Facebook, Twitter and X, while a subdomain such as `m.youtube.com` is written as it is. Names that never existed, such as `m.m.facebook.com` and `music.9gag.com`, are gone, `amp.` is no longer written, and blocking YouTube no longer blocks YouTube Music, which is blocked only when it is a site of its own. [ADR 0015](docs/adr/0015-aliases-only-for-registrable-domains.md) records the decision.

### Fixed

- Allow on the Accessibility permission now removes the entry an older build left in System Settings before asking again. Ad hoc signed builds change their code hash on every release, so macOS showed Grumpo as turned on while refusing the new build.
- The header of the Permissions and Done steps of the welcome no longer splits the mascot from the heading and wraps the heading one word per line when a wide list sits below it.
- A permission row with two actions stacks them, so its text and the Why Grumpo needs this panel keep most of the row, and the panel now opens and closes with a height animation instead of pushing the list down at once.
- Firefox appears in the permission list with its status tied to Accessibility, instead of being left out.

## [0.1.0] - 2026-10-13

### Added

- A landing page in `site/`, built with Astro and Tailwind CSS and deployed to GitHub Pages by `.github/workflows/site.yml`. Grumpo reacts in the hero when you try a blocked site, the unlock section plays the real cooldown ladder and sentence, and the tour and install steps use real screenshots. It ships schema.org data for the app, the FAQ and the install steps, a sitemap, `robots.txt` that welcomes AI crawlers, `llms.txt`, a web manifest and a 404 page, and it sets no cookies and loads no third-party resources.
- `scripts/brand/generate.py`, which renders the README cover, the GitHub social preview, the Open Graph image, the logos, the app icon, the favicons and the site images from `brand/` and the screenshots in `docs/images/`, using the app's own Fredoka and Nunito fonts.
- `scripts/package-dmg.sh`, which packs the signed `Grumpo.app` into `dist/release/Grumpo.dmg` with an Applications link and a SHA-256 checksum, and `.github/workflows/release.yml`, which builds the universal app on a `v*` tag that matches the app version and attaches both files to a draft release. The fixed file name lets the website link to the latest download.
- Dual licensing under MIT or Apache 2.0, `brand/LICENSE.md` for the brand assets, `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, issue and pull request templates, and favicons for the app windows.
- The commands `get_onboarding_state`, `complete_onboarding` and `reset_onboarding` and the `OnboardingState` type, which keep in the settings table whether the person has finished the welcome flow and let Settings show it again.
- A block screen. When enforcement quits a blocked app or closes a blocked tab, the `blocker` window appears with the target, a countdown to the end of the block and whether strict mode is on, and it goes away with Back to work or Escape. The backend adds the `BlockScreen`, `BlockTarget` and `BlockReason` types, the `get_block_screen` and `dismiss_block_screen` commands and the `block-screen` event, plus a `blocker.json` capability that grants that window only those commands. The reason and strict flag come from the daemon status and the enforcement plan, and the notification remains the fallback when the window cannot be shown. [ADR 0012](docs/adr/0012-grumpo-rebrand-and-block-screen.md) records the decision.
- Install and update the Grumpo service from the app with the standard macOS password prompt. `Grumpo.app` now carries the signed `focusd` in `Contents/MacOS` and the installer in `Contents/Resources/installer/`, and the new `install_service` command runs it as root through `osascript` with administrator privileges. It returns `installed` or `cancelled` (the `ServiceInstallOutcome` type), fails with `serviceInstallUnavailable` when no installer is bundled, as in a development build, or `serviceInstallFailed` when it ran and failed, and refreshes the service status right after a successful install. The `ServiceInstaller` port has a macOS implementation and an unsupported stub, the AppleScript is fixed text that reads quoted `argv` values, and the owner is the uid of the app process. [ADR 0011](docs/adr/0011-app-installs-the-service-through-the-admin-prompt.md) records the decision and why `SMAppService.daemon` waits for a Developer ID.
- `focus-core`, a pure library with the rule model, validation, schedule math, strictness checks, the escalating unlock state machine, the enforcement plan, hosts file rendering, the legacy `focus-hosts` importer and the versioned IPC protocol (version 3).
- `focusd`, the privileged LaunchDaemon that owns rules and unlock state, persists them atomically with schema versioning, reconciles the marked `/etc/hosts` block every 15 seconds, on every change and once just after each local midnight, and serves a Unix socket restricted to the owner uid. It has the subcommands `run`, `import-legacy`, `strip-hosts` and `ping`, holds an instance lock so a second daemon or an offline edit cannot race it, honors the `FOCUSD_STATE_DIR`, `FOCUSD_HOSTS_FILE`, `FOCUSD_SOCKET` and `FOCUSD_LOG_FILE` overrides, and reopens its log file after newsyslog rotates it.
- `scripts/install.sh` with `--binary <path>`, `--owner <user|uid>` and `--skip-legacy-import`, `scripts/uninstall.sh` with `--purge` and `--owner`, shared helpers in `scripts/lib/`, and a LaunchDaemon plist and newsyslog configuration in `packaging/macos/`. The installer is idempotent, rolls back a failed install including the saved `state.json`, imports the domain list and schedule of an existing `focus-hosts` setup and then removes it. The uninstaller also removes the Focus login item, under both its current label and the older `Focus.plist`.
- `scripts/build-release.sh`, which builds `focusd` for the host architecture or both with `--universal`, runs `pnpm tauri build` and copies `focusd`, `Grumpo.app` and the disk image into `dist/release/`. It signs the app with `FOCUS_SIGNING_IDENTITY` when set and ad hoc otherwise, then fails unless `codesign --verify --deep --strict` passes and the identifier is `io.github.brian-rey-development.grumpo`. The disk image holds only the app, so the daemon is installed by `scripts/install.sh`.
- Usage storage in SQLite with versioned migrations, day and week reports, retention of 90 days by default (30 to 365 days), JSON export and Delete history.
- A tracking engine made of an observer, a session builder and writer, a 2 second sampler and a daily pruner, with idle and media detection, along with macOS adapters for the frontmost app, browser tabs, app control, icons and permissions behind portable traits.
- Enforcement that runs on its own worker thread and never delays the sampler. It quits a blocked app that is in front, finds blocked apps running behind other windows with a check that repeats every 30 seconds while blocking is active, closes a blocked tab in the frontmost window of Chrome, Brave, Edge and Safari, and tells the person what it did with a notification.
- Daily limits per site or app that count live time, warn once with 5 minutes left and report to the daemon when the allowance runs out, so the block survives an app restart and lasts until local midnight.
- The unlock flow end to end, with an escalating cooldown of 5, 10, 20 and 40 minutes, a notification when the cooldown ends, a typed sentence and a chosen duration of 15, 30 or 60 minutes, and Relock now from the window and the menu bar.
- Protected apps. Grumpo, Finder, the login window, System Settings and the Dock cannot be blocked or limited, are left out of the app picker, and are refused by the daemon.
- A client for the daemon socket with a two second budget per exchange, a status hub that retries blips, orders deliveries and tells the UI when the service is not installed, not responding, outdated, newer than the app or owned by another user, and a poller that wakes at unlock phase deadlines.
- Tauri commands for reports, limit usage, status, rules, unlock, known apps and icons, settings, permissions, export and deletion, and the `usage-changed` and `status-changed` events.
- A menu bar item with the focus state, an unlock countdown, today's total and entries to open the window, relock and quit, which reads "Quit Grumpo (stops app and tab blocking)" while blocking is active, and a main window that hides on close.
- Launch at login through a login item that starts the app hidden with `--autostarted`, enabled by default on the first run of an installed copy and managed from Settings.
- An App Nap activity assertion so the sampler, the poller and enforcement keep their schedule while the window is hidden.
- The Today, Week, Rules and Settings screens and the unlock sheet, built on a design system of tokens, an app shell with navigation and a component library, with loading, empty and error states and keyboard access throughout.
- A typed `ipc` layer with a `CommandMap`, an `EventMap`, TanStack Query hooks, and test fixtures in `app/src/test/ipc`.
- TypeScript bindings generated from Rust with ts-rs, checked for freshness by `scripts/check.sh`.
- Repository platform with a pinned Rust toolchain, EditorConfig, a cargo-deny policy scoped to the Apple targets, GitHub Actions CI that runs the quality gates, shellcheck on the scripts and a signed production build of the app with a strict `codesign` verification, Dependabot configuration that groups Tauri updates, and the MIT license.
- Documentation covering the architecture, privacy, the threat model, development and the architecture decision records.

### Changed

- The product is now called Grumpo, the strict focus blocker whose tagline is "Grumpo says no. You get things done." The bundle is `Grumpo.app` and the window title, tray tooltip and menu, notifications, password prompt, scripts and documents use the new name, with a new app icon and menu bar template made from the brand kit in `brand/`. The app identity is renamed as well, to the bundle id `io.github.brian-rey-development.grumpo` and the main executable `Grumpo`, because System Settings listed the app as Focus with the old icon under Accessibility and as focus-app under Background Activity, while the daemon `focusd`, its label, socket, state and log paths and the crate names keep their identifiers, as [ADR 0012](docs/adr/0012-grumpo-rebrand-and-block-screen.md) explains. Remove the old `/Applications/Focus.app`, which the install command in the README does, and clear its stale permission entries with `tccutil reset All io.github.brian-rey-development.focus`.
- The first start of the renamed app migrates what the old identity left. The app data folder, with the usage database and the settings, is moved to the folder of the new bundle id with a single rename, so a crash cannot split it, nothing is moved on a fresh install or when the new folder already has data, and a refused move keeps the old folder in use until the next start. The login item under the old labels `io.github.brian-rey-development.focus` and `Focus` is replaced by one under the new label, which is enabled before the old property list is deleted. Tests cover a fresh install, old data only, both present, an interrupted start and each login item case. The daemon needed no change, because it authorizes the socket peer by uid, and the protected apps list now holds both bundle ids.
- The administrator prompt that installs the service now says "Grumpo needs your password to install the service that blocks sites." through `with prompt`.
- The limit warning and the block notifications use Grumpo's voice, and a block notification is sent only when the block screen cannot be shown.
- `scripts/build-release.sh` signs `focusd` before the app is sealed, passes a Tauri configuration overlay that bundles it with the installer, so `cargo test`, `tauri dev` and `scripts/check.sh` do not need those files, and fails unless the finished app holds both and `codesign --verify --strict --deep` passes. The README install section now builds, copies the app to `/Applications` and uses Install service in the app, and keeps the Terminal command as the developer alternative.
- `scripts/install.sh` fixes its own `PATH`, so it runs under the minimal environment of the macOS administrator prompt, which sets no `SUDO_USER` and no terminal.
- The threat model and the architecture document describe root running a script from the app bundle, which sits in the administrator-writable `/Applications`.

- `scripts/build-release.sh` signs `focusd` with the same identity as `Grumpo.app`, with the identifier `io.github.brian-rey-development.focusd` and the hardened runtime, and verifies it with `codesign --verify --strict`. It also asserts `Signature=adhoc` or the `Authority` of `FOCUS_SIGNING_IDENTITY` for both, and runs the Tauri build without `APPLE_SIGNING_IDENTITY`, `APPLE_CERTIFICATE` and `APPLE_CERTIFICATE_PASSWORD` so the environment cannot override the identity.
- The app install command removes `/Applications/Grumpo.app` before `ditto` copies the new one, so a stale file never breaks the signature seal. The README now says to run `tccutil reset Accessibility io.github.brian-rey-development.focus` after each ad hoc rebuild, and its opening paragraph says that raising or removing a daily limit always needs an unlock.

### Fixed

- A failed install now deletes the `state.json` the new daemon or the legacy import wrote when there was none before, waits until the new daemon is unloaded before it restores state, and warns when `strip-hosts` fails.
- `install.sh` copies `state.json` only after the old daemon has stopped, so a save made during shutdown is not lost.
- `resolve_owner` reads home directories through `dscl -plist`, so a home path with spaces is no longer cut short.

### Security

- The daemon owns the rules, the unlock state and the hosts block, accepts connections only from the owner uid and root, and computes strictness itself, so a modified app cannot loosen a rule during enforcement without the unlock flow.
- Every message travels in a version envelope that is checked before the body is parsed, and a mismatch is reported as a service that needs updating or an app that needs updating according to which side is older.
- A limit report carries the allowance that ran out, and the daemon ignores a report that no longer matches the rules, so an edit that races the report cannot block a target the person has since freed.
- Saved state is versioned, and its migration drops protected apps from old rules and gives a pending cooldown its real length.
- Raising or removing a daily limit always needs an active unlock, whatever the schedule, while adding or lowering one stays free. An unlock can be requested whenever a requested change is locked, including outside schedule windows.
- Each blocked site is written to the hosts file for the bare domain and the prefixes `www.`, `m.`, `mobile.`, `old.`, `new.`, `music.`, `i.` and `amp.`, in IPv4 and IPv6.
- Setting the clock back cannot stretch an unlock or a cooldown, and day counters reset only when the date moves forward. A limit report carries the reporter's local day, and the daemon ignores one that is not for its current day.
- The daemon quarantines saved state only when it is corrupt, and exits with an error on any other load failure so launchd retries and the existing hosts block stays in place. The installer also keeps a copy of `state.json` and restores it when an install fails.
- `Grumpo.app` is signed with a stable identifier and the Apple Events entitlement, so macOS permissions follow the app across rebuilds when `FOCUS_SIGNING_IDENTITY` names a certificate.
- Web-app wrappers, such as installed Chrome apps and Safari web apps, count as separate apps and escape site limits. The README tells people to block the wrapper as an app.
- Rule validation rejects protected apps, and the enforcement policy skips them as a second line of defense.
- The webview runs under a content security policy with no remote origins and holds only the event, window drag and app command permissions listed in `default.json` and `commands.json`.
- The script that closes a tab embeds the observed address as an escaped AppleScript literal, skips the close when the address contains a control character, and closes the tab only if it still shows the same address.

### Privacy

- Only the app id, the app name, the website domain and timestamps are stored. URLs, paths, queries and titles are never stored or logged, and there are no network calls.
- Private browsing time in Chrome, Brave and Edge is never written to the database. It counts toward limits through an in-memory tally that is forgotten on restart and at midnight. Safari and Firefox private windows cannot be detected.
- Tabs are read and closed only in the frontmost window of the browser, so background windows are never inspected.
- Shortening the retention period deletes older history as soon as the change is saved, and Delete history keeps today's usage only while limits are configured or the service cannot be reached.
- Logs at `info` and above never contain domains or app names.

[Unreleased]: https://github.com/brian-rey-development/grumpo/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/brian-rey-development/grumpo/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/brian-rey-development/grumpo/releases/tag/v0.1.0
