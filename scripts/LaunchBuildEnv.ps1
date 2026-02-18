# Leyline Audio: BUILD ENVIRONMENT SETUP
$ErrorActionPreference = "Stop"

$ewdkRoot = "D:\eWDK_28000"
if (-not (Test-Path $ewdkRoot)) { $ewdkRoot = "C:\eWDK_28000" }
if (-not (Test-Path $ewdkRoot)) { $ewdkRoot = "D:\eWDK" }

if (Test-Path $ewdkRoot) {
    Write-Host "[*] Using eWDK at: $ewdkRoot" -ForegroundColor Gray
    $env:WDK_ROOT = "$ewdkRoot\Program Files\Windows Kits\10"
    $env:eWDK_ROOT_DIR = $ewdkRoot
    
    # Initialize env from SetupBuildEnv.cmd
    $cmd = "`"$ewdkRoot\BuildEnv\SetupBuildEnv.cmd`" amd64 10.0.28000.0 && set"
    $envVars = cmd.exe /c $cmd
    foreach ($line in $envVars) {
        if ($line -match "^([^=]+)=(.*)$") {
            $name = $matches[1]; $value = $matches[2]
            if ($name -eq "PATH") { $env:PATH = $value + ";" + $env:PATH }
            else { [System.Environment]::SetEnvironmentVariable($name, $value, "Process") }
        }
    }
}
else {
    Write-Warning "eWDK not found, falling back to system paths."
    $env:WDK_ROOT = "C:\Program Files (x86)\Windows Kits\10"
}

# Always ensure tool paths are set
$kitsRoot = if ($env:eWDK_ROOT_DIR) { "$env:eWDK_ROOT_DIR\Program Files\Windows Kits\10" } else { "C:\Program Files (x86)\Windows Kits\10" }
$st = Get-ChildItem -Path "$kitsRoot\bin" -Filter signtool.exe -Recurse | Where-Object { $_.FullName -match "x64" } | Select-Object -First 1
$ic = Get-ChildItem -Path "$kitsRoot\bin" -Filter Inf2Cat.exe -Recurse | Select-Object -First 1

if ($st) { $env:SIGNTOOL_EXE = $st.FullName }
if ($ic) { $env:INF2CAT_EXE = $ic.FullName }

Write-Host "[SUCCESS] Environment Set. SignTool=$env:SIGNTOOL_EXE" -ForegroundColor Green
