#requires -Version 5.1
<#
.SYNOPSIS
  Authenticode-sign Korigio Windows build artifacts with signtool.

.DESCRIPTION
  Signs the main release exe and/or the NSIS installer using a certificate
  thumbprint already in Cert:\CurrentUser\My. Used after `tauri build` because
  Tauri's NSIS binary patch can leave the main exe unsigned.

  Never prints private key material.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [string]$Thumbprint,

  [string]$RepoRoot,

  [string]$DigestAlgorithm = "sha256",

  [string]$TimestampUrl = "http://timestamp.digicert.com",

  [switch]$ExeOnly,

  [switch]$NsisOnly
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
  $RepoRoot = Split-Path -Parent $PSScriptRoot
}

$Thumbprint = $Thumbprint.Trim() -replace '\s', ''
if ($Thumbprint.Length -lt 40) {
  Write-Error "certificate thumbprint looks invalid (too short)."
}

function Find-SignTool {
  $cmd = Get-Command signtool.exe -ErrorAction SilentlyContinue
  if ($cmd) {
    return $cmd.Source
  }

  $roots = @(
    ${env:ProgramFiles(x86)},
    $env:ProgramFiles,
    ${env:WindowsSdkDir}
  ) | Where-Object { $_ }

  foreach ($root in $roots) {
    $kits = Join-Path $root "Windows Kits\10\bin"
    if (-not (Test-Path $kits)) { continue }
    $candidates = Get-ChildItem -Path $kits -Filter signtool.exe -Recurse -ErrorAction SilentlyContinue |
      Where-Object { $_.FullName -match '\\x64\\signtool\.exe$' } |
      Sort-Object FullName -Descending
    if ($candidates -and $candidates.Count -gt 0) {
      return $candidates[0].FullName
    }
  }

  Write-Error "signtool.exe not found. Install the Windows 10/11 SDK or run on a runner that has it."
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

function Get-NsisSetupPath {
  param([string]$Root)
  $nsisDir = Join-Path $Root "src-tauri\target\release\bundle\nsis"
  if (-not (Test-Path $nsisDir)) { return $null }
  $files = @(Get-ChildItem -Path $nsisDir -Filter "*-setup.exe" -File | Sort-Object LastWriteTime -Descending)
  if ($files.Count -eq 0) { return $null }
  return $files[0].FullName
}

function Invoke-AuthenticodeSign {
  param(
    [string]$SignTool,
    [string]$FilePath,
    [string]$Sha1,
    [string]$Digest,
    [string]$Timestamp
  )

  Write-Host "Signing $FilePath"
  & $SignTool sign `
    /fd $Digest `
    /td $Digest `
    /tr $Timestamp `
    /sha1 $Sha1 `
    $FilePath
  if ($LASTEXITCODE -ne 0) {
    Write-Error "signtool failed for $FilePath (exit $LASTEXITCODE)."
  }
}

$signTool = Find-SignTool
$targets = [System.Collections.Generic.List[string]]::new()

if (-not $NsisOnly) {
  $exe = Get-MainExePath -Root $RepoRoot
  if (-not $exe) {
    Write-Error "No main exe under src-tauri/target/release (Korigio.exe / korigio.exe / repair-manager.exe)."
  }
  $targets.Add($exe)
}

if (-not $ExeOnly) {
  $nsis = Get-NsisSetupPath -Root $RepoRoot
  if (-not $nsis) {
    Write-Error "No NSIS *-setup.exe under src-tauri/target/release/bundle/nsis."
  }
  $targets.Add($nsis)
}

foreach ($path in $targets) {
  Invoke-AuthenticodeSign `
    -SignTool $signTool `
    -FilePath $path `
    -Sha1 $Thumbprint `
    -Digest $DigestAlgorithm `
    -Timestamp $TimestampUrl
}

Write-Host "Post-signed $($targets.Count) Windows artifact(s)."
[pscustomobject]@{
  Thumbprint = $Thumbprint
  Paths      = @($targets)
}
