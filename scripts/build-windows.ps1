#requires -Version 5.1
<#
.SYNOPSIS
  Build one Windows x64 NSIS installer, sign the restored standalone exe, verify both and the payload.
#>
[CmdletBinding()]
param(
  [ValidateSet('nsis')][string]$Bundles = 'nsis',
  [switch]$RequireSigned,
  [string]$RepoRoot = (Split-Path -Parent $PSScriptRoot),
  [string]$ConfigPath
)
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'Windows production builds must run on Windows.' }
. (Join-Path $PSScriptRoot 'windows-signing-common.ps1')
if (-not $ConfigPath) { $ConfigPath = Join-Path $RepoRoot 'src-tauri\tauri.windows-signing.json' }
$thumbprint = $env:WINDOWS_CERTIFICATE_THUMBPRINT -replace '\s', ''
if (Test-Path -LiteralPath $ConfigPath) {
  $signing = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
  if ($signing.mainBinaryName -or $signing.productName -or $signing.version -or $signing.build) {
    throw 'Signing merge config must not override binary name, product name, version, or build paths.'
  }
  if ($signing.bundle.windows.nsis.installerHooks -or $signing.bundle.windows.nsis.template) {
    throw 'Custom NSIS templates/hooks require a separate reviewed signing pipeline.'
  }
  $configured = [string]$signing.bundle.windows.certificateThumbprint -replace '\s', ''
  if ($thumbprint -and $configured -ne $thumbprint) { throw 'Environment and signing config certificate thumbprints disagree.' }
  $thumbprint = $configured
  if (-not $thumbprint) { throw 'Signing config is missing certificateThumbprint.' }
} elseif ($thumbprint) {
  $signing = @{ bundle = @{ windows = @{
    certificateThumbprint = $thumbprint
    digestAlgorithm = 'sha256'
    timestampUrl = 'http://timestamp.digicert.com'
  } } }
  [System.IO.File]::WriteAllText($ConfigPath, ($signing | ConvertTo-Json -Depth 6), (New-Object System.Text.UTF8Encoding $false))
}
if ($RequireSigned -and -not $thumbprint) { throw 'Signing was required but no signing certificate was configured.' }
if ($thumbprint) {
  $cert = Get-WindowsSigningCertificate -Thumbprint $thumbprint -RequirePrivateKey
  Write-Host "Signing publisher: $($cert.Subject)"
  if ($cert.GetNameInfo([System.Security.Cryptography.X509Certificates.X509NameType]::SimpleName, $false) -ne 'Moritz Alexander Wright' -or
      $cert.GetNameInfo([System.Security.Cryptography.X509Certificates.X509NameType]::EmailName, $false) -ne 'info@korigio.com') {
    Write-Warning 'The existing certificate does not contain the documented publisher name/email. Build metadata cannot change its subject; replace the PFX only if you intend to change identity.'
  }
} else { Write-Host 'No signing certificate configured; building unsigned NSIS.' }
$artifacts = Get-WindowsArtifactPaths -RepoRoot $RepoRoot
# Never let verification select an older successful installer after a failed build.
if (Test-Path -LiteralPath $artifacts.Installer) { Remove-Item -LiteralPath $artifacts.Installer -Force }
Push-Location $RepoRoot
try {
  $arguments = @('run', 'tauri', 'build', '--', '--bundles', $Bundles)
  if ($thumbprint) { $arguments += @('--config', $ConfigPath) }
  & npm.cmd @arguments
  if ($LASTEXITCODE -ne 0) { throw "tauri build failed (exit $LASTEXITCODE)." }
  if ($thumbprint) {
    $digest = $signing.bundle.windows.digestAlgorithm
    if (-not $digest) { $digest = 'sha256' }
    $timestamp = $signing.bundle.windows.timestampUrl
    if (-not $timestamp) { $timestamp = 'http://timestamp.digicert.com' }
    # Tauri 2.11.4 signs after patching, packages, then restores the unsigned original.
    # Do not re-bundle or modify the payload. Verify what NSIS actually contains below.
    & (Join-Path $PSScriptRoot 'sign-windows-artifacts.ps1') -RepoRoot $RepoRoot -Thumbprint $thumbprint -ExeOnly -DigestAlgorithm $digest -TimestampUrl $timestamp
    & (Join-Path $PSScriptRoot 'verify-windows-signature.ps1') -RepoRoot $RepoRoot -Thumbprint $thumbprint
  }
} finally { Pop-Location }
