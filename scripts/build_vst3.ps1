param(
    [string]$ProjectRoot = "$(Split-Path -Parent $PSScriptRoot)",
    [switch]$Install
)

$ErrorActionPreference = "Stop"

Push-Location $ProjectRoot
try {
    # 1. Cargo build
    Write-Host "Building Kyun2Stop in Release mode..." -ForegroundColor Cyan
    cargo build --release --lib

    # 2. Create bundle structure
    $BundleDir = Join-Path $ProjectRoot "target\bundled\Kyun2Stop.vst3\Contents\x86_64-win"
    if (!(Test-Path $BundleDir)) {
        New-Item -ItemType Directory -Path $BundleDir -Force | Out-Null
    }

    # 3. Copy and rename DLL to .vst3
    $SourceDll = Join-Path $ProjectRoot "target\release\Kyun2Stop.dll"
    $TargetVst3 = Join-Path $BundleDir "Kyun2Stop.vst3"
    Copy-Item -Path $SourceDll -Destination $TargetVst3 -Force
    Write-Host "Successfully bundled VST3 at: target\bundled\Kyun2Stop.vst3" -ForegroundColor Green

    # 4. Optional install to system VST3 folder
    if ($Install) {
        $SystemVst3Dir = "C:\Program Files\Common Files\VST3"
        $TargetInstallDir = Join-Path $SystemVst3Dir "Kyun2Stop.vst3"
        Write-Host "Installing to $TargetInstallDir..." -ForegroundColor Cyan
        
        # Check Admin privileges
        $Identity = [Security.Principal.WindowsIdentity]::GetCurrent()
        $Principal = New-Object Security.Principal.WindowsPrincipal($Identity)
        $IsAdmin = $Principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        
        if ($IsAdmin) {
            Copy-Item -Path "target\bundled\Kyun2Stop.vst3" -Destination $SystemVst3Dir -Recurse -Force
            Write-Host "Successfully installed Kyun2Stop.vst3 to VST3 directory!" -ForegroundColor Green
        } else {
            Write-Warning "Administrator privileges are required to copy to '$SystemVst3Dir'."
            Write-Warning "Please re-run this script in a PowerShell window as Administrator using: .\scripts\build_vst3.ps1 -Install"
        }
    }
}
catch {
    Write-Error "Build failed: $_"
}
finally {
    Pop-Location
}
