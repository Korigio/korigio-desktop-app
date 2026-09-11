# macOS release signing

The release workflow supports **Developer ID Application** signing and notarization for the downloadable DMG. This is separate from Windows Authenticode and from App Store distribution.

## Existing certificate request

The local `developer-id-LJH6BT9YQL.csr` is a certificate signing request, not an issued certificate. Its matching `.key` must remain private. Upload the CSR in the Apple Developer portal when creating a **Developer ID Application** certificate, then download the issued `.cer`. The `app-store.*` request/key is for a different distribution route and must not be substituted.

Package the issued certificate and its matching private key into a password-protected `.p12` outside this repository. For a DER-encoded Apple `.cer`:

```sh
openssl x509 -inform DER -in /path/to/developerID_application.cer -out /path/to/developer-id.pem
openssl pkcs12 -export -inkey /path/to/developer-id-LJH6BT9YQL.key \
  -in /path/to/developer-id.pem -out /path/to/korigio-apple.p12
```

OpenSSL prompts for the export password. Never paste private keys, certificate bundles, or passwords into chat or commit them. Reuse the existing matching key; generating another key will not match the issued certificate.

## GitHub Actions secrets

Configure these repository secrets together:

| Secret                       | Value                                                                |
| ---------------------------- | -------------------------------------------------------------------- |
| `APPLE_CERTIFICATE`          | Base64 of the password-protected `.p12`, including its private key   |
| `APPLE_CERTIFICATE_PASSWORD` | The `.p12` export password                                           |
| `APPLE_SIGNING_IDENTITY`     | Full certificate common name, `Developer ID Application: … (TEAMID)` |
| `APPLE_ID`                   | Apple account email for notarization                                 |
| `APPLE_PASSWORD`             | Apple **app-specific password**, not the normal account password     |
| `APPLE_TEAM_ID`              | Developer team ID matching the certificate                           |

The workflow passes these only to the macOS step. Tauri imports the `.p12` into its temporary signing keychain, signs the app with hardened runtime (the existing Tauri default), submits for notarization, and staples the ticket. The build script then runs `codesign --verify --deep --strict`, `xcrun stapler validate`, and Gatekeeper assessment against the app before artifact upload.

When all six secrets are absent, the existing unsigned convenience-build behavior remains. A partial setup fails before building; it cannot silently produce an unsigned release. A certificate issued for App Store distribution is rejected by the direct-download script.

The integration is prepared in the repository; having a CSR and private key alone does not activate Apple signing. No Apple certificate or notarization secret was available during the initial integration, so an actual signed/notarized build still needs verification after those secrets are configured.

References: [Tauri macOS signing](https://v2.tauri.app/distribute/sign/macos/), [Apple Developer certificates](https://developer.apple.com/account/resources/certificates/list).
