param(
    [Parameter(Mandatory = $true)]
    [string]$PluginCode,
    [string]$Root
)

$ErrorActionPreference = "Stop"
$KitDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Cli = Join-Path $KitDir "bin\windows\asapflow\asapflow.exe"
$PluginRoot = $null
$PluginLocation = $null
$SelectedManifest = $null

if ($Root) {
    $PluginRoot = (Resolve-Path $Root).Path
    $PluginLocation = $PluginRoot
    foreach ($CandidateManifest in @(
        (Join-Path $PluginRoot "backend\$PluginCode\manifest.json"),
        (Join-Path $PluginRoot "frontend\src\pages\$PluginCode\manifest.json")
    )) {
        if (Test-Path $CandidateManifest) {
            $SelectedManifest = $CandidateManifest
            break
        }
    }
} else {
    foreach ($Candidate in @(
        (Join-Path $KitDir "plugin-workspace"),
        (Join-Path $KitDir "plugin-workspace\examples")
    )) {
        $BackendManifest = Join-Path $Candidate "backend\$PluginCode\manifest.json"
        $FrontendManifest = Join-Path $Candidate "frontend\src\pages\$PluginCode\manifest.json"
        if ((Test-Path $BackendManifest) -or (Test-Path $FrontendManifest)) {
            $PluginRoot = $Candidate
            $PluginLocation = $Candidate
            $SelectedManifest = if (Test-Path $BackendManifest) { $BackendManifest } else { $FrontendManifest }
            break
        }
    }
}

if (-not $PluginRoot -or -not $SelectedManifest) {
    throw "Plugin '$PluginCode' was not found under plugin-workspace or plugin-workspace\examples."
}

$Manifest = Get-Content -Path $SelectedManifest -Raw -Encoding UTF8 | ConvertFrom-Json
$DeliveryDir = Join-Path $KitDir "dist"
$OutputFile = Join-Path $DeliveryDir "$PluginCode-$($Manifest.version).afplugin"
New-Item -ItemType Directory -Force -Path $DeliveryDir | Out-Null

Write-Host "Plugin location: $PluginLocation"
& $Cli plugin validate --root $PluginRoot --code $PluginCode
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $Cli plugin pack --root $PluginRoot --code $PluginCode --output-file $OutputFile
exit $LASTEXITCODE
