#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Pre-flight check for Leyline Audio Driver installation on host PC
.DESCRIPTION
    Verifies system readiness before installing the Leyline Audio Driver
    on a physical Windows 10/11 machine.
.NOTES
    Run this before Install.ps1 to catch issues early.
#>

param(
    [switch]$Fix,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"

function Write-Check
{
    param(
        [string]$Item,
        [bool]$Passed,
        [string]$Message = ""
    )
    $status = if ($Passed)
    { "✅ PASS" 
    } else
    { "❌ FAIL" 
    }
    $color = if ($Passed)
    { "Green" 
    } else
    { "Red" 
    }
    Write-Host "  [$status] $Item" -ForegroundColor $color
    if ($Message -and -not $Passed)
    {
        Write-Host "       → $Message" -ForegroundColor Yellow
    }
}

function Write-Section
{
    param([string]$Title)
    Write-Host "`n$Title" -ForegroundColor Cyan
    Write-Host ("-" * $Title.Length) -ForegroundColor Cyan
}

Write-Host "`n╔════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Leyline Audio Driver - Pre-Flight Check ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════╝" -ForegroundColor Cyan

$allPassed = $true
$issues = @()

# 1. Administrator Check
Write-Section "1. Administrator Privileges"
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
Write-Check -Item "Running as Administrator" -Passed $isAdmin
if (-not $isAdmin)
{
    $allPassed = $false
    $issues += "Must run as Administrator"
}

# 2. Windows Version Check
Write-Section "2. Operating System"
$osInfo = Get-ComputerInfo
$windowsVersion = $osInfo.WindowsVersion
$isWin10 = $windowsVersion -ge "10.0" -and $osInfo.OsName -like "*Windows 10*"
$isWin11 = $windowsVersion -ge "10.0.22000" -and $osInfo.OsName -like "*Windows 11*"

$osSupported = $isWin10 -or $isWin11
Write-Check -Item "Windows Version: $($osInfo.OsName) ($windowsVersion)" -Passed $osSupported -Message "Requires Windows 10 or 11"
if (-not $osSupported)
{
    $allPassed = $false
    $issues += "Windows 10 or 11 required"
}

# 3. Architecture Check
Write-Section "3. System Architecture"
$is64bit = $osInfo.OsArchitecture -eq "64-bit"
Write-Check -Item "64-bit Windows" -Passed $is64bit -Message "Requires 64-bit Windows"
if (-not $is64bit)
{
    $allPassed = $false
    $issues += "64-bit Windows required"
}

# 4. Test Signing Check
Write-Section "4. Driver Test Signing"
$bcdOutput = bcdedit /enum "{current}" | Out-String
$testSigningOn = $bcdOutput -match "testsigning\s+Yes"
$testSigningNo = $bcdOutput -match "testsigning\s+No"
$testSigningEnabled = $testSigningOn -and -not $testSigningNo

Write-Check -Item "Test Signing Enabled" -Passed $testSigningEnabled
if (-not $testSigningEnabled)
{
    if ($Fix)
    {
        Write-Host "       Attempting to enable test signing..." -ForegroundColor Yellow
        bcdedit /set testsigning on | Out-Null
        Write-Host "       ✅ Test signing enabled - REBOOT REQUIRED!" -ForegroundColor Green
        $issues += "REBOOT REQUIRED: Test signing was just enabled"
    } else
    {
        Write-Host "       Run with -Fix to auto-enable, or run: bcdedit /set testsigning on" -ForegroundColor Yellow
        $allPassed = $false
        $issues += "Test signing must be enabled (requires reboot)"
    }
}

# 5. Secure Boot Check (Warn only)
Write-Section "5. Secure Boot Status"
$secureBoot = Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\SecureBoot\State" -Name "UEFISecureBootEnabled" -ErrorAction SilentlyContinue
$secureBootEnabled = $secureBoot -and $secureBoot.UEFISecureBootEnabled -eq 1
if ($secureBootEnabled)
{
    Write-Host "  ⚠️  WARNING: Secure Boot is enabled" -ForegroundColor Yellow
    Write-Host "       You may need to disable Secure Boot in BIOS for test-signed drivers" -ForegroundColor Yellow
} else
{
    Write-Check -Item "Secure Boot disabled (or not present)" -Passed $true
}

# 6. eWDK Check
Write-Section "6. Enterprise WDK (eWDK)"
$ewdkPaths = @(
    "D:\eWDK_28000",
    "C:\eWDK_28000",
    "$env:SystemDrive\eWDK_28000"
)
$ewdkFound = $null
foreach ($path in $ewdkPaths)
{
    if (Test-Path "$path\LaunchBuildEnv.cmd")
    {
        $ewdkFound = $path
        break
    }
}

Write-Check -Item "eWDK 28000 Installation" -Passed ($null -ne $ewdkFound) -Message "Install eWDK 10.0.28000.0 at D:\eWDK_28000"
if ($ewdkFound)
{
    Write-Host "       Found at: $ewdkFound" -ForegroundColor Gray
    $env:eWDK_ROOT_DIR = $ewdkFound
} else
{
    $allPassed = $false
    $issues += "eWDK not found - install from https://learn.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk"
}

# 7. Rust Toolchain
Write-Section "7. Rust Toolchain"
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
$rustc = Get-Command rustc -ErrorAction SilentlyContinue
$rustOk = $cargo -and $rustc

Write-Check -Item "Rust/Cargo installed" -Passed $rustOk -Message "Install from https://rustup.rs"
if ($rustOk)
{
    $rustVersion = rustc --version
    Write-Host "       Version: $rustVersion" -ForegroundColor Gray

    # Check for cargo-wdk
    $cargoWdk = cargo wdk --version 2>&1
    $hasCargoWdk = $LASTEXITCODE -eq 0
    Write-Check -Item "cargo-wdk installed" -Passed $hasCargoWdk -Message "Run: cargo install cargo-wdk"
    if (-not $hasCargoWdk)
    {
        $allPassed = $false
        $issues += "cargo-wdk not installed"
    }
} else
{
    $allPassed = $false
    $issues += "Rust not installed"
}

# 8. .NET SDK (for HSA)
Write-Section "8. .NET SDK"
$dotnet = Get-Command dotnet -ErrorAction SilentlyContinue
$dotnetOk = $null -ne $dotnet

Write-Check -Item ".NET SDK installed" -Passed $dotnetOk -Message "Install .NET 8.0 SDK from https://dotnet.microsoft.com/download"
if ($dotnetOk)
{
    $dotnetVersion = dotnet --version
    Write-Host "       Version: $dotnetVersion" -ForegroundColor Gray

    # Check version is 8.0 or higher
    $majorVersion = [int]($dotnetVersion -split '\.')[0]
    if ($majorVersion -lt 8)
    {
        Write-Host "       ⚠️  .NET 8.0 or higher recommended" -ForegroundColor Yellow
    }
} else
{
    $allPassed = $false
    $issues += ".NET SDK not installed"
}

# 9. Existing Leyline Check
Write-Section "9. Existing Leyline Installation"
$existingDevices = Get-PnpDevice -PresentOnly:$false | Where-Object {
    $_.HardwareID -contains "Root\LeylineAudio" -or
    $_.FriendlyName -like "*Leyline*"
} -ErrorAction SilentlyContinue

$hasExisting = $existingDevices -and $existingDevices.Count -gt 0
Write-Check -Item "No conflicting Leyline devices" -Passed (-not $hasExisting)
if ($hasExisting)
{
    Write-Host "       Found existing devices:" -ForegroundColor Yellow
    foreach ($dev in $existingDevices)
    {
        Write-Host "         - $($dev.FriendlyName) [$($dev.Status)]" -ForegroundColor Gray
    }
    Write-Host "       Run Uninstall.ps1 first to clean up" -ForegroundColor Yellow
    $issues += "Existing Leyline installation detected"
}

# 10. Disk Space
Write-Section "10. Disk Space"
$sysDrive = Get-CimInstance -ClassName Win32_LogicalDisk -Filter "DeviceID='$env:SystemDrive'"
$freeGB = [math]::Round($sysDrive.FreeSpace / 1GB, 2)
$hasSpace = $freeGB -gt 5

Write-Check -Item "Free disk space: ${freeGB}GB" -Passed $hasSpace -Message "Need at least 5GB free"
if (-not $hasSpace)
{
    $allPassed = $false
    $issues += "Insufficient disk space"
}

# 11. Windows Audio Services
Write-Section "11. Windows Audio Services"
$audioSrv = Get-Service -Name "Audiosrv" -ErrorAction SilentlyContinue
$audioBuilder = Get-Service -Name "AudioEndpointBuilder" -ErrorAction SilentlyContinue

$audioSrvRunning = $audioSrv -and $audioSrv.Status -eq "Running"
$audioBuilderRunning = $audioBuilder -and $audioBuilder.Status -eq "Running"

Write-Check -Item "Windows Audio Service (Audiosrv)" -Passed $audioSrvRunning -Message "Service not running"
Write-Check -Item "Audio Endpoint Builder" -Passed $audioBuilderRunning -Message "Service not running"

if (-not $audioSrvRunning -or -not $audioBuilderRunning)
{
    if ($Fix)
    {
        Write-Host "       Attempting to start services..." -ForegroundColor Yellow
        if ($audioSrv -and $audioSrv.Status -ne "Running")
        { Start-Service Audiosrv 
        }
        if ($audioBuilder -and $audioBuilder.Status -ne "Running")
        { Start-Service AudioEndpointBuilder 
        }
        Write-Host "       ✅ Services started" -ForegroundColor Green
    } else
    {
        $allPassed = $false
        $issues += "Windows Audio services not running (run with -Fix to auto-start)"
    }
}

# Summary
Write-Host "`n══════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "PRE-FLIGHT SUMMARY" -ForegroundColor Cyan
Write-Host "══════════════════════════════════════════" -ForegroundColor Cyan

if ($allPassed -and $issues.Count -eq 0)
{
    Write-Host "`n✅ ALL CHECKS PASSED - Ready to install!" -ForegroundColor Green
    Write-Host "`nNext steps:" -ForegroundColor White
    Write-Host "  1. Run: .\\scripts\\Install.ps1" -ForegroundColor Yellow
    Write-Host "  2. After installation, check Sound Control Panel (mmsys.cpl)" -ForegroundColor Yellow
    Write-Host "  3. Leyline endpoints should appear as audio devices" -ForegroundColor Yellow
} else
{
    Write-Host "`n❌ PRE-FLIGHT CHECK FAILED" -ForegroundColor Red

    if ($issues.Count -gt 0)
    {
        Write-Host "`nIssues found:" -ForegroundColor Yellow
        foreach ($issue in $issues)
        {
            Write-Host "  • $issue" -ForegroundColor Red
        }
    }

    Write-Host "`nTo resolve:" -ForegroundColor Cyan
    if ($issues -contains "Test signing must be enabled (requires reboot)")
    {
        Write-Host "  1. Run: bcdedit /set testsigning on" -ForegroundColor White
        Write-Host "  2. Reboot your PC" -ForegroundColor White
        Write-Host "  3. Re-run this check: .\\scripts\\Test-HostReady.ps1" -ForegroundColor White
    }
    if ($issues -contains "eWDK not found")
    {
        Write-Host "  • Download eWDK 10.0.28000.0 ISO and mount to D:\eWDK_28000" -ForegroundColor White
    }

    Write-Host "`nOr run with -Fix to auto-resolve some issues:" -ForegroundColor Yellow
    Write-Host "  .\\scripts\\Test-HostReady.ps1 -Fix" -ForegroundColor Yellow
}

# Verbose output
if ($Verbose)
{
    Write-Host "`n--- Detailed System Info ---" -ForegroundColor Gray
    Write-Host "Computer: $($osInfo.CsName)" -ForegroundColor Gray
    Write-Host "OS: $($osInfo.OsName) Build $($osInfo.OsBuildNumber)" -ForegroundColor Gray
    Write-Host "Processor: $($osInfo.CsProcessors.Name)" -ForegroundColor Gray
    Write-Host "Memory: $([math]::Round($osInfo.TotalPhysicalMemory/1GB, 1))GB" -ForegroundColor Gray
}

Write-Host ""
