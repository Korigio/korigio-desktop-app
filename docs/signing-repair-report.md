# Release signing repair report

1. **Implemented:** Windows builds once, signs Tauri's restored standalone executable, and verifies the installer plus both standalone and packaged executables. macOS CI supports Developer ID signing and notarization with complete credentials.
2. **Architecture:** Signing stays in build scripts/workflows. No IPC, application, database, or locale changes. Tauri's own packaging/signing order is preserved.
3. **Created:** Shared Windows signing helpers, Windows regression tests, reusable Windows signing check workflow, macOS build wrapper and environment tests, macOS signing documentation, this report.
4. **Changed:** Windows build/import/sign/verify scripts, release and quality workflows, signing/release documentation, download-page signing description.
5. **Review carefully:** Expected signer pinning, temporary self-signed trust cleanup, NSIS payload extraction, and all-or-none Apple credential validation.
6. **Testing:** Four macOS environment tests passed locally; changed JavaScript syntax, formatting, and diff checks passed. Windows regression tests are wired into quality and release workflows. Actual Authenticode execution requires Windows; signed/notarized macOS verification requires the issued certificate and credentials.
7. **Expected behavior:** An unsigned, tampered, wrong-signer, or otherwise invalid Windows artifact blocks the build. Complete Apple credentials produce a signed/notarized app; partial credentials fail before building; absent credentials retain the convenience DMG.
8. **Limitations:** Changes are local and have not been released. The supplied Apple folder contains CSR/private-key files, not an issued certificate; Apple activation remains pending. Existing Windows PFX metadata cannot be changed by build configuration. Self-signing does not establish customer trust.
9. **Performance:** No application runtime impact. CI adds small signing tests and installer payload extraction; removes the second Windows packaging pass.
10. **Dependencies:** No npm/Cargo dependencies added. Windows verification uses 7-Zip; signing tests use Windows SDK and the .NET Framework compiler available on GitHub Windows runners.
11. **Security:** No signing secrets were printed, committed, or replaced. Windows verification temporarily trusts only the configured self-signed public certificate and removes newly added entries in `finally`; pre-existing trust remains. macOS relies on Tauri's temporary signing keychain. Forced process termination can interrupt cleanup, so release signing should use ephemeral runners.
12. **Status:** Repository implementation complete; platform execution and Apple certificate activation remain pending. The full signing setup is not yet verified end to end.
