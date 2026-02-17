#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Diagnoses Leyline Audio Driver endpoint visibility issues
.DESCRIPTION
    Comprehensive diagnostic for when driver loads but audio endpoints
    don't appear in Sound Control Panel. Checks services, device properties,
    registry configuration, and audio stack state.
.NOTES
    Run after driver installation when endpoints aren't visible.
#>

param(
    [switch]$Verbose,
    [switch]$AttemptFix
)

$ErrorActionPreference = "Continue"

function Write-Diag
{
    param(
        [string]$Message,
        [ConsoleColor]$Color = "White",
        [string]$Prefix = "DIAG"
    )
    Write-Host "[$Prefix] $Message" -ForegroundColor $Color
}

function Write-Section
{
    param([string]$Title)
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host " $Title" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
}

function Get-DeviceConfigFlags
{
    param([string]$InstanceId)
    try
    {
        $regPath = "HKLM:\SYSTEM\CurrentControlSet\Enum\$InstanceId"
        $devReg = Get-ItemProperty -Path $regPath -Name "ConfigFlags" -ErrorAction SilentlyContinue
        if ($devReg)
        {
            return $devReg.ConfigFlags
        }
        return $null
    } catch
    {
        return $null
    }
}

Write-Host "`n╔════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║    LEYLINE AUDIO - ENDPOINT DIAGNOSTIC          ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════╝" -ForegroundColor Cyan

# 1. Driver Device Status
Write-Section "1. Driver Device Status"

$leylineDevices = Get-PnpDevice -PresentOnly:$true | Where-Object {
    $_.FriendlyName -like "*Leyline*" -or
    $_.HardwareID -contains "Root\LeylineAudio"
}

if ($leylineDevices)
{
    foreach ($dev in $leylineDevices)
    {
        Write-Diag -Message "Device Found:" -Color Green -Prefix "OK"
        Write-Diag -Message "  Name: $($dev.FriendlyName)" -Color White
        Write-Diag -Message "  Instance ID: $($dev.InstanceId)" -Color Gray
        Write-Diag -Message "  Status: $($dev.Status)" -Color $(if($dev.Status -eq "OK")
            {"Green"
            } else
            {"Red"
            })

        # Get detailed status
        $problemCode = (Get-PnpDeviceProperty -InstanceId $dev.InstanceId -KeyName "DEVPKEY_Device_ProblemCode" -ErrorAction SilentlyContinue).Data
        if ($problemCode)
        {
            Write-Diag -Message "  Problem Code: $problemCode" -Color Red
        }

        # Get configuration flags
        $configFlags = Get-DeviceConfigFlags -InstanceId $dev.InstanceId
        if ($configFlags)
        {
            Write-Diag -Message "  Config Flags: 0x$($configFlags.ToString('X8'))" -Color Yellow
            if ($configFlags -band 0x00000001)
            {
                Write-Diag -Message "    WARNING: CONFIGFLAG_FAILEDINSTALL set" -Color Red
            }
            if ($configFlags -band 0x00000040)
            {
                Write-Diag -Message "    WARNING: CONFIGFLAG_DISABLED set" -Color Red
            }
        }
    }
} else
{
    Write-Diag -Message "No Leyline devices found!" -Color Red -Prefix "FAIL"
    Write-Diag -Message "Install the driver first: .\scripts\Install.ps1" -Color Yellow
    return
}

# 2. Windows Audio Services
Write-Section "2. Windows Audio Services"

$audioServices = @(
    @{Name="Audiosrv"; Display="Windows Audio"},
    @{Name="AudioEndpointBuilder"; Display="Audio Endpoint Builder"},
    @{Name="PlugPlay"; Display="Plug and Play"}
)

foreach ($svcInfo in $audioServices)
{
    $svc = Get-Service -Name $svcInfo.Name -ErrorAction SilentlyContinue
    if ($svc)
    {
        $color = if ($svc.Status -eq "Running")
        { "Green" 
        } else
        { "Red" 
        }
        Write-Diag -Message "$($svcInfo.Display): $($svc.Status)" -Color $color -Prefix $(if($svc.Status -eq "Running")
            {"OK"
            } else
            {"FAIL"
            })

        if ($svc.Status -ne "Running" -and $AttemptFix)
        {
            Write-Diag -Message "  Attempting to start service..." -Color Yellow
            try
            {
                Start-Service -Name $svcInfo.Name
                Write-Diag -Message "  Started successfully" -Color Green
            } catch
            {
                Write-Diag -Message "  FAILED: $_" -Color Red
            }
        }
    } else
    {
        Write-Diag -Message "$($svcInfo.Display): NOT FOUND" -Color Red -Prefix "FAIL"
    }
}

# 3. Audio Endpoint Enumeration
Write-Section "3. Audio Endpoint Enumeration"

