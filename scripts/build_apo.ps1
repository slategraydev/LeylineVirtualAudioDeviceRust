# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

$ErrorActionPreference = "Stop"

# Configuration
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
$APOSource = Join-Path $ProjectRoot "src\APO"
$eWDKPath = "D:\eWDK_28000\LaunchBuildEnv.cmd"

if (-not (Test-Path $eWDKPath))
{
    Write-Error "eWDK build environment not found at $eWDKPath"
    exit 1
}

Write-Host "Building APO in eWDK Environment..." -ForegroundColor Cyan

# Locate nmake.exe
$nmakePath = Get-ChildItem -Path "D:\eWDK_28000" -Filter "nmake.exe" -Recurse -ErrorAction SilentlyContinue |
    Where-Object { $_.DirectoryName -like "*Hostx64\x64*" } |
    Select-Object -First 1

if ($nmakePath)
{
    Write-Host "Found nmake at: $($nmakePath.FullName)" -ForegroundColor Green
    $nmakeDir = $nmakePath.DirectoryName
} else
{
    Write-Warning "nmake.exe not found in Hostx64\x64 path. Attempting generic search..."
    $nmakePath = Get-ChildItem -Path "D:\eWDK_28000" -Filter "nmake.exe" -Recurse -ErrorAction SilentlyContinue |
        Select-Object -First 1

    if ($nmakePath)
    {
        Write-Host "Found nmake at: $($nmakePath.FullName)" -ForegroundColor Yellow
        $nmakeDir = $nmakePath.DirectoryName
    } else
    {
        Write-Error "Could not locate nmake.exe in eWDK."
        exit 1
    }
}

# Locate vcvarsall.bat
$vcvarsPath = "D:\eWDK_28000\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat"
if (-not (Test-Path $vcvarsPath))
{
    Write-Warning "vcvarsall.bat not found at expected location. Attempting generic search..."
    $vcvarsPathObj = Get-ChildItem -Path "D:\eWDK_28000" -Filter "vcvarsall.bat" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($vcvarsPathObj)
    {
        $vcvarsPath = $vcvarsPathObj.FullName
    } else
    {
        Write-Error "Could not locate vcvarsall.bat in eWDK."
        exit 1
    }
}

Write-Host "Found vcvarsall.bat at: $vcvarsPath" -ForegroundColor Green

# Execute nmake within the eWDK environment
# We construct a command that:
# 1. Calls LaunchBuildEnv.cmd to set up the WDK environment
# 2. Calls vcvarsall.bat to set up the C++ toolchain (INCLUDE, LIB, CL)
# 3. Adds the nmake directory to the PATH (redundancy)
# 4. Changes directory to source
# 5. Runs nmake
$buildCmd = "call `"$eWDKPath`" amd64 && call `"$vcvarsPath`" x64 && set `"PATH=$nmakeDir;%PATH%`" && cd /d `"$APOSource`" && nmake"

$process = Start-Process -FilePath "cmd.exe" -ArgumentList "/c", $buildCmd -NoNewWindow -PassThru -Wait

if ($process.ExitCode -ne 0)
{
    Write-Error "APO Build Failed with exit code $($process.ExitCode)"
    exit $process.ExitCode
}

Write-Host "APO Build Successful." -ForegroundColor Green
