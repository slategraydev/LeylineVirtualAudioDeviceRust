#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Diagnostic script to troubleshoot missing DbgPrint output from the Leyline Audio Driver.

.DESCRIPTION
    This script checks all common configuration issues that prevent kernel DbgPrint
    from appearing in DebugView, including:
    - Kernel debugging configuration (bcdedit)
    - Debug Print Filter registry settings
    - Driver load status
    - DebugView process detection
    - Event Log analysis for driver load errors

.EXAMPLE
    .\Debug-Troubleshoot.ps1
    Run this on your test VM to diagnose why DbgPrint output isn't visible.

.NOTES
    File Name      : Debug-Troubleshoot.ps1
    Author         : Leyline Audio Driver Team
    Prerequisite   : PowerShell 5.1 or later, Administrator privileges
#>

[CmdletBinding()]
param()

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Leyline Driver: DbgPrint Diagnostic" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Store results
$issuesFound = @()
$warnings = @()

# Check 1: Administrator privileges
Write-Host "[1/8] Checking Administrator privileges..." -ForegroundColor Yellow
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))
{
    $issuesFound += "Not running as Administrator. DebugView and driver installation require admin rights."
    Write-Host "      ✗ FAIL: Not running as Administrator" -ForegroundColor Red
} else
{
    Write-Host "      ✓ Running as Administrator" -ForegroundColor Green
}

# Check 2: Kernel Debugging Configuration (bcdedit)
Write-Host ""
Write-Host "[2/8] Checking kernel debugging configuration (bcdedit)..." -ForegroundColor Yellow
try
{
    $bcdOutput = bcdedit /enum 2>&1
    $debugLine = $bcdOutput | Select-String "debug"

    if ($debugLine -match "debug.*Yes")
    {
        Write-Host "      ✓ Kernel debugging is enabled" -ForegroundColor Green
    } elseif ($debugLine -match "debug.*No")
    {
        $issuesFound += "Kernel debugging is disabled (bcdedit). Run: bcdedit /debug on"
        Write-Host "      ✗ FAIL: Kernel debugging is disabled" -ForegroundColor Red
        Write-Host "      Fix: Run 'bcdedit /debug on' and reboot" -ForegroundColor Yellow
    } else
    {
        $warnings += "Could not determine kernel debugging status from bcdedit"
        Write-Host "      ? WARNING: Could not parse bcdedit output" -ForegroundColor Yellow
    }
} catch
{
    $issuesFound += "Failed to run bcdedit: $_"
    Write-Host "      ✗ ERROR: Could not run bcdedit" -ForegroundColor Red
}

# Check 3: Debug Print Filter Registry
Write-Host ""
Write-Host "[3/8] Checking Debug Print Filter registry..." -ForegroundColor Yellow
$regPath = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Debug Print Filter"
$regName = "DEFAULT"

if (Test-Path $regPath)
{
    try
    {
        $regValue = Get-ItemProperty -Path $regPath -Name $regName -ErrorAction Stop
        $value = $regValue.$regName

        # Note: In registry, this is a DWORD which can be signed or unsigned
        # 0xFFFFFFFF (-1 as signed int) means ALL debug levels enabled
        # Any non-zero value enables some debug output
        $isAllEnabled = ($value -eq 0xffffffff) -or ($value -eq -1) -or ($value -eq [uint32]::MaxValue)
        $isSomeEnabled = ($value -ne 0)

        if ($isAllEnabled)
        {
            Write-Host "      ✓ Debug Print Filter is set to show ALL output (0xFFFFFFFF)" -ForegroundColor Green
            Write-Host "            All DbgPrint output will be visible" -ForegroundColor Gray
        } elseif ($isSomeEnabled)
        {
            # Partial filtering - show as warning but not critical
            Write-Host "      ? NOTE: Debug Print Filter is set to 0x$($value.ToString('X8'))" -ForegroundColor Yellow
            Write-Host "            Some debug levels may be filtered, but most output should be visible" -ForegroundColor Gray
        } else # $value -eq 0
        {
            $issuesFound += "Debug Print Filter is 0 - all DbgPrint output is suppressed!"
            Write-Host "      ✗ FAIL: Debug Print Filter is 0 (all output suppressed)" -ForegroundColor Red
            Write-Host "            Run: .\scripts\Enable-KernelDebug.ps1" -ForegroundColor Yellow
        }
    } catch
    {
        $issuesFound += "Debug Print Filter registry key exists but value could not be read"
        Write-Host "      ✗ ERROR: Could not read registry value" -ForegroundColor Red
    }
} else
{
    $issuesFound += "Debug Print Filter registry key is missing. This means DbgPrint is filtered by default."
    Write-Host "      ✗ FAIL: Debug Print Filter registry key not found" -ForegroundColor Red
    Write-Host "      Fix: Run .\scripts\Enable-KernelDebug.ps1 and reboot" -ForegroundColor Yellow
}

