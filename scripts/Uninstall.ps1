# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# LEYLINE ACX UNINSTALLER
# Removes the Leyline driver from TestVM via PowerShell Direct.
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

param (
    [PSCredential]$Credential
)

$ErrorActionPreference = "Stop"
$VMName       = "TestVM"
$remotePath   = "C:\LeylineInstall"

# VM Credentials
if (-not $PSBoundParameters.ContainsKey('Credential')) {
    $VMUser = if ($env:LEYLINE_VM_USER) { $env:LEYLINE_VM_USER } else { "USER" }
    $VMPassword = if ($env:LEYLINE_VM_PASS) { $env:LEYLINE_VM_PASS } else { "rd" }
    $secPassword = ConvertTo-SecureString $VMPassword -AsPlainText -Force
    $Credential = New-Object System.Management.Automation.PSCredential ($VMUser, $secPassword)
}

Write-Host "[*] Uninstalling Leyline from '$VMName'..." -ForegroundColor Cyan

try {
    $vmsess = New-PSSession -VMName $VMName -Credential $Credential

    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path)

        Write-Host "    [VM] Removing Leyline device node..."
        # Remove the device node
        pnputil /remove-device "ROOT\MEDIA\LeylineAudio" /force 2>&1 | Out-Null

        # Remove all Leyline driver packages
        Write-Host "    [VM] Removing Leyline driver packages..."
        $driverOutput = pnputil /enum-drivers 2>&1
        $lines = $driverOutput -split "`n"
        for ($i = 0; $i -lt $lines.Count; $i++) {
            if ($lines[$i] -match "leyline") {
                # Walk backwards to find the "Published Name" line
                for ($j = $i; $j -ge [Math]::Max(0, $i - 5); $j--) {
                    if ($lines[$j] -match "(oem\d+\.inf)") {
                        $oemInf = $matches[1]
                        Write-Host "    [VM] Removing driver package: $oemInf"
                        pnputil /delete-driver $oemInf /force 2>&1 | Out-Null
                        break
                    }
                }
            }
        }

        # Restart audio services
        Write-Host "    [VM] Restarting audio services..."
        Restart-Service "AudioEndpointBuilder" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 1
        Restart-Service "Audiosrv" -Force -ErrorAction SilentlyContinue

        # Clean up install directory
        if (Test-Path $path) {
            Write-Host "    [VM] Cleaning install directory..."
            Remove-Item $path -Recurse -Force
        }

        Write-Host "    [VM] Uninstall complete." -ForegroundColor Green
    } -ArgumentList $remotePath
}
catch {
    Write-Error "Uninstall failed: $_"
}
finally {
    if ($vmsess) { Remove-PSSession $vmsess }
}

Write-Host "[*] Done." -ForegroundColor Green
