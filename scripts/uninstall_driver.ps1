# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

# Uninstallation script for Leyline Audio Driver.
# MUST be run as Administrator.

$ErrorActionPreference = "Stop"

function Test-IsAdministrator
{
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))
    {
        Write-Error "This script MUST be run as Administrator."
        exit 1
    }
}

function Uninstall-Driver
{
    Test-IsAdministrator

    Write-Host "--- Removing Virtual Device Nodes ---" -ForegroundColor Cyan
    $devcon = "D:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe"
    & $devcon remove "Root\LeylineAudio"
    & $devcon remove "ROOT\MEDIA*Leyline*"

    Write-Host "--- Locating Installed Driver ---" -ForegroundColor Cyan
    $driver = pnputil /enum-drivers | Select-String "leyline.inf" -Context 3,0

    if ($driver)
    {
        # Extract the Published Name (oemXX.inf)
        $oemName = ($driver.Context.PreContext[0] -split ":" | Select-Object -Last 1).Trim()
        Write-Host "Found driver: $oemName"

        Write-Host "--- Uninstalling Driver ---" -ForegroundColor Cyan
        pnputil /delete-driver $oemName /uninstall /force
    } else
    {
        Write-Warning "Leyline Audio Driver (leyline.inf) not found in the driver store."
    }

    Write-Host "--- Removing Certificate (Optional) ---" -ForegroundColor Cyan
    $certThumbprint = (Get-PfxCertificate (Join-Path $PSScriptRoot "..\package\leyline.cer")).Thumbprint
    if ($certThumbprint)
    {
        # certutil -delstore Root <Thumbprint>
        # certutil -delstore TrustedPublisher <Thumbprint>
        # Note: Manual removal is safer to avoid accidental deletion of shared roots,
        # but for this dev cert we can be specific if needed.
        Write-Host "Certificate thumbprint identified: $certThumbprint"
        Write-Host "Manual certificate removal from 'Root' and 'TrustedPublisher' stores recommended if no longer needed."
    }

    Write-Host "`nUninstallation attempt complete." -ForegroundColor Green
}

Uninstall-Driver
