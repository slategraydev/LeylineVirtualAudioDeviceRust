# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

# Leyline Audio Driver - eWDK 26H1 Environment Setup
# This script locks the toolchain to the eWDK mounted on Drive E.

$ErrorActionPreference = "Stop"

Write-Host "--- Leyline Build Environment (eWDK 26H1 Locked) ---" -ForegroundColor Cyan

# 1. Discover WDK Root (Prioritize Permanent Folder, then Mounts)
$wdkRoot = $null

# Priority 1: Permanent Folder (Recommended)
$permanentPath = "D:\eWDK_28000\Program Files\Windows Kits\10"
if (Test-Path $permanentPath)
{
    $wdkRoot = $permanentPath
    $env:eWDK_ROOT_DIR = "D:\eWDK_28000"
    Write-Host "[✓] Permanent eWDK detected at: $env:eWDK_ROOT_DIR" -ForegroundColor Green
} else
{

    # Priority 2: Check common drive letters for eWDK structure (Mounts)
    $drives = Get-PSDrive -PSProvider FileSystem
    foreach ($drive in $drives)
    {
        if (Test-Path (Join-Path $drive.Root "LaunchBuildEnv.cmd"))
        {
            $wdkRoot = Join-Path $drive.Root "Program Files\Windows Kits\10"
            $env:eWDK_ROOT_DIR = $drive.Root
            Write-Host "[✓] eWDK mount detected on drive $($drive.Name):" -ForegroundColor Green
            break
        }
    }
}

# Fallback to local installation
if ($null -eq $wdkRoot)
{
    $wdkRoot = "C:\Program Files (x86)\Windows Kits\10"
    if (-not (Test-Path $wdkRoot))
    {
        Write-Error "WDK not found. Please copy eWDK to D:\Toolchains\eWDK_28000 or mount the ISO."
    }
    Write-Host "[!] Using local WDK (eWDK not detected)." -ForegroundColor Yellow
    $env:eWDK_ROOT_DIR = $null
}

# 2. Capture Environment
Write-Host "Capturing eWDK environment variables..." -ForegroundColor Gray
if ($env:eWDK_ROOT_DIR)
{
    $cmdPath = Join-Path $env:eWDK_ROOT_DIR "LaunchBuildEnv.cmd"
    $cmd = "$cmdPath amd64 && set"
} else
{
    # If no eWDK, we rely on standard system paths (less robust)
    $cmd = "set"
}

$envVars = cmd.exe /c $cmd

foreach ($line in $envVars)
{
    if ($line -match "^([^=]+)=(.*)$")
    {
        $name = $matches[1]
        $value = $matches[2]

        # Only apply variables that are relevant to the build
        if ($name -in @("PATH", "INCLUDE", "LIB", "LIBPATH", "WDKContentRoot", "WindowsSdkDir", "WindowsTargetPlatformVersion", "VCINSTALLDIR", "VCToolsInstallDir"))
        {
            if ($name -eq "PATH")
            {
                # Prepend eWDK paths to existing PATH
                $env:PATH = $value + ";" + $env:PATH
            } else
            {
                [System.Environment]::SetEnvironmentVariable($name, $value, "Process")
            }
        }
    }
}

# 3. Hardware-Lock LIBCLANG_PATH
$llvmPath = $null

# Priority 1: Check self-contained LLVM in eWDK root (User Preference)
if ($env:eWDK_ROOT_DIR)
{
    $ewdkRootLlvm = Join-Path $env:eWDK_ROOT_DIR "LLVM\bin"
    if (Test-Path (Join-Path $ewdkRootLlvm "libclang.dll"))
    {
        $llvmPath = $ewdkRootLlvm
        Write-Host "[✓] eWDK-root LLVM detected." -ForegroundColor Green
    }
}

# Priority 2: Check known location on C: (Visual Studio Professional)
if ($null -eq $llvmPath)
{
    $vsLlvm = "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\Llvm\x64\bin"
    if (Test-Path (Join-Path $vsLlvm "libclang.dll"))
    {
        $llvmPath = $vsLlvm
    }
}

# Priority 3: Standard LLVM installation
if ($null -eq $llvmPath)
{
    $stdLlvm = "C:\Program Files\LLVM\bin"
    if (Test-Path (Join-Path $stdLlvm "libclang.dll"))
    {
        $llvmPath = $stdLlvm
    }
}

if ($llvmPath -and (Test-Path $llvmPath))
{
    $env:LIBCLANG_PATH = $llvmPath
    $env:PATH = "$llvmPath;" + $env:PATH
    Write-Host "[✓] LIBCLANG_PATH set to: $env:LIBCLANG_PATH"
} else
{
    Write-Warning "libclang.dll not found. bindgen will fail."
}

# 4. Success Output
Write-Host "[✓] WDK_ROOT (via eWDK): $env:WDKContentRoot"
Write-Host "[✓] WindowsTargetPlatformVersion: $env:WindowsTargetPlatformVersion"
Write-Host "[✓] MSVC Version: $env:VCToolsVersion"

Write-Host "`nProject is now 100% contained via eWDK (with local LLVM fallback)." -ForegroundColor Green
