# 0013. Releases carry a self-signed certificate and the app updates itself once a day

Accepted on 2026-10-13.

## Context

The first public release, 0.1.0, was signed ad hoc, which gives every build a different code hash. macOS stores the Accessibility and Automation grants against the designated requirement of the signature, and an ad hoc requirement is that hash, so every update looked like a new app. The owner's own Mac showed the result the same day, with Grumpo switched on under Accessibility in System Settings while the app reported the permission as missing, because the switch belonged to the previous build. Every person who updated would have met the same broken state, and the documented fix was a `tccutil` command.

The project has no budget for the Apple Developer Program, so notarization is out of reach. That leaves Gatekeeper's warning on the first launch of a downloaded app, which only a notarized Developer ID signature removes, but it does not block the two things that hurt most, losing permissions on every update and having to download and approve each new version by hand.

## Decision

Releases are signed with a self-signed code signing certificate, "Grumpo Release Signing", whose key lives outside the repository in `~/.grumpo-signing` and in two GitHub Actions secrets. With a certificate the designated requirement names the bundle identifier and the certificate itself, as `certificate leaf = H"…"`, instead of a hash of the build, so a grant made for one release holds for every later one. `scripts/create-release-keys.sh` creates the certificate and the updater key, `scripts/ci/import-signing-certificate.sh` imports and trusts the certificate in a temporary keychain on the runner, and the release workflow fails if the signature it produced still has a hash requirement. Local builds stay ad hoc unless `FOCUS_SIGNING_IDENTITY` names a certificate.

The app updates itself with the Tauri updater plugin. A worker thread checks `latest.json` of the latest published GitHub release two minutes after launch and then once a day by the wall clock, so a Mac that slept through the night still checks in the morning. When the manifest names a newer version, the worker downloads the archive, verifies its minisign signature against the public key in `tauri.conf.json`, installs it in place and restarts into it as soon as no Grumpo window is visible, starting the new copy with the login argument so no window appears. An update that waits for open windows is announced once with a notification. The release workflow builds the archive and the manifest with `scripts/package-update.sh` and attaches them to a draft, so nothing reaches installed copies until a person publishes the release. A file the app downloads itself is not quarantined, so updates never show the Gatekeeper warning.

Automatic updates are on by default, because an app that enforces rules should not stay on a version with known bugs, and Settings has a switch to turn them off. The worker does nothing in a development build and nothing while the app runs from a disk image or a translocated copy, where there is no installed bundle to replace.

The privacy promise changes from no network requests at all to one request, the daily check, which is a plain HTTPS download of a public file from GitHub that carries no identifier, no usage data and no settings. The webview keeps its content security policy, so only the Rust side ever reaches the network.

## Consequences

People who installed 0.1.0 grant Accessibility and Automation once more when they move to the first certificate-signed release, and never again after that. Losing the certificate would cost everyone one more round of grants, and losing the updater key would strand every installed copy on its version, so both are backed up outside the repository and the script that creates them refuses to replace existing ones.

The first launch still shows Gatekeeper's warning, which the website explains step by step and which the Terminal install command avoids, since `curl` does not quarantine what it downloads. Only notarization would remove it, and this record should be revisited if the project ever gets a Developer ID.

The service keeps running across an app update. It only needs reinstalling when the IPC protocol version changes, which the app already detects and offers to fix with the administrator prompt of [ADR 0011](0011-app-installs-the-service-through-the-admin-prompt.md).
