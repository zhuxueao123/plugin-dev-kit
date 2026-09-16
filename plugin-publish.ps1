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

if ($Root) {
    $PluginRoot = (Resolve-Path $Root).Path
    $PluginLocation = $PluginRoot
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
            break
        }
    }
}

if (-not $PluginRoot) {
    throw "Plugin '$PluginCode' was not found under plugin-workspace or plugin-workspace\examples."
}

Write-Host "Plugin location: $PluginLocation"
& $Cli plugin publish --root $PluginRoot --code $PluginCode
exit $LASTEXITCODE
