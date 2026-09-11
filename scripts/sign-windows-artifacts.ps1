#requires -Version 5.1
<#
.SYNOPSIS
  Sign the restored standalone app executable after Tauri has packaged NSIS.
.DESCRIPTION
  Tauri signs the patched installer payload, then restores the unsigned original.
  Signing this standalone file does not change the already-packaged installer.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)][string]$Thumbprint,
  [string]$RepoRoot = (Split-Path -Parent $PSScriptRoot),
  [string]$DigestAlgorithm = 'sha256',
  [string]$TimestampUrl = 'http://timestamp.digicert.com',
  [switch]$ExeOnly,
  [switch]$NsisOnly
)
$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'windows-signing-common.ps1')
$cert = Get-WindowsSigningCertificate -Thumbprint $Thumbprint -RequirePrivateKey
$signTool = Find-WindowsSignTool
$artifacts = Get-WindowsArtifactPaths -RepoRoot $RepoRoot
if ($ExeOnly -and $NsisOnly) { throw 'Choose ExeOnly or NsisOnly, not both.' }
$paths = @()
if (-not $NsisOnly) { $paths += $artifacts.Exe }
if (-not $ExeOnly) { $paths += $artifacts.Installer }
foreach ($path in $paths) {
  if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing artifact: $path" }
  & $signTool sign /fd $DigestAlgorithm /td $DigestAlgorithm /tr $TimestampUrl /sha1 $cert.Thumbprint /d 'Korigio' /du 'https://www.korigio.com' $path
  if ($LASTEXITCODE -ne 0) { throw "signtool failed for $path (exit $LASTEXITCODE)." }
}
