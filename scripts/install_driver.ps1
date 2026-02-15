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

    Write-Host "--- Creating Virtual Device via DevCon ---" -ForegroundColor Cyan
    # pnputil only stages the driver. We need devcon to create the Root device node.
    $devcon = "D:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe"
    & $devcon install $infPath Root\LeylineAudio

    Write-Host "`nInstallation attempt complete." -ForegroundColor Green
    Write-Host "Note: Ensure 'bcdedit /set testsigning on' has been run and the system was rebooted." -ForegroundColor Yellow
}

Install-Driver
