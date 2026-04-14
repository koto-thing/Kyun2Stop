param(
    [string]$ProjectRoot = "$(Split-Path -Parent $PSScriptRoot)"
)

$ErrorActionPreference = "Stop"

Push-Location $ProjectRoot
try {
    cbindgen --config cbindgen.toml --crate Kyun2Stop --output include/kyun2stop_ffi.h
    Write-Host "Generated include/kyun2stop_ffi.h"
}
finally {
    Pop-Location
}

