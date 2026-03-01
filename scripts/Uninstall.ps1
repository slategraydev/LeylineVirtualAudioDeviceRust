# Leyline Audio: VM UNINSTALLER
# Logic: Remote Scrub of LeylineTestVM ONLY

param (
    [string]$VMName = $(if ($env:LEYLINE_VM_NAME) { $env:LEYLINE_VM_NAME } else { "TestVM" })
)

Write-Host "[*] Triggering Consolidated VM Uninstall..." -ForegroundColor Cyan
& "$PSScriptRoot\Install.ps1" -Uninstall -VMName $VMName
