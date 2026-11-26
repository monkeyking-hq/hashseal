# HashSeal Windows installer — downloads release archive, verifies SHA256SUMS, or builds from source.
# Copyright (c) 2026 MonkeyKing.dev
param(
  [switch]$FromSource,
  [string]$Version = "latest",
  [string]$InstallDir = "$env:LOCALAPPDATA\HashSeal\bin"
)

$ErrorActionPreference = "Stop"
$Repo = if ($env:HASHSEAL_REPO) { $env:HASHSEAL_REPO } else { "https://github.com/monkeyking-hq/hashseal" }

if ($FromSource) {
  cargo install --path rust/hashseal --locked
  cargo install --path rust/hashseal-check --locked
  exit 0
}

$Target = "x86_64-pc-windows-msvc"
$Archive = "hashseal-$Target.zip"
$BaseUrl = if ($Version -eq "latest") { "$Repo/releases/latest/download" } else { "$Repo/releases/download/$Version" }

$Tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("hashseal-install-" + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $Tmp | Out-Null
try {
  $ZipPath = Join-Path $Tmp $Archive
  $SumsPath = Join-Path $Tmp "SHA256SUMS"
  Invoke-WebRequest -Uri "$BaseUrl/$Archive" -OutFile $ZipPath
  Invoke-WebRequest -Uri "$BaseUrl/SHA256SUMS" -OutFile $SumsPath

  $ExpectedLine = (Get-Content $SumsPath | Where-Object { $_ -match [regex]::Escape($Archive) } | Select-Object -First 1)
  if (-not $ExpectedLine) { throw "Missing $Archive entry in SHA256SUMS" }
  $ExpectedHash = ($ExpectedLine -split '\s+')[0].ToLowerInvariant()
  $ActualHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLowerInvariant()
  if ($ActualHash -ne $ExpectedHash) {
    throw "SHA256 mismatch for ${Archive}: expected $ExpectedHash got $ActualHash"
  }

  $Unpack = Join-Path $Tmp "unpack"
  Expand-Archive -Path $ZipPath -DestinationPath $Unpack -Force
  New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
  Copy-Item (Join-Path $Unpack "hashseal.exe") (Join-Path $InstallDir "hashseal.exe") -Force
  Copy-Item (Join-Path $Unpack "hashseal-check.exe") (Join-Path $InstallDir "hashseal-check.exe") -Force
  Write-Host "Installed hashseal.exe and hashseal-check.exe to $InstallDir"
} finally {
  Remove-Item -Path $Tmp -Recurse -Force -ErrorAction SilentlyContinue
}
