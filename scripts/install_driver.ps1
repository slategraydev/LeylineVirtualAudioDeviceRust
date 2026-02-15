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

    Write-Host "--- Installing Driver via PnPUtil ---" -ForegroundColor Cyan
    # /add-driver: Adds the driver to the store
    # /install: Installs it on any matching devices
    pnputil /add-driver $infPath /install

    Write-Host "`nInstallation attempt complete." -ForegroundColor Green
    Write-Host "Note: Ensure 'bcdedit /set testsigning on' has been run and the system was rebooted." -ForegroundColor Yellow
}

Install-Driver
