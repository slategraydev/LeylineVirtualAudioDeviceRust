#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Diagnoses Leyline Audio Driver endpoint visibility issues on Hyper-V VMs
.DESCRIPTION
    Comprehensive diagnostic script to check driver load status, Windows Audio services,
    and endpoint enumeration in Hyper-V virtual machines.
.NOTES
    Must be run as Administrator on the VM where endpoints should appear.
#>

param(
    [switch]$VerboseOutput,
    [switch]$FixServices
)

$ErrorActionPreference = "Continue"

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host " Leyline Audio VM Diagnostic Tool" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Helper function for section headers
function Write-Section
{
    param([string]$Title)
    Write-Host "`n--- $Title ---`n" -ForegroundColor Yellow
}

# Helper for status output
function Write-Status
{
    param(
        [string]$Message,
        [string]$Status,
        [ConsoleColor]$StatusColor
    )
    Write-Host "  $Message" -NoNewline
    Write-Host " [$Status]" -ForegroundColor $StatusColor
}

# Check 1: Driver Installation Status
Write-Section "1. Driver Installation Status"

$leylineDriver = Get-PnpDevice | Where-Object {
    $_.FriendlyName -like "*Leyline*" -or $_.InstanceId -like "*LEYLINE*"
}

if ($leylineDriver)
{
    Write-Status -Message "Leyline driver found" -Status "FOUND" -StatusColor Green
    foreach ($dev in $leylineDriver)
    {
        $status = $dev.Status
        $color = if ($status -eq "OK")
        { "Green" 
        } else
        { "Red" 
        }
        Write-Host "    Device: $($dev.FriendlyName)" -ForegroundColor White
        Write-Host "    Instance ID: $($dev.InstanceId)" -ForegroundColor Gray
        Write-Host "    Status: " -NoNewline
        Write-Host "$status" -ForegroundColor $color
        Write-Host ""
    }
} else
{
    Write-Status -Message "Leyline driver" -Status "NOT FOUND" -StatusColor Red
    Write-Host "    The driver may not be installed or may have failed to load." -ForegroundColor Red
    Write-Host "    Run Install.ps1 on the VM to install the driver." -ForegroundColor Yellow
}

# Check 2: Windows Audio Services
Write-Section "2. Windows Audio Services"

$audioSrv = Get-Service -Name "Audiosrv" -ErrorAction SilentlyContinue
$audioEndpoint = Get-Service -Name "AudioEndpointBuilder" -ErrorAction SilentlyContinue

if ($audioSrv)
{
    $svcColor = if ($audioSrv.Status -eq "Running")
    { "Green" 
    } else
    { "Red" 
    }
    Write-Status -Message "Windows Audio Service (Audiosrv)" -Status $audioSrv.Status -StatusColor $svcColor

    if ($audioSrv.Status -ne "Running" -and $FixServices)
    {
        Write-Host "    Attempting to start service..." -ForegroundColor Yellow
        try
        {
            Start-Service -Name "Audiosrv"
            Write-Host "    Service started successfully" -ForegroundColor Green
        } catch
        {
            Write-Host "    Failed to start: $_" -ForegroundColor Red
        }
    } elseif ($audioSrv.Status -ne "Running")
    {
        Write-Host "    WARNING: Audio service not running. Endpoints won't appear!" -ForegroundColor Red
        Write-Host "    Run with -FixServices to auto-start services." -ForegroundColor Yellow
    }
} else
{
    Write-Status -Message "Windows Audio Service" -Status "NOT FOUND" -StatusColor Red
}

if ($audioEndpoint)
{
    $svcColor = if ($audioEndpoint.Status -eq "Running")
    { "Green" 
    } else
    { "Red" 
    }
    Write-Status -Message "Audio Endpoint Builder" -Status $audioEndpoint.Status -StatusColor $svcColor

    if ($audioEndpoint.Status -ne "Running" -and $FixServices)
    {
        Write-Host "    Attempting to start service..." -ForegroundColor Yellow
        try
        {
            Start-Service -Name "AudioEndpointBuilder"
            Write-Host "    Service started successfully" -ForegroundColor Green
        } catch
        {
            Write-Host "    Failed to start: $_" -ForegroundColor Red
        }
    }
} else
{
    Write-Status -Message "Audio Endpoint Builder" -Status "NOT FOUND" -StatusColor Red
}

# Check 3: Audio Endpoints Enumeration
Write-Section "3. Audio Endpoints Enumeration"

try
{
    # Load required assemblies for MMDevice API
    Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class MMDeviceEnumerator {
    public static dynamic GetDefaultAudioEndpoint(int dataFlow, int role) {
        var enumeratorType = Type.GetTypeFromCLSID(new Guid("BCDE0395-E52F-467C-8E3D-C4579291692E"));
        dynamic enumerator = Activator.CreateInstance(enumeratorType);
        return enumerator.GetDefaultAudioEndpoint(dataFlow, role);
    }
}
"@ -ErrorAction SilentlyContinue
} catch
{
    # Assembly might already be loaded
}

