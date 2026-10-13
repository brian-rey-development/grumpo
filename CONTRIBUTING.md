# Contributing to Grumpo

Thanks for considering a contribution. Grumpo is deliberately strict, so a change that makes an unlock or a bypass easier needs a reason, and the decision records in [docs/adr/](docs/adr/) explain why the current rules exist. Please read the relevant ones before changing behavior, and open an issue first for anything larger than a small fix.

## Setup

Development needs macOS 13 or newer, because the tracker, the tray and the daemon are macOS only. Install Rust through `rustup`, which picks the exact version pinned by `rust-toolchain.toml` once you run `rustup toolchain install` inside the repository, along with Node 22 or newer and pnpm, which is the only package manager used here, so please do not run npm or yarn. The `packageManager` field in `app/package.json` pins the pnpm version, and `corepack enable` provides it.

```sh
cd app
pnpm install --frozen-lockfile
cd ..
cargo build --workspace
```

[docs/development.md](docs/development.md) covers running the app, running the daemon against a temporary root so that it never touches the real `/etc/hosts`, and signing.

## Before you open a pull request

`scripts/check.sh` runs formatting, clippy, the Rust tests, the generated bindings check, and the typecheck, lint, format check and tests of the front end. It must pass, because CI runs the same script. When a change touches the Rust types that cross the IPC boundary, regenerate the TypeScript bindings with `scripts/gen-bindings.sh` and commit the result, since `check.sh` fails on stale bindings.

Keep changes focused, add tests next to the code they cover, and update the docs or add an ADR when a decision changes. Include screenshots for anything visible in the UI.

## Commits

Use conventional commits in English, such as `fix: keep the cooldown after a restart` or `feat: add a weekly report export`. The subject says what changed and the body, when there is one, says why.

## Licensing

The code is licensed under `MIT OR Apache-2.0`, and by contributing you agree that your contribution is licensed the same way, without additional terms.

The brand assets are not open source. The Grumpo name, mascot, stickers, wordmark, cover images and app icon are covered by [brand/LICENSE.md](brand/LICENSE.md), so please do not add your own versions of them to a pull request or reuse them in a fork.

## Conduct

Participation is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). Security problems go through the private process in [SECURITY.md](SECURITY.md) and not through public issues.
