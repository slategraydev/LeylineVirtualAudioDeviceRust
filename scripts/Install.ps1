# Leyline Audio: Simplified & Robust Installer
# Logic: Build (Host) -> Deploy & Verify (VM)

param (
    [switch]$clean,          # Full clean build on HOST
    [switch]$fast,           # Skip reverting VM
    [switch]$Uninstall,      # Only perform uninstallation/scrub on VM
    [string]$VMName = "LeylineTestVM",
    [string]$SnapshotName = "LeylineSnapshot"
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
$remotePath = "C:\LeylineInstall"
$BuildVersion = "1.0.9"

# Host Credentials for VM
$secPassword = ConvertTo-SecureString "REDACTED_VM_PASS" -AsPlainText -Force
$cred = New-Object System.Management.Automation.PSCredential ("USER", $secPassword)

# --- 0. CERTIFICATE GENERATION (Pre-Build Env) ---
# Generate outside eWDK environment to avoid PKI module conflicts
$pfxPath = "$ProjectRoot/leyline.pfx"
$cerPath = "$ProjectRoot/leyline.cer"

if (-not (Test-Path $pfxPath)) {
    Write-Host "[*] Generating Self-Signed Certificate..."
    $cert = New-SelfSignedCertificate -Subject "Leyline Audio" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
    $cert | Export-PfxCertificate -FilePath $pfxPath -Password (ConvertTo-SecureString -String "REDACTED_CERT_PASS" -Force -AsPlainText)
    $cert | Export-Certificate -FilePath $cerPath
}

# --- 1. VM SNAPSHOT HANDLING ---
if (-not $fast -and -not $Uninstall) {
    Write-Host "[*] Reverting VM '$VMName' to snapshot '$SnapshotName'..." -ForegroundColor Cyan
    Try {
        Restore-VMSnapshot -VMName $VMName -Name $SnapshotName -Confirm:$false
        Start-VM -Name $VMName -ErrorAction SilentlyContinue
    }
    Catch {
        Write-Warning "Snapshot revert failed. Ensure the VM and Snapshot exist."
        Return
    }

    Write-Host "    Waiting for VM network..." -NoNewline
    $timeout = 60
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    while ($timer.Elapsed.TotalSeconds -lt $timeout) {
        $vm = Get-VM -Name $VMName
        if ($vm.State -eq 'Running' -and $vm.NetworkAdapters[0].IpAddresses.Count -gt 0) {
            Write-Host " Ready." -ForegroundColor Green
            break
        }
        Start-Sleep -Seconds 1
        Write-Host "." -NoNewline
    }
    Start-Sleep -Seconds 5
}

# --- 2. HOST BUILD ---
if (-not $Uninstall) {
    Write-Host "[*] [HOST] Building Leyline $BuildVersion..." -ForegroundColor Cyan
    . "$PSScriptRoot\LaunchBuildEnv.ps1"

    if ($clean) {
        Write-Host "    -> Cleaning target directories..."
        $targets = @("$ProjectRoot/crates/leyline-kernel/target", "$ProjectRoot/src/HSA/bin", "$ProjectRoot/package")
        foreach ($t in $targets) { if (Test-Path $t) { Remove-Item $t -Recurse -Force } }
    }

    # Build Kernel
    Push-Location "$ProjectRoot/crates/leyline-kernel"
    cargo wdk build --profile release
    if ($LASTEXITCODE -ne 0) { throw "Kernel build failed." }
    Pop-Location

    # Aggregating & Signing
    Write-Host "    -> Packaging and Signing..."
    if (Test-Path "$ProjectRoot/package") { Remove-Item "$ProjectRoot/package" -Recurse -Force }
    New-Item -ItemType Directory -Path "$ProjectRoot/package/HSA" -Force | Out-Null
    
    Copy-Item "$ProjectRoot/target/release/leyline.dll" "$ProjectRoot/package/leyline.sys"
    Copy-Item "$ProjectRoot/crates/leyline-kernel/leyline.inx" "$ProjectRoot/package/leyline.inf"
    # Placeholder for APO/HSA if they exist
    if (Test-Path "$ProjectRoot/src/APO/LeylineAPO.dll") { Copy-Item "$ProjectRoot/src/APO/LeylineAPO.dll" "$ProjectRoot/package/" }
    if (Test-Path "$ProjectRoot/scripts/verification/Verify-AEB-Status.ps1") { Copy-Item "$ProjectRoot/scripts/verification/Verify-AEB-Status.ps1" "$ProjectRoot/package/" }

    # Sign with fixed password "REDACTED_CERT_PASS"
    # Certs generated at start of script
    Copy-Item $cerPath "$ProjectRoot/package/leyline.cer"

    $signArgs = @("sign", "/f", $pfxPath, "/p", "REDACTED_CERT_PASS", "/fd", "SHA256")
    & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot/package/leyline.sys" | Out-Null
    
    # Inf2Cat
    & $env:INF2CAT_EXE /driver:"$ProjectRoot/package" /os:10_X64 | Out-Null
    & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot/package/leyline.cat" | Out-Null
}

# --- 4. VM DEPLOY & VERIFY ---
try {
    Write-Host "[*] [VM] Connecting and Installing..." -ForegroundColor Cyan
    $vmsess = New-PSSession -VMName $VMName -Credential $cred

    # Simplified VM Execution
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path, $isUninstall)
        $ErrorActionPreference = "Continue" # Don't stop on minor failures

        Write-Host "    (VM) Cleaning environment..."
        # Stop and delete existing
        sc.exe stop LeylineAudio | Out-Null
        pnputil /remove-device "ROOT\MEDIA\0000" /force | Out-Null
        
        if ($isUninstall) { return }

        # Setup directory
        if (Test-Path $path) { Remove-Item $path -Recurse -Force }
        New-Item -ItemType Directory -Path $path -Force | Out-Null
    } -ArgumentList $remotePath, $Uninstall

    if ($Uninstall) { Write-Host "[SUCCESS] Uninstalled."; return }

    # Copy files
    Copy-Item -Path "$ProjectRoot/package\*" -Destination $remotePath -ToSession $vmsess -Recurse -Force

    # Final Install Block
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path)
        Set-Location $path
        certutil -addstore -f root leyline.cer | Out-Null
        certutil -addstore -f TrustedPublisher leyline.cer | Out-Null

        Write-Host "    (VM) Registering and Starting..."
        $devcon = "C:\eWDK\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe"
        
        # Cleanup any existing instances of the root device
        & $devcon remove "Root\Media\LeylineAudio" | Out-Null
        
        pnputil /add-driver "leyline.inf" /install | Out-Null
        & $devcon install "leyline.inf" "Root\Media\LeylineAudio" | Out-Null
        
        # Health Check
        Start-Sleep -Seconds 3
        $device = Get-PnpDevice -FriendlyName "*Leyline*" -ErrorAction SilentlyContinue | Where-Object { $_.Status -eq "OK" }
        if ($device) {
            Write-Host "    (VM) Found Device: $($device.FriendlyName) [Status: $($device.Status)]" -ForegroundColor Green
        }
        else {
            Write-Error "    (VM) FAILED: Driver device not appearing or not OK."
        }

        # Endpoint Check
        $endpoints = Get-ChildItem "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render" -ErrorAction SilentlyContinue
        $foundCount = 0
        foreach ($e in $endpoints) {
            $prop = Join-Path $e.Name "Properties"
            if (Test-Path "Registry::$prop") {
                $val = (Get-ItemProperty "Registry::$prop")."{a45c254e-df1c-4efd-8020-67d146a850e0},2"
                if ($val -like "*Leyline*") { 
                    Write-Host "    (VM) Found Endpoint: $val" -ForegroundColor Gray
                    $foundCount++ 
                }
            }
        }
        $color = if ($foundCount -gt 0) { "Green" } else { "Red" }
        Write-Host "    (VM) Total Leyline Audio Endpoints: $foundCount" -ForegroundColor $color
    } -ArgumentList $remotePath

}
catch {
    Write-Error "VM Operation failed: $_"
}
finally {
    if ($vmsess) { Remove-PSSession $vmsess }
    Write-Host "[*] Done."
}
