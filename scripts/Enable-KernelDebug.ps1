#Requires -RunAsAdministrator
<#
.SYNOPSIS
    Enables kernel debug output for viewing DbgPrint messages in DebugView.

.DESCRIPTION
    This script configures Windows to output kernel DbgPrint messages that can be
    captured by Sysinternals DebugView. This is essential for debugging the
    Leyline Audio Driver.

    Run this script on your test VM before using DebugView to see driver output.

.EXAMPLE
    .\Enable-KernelDebug.ps1
    Then reboot the VM.

.NOTES
    File Name      : Enable-KernelDebug.ps1
    Author         : Leyline Audio Driver Team
    Prerequisite   : PowerShell 5.1 or later, Administrator privileges
#>

[CmdletBinding()]
param()

# Error action preference
$ErrorActionPreference = "Stop"

# Store initial directory
$initialDir = Get-Location

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Leyline Driver: Kernel Debug Enabler" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if running as Administrator
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))
{
    Write-Error "This script must be run as Administrator. Right-click PowerShell and select 'Run as Administrator'."
    exit 1
}

try
{
    # Step 1: Enable kernel debugging via bcdedit
    Write-Host "[1/3] Enabling kernel debugging via bcdedit..." -ForegroundColor Yellow
    $bcdOutput = bcdedit /debug on 2>&1
    if ($LASTEXITCODE -eq 0)
    {
        Write-Host "      ✓ Kernel debugging enabled successfully" -ForegroundColor Green
    } else
    {
        Write-Warning "      ⚠ bcdedit returned non-zero exit code: $LASTEXITCODE"
        Write-Warning "      Output: $bcdOutput"
    }

    # Step 2: Set debug print filter to show all DbgPrint output
    Write-Host "[2/3] Setting Debug Print Filter registry key..." -ForegroundColor Yellow

    $regPath = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Debug Print Filter"
    $regName = "DEFAULT"
    $regValue = 0xffffffff  # Show all debug output levels

    # Create the registry key if it doesn't exist
    if (-not (Test-Path $regPath))
    {
        Write-Host "      Creating registry key: $regPath" -ForegroundColor Gray
        $null = New-Item -Path $regPath -Force -ErrorAction Stop
    }

    # Set the registry value
    Set-ItemProperty -Path $regPath -Name $regName -Value $regValue -Type DWord -Force -ErrorAction Stop
    Write-Host "      ✓ Debug Print Filter set to 0xFFFFFFFF (all output enabled)" -ForegroundColor Green

    # Step 3: Display next steps
    Write-Host "[3/3] Configuration complete!" -ForegroundColor Green
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  IMPORTANT: REBOOT REQUIRED" -ForegroundColor Yellow
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor White
    Write-Host "  1. Reboot this VM" -ForegroundColor Yellow
    Write-Host "  2. Download DebugView from Sysinternals:" -ForegroundColor White
    Write-Host "     https://docs.microsoft.com/en-us/sysinternals/downloads/debugview" -ForegroundColor Gray
    Write-Host "  3. Run DebugView as Administrator" -ForegroundColor White
    Write-Host "  4. Enable Capture → Capture Kernel" -ForegroundColor White
    Write-Host "  5. Set Filter to: Leyline*;LeylineTopo*" -ForegroundColor White
    Write-Host "  6. Load the Leyline driver and watch output!" -ForegroundColor Green
    Write-Host ""
    Write-Host "To disable kernel debugging later, run:" -ForegroundColor Gray
    Write-Host "  bcdedit /debug off" -ForegroundColor DarkGray
    Write-Host ""

} catch
{
    Write-Error "Failed to enable kernel debug output: $_"
    exit 1
} finally
{
    # Restore original directory
    Set-Location $initialDir
}
