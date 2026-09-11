#requires -Version 5.1
<#
.SYNOPSIS
  Production Windows NSIS build. Signs only when a thumbprint/config is present.

.DESCRIPTION
  Runs `npm run tauri build -- --bundles nsis` and, when signing config exists,
  lets Tauri invoke signtool, then post-signs the main exe and NSIS installer.

  Post-sign is required because Tauri's NSIS bundle-type binary patch can leave
  the release exe unsigned even when the installer was signed. After signing the
  exe, this script re-bundles NSIS with the exe marked read-only so the patch
  cannot strip the signature again, then signs the fresh installer.

  Does not use a second secret contract — same thumbprint / merge JSON as Tauri.
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

function Get-SigningThumbprintFromConfig {
  param([string]$Path)
  if (-not (Test-Path $Path)) { return $null }
  try {
    $json = Get-Content -LiteralPath $Path -Raw | ConvertFrom-Json
    $value = $json.bundle.windows.certificateThumbprint
    if ($value) { return ([string]$value).Trim() }
  } catch {
    Write-Error "Failed to read certificateThumbprint from $Path"
  }
  return $null
}

function Get-MainExePath {
  param([string]$Root)
  $releaseDir = Join-Path $Root "src-tauri\target\release"
  $candidates = @(
    (Join-Path $releaseDir "Korigio.exe"),
    (Join-Path $releaseDir "korigio.exe"),
    (Join-Path $releaseDir "repair-manager.exe")
  )
  return $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}

if ($useConfig) {
  Write-Host "Tauri will Authenticode-sign with the configured certificate (SHA-256 + timestamp)."
  if (-not $thumbprint) {
    $thumbprint = Get-SigningThumbprintFromConfig -Path $ConfigPath
  }
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

if ($useConfig) {
  if (-not $thumbprint) {
    Write-Error "Signing config is present but certificateThumbprint could not be resolved."
  }

  $signScript = Join-Path $PSScriptRoot "sign-windows-artifacts.ps1"
  $exe = Get-MainExePath -Root $RepoRoot
  if (-not $exe) {
    Write-Error "No main exe under src-tauri/target/release after tauri build."
  }

  # Sign the patched release exe, then re-bundle NSIS while the exe is read-only so
  # Tauri cannot strip the signature via bundle-type binary patching.
  & $signScript -RepoRoot $RepoRoot -Thumbprint $thumbprint -ExeOnly
  $wasReadOnly = (Get-Item -LiteralPath $exe).IsReadOnly
  Set-ItemProperty -LiteralPath $exe -Name IsReadOnly -Value $true
  try {
    $bundleCommand = "npm run tauri bundle -- --bundles $Bundles$configArg"
    Write-Host $bundleCommand
    cmd.exe /c $bundleCommand
    if ($LASTEXITCODE -ne 0) {
      Write-Warning "tauri bundle re-pack failed (exit $LASTEXITCODE). Keeping the previous NSIS output; installer signature still applied below."
    }
  } finally {
    Set-ItemProperty -LiteralPath $exe -Name IsReadOnly -Value $wasReadOnly
  }

  # Always re-sign exe + NSIS so verify passes even if Tauri skipped or stripped a pass.
  & $signScript -RepoRoot $RepoRoot -Thumbprint $thumbprint
}

if ($useConfig -or $RequireSigned) {
  & (Join-Path $PSScriptRoot "verify-windows-signature.ps1") -RepoRoot $RepoRoot
}
