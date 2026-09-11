#requires -Version 5.1
<#
.SYNOPSIS
  Verify the standalone exe, current-version NSIS installer, and extracted app payload.
.DESCRIPTION
  Pins the signer thumbprint and requires Windows Authenticode Status=Valid.
  Self-signed verification temporarily trusts only the selected certificate in
  CurrentUser Root/TrustedPublisher and removes added entries in finally.
  UnknownError, NotTrusted, HashMismatch, and absent/different signers all fail.
.PARAMETER RequireTrusted
  Do not temporarily anchor the self-signed certificate; require existing trust.
#>
[CmdletBinding()]
param(
  [switch]$RequireTrusted,
  [string]$RepoRoot = (Split-Path -Parent $PSScriptRoot),
  [string]$Thumbprint = $env:WINDOWS_CERTIFICATE_THUMBPRINT
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-signing-common.ps1')
if (-not $Thumbprint) {
  $configPath = Join-Path $RepoRoot 'src-tauri\tauri.windows-signing.json'
  if (Test-Path -LiteralPath $configPath) {
    $config = Get-Content -LiteralPath $configPath -Raw | ConvertFrom-Json
    $Thumbprint = $config.bundle.windows.certificateThumbprint
  }
}
$cert = Get-WindowsSigningCertificate -Thumbprint $Thumbprint
$Thumbprint = $cert.Thumbprint
$artifacts = Get-WindowsArtifactPaths -RepoRoot $RepoRoot
$sevenZip = Get-Command 7z.exe -ErrorAction SilentlyContinue
if ($sevenZip) { $sevenZip = $sevenZip.Source }
else { $sevenZip = Join-Path $env:ProgramFiles '7-Zip\7z.exe' }
if (-not (Test-Path -LiteralPath $sevenZip -PathType Leaf)) {
  throw '7-Zip is required to verify the embedded NSIS app without installing it. Install 7-Zip (included on GitHub Windows runners).'
}
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ('korigio-signature-' + [guid]::NewGuid().ToString('N'))
try {
  Invoke-WithWindowsSigningTrust -Certificate $cert -RequireTrusted:$RequireTrusted -Action {
    Assert-WindowsSignature -Path $artifacts.Exe -Thumbprint $Thumbprint
    Assert-WindowsSignature -Path $artifacts.Installer -Thumbprint $Thumbprint
    $exeName = [System.IO.Path]::GetFileName($artifacts.Exe)
    & $sevenZip x $artifacts.Installer "-o$tempDir" "-ir!$exeName" -y | Out-Host
    if ($LASTEXITCODE -ne 0) { throw "NSIS payload extraction failed (exit $LASTEXITCODE)." }
    $payloads = @(Get-ChildItem -LiteralPath $tempDir -Recurse -File -Filter $exeName)
    if ($payloads.Count -ne 1) { throw "Expected exactly one $exeName in NSIS; found $($payloads.Count)." }
    Write-Host 'Checking the application inside the installer...'
    Assert-WindowsSignature -Path $payloads[0].FullName -Thumbprint $Thumbprint
  }
} finally {
  if (Test-Path -LiteralPath $tempDir) { Remove-Item -LiteralPath $tempDir -Recurse -Force }
}
Write-Host 'All three Windows artifacts have valid signatures from the configured certificate.'
