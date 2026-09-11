#requires -Version 5.1
<#
.SYNOPSIS
  Import a Base64-encoded .pfx into CurrentUser\My and write Tauri signing config.

.DESCRIPTION
  Reads WINDOWS_CERTIFICATE (Base64 PFX, optionally certutil PEM wrapping) and
  WINDOWS_CERTIFICATE_PASSWORD. Never prints those values.

  Writes src-tauri/tauri.windows-signing.json (gitignored) with the thumbprint.
#>
[CmdletBinding()]
param(
  [string]$RepoRoot,
  [string]$PfxPath,
  [string]$ConfigPath
)

$ErrorActionPreference = "Stop"

if (-not $RepoRoot) {
  $RepoRoot = Split-Path -Parent $PSScriptRoot
}
if (-not $ConfigPath) {
  $ConfigPath = Join-Path $RepoRoot "src-tauri\tauri.windows-signing.json"
}

if (-not $env:WINDOWS_CERTIFICATE) {
  Write-Error "WINDOWS_CERTIFICATE is not set."
}
if (-not $env:WINDOWS_CERTIFICATE_PASSWORD) {
  Write-Error "WINDOWS_CERTIFICATE_PASSWORD is not set."
}

$b64 = [string]$env:WINDOWS_CERTIFICATE
$b64 = $b64.Trim()
$b64 = $b64 -replace "-----BEGIN CERTIFICATE-----", ""
$b64 = $b64 -replace "-----END CERTIFICATE-----", ""
$b64 = $b64 -replace "-----BEGIN PKCS12-----", ""
$b64 = $b64 -replace "-----END PKCS12-----", ""
$b64 = $b64 -replace "\s", ""

try {
  $bytes = [Convert]::FromBase64String($b64)
} catch {
  Write-Error "WINDOWS_CERTIFICATE is not valid Base64."
}

if (-not $PfxPath) {
  $tempRoot = $env:RUNNER_TEMP
  if (-not $tempRoot) {
    $tempRoot = [System.IO.Path]::GetTempPath()
  }
  $PfxPath = Join-Path $tempRoot ("korigio-codesign-" + [guid]::NewGuid().ToString("N") + ".pfx")
}

if (Test-Path -LiteralPath $PfxPath) { throw 'Refusing to overwrite an existing PFX file.' }
$existingThumbprints = @(Get-ChildItem Cert:\CurrentUser\My | ForEach-Object { $_.Thumbprint })
$imported = @()
try {
  [System.IO.File]::WriteAllBytes($PfxPath, $bytes)

  $password = ConvertTo-SecureString -String $env:WINDOWS_CERTIFICATE_PASSWORD -AsPlainText -Force
  $imported = Import-PfxCertificate `
    -FilePath $PfxPath `
    -CertStoreLocation Cert:\CurrentUser\My `
    -Password $password

  if (-not $imported) {
    Write-Error "Failed to import the PFX into Cert:\CurrentUser\My."
  }

  # Import-PfxCertificate may return an array if the PFX contains a chain.
  $cert = @($imported) | Where-Object { $_.HasPrivateKey } | Select-Object -First 1
  if (-not $cert) {
    throw 'The PFX contains no certificate with a private key.'
  }
  . (Join-Path $PSScriptRoot 'windows-signing-common.ps1')
  $cert = Get-WindowsSigningCertificate -Thumbprint $cert.Thumbprint -RequirePrivateKey

  $thumbprint = $cert.Thumbprint
  $config = @{
    bundle = @{
      windows = @{
        certificateThumbprint = $thumbprint
        digestAlgorithm = "sha256"
        timestampUrl = "http://timestamp.digicert.com"
      }
    }
  }
  $configJson = $config | ConvertTo-Json -Depth 6
  $utf8NoBom = New-Object System.Text.UTF8Encoding $false
  [System.IO.File]::WriteAllText($ConfigPath, $configJson, $utf8NoBom)

  if ($env:GITHUB_OUTPUT) {
    Add-Content -Path $env:GITHUB_OUTPUT -Value "thumbprint=$thumbprint"
    Add-Content -Path $env:GITHUB_OUTPUT -Value "pfx_path=$PfxPath"
    Add-Content -Path $env:GITHUB_OUTPUT -Value "config_path=$ConfigPath"
  }

  Write-Host "Imported code-signing certificate into CurrentUser\My."
  Write-Host "Thumbprint: $thumbprint"
  Write-Host "Wrote Tauri signing merge config (no private key): $ConfigPath"

  # Return values for callers that capture output objects.
  [pscustomobject]@{
    Thumbprint = $thumbprint
    PfxPath    = $PfxPath
    ConfigPath = $ConfigPath
    AddedThumbprints = @($imported | Where-Object { $existingThumbprints -notcontains $_.Thumbprint } | ForEach-Object { $_.Thumbprint })
  }
} catch {
  foreach ($item in @($imported)) {
    if ($item -and $existingThumbprints -notcontains $item.Thumbprint) {
      Remove-Item -LiteralPath "Cert:\CurrentUser\My\$($item.Thumbprint)" -ErrorAction SilentlyContinue
    }
  }
  throw
} finally {
  # The caller never needs the private-key file after import, including failed imports.
  if (Test-Path -LiteralPath $PfxPath) { Remove-Item -LiteralPath $PfxPath -Force }
}
