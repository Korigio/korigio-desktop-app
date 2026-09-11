# Windows code signing

Korigio’s shop installer is the **NSIS** `Korigio_*_x64-setup.exe`. This document covers **Windows Authenticode** signing of that installer and the main app exe. It is not Tauri’s updater key (`TAURI_SIGNING_PRIVATE_KEY`).

A **self-signed** certificate is for development, CI rehearsal, and internal testers. It puts **Moritz Alexander Wright** (organization **Korigio**) on the signature. It does **not** make Microsoft SmartScreen trust the app on a customer PC.

```text
SELF-SIGNED
    ↓
Authenticode signature exists
    ↓
cryptographically signed (identity is visible in certificate details)
    ↓
BUT
    ↓
not automatically trusted on normal customer Windows installations
```

`npm run tauri dev` never needs a certificate.

## Publisher identity (what maps where)

| Identity                    | Where it lives                                           | What users typically see                                          |
| --------------------------- | -------------------------------------------------------- | ----------------------------------------------------------------- |
| **Moritz Alexander Wright** | Cert `CN=`; also `bundle.publisher` in `tauri.conf.json` | Certificate identity; Apps & Features (UAC needs a trusted chain) |
| **Korigio**                 | Cert `O=`; `productName`                                 | Product / installer name; organization in full cert Subject       |
| **info@korigio.com**        | Cert `E=` / `emailAddress=`                              | Full Subject only — not the UAC headline                          |
| **https://www.korigio.com** | `bundle.homepage` (not an Authenticode DN field)         | Installer / app metadata — not the signature publisher line       |

Recommended cert Subject (created by the scripts below):

```text
CN=Moritz Alexander Wright, O=Korigio, E=info@korigio.com
```

## First-time Windows setup

Create the certificate **once** and reuse it. A new certificate every release is a new unknown publisher.

### Option A — Windows (PowerShell)

On a Windows 10/11 machine or VM, from the repo:

```powershell
Set-ExecutionPolicy -Scope Process Bypass
./scripts/create-windows-signing-cert.ps1
```

That:

1. Creates a `CodeSigningCert` (SHA-256) in `Cert:\CurrentUser\My` with subject `CN=Moritz Alexander Wright, O=Korigio, E=info@korigio.com`.
2. Prints the **thumbprint**.
3. Optionally exports a password-protected `.pfx` (password from prompt, or env `WINDOWS_CERTIFICATE_PASSWORD`).

The `.pfx` contains the **private key**. Keep it off Git, chat, and the public downloads repo.

### Option B — macOS (OpenSSL), then CI only

If you have no Windows box, you can still create a PFX on macOS and let GitHub’s Windows runner import it:

```bash
openssl req -newkey rsa:4096 -nodes -keyout korigio-codesign.key -x509 -days 1825 \
  -subj "/CN=Moritz Alexander Wright/O=Korigio/emailAddress=info@korigio.com" \
  -addext "extendedKeyUsage=codeSigning" \
  -addext "keyUsage=digitalSignature" \
  -out korigio-codesign.crt
openssl pkcs12 -export -out korigio-codesign.pfx -inkey korigio-codesign.key -in korigio-codesign.crt
```

Delete `korigio-codesign.key` after the `.pfx` exists. Those filenames are gitignored.

## Find certificate thumbprint

On Windows, after import:

```powershell
Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert
```

Use the `Thumbprint` column (SHA-1 hash of the cert). That value is public; the private key is not.

Copy `src-tauri/tauri.windows-signing.example.json` to `src-tauri/tauri.windows-signing.json` (gitignored) and put the thumbprint there, **or** set:

```powershell
$env:WINDOWS_CERTIFICATE_THUMBPRINT = "YOUR_THUMBPRINT"
```

Do not commit a machine-specific thumbprint into `tauri.conf.json`. Without a thumbprint, Windows `tauri build` stays unsigned so other machines keep working.

## Build

Unsigned (macOS, Linux, or Windows without a cert) — unchanged:

```bash
npm run tauri build
```

Signed NSIS on **Windows** when a thumbprint/config is present:

```bash
npm run build:windows
```

That runs:

1. One `tauri build --bundles nsis`. Tauri patches the app for NSIS, signs that payload, builds/signs the installer, then restores the original unsigned executable.
2. Signs the restored standalone `repair-manager.exe`, without re-bundling.
3. Verifies the standalone exe, exact current-version installer, and the app extracted from that installer using 7-Zip. Extraction does not launch or install anything.

