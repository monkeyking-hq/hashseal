# Build release hashseal + hashseal-check on Windows.
# Copyright (c) 2026 MonkeyKing.dev

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

Write-Host "Building hashseal (release)..."
cargo build -p hashseal --release
Write-Host "Building hashseal-check (release)..."
cargo build -p hashseal-check --release

Write-Host "Dependency tree (hashseal-check):"
cargo tree -p hashseal-check --edges normal

$rel = Join-Path (Get-Location) "target\release"
Write-Host ""
Write-Host "Artifacts:"
Write-Host "  $rel\hashseal.exe"
Write-Host "  $rel\hashseal-check.exe"
Write-Host ""
Write-Host "Add to PATH or: `$env:HASHSEAL_BIN = '$rel\hashseal.exe'"
