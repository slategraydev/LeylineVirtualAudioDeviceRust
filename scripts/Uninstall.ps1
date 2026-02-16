# Leyline Audio: FULL SYSTEM PURGE
# MUST be run as Administrator.

$ErrorActionPreference = "SilentlyContinue"
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
Push-Location $ProjectRoot

# 0. Administrator Guard
$currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
    Write-Error "This script MUST be run as Administrator."
    Pop-Location
    return
}

Write-Host "`n--- Leyline Audio: UNINSTALLING ---" -ForegroundColor Red

# 1. Device Removal
Write-Host "[*] Removing PnP Devices (Leyline and legacy samples)..."

# Use devcon to find devices by Hardware ID, as it's more reliable for root-enumerated "ghosts"
$devconPath = "D:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe"
if (Test-Path $devconPath) {
    # findall is crucial to catch non-present/ghost nodes
    $ids = & $devconPath findall "Root\LeylineAudio" | Where-Object { $_ -match "(.*):" -or $_ -match "^ROOT\\" } | ForEach-Object { 
        if ($_ -match "(.*):") { $matches[1].Trim() } else { $_.Trim() }
    }
    foreach ($id in ($ids | Select-Object -Unique)) {
        if ($id -match "ROOT\\") {
            Write-Host "    -> Removing Device (devcon): $id"
            & $devconPath remove "@$id" | Out-Null
        }
    }
}

# Fallback/Safety: Clean up by class and name
$pnpDevices = pnputil /enum-devices /class MEDIA
$toRemove = @()
$currentId = ""
foreach ($line in $pnpDevices) {
    if ($line -match "Instance ID:\s+(.*)") { $currentId = $matches[1].Trim() }
    if ($line -match "(Leyline|simpleaudiosample|SimpleAudioDriver)") {
        if ($currentId) { $toRemove += $currentId; $currentId = "" }
    }
}
foreach ($id in ($toRemove | Select-Object -Unique)) {
    Write-Host "    -> Removing Device (pnputil): $id"
    pnputil /remove-device $id | Out-Null
}

# 2. Service & Registry Cleanup
Write-Host "[*] Deleting Services and Registry Bloat..."
foreach ($svc in @("Leyline", "LEYLINEAUDIO", "simpleaudiosample")) {
    if (Get-Service $svc -ErrorAction SilentlyContinue) {
        Write-Host "    -> Stopping & Deleting Service: $svc"
        sc.exe stop $svc | Out-Null
        sc.exe delete $svc | Out-Null
    }
}
# Remove APO Registration
Remove-Item -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Audio\*" -Include "*Leyline*", "*simpleaudiosample*" -Recurse -ErrorAction SilentlyContinue

# 3. Driver Store Purge
Write-Host "[*] Purging Driver Store (OEM INFs)..."
$drivers = pnputil /enum-drivers
$oemInfs = @()
for ($i = 0; $i -lt $drivers.Count; $i++) {
    if ($drivers[$i] -match "Original Name:\s+(leyline\.inf|simpleaudiosample\.inf)") {
        # Look back up to 2 lines for the Published Name
        for ($j = 1; $j -le 2; $j++) {
            if ($i -ge $j -and $drivers[$i-$j] -match "Published Name:\s+(oem\d+\.inf)") {
                $oemInfs += $matches[1]
                break
            }
        }
    }
}

foreach ($inf in ($oemInfs | Select-Object -Unique)) {
    Write-Host "    -> Deleting Driver Package: $inf"
    pnputil /delete-driver $inf /force | Out-Null
}

# 4. Certificate Removal
Write-Host "[*] Removing Test Certificates..."
Get-ChildItem Cert:\CurrentUser\My, Cert:\LocalMachine\Root, Cert:\LocalMachine\TrustedPublisher | Where-Object { $_.Subject -like "*Leyline*" } | ForEach-Object {
    Write-Host "    -> Removing Cert: $($_.Thumbprint)"
    Remove-Item $_.PSPath -Force
}

# 5. Build Artifact Cleanup
if (Test-Path "package") { Remove-Item "package" -Recurse -Force }

Pop-Location
Write-Host "`n[SUCCESS] All Leyline artifacts removed. REBOOT RECOMMENDED." -ForegroundColor Green
