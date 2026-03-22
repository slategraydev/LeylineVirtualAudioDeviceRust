# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# BUILD ENVIRONMENT SETUP
# Initializes eWDK and Rust environment variables for the current session.
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

$ErrorActionPreference = "Stop"

# eWDK location — C:\EWDK
$ewdkRoot = if ($env:LEYLINE_EWDK_ROOT) { $env:LEYLINE_EWDK_ROOT } else { "C:\EWDK" }
$sdkVersion = if ($env:LEYLINE_SDK_VERSION) { $env:LEYLINE_SDK_VERSION } else { "10.0.28000.0" }

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

    # Add User Mode and UCRT libraries to LIB path for build.rs compilation
    $umLibPath = "$ewdkRoot\Program Files\Windows Kits\10\Lib\$sdkVersion\um\x64"
    $ucrtLibPath = "$ewdkRoot\Program Files\Windows Kits\10\Lib\$sdkVersion\ucrt\x64"

    $currentLib = [System.Environment]::GetEnvironmentVariable("LIB", "Process")
    $newLib = $currentLib

    if (Test-Path $umLibPath) { $newLib = "$newLib;$umLibPath" }
    if (Test-Path $ucrtLibPath) { $newLib = "$newLib;$ucrtLibPath" }

    [System.Environment]::SetEnvironmentVariable("LIB", $newLib, "Process")
}
else {
    Write-Warning "eWDK not found at '$ewdkRoot'. Set LEYLINE_EWDK_ROOT or install to C:\EWDK."
    throw "eWDK required for kernel driver builds."
}

# Locate signing and packaging tools
$kitsRoot = "$ewdkRoot\Program Files\Windows Kits\10"
$st = Get-ChildItem -Path "$kitsRoot\bin" -Filter signtool.exe -Recurse | Where-Object { $_.FullName -match "x64" } | Select-Object -First 1
$ic = Get-ChildItem -Path "$kitsRoot\bin" -Filter Inf2Cat.exe -Recurse | Select-Object -First 1

if ($st) { $env:SIGNTOOL_EXE = $st.FullName }
if ($ic) { $env:INF2CAT_EXE = $ic.FullName }

Write-Host "[SUCCESS] Environment Set. SignTool=$env:SIGNTOOL_EXE" -ForegroundColor Green
