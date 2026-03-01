# Leyline Audio: BUILD ENVIRONMENT SETUP
$ErrorActionPreference = "Stop"

# Use environment variable if set, otherwise try common locations
$ewdkRoot = $env:LEYLINE_EWDK_ROOT
if (-not $ewdkRoot) {
    $ewdkRoot = "D:\eWDK_28000"
    if (-not (Test-Path $ewdkRoot)) { $ewdkRoot = "C:\eWDK_28000" }
    if (-not (Test-Path $ewdkRoot)) { $ewdkRoot = "D:\eWDK" }
    if (-not (Test-Path $ewdkRoot)) { $ewdkRoot = "C:\eWDK" }
}

$sdkVersion = $env:LEYLINE_SDK_VERSION
if (-not $sdkVersion) { $sdkVersion = "10.0.28000.0" }

if (Test-Path $ewdkRoot) {
    Write-Host "[*] Using eWDK at: $ewdkRoot (SDK: $sdkVersion)" -ForegroundColor Gray
    $env:WDK_ROOT = "$ewdkRoot\Program Files\Windows Kits\10"
    $env:WDKContentRoot = $env:WDK_ROOT
    $env:eWDK_ROOT_DIR = $ewdkRoot

    # Initialize env from SetupBuildEnv.cmd
    $cmd = "`"$ewdkRoot\BuildEnv\SetupBuildEnv.cmd`" amd64 $sdkVersion && set"
    $envVars = cmd.exe /c $cmd
    foreach ($line in $envVars) {
        if ($line -match "^([^=]+)=(.*)$") {
            $name = $matches[1]; $value = $matches[2]
            if ($name -eq "PATH") { $env:PATH = $value + ";" + $env:PATH }
            else { [System.Environment]::SetEnvironmentVariable($name, $value, "Process") }
        }
    }

    # CRITICAL FIX: Add User Mode libraries to LIB path for build.rs compilation
    $umLibPath = "$ewdkRoot\Program Files\Windows Kits\10\Lib\$sdkVersion\um\x64"
    $ucrtLibPath = "$ewdkRoot\Program Files\Windows Kits\10\Lib\$sdkVersion\ucrt\x64"

    $currentLib = [System.Environment]::GetEnvironmentVariable("LIB", "Process")
    $newLib = $currentLib

    if (Test-Path $umLibPath) { $newLib = "$newLib;$umLibPath" }
    if (Test-Path $ucrtLibPath) { $newLib = "$newLib;$ucrtLibPath" }

    [System.Environment]::SetEnvironmentVariable("LIB", $newLib, "Process")
}
else {
    Write-Warning "eWDK not found at '$ewdkRoot', falling back to system paths."
    $env:WDK_ROOT = "C:\Program Files (x86)\Windows Kits\10"
}

# Always ensure tool paths are set
$kitsRoot = if ($env:eWDK_ROOT_DIR) { "$env:eWDK_ROOT_DIR\Program Files\Windows Kits\10" } else { "C:\Program Files (x86)\Windows Kits\10" }
$st = Get-ChildItem -Path "$kitsRoot\bin" -Filter signtool.exe -Recurse | Where-Object { $_.FullName -match "x64" } | Select-Object -First 1
$ic = Get-ChildItem -Path "$kitsRoot\bin" -Filter Inf2Cat.exe -Recurse | Select-Object -First 1

if ($st) { $env:SIGNTOOL_EXE = $st.FullName }
if ($ic) { $env:INF2CAT_EXE = $ic.FullName }

Write-Host "[SUCCESS] Environment Set. SignTool=$env:SIGNTOOL_EXE" -ForegroundColor Green

