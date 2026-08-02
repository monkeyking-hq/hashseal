# Point this clone at repo-managed hooks under scripts/git-hooks/.
# Copyright (c) 2026 MonkeyKing.dev
# Requires: Git for Windows (runs hook scripts via sh).

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
Set-Location -LiteralPath $root

$hooksPath = "scripts/git-hooks"
$preCommit = Join-Path $hooksPath "pre-commit"
if (-not (Test-Path -LiteralPath $preCommit)) {
    Write-Error "Missing $preCommit"
}

git config core.hooksPath $hooksPath
if ($LASTEXITCODE -ne 0) {
    Write-Error "git config core.hooksPath failed"
}

Write-Host "Configured core.hooksPath=$hooksPath"
Write-Host "Active checks:"
Write-Host "  pre-commit → cargo fmt --all -- --check (when .rs files are staged)"