# Try to enumerate audio devices using PowerShell
$audioDevices = Get-PnpDevice -Class MEDIA | Where-Object { $_.Status -eq "OK" }

$renderEndpoints = $audioDevices | Where-Object { $_.FriendlyName -like "*output*" -or $_.FriendlyName -like "*speaker*" -or $_.FriendlyName -like "*leyline*" }
$captureEndpoints = $audioDevices | Where-Object { $_.FriendlyName -like "*input*" -or $_.FriendlyName -like "*microphone*" -or $_.FriendlyName -like "*leyline*" }

Write-Host "  All Active Audio Devices:" -ForegroundColor Cyan
$audioDevices | ForEach-Object {
    $isLeyline = if ($_.FriendlyName -like "*Leyline*")
    { " (LEYLINE)" 
    } else
    { "" 
    }
    Write-Host "    - $($_.FriendlyName)$isLeyline" -ForegroundColor $(if ($isLeyline)
        { "Green" 
        } else
        { "Gray" 
        })
}

if ($renderEndpoints.Count -eq 0)
{
    Write-Status -Message "Render (Output) Endpoints" -Status "NONE FOUND" -StatusColor Red
} else
{
    Write-Status -Message "Render (Output) Endpoints" -Status "$($renderEndpoints.Count) found" -StatusColor Green
}

if ($captureEndpoints.Count -eq 0)
{
    Write-Status -Message "Capture (Input) Endpoints" -Status "NONE FOUND" -StatusColor Red
} else
{
    Write-Status -Message "Capture (Input) Endpoints" -Status "$($captureEndpoints.Count) found" -StatusColor Green
}

# Check 4: Device Manager Deep Dive
Write-Section "4. Device Manager - Audio Devices"

$allAudio = Get-PnpDevice -Class MEDIA
$leylineDevs = $allAudio | Where-Object { $_.FriendlyName -like "*Leyline*" }

if ($leylineDevs)
{
    Write-Host "  Leyline devices in Device Manager:" -ForegroundColor Green
    foreach ($dev in $leylineDevs)
    {
        Write-Host "    [$(($dev.FriendlyName).PadRight(30))] Status: $($dev.Status)" -ForegroundColor White
    }
} else
{
    Write-Host "  No Leyline devices found in MEDIA class" -ForegroundColor Red

    # Check if driver file exists
    $driverPath = "C:\Windows\System32\drivers\leyline.sys"
    if (Test-Path $driverPath)
    {
        Write-Host "    Driver file exists at: $driverPath" -ForegroundColor Yellow
        $driverInfo = Get-Item $driverPath
        Write-Host "    Version: $($driverInfo.VersionInfo.FileVersion)" -ForegroundColor Gray
        Write-Host "    Last Modified: $($driverInfo.LastWriteTime)" -ForegroundColor Gray
    }
}

# Check 5: Event Log Analysis
Write-Section "5. Driver Event Log Analysis"

$startTime = (Get-Date).AddHours(-2)
$driverEvents = Get-WinEvent -FilterHashtable @{
    LogName = 'System'
    StartTime = $startTime
    ID = 7034, 7035, 7036, 7000, 7001, 7002, 7009, 7010, 1014
} -ErrorAction SilentlyContinue | Where-Object {
    $_.Message -like "*Leyline*" -or $_.Message -like "*audio*" -or $_.Message -like "*driver*"
}

if ($driverEvents)
{
    Write-Host "  Recent driver-related events:" -ForegroundColor Yellow
    $driverEvents | Select-Object -First 5 | ForEach-Object {
        $levelColor = switch ($_.Level)
        {
            1
            { "Red" 
            }      # Critical
            2
            { "Red" 
            }      # Error
            3
            { "Yellow" 
            }   # Warning
            default
            { "Gray" 
            }
        }
        Write-Host "    [$($_.TimeCreated.ToString('HH:mm:ss'))] " -NoNewline
        Write-Host "$($_.LevelDisplayName): " -NoNewline -ForegroundColor $levelColor
        Write-Host ($_.Message -split "`n")[0] -ForegroundColor Gray
    }
} else
{
    Write-Host "  No recent driver events found (last 2 hours)" -ForegroundColor Green
}

# Check 6: Hyper-V Specific Checks
Write-Section "6. Hyper-V Environment Check"

