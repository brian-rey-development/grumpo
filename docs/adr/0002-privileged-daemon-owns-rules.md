# 0002. A privileged daemon owns rules and the hosts file

Accepted on 2026-10-12.

## Context

Blocking websites by editing `/etc/hosts` requires root, whereas the UI, the tracker and the app control (quitting apps, closing tabs) must run as the logged-in user. The previous setup, the `focus-hosts` bash LaunchDaemon, kept its configuration in a script that the user could simply edit, so every restriction could be removed in seconds. If the rules lived in the user's process or home directory, any cooldown added to the UI would be decoration, because the same user could rewrite the rules file directly.

## Decision

A LaunchDaemon named `focusd` runs as root and is the single owner of the rules, the unlock state and the marked block in `/etc/hosts`. It persists its state in a root-owned file, reconciles the hosts file every 15 seconds and on every mutation, and exposes a small request API on a Unix socket. The user app is a client of that API for everything that affects enforcement, and it keeps only usage history locally.

## Consequences

Loosening a rule now requires going through code the user session cannot modify, which is what makes the strictness and unlock design meaningful. The price is an installer that needs `sudo`, a second binary to version and ship, and a protocol that must stay compatible across upgrades, as covered by [ADR 0007](0007-versioned-ipc-envelope-over-unix-socket.md). Any person with `sudo` can still defeat the system, which is an accepted limit described in [threat-model.md](../threat-model.md).
