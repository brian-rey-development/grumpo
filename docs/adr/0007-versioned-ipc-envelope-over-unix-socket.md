# 0007. A versioned IPC envelope over a Unix socket

Accepted on 2026-10-12.

## Context

The app and the daemon are installed and upgraded separately, with the daemon running as root and surviving across app updates, so there will regularly be a moment when the two disagree about the protocol. The transport also decides who can talk to the daemon at all, and it must work without opening any network port.

## Decision

The daemon listens on a Unix domain socket and checks the peer's uid with `getpeereid`, rejecting everyone except the owner uid and root. Each connection carries exactly one request and one response, each a single newline-terminated JSON frame read through a bounded reader with caps of 256 KiB and 1 MiB, and one time budget covers the request and the reply on each side, shared through `focus_core::ipc::transport`. Both directions use an `Envelope { protocolVersion, body }`, and the receiver parses and compares the version against `PROTOCOL_VERSION` before touching the body. A mismatch returns the typed error `version_mismatch`, and the app compares the two numbers to tell the person which side is behind, showing "the Grumpo service needs updating" when the daemon is older and "Grumpo needs updating" when the daemon is newer. `PROTOCOL_VERSION` in `focus-core` says which version is current. Rule payloads reject unknown fields. The envelope, request and response types stay on the Rust side and are not exported to TypeScript, as recorded in [ADR 0006](0006-typescript-bindings-generated-from-rust.md).

## Consequences

An upgrade that changes the protocol degrades into a clear message instead of a parse failure, and the version bump is the single signal that compatibility has broken. One request per connection keeps the server stateless and simple to reason about, at the cost of a new connection for each poll, which is negligible at one status request every 5 seconds. JSON over newline framing is easy to debug with standard tools, and no port is ever exposed to the network. Forward compatibility is intentionally strict, so adding a field is a coordinated change of both binaries.
