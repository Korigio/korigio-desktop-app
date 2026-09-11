#requires -Version 5.1
# Shared artifact discovery and certificate validation for Windows release scripts.
function Get-WindowsArtifactPaths {
  param([string]$RepoRoot)
  $config = Get-Content -LiteralPath (Join-Path $RepoRoot 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
  $binary = 'repair-manager'
  if ($config.mainBinaryName) { $binary = $config.mainBinaryName }
  $release = Join-Path $RepoRoot 'src-tauri\target\release'
  [pscustomobject]@{
    Exe = Join-Path $release "$binary.exe"
    Installer = Join-Path $release "bundle\nsis\$($config.productName)_$($config.version)_x64-setup.exe"
  }
}

function Find-WindowsSignTool {
  $command = Get-Command signtool.exe -ErrorAction SilentlyContinue
  if ($command) { return $command.Source }
  $roots = @(
    (Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'),
    (Join-Path $env:ProgramFiles 'Windows Kits\10\bin')
  )
  if ($env:WindowsSdkDir) { $roots += Join-Path $env:WindowsSdkDir 'bin' }
  foreach ($root in $roots) {
    if (-not (Test-Path -LiteralPath $root)) { continue }
    $candidate = Get-ChildItem -LiteralPath $root -Filter signtool.exe -Recurse |
      Where-Object { $_.FullName -match '\\x64\\signtool\.exe$' } |
      Sort-Object FullName -Descending | Select-Object -First 1
    if ($candidate) { return $candidate.FullName }
  }
  throw 'signtool.exe not found. Install the Windows 10/11 SDK.'
}

function Get-WindowsSigningCertificate {
  param([string]$Thumbprint, [switch]$RequirePrivateKey)
  $normalized = $Thumbprint -replace '\s', ''
  if ($normalized -notmatch '^[0-9a-fA-F]{40}$') { throw 'Expected a 40-digit certificate thumbprint.' }
  $cert = Get-Item -LiteralPath "Cert:\CurrentUser\My\$normalized" -ErrorAction Stop
  if ($RequirePrivateKey -and -not $cert.HasPrivateKey) { throw 'The selected certificate has no private key.' }
  # Read the standard X.509 EKU extension, not PowerShell's display-oriented
  # EnhancedKeyUsageList (whose ObjectId can be a string rather than an Oid).
  $eku = @(
    foreach ($extension in $cert.Extensions) {
      if ($extension.Oid.Value -eq '2.5.29.37') {
        $usage = New-Object System.Security.Cryptography.X509Certificates.X509EnhancedKeyUsageExtension
        $usage.CopyFrom($extension)
        $usage.EnhancedKeyUsages | ForEach-Object { $_.Value }
      }
    }
  )
  if ($eku -notcontains '1.3.6.1.5.5.7.3.3') { throw 'The selected certificate is not a code-signing certificate.' }
  return $cert
}

function Assert-WindowsSignature {
  param([string]$Path, [string]$Thumbprint)
  if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "Missing artifact: $Path" }
  $signature = Get-AuthenticodeSignature -LiteralPath $Path
  if (-not $signature.SignerCertificate -or $signature.SignerCertificate.Thumbprint -ne $Thumbprint) {
    throw "Artifact is unsigned or signed by a different certificate: $Path"
  }
  if ($signature.Status -ne 'Valid') {
    throw "Invalid Authenticode signature ($($signature.Status)): $Path. $($signature.StatusMessage)"
  }
  Write-Host "VALID: $Path"
  Write-Host "  Publisher: $($signature.SignerCertificate.Subject)"
}

function Invoke-WithWindowsSigningTrust {
  param($Certificate, [switch]$RequireTrusted, [scriptblock]$Action)
  # Windows must validate the Authenticode file digest, chain, and signing policy.
  # Temporarily anchor ONLY the explicitly selected self-signed certificate.
  # Copy public bytes so trust stores never receive a private-key association.
  # CurrentUser Root can display a protected-root confirmation dialog and hang CI.
  # Only disposable GitHub-hosted VMs use machine trust; local/self-hosted builds
  # retain user-scoped trust. Hosted Windows runners are already administrators.
  $location = 'CurrentUser'
  if ($env:GITHUB_ACTIONS -eq 'true' -and $env:RUNNER_ENVIRONMENT -eq 'github-hosted') {
    $location = 'LocalMachine'
  }
  $added = [System.Collections.Generic.List[string]]::new()
  $publicCert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2 -ArgumentList @(,$Certificate.RawData)
  try {
    if (-not $RequireTrusted -and $Certificate.Subject -eq $Certificate.Issuer) {
      foreach ($name in @('Root', 'TrustedPublisher')) {
        $store = New-Object System.Security.Cryptography.X509Certificates.X509Store -ArgumentList $name, $location
        try {
          $store.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
          if (-not ($store.Certificates | Where-Object { $_.Thumbprint -eq $Certificate.Thumbprint })) {
            $store.Add($publicCert)
            $added.Add($name)
          }
        } finally { $store.Close() }
      }
      Write-Host "Verifying with the selected self-signed certificate temporarily trusted in $location; this does not establish customer trust."
    }
    & $Action
  } finally {
    foreach ($name in $added) {
      $store = New-Object System.Security.Cryptography.X509Certificates.X509Store -ArgumentList $name, $location
      try {
        $store.Open([System.Security.Cryptography.X509Certificates.OpenFlags]::ReadWrite)
        $store.Remove($publicCert)
      } finally { $store.Close() }
    }
    $publicCert.Dispose()
  }
}
