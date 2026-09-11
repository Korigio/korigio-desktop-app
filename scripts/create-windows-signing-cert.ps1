#requires -Version 5.1
<#
.SYNOPSIS
  Create a development/testing Windows Authenticode certificate for Korigio.

.DESCRIPTION
  One-time script. Reuse this certificate for every build. Do not generate a new
  certificate per release — each new cert is a new untrusted publisher.

  This is NOT a publicly trusted certificate. Customer PCs will still show
  SmartScreen / unknown-publisher warnings until you switch this pipeline to
  OV/EV or Microsoft Artifact Signing.

.NOTES
  The .pfx contains the private key. Never commit it or paste it into chat logs.
#>
[CmdletBinding()]
param(
  [switch]$ExportPfx,
  [string]$PfxPath,
  [int]$ValidYears = 5
)

$ErrorActionPreference = "Stop"

if (-not $IsWindows -and $env:OS -ne "Windows_NT") {
  Write-Error "This script must run on Windows (PowerShell with the certificate store)."
}

# CN is what Windows usually shows as the publisher name (UAC / signature details).
# O / E are visible in the full Subject. Website URL is not an Authenticode DN field —
# use bundle.homepage in tauri.conf.json (https://www.korigio.com).
$subject = "CN=Moritz Alexander Wright, O=Korigio, E=info@korigio.com"
$notAfter = (Get-Date).AddYears($ValidYears)
$repoRoot = Split-Path -Parent $PSScriptRoot
if (-not $PfxPath) {
  $PfxPath = Join-Path $repoRoot "korigio-codesign.pfx"
}

Write-Host "Creating self-signed code-signing certificate..."
Write-Host "  Subject:     $subject"
Write-Host "  Store:       CurrentUser\My"
Write-Host "  Hash:        SHA256"
Write-Host "  Valid until: $($notAfter.ToString('yyyy-MM-dd'))"
Write-Host ""

$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject $subject `
  -FriendlyName "Korigio Windows code signing (development)" `
  -HashAlgorithm SHA256 `
  -KeyExportPolicy Exportable `
  -CertStoreLocation "Cert:\CurrentUser\My" `
  -NotAfter $notAfter

Write-Host "Certificate created."
Write-Host "  Thumbprint: $($cert.Thumbprint)"
Write-Host ""
Write-Host "List it later with:"
Write-Host "  Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert"
Write-Host ""
Write-Host "Reuse this certificate. Do not create a new one for every build."
Write-Host ""

$shouldExport = [bool]$ExportPfx
if (-not $shouldExport) {
  $answer = Read-Host "Export a password-protected .pfx for GitHub Actions / backup? [y/N]"
  $shouldExport = $answer -match '^[Yy]'
}

if (-not $shouldExport) {
  Write-Host "Skipped .pfx export. The certificate is in Cert:\CurrentUser\My only."
  Write-Host "Local signed builds: set WINDOWS_CERTIFICATE_THUMBPRINT=$($cert.Thumbprint)"
  exit 0
}

$password = $null
if ($env:WINDOWS_CERTIFICATE_PASSWORD) {
  $password = ConvertTo-SecureString -String $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force
} else {
  $password = Read-Host "Enter a new .pfx password (input hidden)" -AsSecureString
  if (-not $password -or $password.Length -eq 0) {
    Write-Error "A password is required to export the .pfx. Aborting export (certificate remains in the store)."
  }
}

$exportDir = Split-Path -Parent $PfxPath
if ($exportDir -and -not (Test-Path $exportDir)) {
  New-Item -ItemType Directory -Path $exportDir | Out-Null
}

Export-PfxCertificate -Cert $cert -FilePath $PfxPath -Password $password | Out-Null

Write-Host ""
Write-Host "Exported: $PfxPath"
Write-Host ""
Write-Host "SECURITY"
Write-Host "  - The .pfx contains the PRIVATE KEY."
Write-Host "  - Never commit it to Git, never put it in the website repo, never paste it into tickets."
Write-Host "  - For GitHub Actions, store Base64(.pfx) as secret WINDOWS_CERTIFICATE"
Write-Host "    and the password as secret WINDOWS_CERTIFICATE_PASSWORD."
Write-Host "  - Encode on Windows with:  certutil -encode `"$PfxPath`" korigio-codesign.b64.txt"
Write-Host "    Then copy only the Base64 body into the secret (not the CERTIFICATE header lines if you strip them,"
Write-Host "    or include the full certutil file — the importer accepts either)."
Write-Host ""
Write-Host "This certificate will NOT make SmartScreen trust Korigio on your client's PC."
Write-Host "It only attaches publisher `"Moritz Alexander Wright`" (O=Korigio, E=info@korigio.com) to the Authenticode signature."
Write-Host ""
Write-Host "Thumbprint: $($cert.Thumbprint)"