$computerInfo = Get-ComputerInfo
$isVM = $computerInfo.HyperVRequirementVirtualizationFirmwareEnabled -eq $false -or
(Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Virtual Machine\Guest\Parameters" -ErrorAction SilentlyContinue)

if ($computerInfo.HypervisorPresent -or $isVM)
{
    Write-Status -Message "Running in Virtual Machine" -Status "YES" -StatusColor Yellow

    # Check for Hyper-V Enhanced Session Mode
    $enhancedSession = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Virtual Machine\Guest\Parameters" -Name "VirtualMachineName" -ErrorAction SilentlyContinue
    if ($enhancedSession)
    {
        Write-Host "    VM Name: $($enhancedSession.VirtualMachineName)" -ForegroundColor Gray
    }

    Write-Host "`n  IMPORTANT: Hyper-V VMs require Enhanced Session Mode for local audio." -ForegroundColor Yellow
    Write-Host "  If using RDP: Audio redirection should provide endpoints via RDP." -ForegroundColor Yellow
    Write-Host "  If using console: Enable Enhanced Session Mode in Hyper-V settings." -ForegroundColor Yellow

    # Check if this is a generation 2 VM
    try
    {
        $vmGeneration = (Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Virtual Machine\Guest\Parameters" -Name "VirtualMachineName" -ErrorAction SilentlyContinue)
        if ($vmGeneration)
        {
            Write-Host "`n  To enable audio in Hyper-V console:" -ForegroundColor Cyan
            Write-Host "    1. Close VM connection window" -ForegroundColor White
            Write-Host "    2. Hyper-V Manager → VM Settings" -ForegroundColor White
            Write-Host "    3. Check 'Enhanced Session Mode' under 'Integration Services'" -ForegroundColor White
            Write-Host "    4. Or connect via RDP for audio redirection" -ForegroundColor White
        }
    } catch
    {
    }

} else
{
    Write-Status -Message "Running in Virtual Machine" -Status "NO" -StatusColor Green
}

# Check 7: Sound Control Panel
Write-Section "7. Sound Control Panel Status"

$soundPath = "mmsys.cpl"
$soundApplet = Get-Command "mmsys.cpl" -ErrorAction SilentlyContinue
if ($soundApplet)
{
    Write-Host "  Sound applet available: $soundPath" -ForegroundColor Green
    Write-Host "  Run 'mmsys.cpl' to open Sound settings and check for Leyline endpoints." -ForegroundColor Cyan
} else
{
    Write-Host "  Sound applet path not found" -ForegroundColor Yellow
}

# Summary
Write-Section "DIAGNOSTIC SUMMARY"

$issues = @()

if (-not $leylineDriver)
{
    $issues += "Driver not found in PnP devices - may not be installed or loaded"
}
if ($audioSrv.Status -ne "Running")
{
    $issues += "Windows Audio service not running - endpoints cannot appear"
}
if ($audioEndpoint.Status -ne "Running")
{
    $issues += "Audio Endpoint Builder not running - endpoints cannot be enumerated"
}
if ($renderEndpoints.Count -eq 0 -and $leylineDriver)
{
    $issues += "Driver present but no render endpoints - possible topology issue"
}

if ($issues.Count -eq 0)
{
    Write-Host "  ✅ No critical issues found!" -ForegroundColor Green
    Write-Host "  If endpoints still don't appear in applications:" -ForegroundColor Cyan
    Write-Host "    1. Check Sound Control Panel (mmsys.cpl)" -ForegroundColor White
    Write-Host "    2. Restart Windows Audio service: Restart-Service Audiosrv -Force" -ForegroundColor White
    Write-Host "    3. Check if Hyper-V Enhanced Session Mode is enabled" -ForegroundColor White
} else
{
    Write-Host "  ❌ Issues Found:" -ForegroundColor Red
    foreach ($issue in $issues)
    {
        Write-Host "     - $issue" -ForegroundColor Red
    }

    Write-Host "`n  RECOMMENDED ACTIONS:" -ForegroundColor Cyan
    if (-not $leylineDriver)
    {
        Write-Host "    1. Install driver: .\\scripts\\Install.ps1" -ForegroundColor Yellow
    }
    if ($audioSrv.Status -ne "Running" -or $audioEndpoint.Status -ne "Running")
    {
        Write-Host "    2. Start audio services:" -ForegroundColor Yellow
        Write-Host "       Start-Service Audiosrv" -ForegroundColor Gray
        Write-Host "       Start-Service AudioEndpointBuilder" -ForegroundColor Gray
        Write-Host "    Or run this script with -FixServices parameter" -ForegroundColor Gray
    }
    if ($leylineDriver -and ($audioSrv.Status -eq "Running"))
    {
        Write-Host "    3. Restart audio subsystem:" -ForegroundColor Yellow
        Write-Host "       Restart-Service Audiosrv -Force" -ForegroundColor Gray
    }
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host " Diagnostic Complete" -ForegroundColor Cyan
Write-Host "========================================`n" -ForegroundColor Cyan

# Verbose output
if ($VerboseOutput)
{
    Write-Section "VERBOSE: All PnP Devices (MEDIA class)"
    Get-PnpDevice -Class MEDIA | Format-Table -Property FriendlyName, InstanceId, Status -AutoSize

    Write-Section "VERBOSE: Driver File Details"
    $sysPath = "C:\Windows\System32\drivers\leyline.sys"
    if (Test-Path $sysPath)
    {
        Get-ItemProperty $sysPath | Select-Object Name, Length, LastWriteTime, VersionInfo | Format-List
    }
}
