#Requires -RunAsAdministrator

<#
.SYNOPSIS
    System-Only Uninstall: Removes all Leyline Audio Driver artifacts from the machine.
.DESCRIPTION
    Aggressive system cleanup for Leyline driver. Handles multiple device instances,
    "marked for deletion" services, and deep registry/file cleanup.
    Does NOT touch project files - use Install.ps1 -clean for that.
    REBOOT RECOMMENDED after running if driver was previously loaded.
.NOTES
    Must be run as Administrator.
#>

param(
    [switch]$AutoReboot,
    [switch]$WhatIf
)

$ErrorActionPreference = "Continue"

$initialDir = Get-Location
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."

function Write-Uninstall
{
    param(
        [string]$Message,
        [ConsoleColor]$Color = "Yellow"
    )
    Write-Host "[UNINSTALL] $Message" -ForegroundColor $Color
}

function Write-Divider
{
    Write-Host "`n========================================" -ForegroundColor Cyan
}

try
{
    Push-Location $ProjectRoot

    # 0. Administrator Guard
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))
    {
        throw "This script MUST be run as Administrator."
    }

    Write-Host "`n╔════════════════════════════════════════════════╗" -ForegroundColor Yellow
    Write-Host "║     LEYLINE AUDIO DRIVER - SYSTEM UNINSTALL    ║" -ForegroundColor Yellow
    Write-Host "╚════════════════════════════════════════════════╝" -ForegroundColor Yellow

    if ($WhatIf)
    {
        Write-Host "`n[WHATIF MODE] Previewing what would be removed...`n" -ForegroundColor Magenta
    }

    Write-Divider
    Write-Uninstall "STEP 1/7: Stopping Audio Services" -Color Cyan

    # Stop all audio-related services first
    # Use sc.exe query to check existence first (more reliable than Get-Service)
    $audioServices = @("Audiosrv", "AudioEndpointBuilder", "Leyline", "LEYLINEAUDIO")
    foreach ($svcName in $audioServices)
    {
        # Check if service exists using sc.exe (avoids Get-Service errors)
        $svcCheck = sc.exe query $svcName 2>&1
        $svcExists = $svcCheck -notmatch "FAILED 1060"

        if ($svcExists)
        {
            try
            {
                $svc = Get-Service -Name $svcName -ErrorAction Stop
                if ($svc.Status -eq "Running")
                {
                    Write-Uninstall "  Stopping service: $svcName" -Color Yellow
                    if (-not $WhatIf)
                    {
                        Stop-Service -Name $svcName -Force -ErrorAction SilentlyContinue
                    }
                }
            } catch
            {
                # Service registry might be corrupted, try sc.exe stop directly
                Write-Uninstall "  Service $svcName registry issue, using sc.exe" -Color Yellow
                if (-not $WhatIf)
                {
                    sc.exe stop $svcName 2>&1 | Out-Null
                }
            }
        }
    }

    # Wait a moment for services to release handles
    if (-not $WhatIf)
    {
        Start-Sleep -Seconds 2
    }

    Write-Divider
    Write-Uninstall "STEP 2/7: Removing All Device Instances" -Color Cyan

    # Find ALL Leyline-related devices (including multiple instances and SWD\DEVGEN)
    $allDevices = Get-PnpDevice -PresentOnly:$false | Where-Object {
        $_.FriendlyName -like "*Leyline*" -or
        $_.FriendlyName -like "*Generic software device*" -or
        $_.InstanceId -like "*LEYLINE*" -or
        $_.InstanceId -like "*Leyline*" -or
        $_.HardwareID -contains "Root\LeylineAudio" -or
        $_.InstanceId -like "SWD\DEVGEN*"
    }

    # Also catch legacy simpleaudiosample devices
    $legacyDevices = Get-PnpDevice -PresentOnly:$false | Where-Object {
        $_.FriendlyName -like "*simpleaudiosample*" -or
        $_.FriendlyName -like "*SimpleAudioSample*" -or
        $_.HardwareID -contains "Root\simpleaudiosample"
    }

    # Also catch SWD\DEVGEN devices (created by devgen.exe)
    $devgenDevices = Get-PnpDevice -PresentOnly:$false | Where-Object {
        $_.InstanceId -like "SWD\DEVGEN*" -and
        ($_.FriendlyName -like "*Leyline*" -or $_.FriendlyName -like "*Generic software device*")
    }

    $allDevices = @($allDevices) + @($legacyDevices) + @($devgenDevices) | Select-Object -Unique

    Write-Uninstall "  Found $($allDevices.Count) device instance(s) to remove" -Color Yellow

    foreach ($dev in $allDevices)
    {
        Write-Uninstall "  Removing: $($dev.InstanceId)" -Color Yellow
        Write-Uninstall "    Name: $($dev.FriendlyName) [Status: $($dev.Status)]" -Color Gray

        if (-not $WhatIf)
        {
            # Try devcon first (most reliable for root-enumerated devices)
            $devconPaths = @(
                "D:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe",
                "C:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe",
                "$env:SystemDrive\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe"
            )

            $devconFound = $null
            foreach ($path in $devconPaths)
            {
                if (Test-Path $path)
                {
                    $devconFound = $path
                    break
                }
            }

            if ($devconFound)
            {
                & $devconFound remove "@$($dev.InstanceId)" 2>&1 | Out-Null
            }

            # Fallback to pnputil
            pnputil /remove-device $dev.InstanceId 2>&1 | Out-Null

            # Last resort: WMI force removal
            try
            {
                $escapedId = $dev.InstanceId -replace '\\', '\\'
                $wmiQuery = "SELECT * FROM Win32_PnPEntity WHERE DeviceID = '$escapedId'"
                $wmiDev = Get-WmiObject -Query $wmiQuery -ErrorAction SilentlyContinue
                if ($wmiDev)
                {
                    $wmiDev.Delete() | Out-Null
                }
            } catch
            {
                # WMI removal failed, continue
            }
        }
    }

    # Also remove orphaned SWD\DEVGEN devices that might not have Leyline in the name
    $orphanedDevGen = Get-PnpDevice -PresentOnly:$false | Where-Object {
        $_.InstanceId -like "SWD\DEVGEN*" -and $_.FriendlyName -eq "Generic software device"
    }
    foreach ($dev in $orphanedDevGen)
    {
        Write-Uninstall "  Removing orphaned DEVGEN device: $($dev.InstanceId)" -Color Yellow
        if (-not $WhatIf)
        {
            pnputil /remove-device $dev.InstanceId 2>&1 | Out-Null
        }
    }

    Write-Divider
    Write-Uninstall "STEP 3/7: Purging Driver Store (All Leyline INFs)" -Color Cyan

    # Get all driver packages and find Leyline-related ones
    $driverOutput = pnputil /enum-drivers | Out-String
    $driverLines = $driverOutput -split "`r?`n"

    $oemInfsToDelete = @()
    $currentOem = ""

    foreach ($line in $driverLines)
    {
        if ($line -match "Published Name:\s+(oem\d+\.inf)")
        {
            $currentOem = $matches[1]
        } elseif ($line -match "Original Name:\s+(leyline\.inf|simpleaudiosample\.inf)")
        {
            if ($currentOem)
            {
                $oemInfsToDelete += $currentOem
                Write-Uninstall "  Found driver: $currentOem ($($matches[1]))" -Color Yellow
            }
        } elseif ($line -match "Provider Name:\s+(Leyline|SimpleAudioSample)")
        {
            if ($currentOem -and $oemInfsToDelete -notcontains $currentOem)
            {
                $oemInfsToDelete += $currentOem
                Write-Uninstall "  Found driver by provider: $currentOem" -Color Yellow
            }
        }
    }

    # Remove unique INF packages with force and uninstall flags
    foreach ($inf in ($oemInfsToDelete | Select-Object -Unique))
    {
        Write-Uninstall "  Deleting driver package: $inf" -Color Yellow
        if (-not $WhatIf)
        {
            # Use /force /uninstall to completely remove from driver store and uninstall from devices
            pnputil /delete-driver $inf /force /uninstall 2>&1 | Out-Null
        }
    }

    Write-Divider
    Write-Uninstall "STEP 4/7: Deleting Services (including 'marked for deletion')" -Color Cyan

    $servicesToKill = @("Leyline", "LEYLINEAUDIO", "LeylineAudio", "SimpleAudioSample")
    foreach ($svcName in $servicesToKill)
    {
        $svcExists = sc.exe query $svcName 2>&1
        if ($svcExists -notmatch "FAILED 1060")
        {
            Write-Uninstall "  Stopping and deleting service: $svcName" -Color Yellow

            if (-not $WhatIf)
            {
                # Stop the service
                sc.exe stop $svcName 2>&1 | Out-Null

                # Delete the service
                $deleteResult = sc.exe delete $svcName 2>&1
                if ($deleteResult -match "marked for deletion")
                {
                    Write-Uninstall "    Note: Service marked for deletion (will clear on reboot)" -Color Magenta
                }
            }
        }
    }

    Write-Divider
    Write-Uninstall "STEP 5/7: Registry Cleanup" -Color Cyan

    $registryPaths = @(
        "HKLM:\SYSTEM\CurrentControlSet\Services\Leyline",
        "HKLM:\SYSTEM\CurrentControlSet\Services\LEYLINEAUDIO",
        "HKLM:\SYSTEM\CurrentControlSet\Services\SimpleAudioSample",
        "HKLM:\SOFTWARE\Leyline",
        "HKLM:\SOFTWARE\LeylineAudio",
        "HKLM:\SOFTWARE\SimpleAudioSample",
        "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Audio\Leyline*",
        "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Audio\SimpleAudioSample*"
    )

    foreach ($regPath in $registryPaths)
    {
        if (Test-Path $regPath)
        {
            Write-Uninstall "  Deleting registry: $regPath" -Color Yellow
            if (-not $WhatIf)
            {
                Remove-Item -Path $regPath -Recurse -Force -ErrorAction SilentlyContinue
            }
        }
    }

    # Clean CriticalDeviceDatabase
    if (-not $WhatIf)
    {
        $cddPath = "HKLM:\SYSTEM\CurrentControlSet\Control\CriticalDeviceDatabase"
        Get-ChildItem $cddPath -ErrorAction SilentlyContinue | Where-Object {
            $_.Name -like "*Leyline*" -or $_.Name -like "*simpleaudiosample*"
        } | ForEach-Object {
            Write-Uninstall "  Deleting CriticalDevice entry: $($_.Name)" -Color Yellow
            Remove-Item -Path $_.PSPath -Recurse -Force -ErrorAction SilentlyContinue
        }
    }

    Write-Divider
    Write-Uninstall "STEP 6/7: System File Cleanup" -Color Cyan

    # Only delete files from Windows system directories (not project files)
    $systemFilesToDelete = @(
        "C:\Windows\System32\drivers\leyline.sys",
        "C:\Windows\System32\drivers\LeylineAudio.sys",
        "C:\Windows\System32\drivers\simpleaudiosample.sys",
        "C:\Windows\System32\LeylineAPO.dll",
        "C:\Windows\SysWOW64\LeylineAPO.dll",
        "C:\Windows\System32\LeylineHSA.exe"
    )

    foreach ($file in $systemFilesToDelete)
    {
        if (Test-Path $file)
        {
            Write-Uninstall "  Deleting: $file" -Color Yellow
            if (-not $WhatIf)
            {
                # Take ownership and grant permissions before deleting
                takeown /f $file 2>&1 | Out-Null
                icacls $file /grant administrators:F 2>&1 | Out-Null
                Remove-Item $file -Force -ErrorAction SilentlyContinue
            }
        }
    }

    Write-Divider
    Write-Uninstall "STEP 7/7: Certificate Cleanup" -Color Cyan

    $certStores = @(
        "Cert:\CurrentUser\My",
        "Cert:\CurrentUser\Root",
        "Cert:\CurrentUser\TrustedPublisher",
        "Cert:\LocalMachine\My",
        "Cert:\LocalMachine\Root",
        "Cert:\LocalMachine\TrustedPublisher"
    )

    foreach ($store in $certStores)
    {
        $certs = Get-ChildItem $store -ErrorAction SilentlyContinue | Where-Object {
            $_.Subject -like "*Leyline*" -or
            $_.FriendlyName -like "*Leyline*" -or
            $_.Subject -like "*SimpleAudioSample*" -or
            $_.FriendlyName -like "*SimpleAudioSample*"
        }

        foreach ($cert in $certs)
        {
            Write-Uninstall "  Removing certificate: $($cert.Thumbprint)" -Color Yellow
            if (-not $WhatIf)
            {
                # Root store requires certutil (PowerShell Remove-Item triggers UI restriction)
                if ($store -match "Root")
                {
                    $storeName = "ROOT"
                    certutil -delstore $storeName "$($cert.Thumbprint)" 2>&1 | Out-Null
                } else
                {
                    Remove-Item $cert.PSPath -Force -ErrorAction SilentlyContinue
                }
            }
        }
    }

    Write-Divider

    # Restart Windows Audio services
    if (-not $WhatIf)
    {
        Write-Uninstall "Restarting Windows Audio services..." -Color Green
        foreach ($svcName in @("Audiosrv", "AudioEndpointBuilder"))
        {
            $svc = Get-Service -Name $svcName -ErrorAction SilentlyContinue
            if ($svc)
            {
                Set-Service -Name $svcName -StartupType Automatic -ErrorAction SilentlyContinue
                Start-Service -Name $svcName -ErrorAction SilentlyContinue
            }
        }
    }

    Write-Host "`n╔════════════════════════════════════════════════╗" -ForegroundColor Green
    Write-Host "║         SYSTEM UNINSTALL COMPLETE              ║" -ForegroundColor Green
    Write-Host "╚════════════════════════════════════════════════╝" -ForegroundColor Green

    if ($WhatIf)
    {
        Write-Host "`n[WHATIF MODE] No changes were made." -ForegroundColor Magenta
        Write-Host "Run without -WhatIf to execute the uninstall." -ForegroundColor Yellow
    } else
    {
        Write-Host "`n✅ All Leyline driver artifacts removed from system." -ForegroundColor Green

        Write-Host "`n⚠️  IMPORTANT: If you saw 'marked for deletion' messages," -ForegroundColor Yellow
        Write-Host "    a REBOOT is required before reinstalling the driver." -ForegroundColor Yellow

        Write-Host "`nNext steps:" -Color Cyan
        Write-Host "  1. (Optional) Reboot if 'marked for deletion' was reported" -ForegroundColor White
        Write-Host "  2. Run: .\\scripts\\Test-HostReady.ps1" -ForegroundColor White
        Write-Host "  3. Run: .\\scripts\\Install.ps1" -ForegroundColor White

        if ($AutoReboot)
        {
            Write-Host "`n🔄 Auto-rebooting in 10 seconds..." -ForegroundColor Magenta
            Start-Sleep -Seconds 10
            Restart-Computer -Force
        }
    }

} finally
{
    Set-Location $initialDir
}
