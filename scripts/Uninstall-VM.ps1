# Leyline Audio: VM REMOTE UNINSTALLER
# Cleans up Leyline artifacts from a target VM.

param (
    [string]$VMName = "LeylineTestVM",
    [string]$UserName = "USER"
)

$ErrorActionPreference = "Stop"

# Create credential object for blank password
$secPassword = ConvertTo-SecureString "REDACTED_VM_PASS" -AsPlainText -Force
$cred = New-Object System.Management.Automation.PSCredential ($UserName, $secPassword)

$vmsess = $null

try
{
    Write-Host "[*] Targeting VM: $VMName for Purge..." -ForegroundColor Red

    $vmsess = New-PSSession -VMName $VMName -Credential $cred

    Invoke-Command -Session $vmsess -ScriptBlock {
        Write-Host "[VM] Removing PnP Devices..."
        $ids = pnputil /enum-devices /class MEDIA | Select-String "Instance ID:\s+(.*)" | ForEach-Object { $ms = $_.Matches[0].Groups[1].Value.Trim(); $ms }
        # Note: This is a broad filter, normally we'd check FriendlyName but pnputil output is multi-line
        # Better approach:
        Get-PnpDevice -PresentOnly:$false | Where-Object { $_.HardwareID -contains "Root\LeylineAudio" } | ForEach-Object {
            Write-Host "[VM] Removing $($_.InstanceId)"
            pnputil /remove-device $_.InstanceId | Out-Null
        }

        Write-Host "[VM] Deleting Services..."
        foreach ($svc in @("Leyline", "LEYLINEAUDIO"))
        {
            if (Get-Service $svc -ErrorAction SilentlyContinue)
            {
                sc.exe stop $svc | Out-Null
                sc.exe delete $svc | Out-Null
            }
        }

        Write-Host "[VM] Purging Driver Store..."
        $drivers = pnputil /enum-drivers
        for ($i = 0; $i -lt $drivers.Count; $i++)
        {
            if ($drivers[$i] -match "Original Name:\s+leyline\.inf")
            {
                if ($drivers[$i-1] -match "Published Name:\s+(oem\d+\.inf)")
                {
                    $inf = $matches[1]
                    Write-Host "[VM] Deleting $inf"
                    pnputil /delete-driver $inf /force | Out-Null
                }
            }
        }

        Write-Host "[VM] Removing Certificates..."
        Get-ChildItem Cert:\LocalMachine\Root, Cert:\LocalMachine\TrustedPublisher | Where-Object { $_.Subject -like "*Leyline*" } | ForEach-Object {
            Remove-Item $_.PSPath -Force
        }

        if (Test-Path "C:\LeylineInstall")
        { Remove-Item "C:\LeylineInstall" -Recurse -Force
        }
    }

    Write-Host "[SUCCESS] VM $VMName has been scrubbed." -ForegroundColor Green
} catch
{
    Write-Error "Failed to uninstall from VM: $($_.Exception.Message)"
} finally
{
    if ($vmsess)
    { Remove-PSSession $vmsess
    }
}
