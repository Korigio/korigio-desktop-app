# Windows code signing

Korigio’s shop installer is the **NSIS** `Korigio_*_x64-setup.exe`. This document covers **Windows Authenticode** signing of that installer and the main app exe. It is not Tauri’s updater key (`TAURI_SIGNING_PRIVATE_KEY`).

A **self-signed** certificate is for development, CI rehearsal, and internal testers. It puts **Moritz Alexander Wright** (organization **Korigio**) on the signature. It does **not** make Microsoft SmartScreen trust the app on a customer PC.

```text
SELF-SIGNED
    ↓
Authenticode signature exists
    ↓
cryptographically signed (publisher name is visible in signature details / UAC)
    ↓
BUT
    ↓
not automatically trusted on normal customer Windows installations
```

`npm run tauri dev` never needs a certificate.

## Publisher identity (what maps where)

| Identity                    | Where it lives                                           | What users typically see                                    |
| --------------------------- | -------------------------------------------------------- | ----------------------------------------------------------- |
| **Moritz Alexander Wright** | Cert `CN=`; also `bundle.publisher` in `tauri.conf.json` | UAC / “Publisher” in signature details; Apps & Features     |
| **Korigio**                 | Cert `O=`; `productName`                                 | Product / installer name; organization in full cert Subject |
| **info@korigio.com**        | Cert `E=` / `emailAddress=`                              | Full Subject only — not the UAC headline                    |
| **https://www.korigio.com** | `bundle.homepage` (not an Authenticode DN field)         | Installer / app metadata — not the signature publisher line |

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

1. `tauri build --bundles nsis` (Tauri’s own `signtool` pass).
2. **Post-sign** the main release exe (Tauri’s NSIS bundle-type patch can leave it unsigned).
3. Re-run `tauri bundle` with the exe marked read-only so the patch cannot strip that signature, then sign the NSIS installer again.
4. `scripts/verify-windows-signature.ps1` on exe + installer.

## Verify

After a Windows build:

```powershell
./scripts/verify-windows-signature.ps1
```

Expected (only artifacts this repo builds — NSIS, not MSI):

```text
Checking application executable...
SIGNED (NotTrusted)
  Publisher: CN=Moritz Alexander Wright, O=Korigio, E=info@korigio.com

Checking NSIS installer...
SIGNED (NotTrusted)
  Publisher: CN=Moritz Alexander Wright, O=Korigio, E=info@korigio.com
```

`NotTrusted` / `UnknownError` is normal for self-signed. `UNSIGNED` or `HASH_MISMATCH` fails the script.

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

If **both** secrets are set, the Windows job: decodes the PFX in the runner temp dir → imports into `Cert:\CurrentUser\My` → Tauri signs → post-signs exe + NSIS → verifies → deletes the PFX, merge config, and store cert. Logs must never print the password, PFX bytes, or Base64.

If the secrets are **missing**, the job still produces an **unsigned** NSIS installer (same as before), with a warning.

### Regenerating after a Subject change

If you change `CN` / `O` / `E` (for example to match this doc), create a **new** PFX once, then replace `WINDOWS_CERTIFICATE` (and the password secret if the password changed). Reusing an old PFX keeps the old Subject in Signature Details.

To send a tester build without tagging a version: **Actions → Release → Run workflow** (`workflow_dispatch`). Download the Windows NSIS artifact. A `v*` tag is still what publishes to GitHub Releases and the public download page.

`npm run tauri dev` and macOS/Linux jobs are unchanged.

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

To actually reduce SmartScreen for customers, switch this pipeline to a publicly trusted identity (Azure Artifact Signing or an OV certificate). `bundle.windows.signCommand` is the Tauri 2 hook for Artifact Signing (`%1` = file to sign). No application code change.

## Later: Microsoft Artifact Signing or OV/EV

Keep using `npm run tauri build` / `build:windows` / the Release workflow. Change only how the Windows job obtains a signer:

1. **OV/EV PFX** — same secrets (`WINDOWS_CERTIFICATE` / `WINDOWS_CERTIFICATE_PASSWORD`), different file.
2. **Artifact Signing** — stop importing a PFX; pass `bundle.windows.signCommand` (for example `artifact-signing-cli ... %1`) via the same merge JSON. See [Tauri Windows signing](https://v2.tauri.app/distribute/sign/windows/).
