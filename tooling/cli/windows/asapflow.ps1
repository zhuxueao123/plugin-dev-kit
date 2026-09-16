param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$ArgsList
)

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$exePath = Join-Path $scriptDir "asapflow.exe"

if (-not (Test-Path $exePath)) {
    throw "asapflow.exe was not found. Place asapflow.ps1 and asapflow.exe in the same directory."
}

$hasOutput = $false
for ($i = 0; $i -lt $ArgsList.Count; $i++) {
    if ($ArgsList[$i] -eq "--output") {
        $hasOutput = $true
        break
    }
}

if ($hasOutput) {
    & $exePath @ArgsList
    exit $LASTEXITCODE
}

$tempPath = Join-Path ([System.IO.Path]::GetTempPath()) ("asapflow-" + [Guid]::NewGuid().ToString("N") + ".json")

try {
    $finalArgs = @("--output", $tempPath) + $ArgsList
    & $exePath @finalArgs
    $exitCode = $LASTEXITCODE

    if (Test-Path $tempPath) {
        Get-Content -Path $tempPath -Raw -Encoding UTF8
    }

    exit $exitCode
}
finally {
    if (Test-Path $tempPath) {
        Remove-Item -Path $tempPath -Force -ErrorAction SilentlyContinue
    }
}
