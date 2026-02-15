# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

# Installation script for Leyline Audio Driver.
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

function Install-Driver
{
    Test-IsAdministrator

    $packageDir = Join-Path $PSScriptRoot "..\package"
    $certPath = Join-Path $packageDir "leyline.cer"
    $infPath = Join-Path $packageDir "leyline.inf"

    if (-not (Test-Path $certPath))
    {
        Write-Error "Certificate not found at $certPath. Run package_driver.ps1 first."
    }

    Write-Host "--- Installing Certificate ---" -ForegroundColor Cyan
    certutil -addstore -f "Root" $certPath
    certutil -addstore -f "TrustedPublisher" $certPath

    Write-Host "--- Managing Virtual Device via DevCon ---" -ForegroundColor Cyan
    $devcon = "D:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe"

    # 1. Broad cleanup: Find and remove ANY device matching our Hardware ID or Name
    Write-Host "Searching for all Leyline-related device nodes..." -ForegroundColor Gray
    # Try removing by Hardware ID
    & $devcon remove "Root\LeylineAudio"
    # Try removing by a wildcard that captures ROOT\MEDIA variants
    & $devcon remove "ROOT\MEDIA*Leyline*"

    # 2. Pnputil Cleanup: Remove old INF versions from the driver store to prevent "Driver Rollback" issues
    Write-Host "Purging old driver versions from Windows Driver Store..." -ForegroundColor Gray
    $oldDrivers = pnputil /enum-drivers | Select-String -Pattern "leyline.inf" -Context 3,0
    foreach ($match in $oldDrivers)
    {
        if ($match.Context.PreContext[0] -match "oem\d+\.inf")
        {
            $oemName = $matches[0]
            Write-Host "Deleting $oemName..."
            pnputil /delete-driver $oemName /force | Out-Null
        }
    }

    # 3. Final Install
    Write-Host "Installing fresh driver instance..." -ForegroundColor Yellow
    & $devcon install $infPath Root\LeylineAudio
    Write-Host "`nInstallation attempt complete." -ForegroundColor Green
    Write-Host "Note: Ensure 'bcdedit /set testsigning on' has been run and the system was rebooted." -ForegroundColor Yellow
}

Install-Driver
