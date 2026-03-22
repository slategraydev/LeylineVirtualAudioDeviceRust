# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# LEYLINE ACX INSTALLER
# Builds the KMDF/ACX kernel driver on the host and deploys it to the
# Hyper-V test VM (TestVM / Leyline snapshot).
#
# Usage:
#   .\Install.ps1              # Full cycle: revert snapshot → build → deploy
#   .\Install.ps1 -fast        # Build and deploy without reverting snapshot
#   .\Install.ps1 -clean       # Clean build artifacts first, then full cycle
#   .\Install.ps1 -Uninstall   # Remote-uninstall the driver from the VM
#   .\Install.ps1 -BuildOnly   # Build only, do not deploy to VM
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

param (
    [switch]$clean,          # Full clean build on HOST
    [switch]$fast,           # Skip reverting VM snapshot
    [switch]$Uninstall,      # Only perform uninstallation/scrub on VM
    [switch]$BuildOnly,      # Build only, do not deploy to VM
    [PSCredential]$Credential
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# CONFIGURATION
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

$VMName        = "TestVM"
$SnapshotName  = "Leyline"
$remotePath    = "C:\LeylineInstall"
$BuildVersion  = "0.2.0"

# VM Credentials (default to USER / rd for the test VM)
if (-not $PSBoundParameters.ContainsKey('Credential')) {
    $VMUser = if ($env:LEYLINE_VM_USER) { $env:LEYLINE_VM_USER } else { "USER" }
    $VMPassword = if ($env:LEYLINE_VM_PASS) { $env:LEYLINE_VM_PASS } else { "rd" }
    $secPassword = ConvertTo-SecureString $VMPassword -AsPlainText -Force
    $Credential = New-Object System.Management.Automation.PSCredential ($VMUser, $secPassword)
}

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# 0. CERTIFICATE GENERATION
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

$pfxPath = "$ProjectRoot\leyline.pfx"
$cerPath = "$ProjectRoot\leyline.cer"
$CertPassword = if ($env:LEYLINE_CERT_PASS) { $env:LEYLINE_CERT_PASS } else { "leyline" }

if (-not (Test-Path $pfxPath)) {
    Write-Host "[*] Generating Self-Signed Certificate..." -ForegroundColor Cyan
    $cert = New-SelfSignedCertificate -Subject "Leyline Audio" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
    $secCertPass = ConvertTo-SecureString -String $CertPassword -Force -AsPlainText
    $cert | Export-PfxCertificate -FilePath $pfxPath -Password $secCertPass
    $cert | Export-Certificate -FilePath $cerPath
}

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# 1. VM SNAPSHOT HANDLING
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

if (-not $fast -and -not $Uninstall -and -not $BuildOnly) {
    Write-Host "[*] Reverting VM '$VMName' to snapshot '$SnapshotName'..." -ForegroundColor Cyan
    try {
        Restore-VMSnapshot -VMName $VMName -Name $SnapshotName -Confirm:$false
        Start-VM -Name $VMName -ErrorAction SilentlyContinue
    }
    catch {
        Write-Warning "Snapshot revert failed: $_"
        Write-Warning "Ensure VM '$VMName' and snapshot '$SnapshotName' exist."
        return
    }

    Write-Host "    Waiting for VM heartbeat..." -NoNewline
    $timeout = 90
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    while ($timer.Elapsed.TotalSeconds -lt $timeout) {
        $vm = Get-VM -Name $VMName
        if ($vm.State -eq 'Running' -and $vm.Heartbeat -eq 'OkApplicationsHealthy') {
            Write-Host " Ready." -ForegroundColor Green
            break
        }
        Start-Sleep -Seconds 2
        Write-Host "." -NoNewline
    }
    if ($timer.Elapsed.TotalSeconds -ge $timeout) {
        Write-Warning "VM did not reach healthy state within ${timeout}s."
    }
    Start-Sleep -Seconds 3
}

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# 2. HOST BUILD — KMDF/ACX Kernel Driver Only
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

if (-not $Uninstall) {
    Write-Host "[*] [HOST] Building Leyline ACX v$BuildVersion..." -ForegroundColor Cyan
    . "$PSScriptRoot\LaunchBuildEnv.ps1"

    if ($clean) {
        Write-Host "    -> Cleaning build artifacts..."
        $targets = @(
            "$ProjectRoot\crates\leyline-kernel\target",
            "$ProjectRoot\target",
            "$ProjectRoot\package\leyline.sys",
            "$ProjectRoot\package\leyline.pdb",
            "$ProjectRoot\package\leyline.map",
            "$ProjectRoot\package\leyline.cat"
        )
        foreach ($t in $targets) {
            if (Test-Path $t) { Remove-Item $t -Recurse -Force }
        }
    }

    # Build Kernel (Rust + ACX)
    Write-Host "    -> Building Kernel (Rust/ACX)..."
    Push-Location "$ProjectRoot\crates\leyline-kernel"
    cargo wdk build --profile release
    if ($LASTEXITCODE -ne 0) { Pop-Location; throw "Kernel build failed." }
    Pop-Location

    # Stage the package
    Write-Host "    -> Staging package..."
    if (-not (Test-Path "$ProjectRoot\package")) {
        New-Item -ItemType Directory -Path "$ProjectRoot\package" | Out-Null
    }

    # Copy built artifacts from cargo-wdk output
    $wdkOutDir = "$ProjectRoot\target\release\leyline_package"
    if (Test-Path $wdkOutDir) {
        Copy-Item -Path "$wdkOutDir\*" -Destination "$ProjectRoot\package" -Recurse -Force
    }
    else {
        # Fallback: find .sys directly in target/release
        $sysFile = "$ProjectRoot\target\x86_64-pc-windows-msvc\release\leyline.sys"
        if (-not (Test-Path $sysFile)) {
            $sysFile = "$ProjectRoot\target\release\leyline.sys"
        }
        if (Test-Path $sysFile) {
            Copy-Item -Path $sysFile -Destination "$ProjectRoot\package\leyline.sys" -Force
        }
    }

    # Ensure INF is in the package
    Copy-Item -Path "$ProjectRoot\package\leyline.inf" -Destination "$ProjectRoot\package\leyline.inf" -Force -ErrorAction SilentlyContinue

    # Sign the driver binary
    if ($env:SIGNTOOL_EXE -and (Test-Path "$ProjectRoot\package\leyline.sys")) {
        Write-Host "    -> Signing leyline.sys..."
        $signArgs = @("sign", "/f", $pfxPath, "/p", $CertPassword, "/fd", "SHA256")
        & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot\package\leyline.sys" | Out-Null
    }

    # Generate catalog file
    if ($env:INF2CAT_EXE -and (Test-Path "$ProjectRoot\package\leyline.inf")) {
        Write-Host "    -> Generating catalog..."
        & $env:INF2CAT_EXE /driver:"$ProjectRoot\package" /os:10_x64 /verbose 2>&1 | Out-Null
        if (Test-Path "$ProjectRoot\package\leyline.cat") {
            if ($env:SIGNTOOL_EXE) {
                & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot\package\leyline.cat" | Out-Null
            }
        }
    }

    # Copy certificate into the package
    Copy-Item -Path $cerPath -Destination "$ProjectRoot\package\leyline.cer" -Force

    # Copy devcon.exe if available
    $sdkVersion = if ($env:LEYLINE_SDK_VERSION) { $env:LEYLINE_SDK_VERSION } else { "10.0.28000.0" }
    $devconHostPath = Join-Path $env:WDK_ROOT "Tools\$sdkVersion\x64\devcon.exe"
    if (Test-Path $devconHostPath) {
        Copy-Item -Path $devconHostPath -Destination "$ProjectRoot\package\devcon.exe" -Force
    }

    Write-Host "[*] Build complete." -ForegroundColor Green

    if ($BuildOnly) {
        Write-Host "[*] -BuildOnly specified. Skipping VM deployment."
        return
    }
}

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# 3. VM DEPLOY & VERIFY
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

try {
    Write-Host "[*] [VM] Connecting to '$VMName'..." -ForegroundColor Cyan
    $vmsess = New-PSSession -VMName $VMName -Credential $Credential

    # --- Uninstall existing driver ---
    Write-Host "    -> Removing previous Leyline installation..."
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path, $isUninstall)

        # Remove existing device node
        $devices = pnputil /enum-devices /class "MEDIA" 2>&1 | Select-String "LeylineAudio"
        if ($devices) {
            pnputil /remove-device "ROOT\MEDIA\LeylineAudio" /force 2>&1 | Out-Null
        }

        # Remove any existing driver package
        $packages = pnputil /enum-drivers 2>&1 | Select-String -Pattern "leyline" -Context 1
        foreach ($pkg in $packages) {
            $oem = ($pkg.Context.PreContext | Select-String "oem\d+\.inf").Matches.Value
            if ($oem) { pnputil /delete-driver $oem /force 2>&1 | Out-Null }
        }

        if ($isUninstall) { return }

        # Prepare install directory
        if (Test-Path $path) { Remove-Item $path -Recurse -Force }
        New-Item -ItemType Directory -Path $path -Force | Out-Null
    } -ArgumentList $remotePath, $Uninstall

    if ($Uninstall) {
        Write-Host "[*] Uninstall complete." -ForegroundColor Green
        return
    }

    # --- Copy package to VM ---
    Write-Host "    -> Copying package to VM..."
    Copy-Item -Path "$ProjectRoot\package\*" -Destination $remotePath -ToSession $vmsess -Recurse -Force

    # --- Install on VM ---
    Write-Host "    -> Installing driver on VM..." -ForegroundColor Yellow
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path)
        Set-Location $path

        # Trust the certificate
        certutil -addstore -f root leyline.cer 2>&1 | Out-Null
        certutil -addstore -f TrustedPublisher leyline.cer 2>&1 | Out-Null

        # Install the driver package
        Write-Host "      [VM] pnputil /add-driver leyline.inf /install"
        $result = pnputil /add-driver "leyline.inf" /install 2>&1
        Write-Host $result

        # Create the device node if it doesn't exist
        if (Test-Path "$path\devcon.exe") {
            Write-Host "      [VM] devcon install leyline.inf ROOT\MEDIA\LeylineAudio"
            $devconResult = & "$path\devcon.exe" install leyline.inf "ROOT\MEDIA\LeylineAudio" 2>&1
            Write-Host $devconResult
        }

        # Restart audio services to pick up the new driver
        Write-Host "      [VM] Restarting audio services..."
        Restart-Service "AudioEndpointBuilder" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2
        Restart-Service "Audiosrv" -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 2

        # Verify endpoints
        Write-Host "      [VM] Checking audio endpoints..."
        $endpoints = Get-PnpDevice -Class "MEDIA" -Status OK 2>&1
        $leylineDevices = $endpoints | Where-Object { $_.FriendlyName -match "Leyline" }
        if ($leylineDevices) {
            Write-Host "      [VM] Leyline endpoints found:" -ForegroundColor Green
            $leylineDevices | ForEach-Object { Write-Host "        - $($_.FriendlyName) [$($_.Status)]" }
        }
        else {
            Write-Warning "      [VM] No Leyline endpoints detected. Check driver installation."
            # Dump driver status for debugging
            Write-Host "      [VM] pnputil /enum-drivers (Leyline):"
            pnputil /enum-drivers 2>&1 | Select-String -Pattern "leyline" -Context 3
        }

    } -ArgumentList $remotePath

    Write-Host "[*] Deployment complete." -ForegroundColor Green
}
catch {
    Write-Error "VM deployment failed: $_"
}
finally {
    if ($vmsess) { Remove-PSSession $vmsess }
}