The ordering is documented in [the pinned Tauri CLI 2.11.4 bundler source](https://github.com/tauri-apps/tauri/blob/tauri-cli-v2.11.4/crates/tauri-bundler/src/bundle.rs#L137-L188). An unsigned file in `target/release` after Tauri exits does not by itself mean the installer contains an unsigned app. The standalone and embedded executables can have different hashes because only the embedded copy has the NSIS bundle marker.

The previous read-only/re-bundle workaround was unsafe: a failed re-bundle could leave the old installer in place. The build now fails on every build/signing/verification error and deletes the expected installer before building so stale output cannot pass verification. Only Windows x64 NSIS with the standard target/release output directory is supported by this wrapper. Signing merge configs cannot override artifact names, versions, build paths, or NSIS templates/hooks.

## Verify

After a Windows build:

```powershell
./scripts/verify-windows-signature.ps1
```

The verifier requires the configured certificate in `Cert:\CurrentUser\My`, the Windows SDK signing tool for builds, and 7-Zip for payload extraction (available on GitHub Windows runners). It checks the selected certificate thumbprint on all three artifacts and requires Windows Authenticode `Status=Valid`. Unsigned files, different signers, tampered bytes, and `UnknownError` all fail.

For a self-signed certificate, verification temporarily adds **only the selected certificate's public bytes** to `CurrentUser\Root` and `CurrentUser\TrustedPublisher`. It removes only entries it added, in `finally`, including on verification failure. Existing trust is preserved. This validates file integrity under the selected test identity; it does **not** establish public/customer trust. Abruptly killing the process can prevent cleanup; use an ephemeral CI runner for release builds.

Use `-RequireTrusted` to disable temporary trust and require the machine's existing trust policy. This does not necessarily imply a publicly issued certificate if the machine already trusts a private certificate.

Inspect one file:

```powershell
Get-AuthenticodeSignature src-tauri\target\release\bundle\nsis\Korigio_*_x64-setup.exe |
  Format-List *
```

## GitHub Actions setup

`.github/workflows/release.yml` already builds NSIS on `windows-latest` for `v*` tags.

Add two **repository secrets** (Settings → Secrets and variables → Actions):

| Secret                         | Contents                                |
| ------------------------------ | --------------------------------------- |
| `WINDOWS_CERTIFICATE`          | Base64 of the `.pfx` (not the password) |
| `WINDOWS_CERTIFICATE_PASSWORD` | PFX password                            |

Encode on Windows:

```powershell
certutil -encode korigio-codesign.pfx korigio-codesign.b64.txt
```

Put the file contents into `WINDOWS_CERTIFICATE`. The importer accepts raw Base64 or `certutil` `BEGIN CERTIFICATE` wrapping.

Encode on macOS:

```bash
base64 -i korigio-codesign.pfx | pbcopy
```

If **both** secrets are set, the Windows job: decodes the PFX in the runner temp dir → imports into `Cert:\CurrentUser\My` → Tauri signs/packages NSIS → signs the restored standalone exe → verifies all three artifacts → deletes the PFX, merge config, and store cert. Logs must never print the password, PFX bytes, or Base64.

If the secrets are **missing**, the job still produces an **unsigned** NSIS installer (same as before), with a warning.

### Regenerating after a Subject change

If you change `CN` / `O` / `E` (for example to match this doc), create a **new** PFX once, then replace `WINDOWS_CERTIFICATE` (and the password secret if the password changed). Reusing an old PFX keeps the old Subject in Signature Details.

To send a tester build without tagging a version: **Actions → Release → Run workflow** (`workflow_dispatch`). Download the Windows NSIS artifact. A `v*` tag is still what publishes to GitHub Releases and the public download page.

`npm run tauri dev` does not use signing. macOS uses a separate Apple certificate; see [Apple code signing](MACOS_CODE_SIGNING.md).

## Linux (AppImage)

There is **no** Windows-like “publisher name at install” UX for the convenience AppImage this repo builds.

- Tauri can optionally GPG-sign AppImages (`SIGN` / `SIGN_KEY` / related env vars) for **integrity**, not a friendly installer publisher dialog.
- AppImage does not verify that signature on launch by itself.
- This project keeps Linux **unsigned by design** (convenience download only). Do not expect `Moritz Alexander Wright` to appear in a Linux install UI from Authenticode-style signing.

## Security

- `.pfx` / `.p12` / `korigio-codesign.key` = private key. Never commit.
- Thumbprint and `digestAlgorithm` / `timestampUrl` are not secrets.
- Do not paste Base64 certificates into issues, the download page, or chat.
- Later, replace this cert with OV/EV or Microsoft Artifact Signing **in this same build layer**. Do not put signing into the React/Rust app.

## SmartScreen limitation

| Audience                                 | What they see                                                                                                                         |
| ---------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Your Windows PC (cert in the store)      | Signature details show Moritz Alexander Wright / Korigio                                                                              |
| Client PC without your cert installed    | Still SmartScreen / untrusted publisher. The signature can show your name under “More info”, but Windows will not treat it as trusted |
| After they install and launch from Start | Usually no warning on later launches of the **installed** app (true for unsigned builds too)                                          |
| Next version’s installer                 | Warning can appear again                                                                                                              |

To actually reduce SmartScreen for customers, switch this pipeline to a publicly trusted identity (Azure Artifact Signing or an OV certificate). `bundle.windows.signCommand` is the Tauri 2 hook for Artifact Signing (`%1` = file to sign). No application code change. The current wrapper requires a certificate thumbprint; adopting a custom signing service also requires updating its signing/verification contract.

## Later: Microsoft Artifact Signing or OV/EV

Keep using `npm run tauri build` / `build:windows` / the Release workflow. Change only how the Windows job obtains a signer:

1. **OV/EV PFX** — same secrets (`WINDOWS_CERTIFICATE` / `WINDOWS_CERTIFICATE_PASSWORD`), different file.
2. **Artifact Signing** — stop importing a PFX; pass `bundle.windows.signCommand` (for example `artifact-signing-cli ... %1`) via merge JSON and adapt the wrapper (it currently requires a thumbprint). See [Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/).

## Regression checks

On Windows, `./scripts/test-windows-signing.ps1` creates disposable certificates and a tiny executable, then checks valid, unsigned, wrong-signer, tampered, and `UnknownError` cases plus trust cleanup after success/failure and preservation of existing trust. It needs the Windows SDK and .NET Framework compiler; no release secrets or network timestamp are used.

A release rehearsal with the real PFX still needs `./scripts/build-windows.ps1 -RequireSigned` on Windows. The final verification extracts the actual NSIS payload and will block a release if any of the three signatures fail.
