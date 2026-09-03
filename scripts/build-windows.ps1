#requires -Version 5.1
<#
.SYNOPSIS
  Production Windows NSIS build. Signs only when a thumbprint/config is present.

.DESCRIPTION
  Runs `npm run tauri build -- --bundles nsis` and, when signing config exists,
  lets Tauri invoke signtool (exe + installer). Verification runs afterward
  when signing was requested.

  Does not duplicate signing with a second signtool pass.
#>
[CmdletBinding()]
param(
  [string]$Bundles = "nsis",
  [switch]$RequireSigned,
  [string]$RepoRoot,
  [string]$ConfigPath
)

$ErrorActionPreference = "Stop"

if (-not $IsWindows -and $env:OS -ne "Windows_NT") {
  Write-Error "Windows production builds must run on Windows (or GitHub windows-latest)."
}

if (-not $RepoRoot) {
  $RepoRoot = Split-Path -Parent $PSScriptRoot
}
if (-not $ConfigPath) {
  $ConfigPath = Join-Path $RepoRoot "src-tauri\tauri.windows-signing.json"
}

Set-Location $RepoRoot

$thumbprint = $env:WINDOWS_CERTIFICATE_THUMBPRINT
if ($thumbprint) {
  $thumbprint = $thumbprint.Trim()
}

$useConfig = $false
if (Test-Path $ConfigPath) {
  $useConfig = $true
} elseif ($thumbprint) {
  $config = @{
    bundle = @{
      windows = @{
        certificateThumbprint = $thumbprint
        digestAlgorithm = "sha256"
        timestampUrl = "http://timestamp.digicert.com"
      }
    }
  }
  $utf8NoBom = New-Object System.Text.UTF8Encoding $false
  [System.IO.File]::WriteAllText($ConfigPath, ($config | ConvertTo-Json -Depth 6), $utf8NoBom)
  $useConfig = $true
  Write-Host "Wrote $ConfigPath from WINDOWS_CERTIFICATE_THUMBPRINT."
}

if ($useConfig) {
  Write-Host "Tauri will Authenticode-sign with the configured certificate (SHA-256 + timestamp)."
} else {
  Write-Host "No signing thumbprint/config. Building an unsigned NSIS installer."
  if ($RequireSigned) {
    Write-Error "Signing was required but no WINDOWS_CERTIFICATE_THUMBPRINT or $ConfigPath was found."
  }
}

$configArg = ""
if ($useConfig) {
  $configArg = " --config `"$ConfigPath`""
}

$command = "npm run tauri build -- --bundles $Bundles$configArg"
Write-Host $command
cmd.exe /c $command
if ($LASTEXITCODE -ne 0) {
  Write-Error "tauri build failed with exit code $LASTEXITCODE."
}

if ($useConfig -or $RequireSigned) {
  & (Join-Path $PSScriptRoot "verify-windows-signature.ps1") -RepoRoot $RepoRoot
}