try
{
    # Use MMDeviceEnumerator via PowerShell
    Add-Type -TypeDefinition @"
        using System;
        using System.Runtime.InteropServices;
        using System.Collections.Generic;

        [Guid("BCDE0395-E52F-467C-8E3D-C4579291692E"), ComImport]
        public class MMDeviceEnumerator { }

        [Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        public interface IMMDeviceEnumerator {
            [PreserveSig]
            int EnumAudioEndpoints(int dataFlow, int dwStateMask, out IMMDeviceCollection ppDevices);
            [PreserveSig]
            int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice ppEndpoint);
        }

        [Guid("0BD7A1BE-7A1A-44DB-8397-CC5392387B5E"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        public interface IMMDeviceCollection {
            [PreserveSig]
            int GetCount(out int pcDevices);
            [PreserveSig]
            int Item(int nDevice, out IMMDevice ppDevice);
        }

        [Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
        public interface IMMDevice {
            [PreserveSig]
            int Activate(ref Guid iid, int dwClsCtx, IntPtr pActivationParams, [MarshalAs(UnmanagedType.IUnknown)] out object ppInterface);
            [PreserveSig]
            int OpenPropertyStore(int stgmAccess, out IntPtr ppProperties);
            [PreserveSig]
            int GetId([MarshalAs(UnmanagedType.LPWStr)] out string ppstrId);
            [PreserveSig]
            int GetState(out int pdwState);
        }
"@ -ErrorAction SilentlyContinue

    $enumeratorType = [Type]::GetTypeFromCLSID([Guid]"BCDE0395-E52F-467C-8E3D-C4579291692E")
    $enumerator = [Activator]::CreateInstance($enumeratorType)

    # 0 = eRender, 1 = eCapture
    # 1 = DEVICE_STATE_ACTIVE
    $renderCollection = $null
    $captureCollection = $null

    try
    {
        $enumerator.EnumAudioEndpoints(0, 1, [ref]$renderCollection) | Out-Null
    } catch
    {
    }

    try
    {
        $enumerator.EnumAudioEndpoints(1, 1, [ref]$captureCollection) | Out-Null
    } catch
    {
    }

    $renderCount = 0
    $captureCount = 0

    if ($renderCollection)
    {
        $renderCollection.GetCount([ref]$renderCount) | Out-Null
    }
    if ($captureCollection)
    {
        $captureCollection.GetCount([ref]$captureCount) | Out-Null
    }

    Write-Diag -Message "Active Render (Output) Endpoints: $renderCount" -Color White
    Write-Diag -Message "Active Capture (Input) Endpoints: $captureCount" -Color White

    # Check for Leyline in endpoints
    $leylineInRender = $false
    $leylineInCapture = $false

    for ($i = 0; $i -lt $renderCount; $i++)
    {
        $device = $null
        try
        {
            $renderCollection.Item($i, [ref]$device) | Out-Null
            if ($device)
            {
                $id = ""
                $device.GetId([ref]$id) | Out-Null
                if ($id -like "*Leyline*" -or $id -like "*leyline*")
                {
                    $leylineInRender = $true
                    Write-Diag -Message "Found Leyline in Render: $id" -Color Green -Prefix "OK"
                }
            }
        } catch
        {
        }
    }

    for ($i = 0; $i -lt $captureCount; $i++)
    {
        $device = $null
        try
        {
            $captureCollection.Item($i, [ref]$device) | Out-Null
            if ($device)
            {
                $id = ""
                $device.GetId([ref]$id) | Out-Null
                if ($id -like "*Leyline*" -or $id -like "*leyline*")
                {
                    $leylineInCapture = $true
                    Write-Diag -Message "Found Leyline in Capture: $id" -Color Green -Prefix "OK"
                }
            }
        } catch
        {
        }
    }

    if (-not $leylineInRender -and -not $leylineInCapture)
    {
        Write-Diag -Message "Leyline endpoints NOT found in audio enumeration!" -Color Red -Prefix "FAIL"
        Write-Diag -Message "This indicates WaveRT miniport registration issue" -Color Yellow
    }

} catch
{
    Write-Diag -Message "Could not enumerate audio endpoints via MMDevice API" -Color Red
    Write-Diag -Message "Error: $_" -Color Gray
}

# 4. KS (Kernel Streaming) Device Check
Write-Section "4. Kernel Streaming (KS) Devices"

$ksDevices = Get-PnpDevice -Class MEDIA | Where-Object {
    $_.FriendlyName -like "*Leyline*" -or
    $_.InstanceId -like "*Leyline*"
}

if ($ksDevices)
{
    foreach ($dev in $ksDevices)
    {
        Write-Diag -Message "KS Device: $($dev.FriendlyName)" -Color Green -Prefix "OK"
        Write-Diag -Message "  Status: $($dev.Status)" -Color $(if($dev.Status -eq "OK")
            {"Green"
            } else
            {"Red"
            })
        Write-Diag -Message "  Class: $($dev.Class)" -Color Gray

        # Check if audio device properties exist
        $props = Get-PnpDeviceProperty -InstanceId $dev.InstanceId -ErrorAction SilentlyContinue
        $audioProps = $props | Where-Object { $_.KeyName -like "*Audio*" -or $_.KeyName -like "*Endpoint*" }
        if ($audioProps)
        {
            Write-Diag -Message "  Audio Properties Found: $($audioProps.Count)" -Color Green
        } else
        {
            Write-Diag -Message "  No Audio Properties (may indicate missing WaveRT registration)" -Color Yellow
        }
    }
} else
{
    Write-Diag -Message "No Leyline KS devices in MEDIA class!" -Color Red -Prefix "FAIL"
}

# 5. Registry Audio Endpoint Configuration
Write-Section "5. Audio Endpoint Registry Configuration"

$audioEndpointsReg = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render"
if (Test-Path $audioEndpointsReg)
{
    $renderEndpoints = Get-ChildItem $audioEndpointsReg -ErrorAction SilentlyContinue
    $leylineEndpoints = $renderEndpoints | Where-Object {
        $props = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
        $props.DeviceDesc -like "*Leyline*" -or $props.FriendlyName -like "*Leyline*"
    }

    if ($leylineEndpoints)
    {
        Write-Diag -Message "Found $($leylineEndpoints.Count) Leyline render endpoint(s) in registry" -Color Green -Prefix "OK"
    } else
    {
        Write-Diag -Message "No Leyline render endpoints in MMDevices registry" -Color Red -Prefix "FAIL"
    }
} else
{
    Write-Diag -Message "MMDevices registry path not found!" -Color Red
}

# 6. Driver File Verification
Write-Section "6. Driver File Verification"

$driverFiles = @{
    "Driver SYS" = "C:\Windows\System32\drivers\leyline.sys"
    "APO DLL" = "C:\Windows\System32\LeylineAPO.dll"
}

foreach ($fileType in $driverFiles.Keys)
{
    $path = $driverFiles[$fileType]
    if (Test-Path $path)
    {
        $info = Get-Item $path
        Write-Diag -Message "$fileType`: FOUND" -Color Green -Prefix "OK"
        Write-Diag -Message "  Path: $path" -Color Gray
        Write-Diag -Message "  Modified: $($info.LastWriteTime)" -Color Gray
        Write-Diag -Message "  Size: $([math]::Round($info.Length/1KB, 2)) KB" -Color Gray
    } else
    {
        Write-Diag -Message "$fileType`: NOT FOUND at $path" -Color Yellow
    }
}

# 7. Event Log Analysis
Write-Section "7. Recent Audio-Related Events"

$startTime = (Get-Date).AddHours(-1)
$audioEvents = Get-WinEvent -FilterHashtable @{
    LogName = 'System'
    StartTime = $startTime
    Level = 1,2,3  # Critical, Error, Warning
} -ErrorAction SilentlyContinue | Where-Object {
    $_.Message -like "*Leyline*" -or
    $_.Message -like "*audio*" -or
    $_.Message -like "*PortCls*" -or
    $_.Message -like "*WaveRT*"
} | Select-Object -First 5

if ($audioEvents)
{
    Write-Diag -Message "Recent audio-related events:" -Color Yellow
    foreach ($evt in $audioEvents)
    {
        $color = switch ($_.Level)
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
        Write-Diag -Message "[$($evt.TimeCreated.ToString('HH:mm:ss'))] $($evt.LevelDisplayName): $($evt.Message.Substring(0,[Math]::Min(80,$evt.Message.Length)))..." -Color $color
    }
} else
{
    Write-Diag -Message "No critical audio events in last hour" -Color Green -Prefix "OK"
}

# 8. Recommendations
Write-Section "DIAGNOSTIC SUMMARY & RECOMMENDATIONS"

Write-Diag -Message "Based on diagnostic results:" -Color Cyan

$issuesFound = @()

# Check for specific issues
if (-not $leylineInRender -and -not $leylineInCapture)
{
    $issuesFound += "Endpoints not enumerated by Windows Audio subsystem"
}
if ($audioServices | Where-Object { (Get-Service $_.Name -EA SilentlyContinue).Status -ne "Running" })
{
    $issuesFound += "Audio services not running"
}

if ($issuesFound.Count -eq 0)
{
    Write-Diag -Message "✅ No critical issues detected!" -Color Green
    Write-Diag -Message "If endpoints still don't appear:" -Color White
    Write-Diag -Message "  1. Open Sound Control Panel: mmsys.cpl" -Color Yellow
    Write-Diag -Message "  2. Right-click in empty area → Show Disabled Devices" -Color Yellow
    Write-Diag -Message "  3. Check if Leyline endpoints are disabled" -Color Yellow
} else
{
    Write-Diag -Message "❌ Issues Found:" -Color Red
    foreach ($issue in $issuesFound)
    {
        Write-Diag -Message "  • $issue" -Color Red
    }

    Write-Diag -Message "`nRecommended Actions:" -Color Cyan
    Write-Diag -Message "  1. Restart Windows Audio service:" -Color White
    Write-Diag -Message "     Restart-Service Audiosrv -Force" -Color Yellow
    Write-Diag -Message "  2. Check Device Manager for disabled devices" -Color White
    Write-Diag -Message "  3. Verify WaveRT miniport is registered in driver code" -Color White
    Write-Diag -Message "     Check adapter.rs for PcRegisterSubdevice calls" -Color Yellow
}

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Diag -Message "Diagnostic complete" -Color Green
Write-Host "========================================`n" -ForegroundColor Cyan
