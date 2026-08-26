# Security

The app stores personal customer data. Security is designed from Phase 0; hardening lands mainly in Phase 12.

## Baseline (all phases)

- Validate every Tauri command input in Rust.
- Parameterized SQL only.
- Least-privilege Tauri capabilities; no shell execution; filesystem scoped to AppData paths we own.
- Content Security Policy for the WebView.
- No secrets in source control.
- Do not log phones, addresses, private notes, or full customer records.
- Mutable data only under AppData.

## Encryption

At-rest DB encryption is **deferred** until Phase 12 after measuring impact. See [ADR 004](decisions/004-encryption-deferred.md).

Options to evaluate then: none + OS account protection, SQLCipher, DPAPI-wrapped keys. Never invent cryptography.

## Auth

v1 assumption: single workstation, no required login. Optional unlock password may be added later without redesigning domains.

## Installer

Customer installer uses embedded WebView2 offline installer — no network required for install when WebView2 is missing.
