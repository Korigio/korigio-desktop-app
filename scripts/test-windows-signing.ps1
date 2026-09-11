#requires -Version 5.1
# Windows-only integration tests with throwaway certificates, never release secrets.
$ErrorActionPreference = 'Stop'
if ($env:OS -ne 'Windows_NT') { throw 'Run these Authenticode tests on Windows.' }
. (Join-Path $PSScriptRoot 'windows-signing-common.ps1')
function Assert-Rejected {
  param([scriptblock]$Action, [string]$Label)
  $rejected = $false
  try { & $Action } catch { $rejected = $true }
  if (-not $rejected) { throw "Expected rejection: $Label" }
  Write-Host "PASS: $Label rejected"
}
function Assert-NoTemporaryTrust {
  param([string]$Thumbprint)
  foreach ($location in @('CurrentUser', 'LocalMachine')) {
    foreach ($store in @('Root', 'TrustedPublisher')) {
      if (Test-Path -LiteralPath "Cert:\$location\$store\$Thumbprint") { throw "Temporary trust leaked into $location/$store." }
    }
  }
}
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ('korigio-signing-test-' + [guid]::NewGuid().ToString('N'))
$cert = $null
$otherCert = $null
$tlsCert = $null
try {
  New-Item -ItemType Directory -Path $tempDir | Out-Null
  $repoRoot = Split-Path -Parent $PSScriptRoot
  $actual = Get-WindowsArtifactPaths -RepoRoot $repoRoot
  if ((Split-Path -Leaf $actual.Exe) -ne 'korigio.exe') { throw 'Repository application executable must be korigio.exe.' }
  # Cargo metadata must honor target names and default-run rather than package-name guesses.
  $fixtureRoot = Join-Path $tempDir 'cargo-fixture'
  $tauriDir = Join-Path $fixtureRoot 'src-tauri'
  New-Item -ItemType Directory -Path $tauriDir | Out-Null
  Set-Content -LiteralPath (Join-Path $tauriDir 'app.rs') -Value 'fn main() {}'
  $manifest = Join-Path $tauriDir 'Cargo.toml'
  $configPath = Join-Path $tauriDir 'tauri.conf.json'
  Set-Content -LiteralPath $configPath -Value '{"productName":"Fixture","version":"1.0.0"}'
  $single = @'
[package]
name = "different-package-name"
version = "1.0.0"
[[bin]]
name = "actual-app"
path = "app.rs"
'@
  Set-Content -LiteralPath $manifest -Value $single
  if ((Split-Path -Leaf (Get-WindowsArtifactPaths -RepoRoot $fixtureRoot).Exe) -ne 'actual-app.exe') { throw 'Single Cargo binary target was not selected.' }
  $multiple = $single + "`n[[bin]]`nname = `"helper`"`npath = `"app.rs`"`n"
  Set-Content -LiteralPath $manifest -Value $multiple
  Assert-Rejected -Label 'ambiguous Cargo binary targets' -Action { Get-WindowsArtifactPaths -RepoRoot $fixtureRoot }
  Set-Content -LiteralPath $manifest -Value ($multiple.Replace('[package]', "[package]`ndefault-run = `"actual-app`""))
  if ((Split-Path -Leaf (Get-WindowsArtifactPaths -RepoRoot $fixtureRoot).Exe) -ne 'actual-app.exe') { throw 'Cargo default-run was not selected.' }
  Set-Content -LiteralPath $configPath -Value '{"productName":"Fixture","version":"1.0.0","mainBinaryName":"renamed-app"}'
  if ((Split-Path -Leaf (Get-WindowsArtifactPaths -RepoRoot $fixtureRoot).Exe) -ne 'renamed-app.exe') { throw 'Tauri mainBinaryName was not selected.' }
  Write-Host 'PASS: repository and fixture application binary resolution'
  $cert = New-SelfSignedCertificate -Type CodeSigningCert -KeyExportPolicy Exportable -Subject "CN=Korigio test $([guid]::NewGuid())" -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddDays(1)
  $otherCert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=Korigio other test $([guid]::NewGuid())" -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddDays(1)
  # Exercise the same certificate lookup used by the release import path.
  $cert = Get-WindowsSigningCertificate -Thumbprint $cert.Thumbprint -RequirePrivateKey
  $pfx = Join-Path $tempDir 'fixture.pfx'
  $password = ConvertTo-SecureString ([guid]::NewGuid().ToString('N')) -AsPlainText -Force
  Export-PfxCertificate -Cert $cert -FilePath $pfx -Password $password | Out-Null
  Remove-Item -LiteralPath "Cert:\CurrentUser\My\$($cert.Thumbprint)" -Force
  Import-PfxCertificate -FilePath $pfx -Password $password -CertStoreLocation Cert:\CurrentUser\My | Out-Null
  $cert = Get-WindowsSigningCertificate -Thumbprint $cert.Thumbprint -RequirePrivateKey
  Write-Host 'PASS: generated and PFX-imported code-signing certificates accepted'
  $tlsCert = New-SelfSignedCertificate -Type SSLServerAuthentication -Subject "CN=Korigio TLS test $([guid]::NewGuid())" -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddDays(1)
  Assert-Rejected -Label 'TLS certificate without code-signing EKU' -Action {
    Get-WindowsSigningCertificate -Thumbprint $tlsCert.Thumbprint -RequirePrivateKey
  }
  $source = Join-Path $tempDir 'fixture.cs'
  $unsigned = Join-Path $tempDir 'unsigned.exe'
  Set-Content -LiteralPath $source -Value 'public class Fixture { public static void Main() { System.Console.WriteLine("signing test"); } }'
  & "$env:WINDIR\Microsoft.NET\Framework64\v4.0.30319\csc.exe" /nologo /target:exe "/out:$unsigned" $source
  if ($LASTEXITCODE -ne 0) { throw 'Fixture compilation failed.' }
  $signed = Join-Path $tempDir 'signed.exe'
  Copy-Item -LiteralPath $unsigned -Destination $signed
  $signTool = Find-WindowsSignTool
  & $signTool sign /fd sha256 /sha1 $cert.Thumbprint $signed
  if ($LASTEXITCODE -ne 0) { throw 'Fixture signing failed.' }
  Invoke-WithWindowsSigningTrust -Certificate $cert -Action {
    Assert-WindowsSignature -Path $signed -Thumbprint $cert.Thumbprint
    Assert-Rejected -Label 'unsigned executable' -Action { Assert-WindowsSignature -Path $unsigned -Thumbprint $cert.Thumbprint }
    Assert-Rejected -Label 'different signer' -Action { Assert-WindowsSignature -Path $signed -Thumbprint $otherCert.Thumbprint }
    $tampered = Join-Path $tempDir 'tampered.exe'
    $bytes = [System.IO.File]::ReadAllBytes($signed)
    # The DOS stub is covered by Authenticode and outside the checksum/certificate exceptions.
    $bytes[80] = $bytes[80] -bxor 1
    [System.IO.File]::WriteAllBytes($tampered, $bytes)
    Assert-Rejected -Label 'tampered executable' -Action { Assert-WindowsSignature -Path $tampered -Thumbprint $cert.Thumbprint }
  }
  Assert-NoTemporaryTrust -Thumbprint $cert.Thumbprint
  Assert-Rejected -Label 'verification failure' -Action {
    Invoke-WithWindowsSigningTrust -Certificate $cert -Action { throw 'Deliberate verifier failure.' }
  }
  Assert-NoTemporaryTrust -Thumbprint $cert.Thumbprint
  # Exercise the formerly permissive branch even on machines whose trust APIs use a different error enum.
  & {
    function Get-AuthenticodeSignature { [pscustomobject]@{ SignerCertificate = $cert; Status = 'UnknownError'; StatusMessage = 'Synthetic provider error' } }
    Assert-Rejected -Label 'UnknownError with expected signer' -Action { Assert-WindowsSignature -Path $signed -Thumbprint $cert.Thumbprint }
  }
  # Existing trust must survive the verifier in its selected store location.
  $trustLocation = 'CurrentUser'
  if ($env:GITHUB_ACTIONS -eq 'true' -and $env:RUNNER_ENVIRONMENT -eq 'github-hosted') {
    $trustLocation = 'LocalMachine'
  }
  $store = New-Object System.Security.Cryptography.X509Certificates.X509Store -ArgumentList 'TrustedPublisher', $trustLocation
  try { $store.Open('ReadWrite'); $store.Add($cert) } finally { $store.Close() }
  try {
    Invoke-WithWindowsSigningTrust -Certificate $cert -Action { Assert-WindowsSignature -Path $signed -Thumbprint $cert.Thumbprint }
    if (-not (Test-Path "Cert:\$trustLocation\TrustedPublisher\$($cert.Thumbprint)")) { throw 'Existing trust was removed.' }
  } finally { Remove-Item -LiteralPath "Cert:\$trustLocation\TrustedPublisher\$($cert.Thumbprint)" -ErrorAction SilentlyContinue }
  Assert-NoTemporaryTrust -Thumbprint $cert.Thumbprint
  Write-Host 'Windows Authenticode regression tests passed.'
} finally {
  foreach ($item in @($cert, $otherCert, $tlsCert)) {
    if ($item) {
      foreach ($store in @('My', 'Root', 'TrustedPublisher')) {
        $path = "Cert:\CurrentUser\$store\$($item.Thumbprint)"
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
      }
    }
  }
  if (Test-Path -LiteralPath $tempDir) { Remove-Item -LiteralPath $tempDir -Recurse -Force }
}
