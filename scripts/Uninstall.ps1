# Leyline Audio: VM UNINSTALLER
# Logic: Remote Scrub of LeylineTestVM ONLY

param (
    [string]$VMName = ($env:LEYLINE_VM_NAME -or "LeylineTestVM")
)

Write-Host "[*] Triggering Consolidated VM Uninstall..." -ForegroundColor Cyan
& "$PSScriptRoot\Install.ps1" -Uninstall -VMName $VMName
