#requires -Version 5.1
<#
.SYNOPSIS
  Verify Authenticode signatures on Korigio Windows build artifacts.

.DESCRIPTION
  Checks the app executable and the NSIS installer this repo actually produces.
  MSI is not built and is not checked.

  A self-signed signature is usually Status=NotTrusted or UnknownError on
  machines that do not trust the cert. That still counts as SIGNED. NotSigned
  or HashMismatch fails.

.PARAMETER RequireTrusted
  Also fail unless Windows reports Status=Valid (publicly trusted chain).
#>
[CmdletBinding()]
param(
  [switch]$RequireTrusted,
  [string]$RepoRoot
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
  $RepoRoot = Split-Path -Parent $PSScriptRoot
}

function Get-SignatureLabel {
  param($Signature)
  if (-not $Signature -or $Signature.Status -eq "NotSigned" -or -not $Signature.SignerCertificate) {
    return "UNSIGNED"
  }
  if ($Signature.Status -eq "HashMismatch") {
    return "HASH_MISMATCH"
  }
  if ($Signature.Status -eq "Valid") {
    return "SIGNED"
  }
  return "SIGNED ($($Signature.Status))"
}

function Test-IsSigned {
  param($Signature)
  if (-not $Signature) { return $false }
  if ($Signature.Status -eq "NotSigned") { return $false }
  if ($Signature.Status -eq "HashMismatch") { return $false }
  if (-not $Signature.SignerCertificate) { return $false }
  if ($RequireTrusted -and $Signature.Status -ne "Valid") { return $false }
  return $true
}

$targets = [System.Collections.Generic.List[object]]::new()

$releaseDir = Join-Path $RepoRoot "src-tauri\target\release"
$exeCandidates = @(
  (Join-Path $releaseDir "Korigio.exe"),
  (Join-Path $releaseDir "korigio.exe"),
  (Join-Path $releaseDir "repair-manager.exe")
)
$exe = $exeCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if ($exe) {
  $targets.Add([pscustomobject]@{ Label = "application executable"; Path = $exe })
} else {
  Write-Host "Checking application executable..."
  Write-Host "MISSING"
  Write-Error "No Korigio.exe (or korigio.exe) under src-tauri/target/release. Build first."
}

$nsisDir = Join-Path $RepoRoot "src-tauri\target\release\bundle\nsis"
$nsis = @()
if (Test-Path $nsisDir) {
  $nsis = @(Get-ChildItem -Path $nsisDir -Filter "*-setup.exe" -File | Sort-Object LastWriteTime -Descending)
}
if ($nsis.Count -eq 0) {
  Write-Host ""
  Write-Host "Checking NSIS installer..."
  Write-Host "MISSING"
  Write-Error "No NSIS *-setup.exe under src-tauri/target/release/bundle/nsis. Build first."
}
$targets.Add([pscustomobject]@{ Label = "NSIS installer"; Path = $nsis[0].FullName })

$failed = $false
foreach ($target in $targets) {
  Write-Host "Checking $($target.Label)..."
  $signature = Get-AuthenticodeSignature -FilePath $target.Path
  $label = Get-SignatureLabel $signature
  Write-Host $label
  if ($signature.SignerCertificate) {
    Write-Host "  Publisher: $($signature.SignerCertificate.Subject)"
  }
  Write-Host "  Path: $($target.Path)"
  Write-Host ""
  if (-not (Test-IsSigned $signature)) {
    $failed = $true
  }
}

if ($failed) {
  Write-Error "One or more Windows artifacts are unsigned or the signature is invalid."
}

Write-Host "All checked Windows artifacts are signed."