# Check 4: Driver Load Status
Write-Host ""
Write-Host "[4/8] Checking Leyline driver load status..." -ForegroundColor Yellow
$driverService = Get-Service -Name "LeylineAudio" -ErrorAction SilentlyContinue
$driverInRegistry = Test-Path "HKLM:\SYSTEM\CurrentControlSet\Services\LeylineAudio"

if ($driverInRegistry)
{
    Write-Host "      ✓ Driver is registered in registry" -ForegroundColor Green

    if ($driverService)
    {
        Write-Host "      ✓ Driver service exists: $($driverService.Status)" -ForegroundColor Green

        if ($driverService.Status -eq "Running")
        {
            Write-Host "      ✓ Driver is RUNNING" -ForegroundColor Green
        } else
        {
            $warnings += "Driver service exists but is not running (Status: $($driverService.Status))"
            Write-Host "      ? WARNING: Driver service status is: $($driverService.Status)" -ForegroundColor Yellow
        }
    } else
    {
        $issuesFound += "Driver is registered but service object not found. Driver may have failed to start."
        Write-Host "      ✗ FAIL: Driver registry exists but service not found" -ForegroundColor Red
    }

    # Check Device Manager
    $device = Get-PnpDevice | Where-Object { $_.FriendlyName -like "*Leyline*" -or $_.InstanceId -like "*Leyline*" } | Select-Object -First 1
    if ($device)
    {
        Write-Host "      ✓ Device found in Device Manager: $($device.FriendlyName) [$($device.Status)]" -ForegroundColor Green
    } else
    {
        $warnings += "Driver service exists but no PnP device found in Device Manager"
        Write-Host "      ? WARNING: No Leyline device found in Device Manager" -ForegroundColor Yellow
    }
} else
{
    $issuesFound += "Leyline driver is not installed. Run Install.ps1 first."
    Write-Host "      ✗ FAIL: Leyline driver not found in registry" -ForegroundColor Red
    Write-Host "      Fix: Run .\scripts\Install.ps1 to install the driver" -ForegroundColor Yellow
}

# Check 5: DebugView Process
Write-Host ""
Write-Host "[5/8] Checking if DebugView is running..." -ForegroundColor Yellow
$debugView = Get-Process | Where-Object {
    $_.ProcessName -match "DebugView" -or
    $_.ProcessName -match "Dbgview" -or
    $_.ProcessName -match "DbgView" -or
    $_.ProcessName -eq "DebugView64" -or
    $_.ProcessName -eq "DebugView"
}
if ($debugView)
{
    Write-Host "      ✓ DebugView is running (PID: $($debugView.Id), Name: $($debugView.ProcessName))" -ForegroundColor Green

    # Check if running as admin
    $debugViewProcess = Get-WmiObject -Class Win32_Process -Filter "ProcessId=$($debugView.Id)" -ErrorAction SilentlyContinue
    if ($debugViewProcess)
    {
        $owner = $debugViewProcess.GetOwner()
        if ($owner.Domain -and $owner.User)
        {
            Write-Host "      ℹ DebugView running as: $($owner.Domain)\$($owner.User)" -ForegroundColor Gray
        }
    }
} else
{
    $warnings += "DebugView is not running. You need to start it to see DbgPrint output."
    Write-Host "      ? WARNING: DebugView is not running" -ForegroundColor Yellow
    Write-Host "      Action: Download and run DebugView as Administrator" -ForegroundColor Yellow
}

# Check 6: System Event Log for Driver Errors
Write-Host ""
Write-Host "[6/8] Checking System Event Log for driver errors..." -ForegroundColor Yellow
$startTime = (Get-Date).AddHours(-1)
$events = Get-WinEvent -FilterHashtable @{LogName='System'; StartTime=$startTime} -ErrorAction SilentlyContinue |
    Where-Object { $_.Message -like "*Leyline*" -or $_.Message -like "*LeylineAudio*" } |
    Select-Object -First 5

if ($events)
{
    Write-Host "      ! Recent events found:" -ForegroundColor Yellow
    foreach ($event in $events)
    {
        $level = switch ($event.Level)
        {
            1
            { "CRITICAL"
            }
            2
            { "ERROR"
            }
            3
            { "WARNING"
            }
            4
            { "INFO"
            }
            default
            { "UNKNOWN"
            }
        }
        $color = if ($event.Level -le 2)
        { "Red"
        } elseif ($event.Level -eq 3)
        { "Yellow"
        } else
        { "Gray"
        }
        Write-Host "        [$level] $($event.TimeCreated): $($event.Message.Substring(0, [Math]::Min(80, $event.Message.Length)))..." -ForegroundColor $color
    }
} else
{
    Write-Host "      ✓ No recent driver events in System log" -ForegroundColor Green
}

