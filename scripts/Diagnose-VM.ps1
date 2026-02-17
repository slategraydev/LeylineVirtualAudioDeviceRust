#Requires -RunAsAdministrator

<#
.SYNOPSIS
    Remote diagnostic tool for Leyline Audio Driver VM testing.
.DESCRIPTION
    Connects to a VM via PowerShell Direct and runs comprehensive diagnostics
    to determine why audio endpoints aren't appearing. Returns results to host console.
.PARAMETER VMName
    Name of the VM to diagnose (default: LeylineTestVM)
.PARAMETER UserName
    Username for VM connection (default: USER)
.EXAMPLE
    .\Diagnose-VM.ps1 -VMName "LeylineTestVM"
#>

param (
    [string]$VMName = "LeylineTestVM",
    [string]$UserName = "USER"
)

$ErrorActionPreference = "Stop"
$initialDir = Get-Location

# Create credential object for blank password
$secPassword = ConvertTo-SecureString "REDACTED_VM_PASS" -AsPlainText -Force
$cred = New-Object System.Management.Automation.PSCredential ($UserName, $secPassword)

$vmsess = $null

try
{
    Write-Host "`n╔════════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║    LEYLINE VM REMOTE DIAGNOSTIC                 ║" -ForegroundColor Cyan
    Write-Host "╚════════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""

    # Check VM Status
    Write-Host "[*] Checking VM: $VMName..." -ForegroundColor Cyan
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if (-not $vm)
    { throw "VM '$VMName' not found."
    }
    if ($vm.State -ne 'Running')
    { throw "VM '$VMName' is not running. Current state: $($vm.State)"
    }
    Write-Host "    -> VM is running" -ForegroundColor Green

    # Create Session
    $vmsess = New-PSSession -VMName $VMName -Credential $cred
    Write-Host "    -> Connected to VM session" -ForegroundColor Green

    # Run Diagnostics
    Write-Host "`n--- [Running Remote Diagnostics] ---" -ForegroundColor Cyan

    $results = Invoke-Command -Session $vmsess -ScriptBlock {
        $output = @()

        # Helper function to collect output
        function Add-Output($Section, $Status, $Details)
        {
            return [PSCustomObject]@{
                Section = $Section
                Status = $Status
                Details = $Details
            }
        }

        # 1. Check Device Status
        $leylineDevices = Get-PnpDevice -PresentOnly:$true | Where-Object {
            $_.FriendlyName -like "*Leyline*" -or
            $_.HardwareID -contains "Root\LeylineAudio"
        }

        if ($leylineDevices)
        {
            foreach ($dev in $leylineDevices)
            {
                # Get detailed properties
                $props = Get-PnpDeviceProperty -InstanceId $dev.InstanceId -ErrorAction SilentlyContinue
                $capabilities = ($props | Where-Object { $_.KeyName -eq "DEVPKEY_Device_Capabilities" }).Data
                $configFlags = ($props | Where-Object { $_.KeyName -eq "DEVPKEY_Device_ConfigFlags" }).Data
                $problemCode = ($props | Where-Object { $_.KeyName -eq "DEVPKEY_Device_ProblemCode" }).Data

                $details = "InstanceId: $($dev.InstanceId)`n"
                $details += "HardwareID: $($dev.HardwareID -join ', ')`n"
                $details += "Status: $($dev.Status)`n"
                if ($capabilities -ne $null)
                { $details += "Capabilities: 0x$($capabilities.ToString('X8'))`n" 
                }
                if ($configFlags -ne $null)
                { $details += "ConfigFlags: 0x$($configFlags.ToString('X8'))`n" 
                }
                if ($problemCode)
                { $details += "ProblemCode: $problemCode (ERROR)`n" 
                }

                $audioProps = $props | Where-Object { $_.KeyName -like "*Audio*" -or $_.KeyName -like "*Endpoint*" -or $_.KeyName -like "*FX*" }
                if ($audioProps)
                {
                    $details += "Audio Properties Found: $($audioProps.Count)`n"
                } else
                {
                    $details += "WARNING: No Audio/Endpoint Properties Found`n"
                }

                $output += Add-Output -Section "Device Status" -Status $(if($dev.Status -eq "OK")
                    {"OK"
                    } else
                    {"FAIL"
                    }) -Details $details
            }
        } else
        {
            $output += Add-Output -Section "Device Status" -Status "FAIL" -Details "No Leyline devices found in PnP"
        }

        # 2. Check Audio Services
        $services = @(
            @{Name="Audiosrv"; Display="Windows Audio"},
            @{Name="AudioEndpointBuilder"; Display="Audio Endpoint Builder"},
            @{Name="PlugPlay"; Display="Plug and Play"}
        )

        foreach ($svcInfo in $services)
        {
            $svc = Get-Service -Name $svcInfo.Name -ErrorAction SilentlyContinue
            if ($svc)
            {
                $status = if ($svc.Status -eq "Running")
                { "OK" 
                } else
                { "FAIL" 
                }
                $output += Add-Output -Section "Service: $($svcInfo.Display)" -Status $status -Details "Status: $($svc.Status), StartType: $($svc.StartType)"
            } else
            {
                $output += Add-Output -Section "Service: $($svcInfo.Display)" -Status "FAIL" -Details "Service not found"
            }
        }

        # 3. Check MMDevices Registry
        $renderPath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render"
        $capturePath = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture"

        $renderCount = 0
        $captureCount = 0
        $leylineInRender = $false
        $leylineInCapture = $false

        if (Test-Path $renderPath)
        {
            $renderDevices = Get-ChildItem $renderPath -ErrorAction SilentlyContinue
            $renderCount = $renderDevices.Count
            $leylineRender = $renderDevices | Where-Object {
                $props = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
                $props.DeviceDesc -like "*Leyline*" -or $props.FriendlyName -like "*Leyline*"
            }
            if ($leylineRender)
            { $leylineInRender = $true 
            }
        }

        if (Test-Path $capturePath)
        {
            $captureDevices = Get-ChildItem $capturePath -ErrorAction SilentlyContinue
            $captureCount = $captureDevices.Count
            $leylineCapture = $captureDevices | Where-Object {
                $props = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
                $props.DeviceDesc -like "*Leyline*" -or $props.FriendlyName -like "*Leyline*"
            }
            if ($leylineCapture)
            { $leylineInCapture = $true 
            }
        }

        $details = "Total Render Endpoints: $renderCount`n"
        $details += "Total Capture Endpoints: $captureCount`n"
        $details += "Leyline in Render: $leylineInRender`n"
        $details += "Leyline in Capture: $leylineInCapture`n"
        if (-not $leylineInRender -and -not $leylineInCapture)
        {
            $details += "CRITICAL: Leyline not found in MMDevices registry`n"
        }

        $status = if ($leylineInRender -or $leylineInCapture)
        { "OK" 
        } else
        { "FAIL" 
        }
        $output += Add-Output -Section "MMDevices Registry" -Status $status -Details $details

        # 4. Check Audio Endpoints via MMDevice API
        try
        {
            Add-Type -TypeDefinition @"
                using System;
                using System.Runtime.InteropServices;
                [Guid("BCDE0395-E52F-467C-8E3D-C4579291692E"), ComImport]
                public class MMDeviceEnumerator { }
                [Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
                public interface IMMDeviceEnumerator {
                    [PreserveSig] int EnumAudioEndpoints(int dataFlow, int dwStateMask, out IMMDeviceCollection ppDevices);
                }
                [Guid("0BD7A1BE-7A1A-44DB-8397-CC5392387B5E"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
                public interface IMMDeviceCollection {
                    [PreserveSig] int GetCount(out int pcDevices);
                    [PreserveSig] int Item(int nDevice, out IMMDevice ppDevice);
                }
                [Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
                public interface IMMDevice {
                    [PreserveSig] int GetId([MarshalAs(UnmanagedType.LPWStr)] out string ppstrId);
                }
"@ -ErrorAction SilentlyContinue

            $enumeratorType = [Type]::GetTypeFromCLSID([Guid]"BCDE0395-E52F-467C-8E3D-C4579291692E")
            $enumerator = [Activator]::CreateInstance($enumeratorType)

            $renderCollection = $null
            $captureCollection = $null
            $enumerator.EnumAudioEndpoints(0, 1, [ref]$renderCollection) | Out-Null
            $enumerator.EnumAudioEndpoints(1, 1, [ref]$captureCollection) | Out-Null

            $renderCount = 0
            $captureCount = 0
            if ($renderCollection)
            { $renderCollection.GetCount([ref]$renderCount) | Out-Null 
            }
            if ($captureCollection)
            { $captureCollection.GetCount([ref]$captureCount) | Out-Null 
            }

            $leylineRenderEP = $false
            $leylineCaptureEP = $false

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
                        { $leylineRenderEP = $true 
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
                        { $leylineCaptureEP = $true 
                        }
                    }
                } catch
                {
                }
            }

            $details = "Active Render Endpoints: $renderCount`n"
            $details += "Active Capture Endpoints: $captureCount`n"
            $details += "Leyline in Render Endpoints: $leylineRenderEP`n"
            $details += "Leyline in Capture Endpoints: $leylineCaptureEP`n"

            $status = if ($leylineRenderEP -or $leylineCaptureEP)
            { "OK" 
            } else
            { "FAIL" 
            }
            $output += Add-Output -Section "MMDevice API" -Status $status -Details $details
        } catch
        {
            $output += Add-Output -Section "MMDevice API" -Status "FAIL" -Details "Could not enumerate: $_"
        }

        # 5. Check Driver Files
        $driverFiles = @(
            "C:\Windows\System32\drivers\leyline.sys",
            "C:\Windows\System32\LeylineAPO.dll"
        )

        foreach ($file in $driverFiles)
        {
            if (Test-Path $file)
            {
                $info = Get-Item $file
                $details = "Found: $file`n"
                $details += "Modified: $($info.LastWriteTime)`n"
                $details += "Size: $([math]::Round($info.Length/1KB, 2)) KB"
                $output += Add-Output -Section "Driver File: $(Split-Path $file -Leaf)" -Status "OK" -Details $details
            } else
            {
                $output += Add-Output -Section "Driver File: $(Split-Path $file -Leaf)" -Status "WARN" -Details "Not found at $file"
            }
        }

        # 6. Check for Audio-Related Events
        $startTime = (Get-Date).AddHours(-1)
        $audioEvents = Get-WinEvent -FilterHashtable @{
            LogName = 'System'
            StartTime = $startTime
            Level = 1,2,3
        } -ErrorAction SilentlyContinue | Where-Object {
            $_.Message -like "*Leyline*" -or
            $_.Message -like "*audio*" -or
            $_.Message -like "*PortCls*" -or
            $_.Message -like "*WaveRT*"
        } | Select-Object -First 3

        if ($audioEvents)
        {
            $details = ($audioEvents | ForEach-Object { "[$($_.LevelDisplayName)] $($_.Message.Substring(0,[Math]::Min(60,$_.Message.Length)))..." }) -join "`n"
            $output += Add-Output -Section "Recent Audio Events" -Status "INFO" -Details $details
        }

        return $output
    }

    # Display Results
    Write-Host "`n========================================" -ForegroundColor Cyan
    Write-Host " DIAGNOSTIC RESULTS" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan

    $hasErrors = $false

    foreach ($result in $results)
    {
        $color = switch ($result.Status)
        {
            "OK"
            { "Green" 
            }
            "FAIL"
            { "Red"; $hasErrors = $true 
            }
            "WARN"
            { "Yellow" 
            }
            "INFO"
            { "Gray" 
            }
            default
            { "White" 
            }
        }

        Write-Host "`n[$($result.Section)]" -ForegroundColor Cyan
        Write-Host "Status: $($result.Status)" -ForegroundColor $color
        Write-Host $result.Details -ForegroundColor White
    }

    Write-Host "`n========================================" -ForegroundColor Cyan

    if ($hasErrors)
    {
        Write-Host " DIAGNOSTIC COMPLETE - ISSUES FOUND" -ForegroundColor Red
        Write-Host "========================================`n" -ForegroundColor Cyan
        Write-Host "RECOMMENDATIONS:" -ForegroundColor Yellow
        Write-Host "  1. Check if driver is properly signed" -ForegroundColor White
        Write-Host "  2. Verify Windows Audio services are running" -ForegroundColor White
        Write-Host "  3. Check MMDevices registry for Leyline entries" -ForegroundColor White
        Write-Host "  4. Run DebugView to capture kernel DbgPrint" -ForegroundColor White
        Write-Host "  5. Try -UseRootMedia switch for Root\Media enumeration" -ForegroundColor White
    } else
    {
        Write-Host " DIAGNOSTIC COMPLETE - ALL CHECKS PASSED" -ForegroundColor Green
        Write-Host "========================================`n" -ForegroundColor Cyan
    }

} finally
{
    if ($vmsess)
    {
        Remove-PSSession $vmsess
    }
    Set-Location $initialDir
}
