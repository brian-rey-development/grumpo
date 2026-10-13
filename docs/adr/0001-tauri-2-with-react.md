# 0001. Tauri 2 with React for a cross-platform UI

Accepted on 2026-10-12.

## Context

Grumpo ships on macOS first, but the tracking and enforcement logic should not be rewritten if Linux or Windows support is added later. The UI needs rich charts, a timeline and animated dialogs, which are quick to build with web technology, and the app should stay small and idle cheaply because it runs all day in the menu bar. Electron would bundle a full Chromium and a Node runtime for what is mostly a background sampler, and a native SwiftUI app would tie the whole product to Apple platforms.

## Decision

The user-facing app is built with Tauri 2, with a Rust backend and a React and TypeScript front end rendered in the system webview. The sampler, the SQLite store, the enforcement executor and the daemon client are written in Rust, so they are shared across platforms, and only the thin adapters behind the `platform/` traits are OS specific. The front end talks to the backend exclusively through typed Tauri commands in `app/src/ipc`.

## Consequences

The bundle is small and the idle memory cost is low because Tauri reuses the operating system's webview, and the same Rust code can serve other platforms once adapters exist. The cost is that macOS webview behavior (WKWebView) is the rendering baseline for now, and that Tauri crates and JavaScript packages must be kept at matching versions, which is why they are pinned exactly. Contributors need both Rust and TypeScript skills, and macOS specific features such as the tray, vibrancy and private APIs need explicit Tauri configuration.
