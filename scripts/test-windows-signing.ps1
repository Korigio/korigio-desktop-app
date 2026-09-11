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
  foreach ($store in @('Root', 'TrustedPublisher')) {
    if (Test-Path -LiteralPath "Cert:\CurrentUser\$store\$Thumbprint") { throw "Temporary trust leaked into $store." }
  }
}
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ('korigio-signing-test-' + [guid]::NewGuid().ToString('N'))
$cert = $null
$otherCert = $null
try {
  New-Item -ItemType Directory -Path $tempDir | Out-Null
  $cert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=Korigio test $([guid]::NewGuid())" -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddDays(1)
  $otherCert = New-SelfSignedCertificate -Type CodeSigningCert -Subject "CN=Korigio other test $([guid]::NewGuid())" -CertStoreLocation Cert:\CurrentUser\My -NotAfter (Get-Date).AddDays(1)
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
  # Existing user trust must survive the verifier.
  $store = New-Object System.Security.Cryptography.X509Certificates.X509Store -ArgumentList 'TrustedPublisher', 'CurrentUser'
  try { $store.Open('ReadWrite'); $store.Add($cert) } finally { $store.Close() }
  try {
    Invoke-WithWindowsSigningTrust -Certificate $cert -Action { Assert-WindowsSignature -Path $signed -Thumbprint $cert.Thumbprint }
    if (-not (Test-Path "Cert:\CurrentUser\TrustedPublisher\$($cert.Thumbprint)")) { throw 'Existing trust was removed.' }
  } finally { Remove-Item -LiteralPath "Cert:\CurrentUser\TrustedPublisher\$($cert.Thumbprint)" -ErrorAction SilentlyContinue }
  Assert-NoTemporaryTrust -Thumbprint $cert.Thumbprint
  Write-Host 'Windows Authenticode regression tests passed.'
} finally {
  foreach ($item in @($cert, $otherCert)) {
    if ($item) {
      foreach ($store in @('My', 'Root', 'TrustedPublisher')) {
        $path = "Cert:\CurrentUser\$store\$($item.Thumbprint)"
        if (Test-Path -LiteralPath $path) { Remove-Item -LiteralPath $path -Force }
      }
    }
  }
  if (Test-Path -LiteralPath $tempDir) { Remove-Item -LiteralPath $tempDir -Recurse -Force }
}
