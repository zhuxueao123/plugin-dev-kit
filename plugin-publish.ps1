param(
    [Parameter(Mandatory = $true)][string]$PluginCode,
    [string]$Root = (Get-Location).Path
)
$ErrorActionPreference = "Stop"
$Cli = Join-Path $PSScriptRoot "bin\windows\asapflow\asapflow.exe"
& $Cli plugin validate --root $Root --code $PluginCode
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& $Cli plugin publish --root $Root --code $PluginCode
exit $LASTEXITCODE
