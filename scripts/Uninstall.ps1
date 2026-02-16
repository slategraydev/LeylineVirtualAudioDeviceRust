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
$legacyIds = @("Root\LeylineAudio", "Root\simpleaudiosample", "Root\SimpleAudioDriver")
Get-PnpDevice -PresentOnly:$false | Where-Object { 
    $hwid = $_.HardwareID
    $match = $false
    foreach ($id in $legacyIds) { if ($hwid -contains $id) { $match = $true; break } }
    $match
} | ForEach-Object {
    Write-Host "    -> Removing $($_.InstanceId) ($($_.FriendlyName))"
    pnputil /remove-device $_.InstanceId | Out-Null
}

# 2. Service & Registry Cleanup
Write-Host "[*] Deleting Services and Registry Bloat..."
foreach ($svc in @("Leyline", "LEYLINEAUDIO", "simpleaudiosample")) {
    sc.exe stop $svc | Out-Null
    sc.exe delete $svc | Out-Null
}
# Remove APO Registration
Remove-Item -Path "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Audio\*" -Include "*Leyline*", "*simpleaudiosample*" -Recurse -ErrorAction SilentlyContinue

# 3. Driver Store Purge
Write-Host "[*] Purging Driver Store (OEM INFs)..."
pnputil /enum-drivers | Select-String "Published Name:\s+(oem\d+\.inf)" -Context 0,2 | ForEach-Object {
    if ($_.Context.PostContext -match "leyline.inf" -or $_.Context.PostContext -match "simpleaudiosample.inf") {
        $name = $_.Matches[0].Groups[1].Value
        Write-Host "    -> Deleting $name"
        pnputil /delete-driver $name /force | Out-Null
    }
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