# Check 7: Kernel Debug Settings
Write-Host ""
Write-Host "[7/8] Checking additional kernel debug settings..." -ForegroundColor Yellow

# Check if booted with debugging
$debugMode = (Get-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control" -Name "SystemStartOptions" -ErrorAction SilentlyContinue).SystemStartOptions
if ($debugMode -and $debugMode -match "DEBUG")
{
    Write-Host "      ✓ System booted with DEBUG option" -ForegroundColor Green
} else
{
    $warnings += "System may not have been booted with kernel debugging enabled (requires reboot after bcdedit)"
    Write-Host "      ? WARNING: DEBUG option not detected in boot parameters" -ForegroundColor Yellow
    Write-Host "            This is normal if you haven't rebooted since running Enable-KernelDebug.ps1" -ForegroundColor Gray
}

# Check 8: Common Issues Summary
Write-Host ""
Write-Host "[8/8] Common issues check..." -ForegroundColor Yellow

# Check Windows version (DbgPrint behavior changed in Win10)
$osVersion = [System.Environment]::OSVersion.Version
if ($osVersion -ge [System.Version]"10.0.10240")
{
    Write-Host "      ℹ Windows 10/11 detected (DbgPrint is filtered by default)" -ForegroundColor Cyan
    Write-Host "        The registry/bcdedit settings above are REQUIRED to see output" -ForegroundColor Gray
}

# Final Summary
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  DIAGNOSTIC SUMMARY" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

if ($issuesFound.Count -eq 0 -and $warnings.Count -eq 0)
{
    Write-Host "✅ ALL CHECKS PASSED" -ForegroundColor Green
    Write-Host ""
    Write-Host "Configuration appears correct. If you still don't see output:"
    Write-Host "  1. Make sure DebugView is running as Administrator"
    Write-Host "  2. In DebugView, enable Capture → Capture Kernel"
    Write-Host "  3. In DebugView, set filter to: Leyline*;LeylineTopo*"
    Write-Host "  4. Try loading the driver again with Install.ps1"
    Write-Host "  5. Check if driver appears in Device Manager (may show Code 10)"
    Write-Host ""
    Write-Host "If driver shows Code 10 in Device Manager but you see NO DebugView output,"
    Write-Host "the driver may be failing before DriverEntry completes. Check System Event Log."
} else
{
    if ($issuesFound.Count -gt 0)
    {
        Write-Host "❌ CRITICAL ISSUES FOUND ($($issuesFound.Count)):" -ForegroundColor Red
        Write-Host ""
        for ($i = 0; $i -lt $issuesFound.Count; $i++)
        {
            Write-Host "  $($i+1). $($issuesFound[$i])" -ForegroundColor Red
        }
        Write-Host ""
    }

    if ($warnings.Count -gt 0)
    {
        Write-Host "⚠️  WARNINGS ($($warnings.Count)):" -ForegroundColor Yellow
        Write-Host ""
        for ($i = 0; $i -lt $warnings.Count; $i++)
        {
            Write-Host "  $($i+1). $($warnings[$i])" -ForegroundColor Yellow
        }
        Write-Host ""
    }

    Write-Host "RECOMMENDED FIXES:" -ForegroundColor Cyan
    Write-Host ""

    if ($issuesFound -match "bcdedit")
    {
        Write-Host "1. Enable kernel debugging:" -ForegroundColor White
        Write-Host "   bcdedit /debug on" -ForegroundColor Gray
        Write-Host "   (Then REBOOT)" -ForegroundColor Yellow
        Write-Host ""
    }

    if ($issuesFound -match "Debug Print Filter")
    {
        Write-Host "2. Set Debug Print Filter:" -ForegroundColor White
        Write-Host "   .\scripts\Enable-KernelDebug.ps1" -ForegroundColor Gray
        Write-Host "   (Then REBOOT)" -ForegroundColor Yellow
        Write-Host ""
    }

    if ($issuesFound -match "not installed")
    {
        Write-Host "3. Install the driver:" -ForegroundColor White
        Write-Host "   .\scripts\Install.ps1" -ForegroundColor Gray
        Write-Host ""
    }

    Write-Host "QUICK FIX - Run these commands as Administrator, then REBOOT:" -ForegroundColor Cyan
    Write-Host "   bcdedit /debug on" -ForegroundColor Yellow
    Write-Host "   reg add 'HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Debug Print Filter' /v DEFAULT /t REG_DWORD /d 0xffffffff /f" -ForegroundColor Yellow
    Write-Host ""
}

Write-Host "For detailed instructions, see TEST_REVIEW.md section 'DebugView Setup'" -ForegroundColor Gray
Write-Host ""
