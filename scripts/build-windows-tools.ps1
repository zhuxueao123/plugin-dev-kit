param(
    [switch]$SkipTests
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$KitRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$CliManifest = Join-Path $KitRoot "tooling\cli\Cargo.toml"
$LegacyManifest = Join-Path $KitRoot "tooling\legacy-export\Cargo.toml"
$CliBuild = Join-Path $KitRoot "tooling\cli\target\release\asapflow.exe"
$LegacyBuild = Join-Path $KitRoot "tooling\legacy-export\target\release\legacy-export.exe"
$CliDeliveryDir = Join-Path $KitRoot "bin\windows\asapflow"
$LegacyDeliveryDir = Join-Path $KitRoot "optional\migration\legacy-export\bin\windows"
$CliDelivery = Join-Path $CliDeliveryDir "asapflow.exe"
$LegacyDelivery = Join-Path $LegacyDeliveryDir "legacy-export.exe"
$ObsoleteCrossTarget = Join-Path $KitRoot "tooling\cli\target\x86_64-pc-windows-gnu"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "cargo was not found. Install Rust from https://rustup.rs and reopen PowerShell."
}

if (Test-Path $ObsoleteCrossTarget) {
    Remove-Item -Recurse -Force $ObsoleteCrossTarget
}

if (-not $SkipTests) {
    & cargo test --manifest-path $CliManifest
    if ($LASTEXITCODE -ne 0) { throw "CLI tests failed with exit code $LASTEXITCODE." }
    & cargo test --manifest-path $LegacyManifest
    if ($LASTEXITCODE -ne 0) { throw "legacy-export tests failed with exit code $LASTEXITCODE." }
}

& cargo build --release --manifest-path $CliManifest
if ($LASTEXITCODE -ne 0 -or -not (Test-Path $CliBuild)) {
    throw "Windows CLI build failed or did not produce $CliBuild."
}
& cargo build --release --manifest-path $LegacyManifest
if ($LASTEXITCODE -ne 0 -or -not (Test-Path $LegacyBuild)) {
    throw "Windows legacy-export build failed or did not produce $LegacyBuild."
}

New-Item -ItemType Directory -Force -Path $CliDeliveryDir | Out-Null
New-Item -ItemType Directory -Force -Path $LegacyDeliveryDir | Out-Null
Copy-Item -Force $CliBuild $CliDelivery
Copy-Item -Force $LegacyBuild $LegacyDelivery
Copy-Item -Force (Join-Path $KitRoot "bin\macos\asapflow\config.example.json") (Join-Path $CliDeliveryDir "config.example.json")
Copy-Item -Force (Join-Path $KitRoot "tooling\cli\windows\asapflow.ps1") (Join-Path $CliDeliveryDir "asapflow.ps1")

$PluginHelp = (& $CliDelivery plugin --help 2>&1 | Out-String)
foreach ($RequiredCommand in @("pack", "publish", "rollback")) {
    if ($PluginHelp -notmatch "(?m)^\s*$RequiredCommand\s") {
        throw "Built CLI is missing required plugin command '$RequiredCommand'."
    }
}
$EntityHelp = (& $CliDelivery system list-entities --help 2>&1 | Out-String)
if ($EntityHelp -notmatch "--limit") {
    throw "Built CLI is missing system list-entities --limit."
}
$LegacyHelp = (& $LegacyDelivery --help 2>&1 | Out-String)
if ($LegacyHelp -notmatch "export-function") {
    throw "Built legacy-export.exe failed its help smoke test."
}

$CliHash = (Get-FileHash -Algorithm SHA256 $CliDelivery).Hash.ToLowerInvariant()
$LegacyHash = (Get-FileHash -Algorithm SHA256 $LegacyDelivery).Hash.ToLowerInvariant()
Write-Host "Windows tools built successfully."
Write-Host "CLI: $CliDelivery"
Write-Host "CLI SHA-256: $CliHash"
Write-Host "legacy-export: $LegacyDelivery"
Write-Host "legacy-export SHA-256: $LegacyHash"
