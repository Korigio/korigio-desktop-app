#requires -Version 5.1
<#
.SYNOPSIS
  GitHub Actions Windows job: optional PFX import, Tauri NSIS build, verify, cleanup.
#>
[CmdletBinding()]
param(
  [string]$Bundles = "nsis"
)

$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $RepoRoot

$hasCert = -not [string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE)
$hasPassword = -not [string]::IsNullOrWhiteSpace($env:WINDOWS_CERTIFICATE_PASSWORD)

if ($hasCert -xor $hasPassword) {
  Write-Error "Set both WINDOWS_CERTIFICATE and WINDOWS_CERTIFICATE_PASSWORD, or neither."
}

$imported = $null
try {
  if ($hasCert -and $hasPassword) {
    $imported = & (Join-Path $PSScriptRoot "import-windows-certificate.ps1") -RepoRoot $RepoRoot | Select-Object -Last 1
    & (Join-Path $PSScriptRoot "build-windows.ps1") -RepoRoot $RepoRoot -Bundles $Bundles -RequireSigned
  } else {
    Write-Host "WINDOWS_CERTIFICATE secrets are not set; building unsigned NSIS (existing release behavior)."
    & (Join-Path $PSScriptRoot "build-windows.ps1") -RepoRoot $RepoRoot -Bundles $Bundles
  }
} finally {
  $configPath = Join-Path $RepoRoot "src-tauri\tauri.windows-signing.json"
  if (Test-Path $configPath) {
    Remove-Item -LiteralPath $configPath -Force -ErrorAction SilentlyContinue
  }
  if ($imported -and $imported.PfxPath -and (Test-Path $imported.PfxPath)) {
    Remove-Item -LiteralPath $imported.PfxPath -Force -ErrorAction SilentlyContinue
  }
  if ($imported -and $imported.Thumbprint) {
    Get-ChildItem Cert:\CurrentUser\My |
      Where-Object { $_.Thumbprint -eq $imported.Thumbprint } |
      Remove-Item -ErrorAction SilentlyContinue
  }
}
