# Leyline Audio: Simplified & Robust Installer
# Logic: Build (Host) -> Deploy & Verify (VM)

param (
    [switch]$clean,          # Full clean build on HOST
    [switch]$fast,           # Skip reverting VM
    [switch]$Uninstall,      # Only perform uninstallation/scrub on VM
    [string]$VMName = ($env:LEYLINE_VM_NAME -or "LeylineTestVM"),
    [string]$SnapshotName = ($env:LEYLINE_VM_SNAPSHOT -or "LeylineSnapshot"),
    [PSCredential]$Credential
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
$remotePath = "C:\LeylineInstall"
$BuildVersion = "0.1.0"

# Fallback for VM Credentials if not provided
if (-not $PSBoundParameters.ContainsKey('Credential')) {
    $VMUser = ($env:LEYLINE_VM_USER -or "USER")
    $VMPassword = ($env:LEYLINE_VM_PASS -or "REDACTED_VM_PASS")
    $secPassword = ConvertTo-SecureString $VMPassword -AsPlainText -Force
    $Credential = New-Object System.Management.Automation.PSCredential ($VMUser, $secPassword)
}

# --- 0. CERTIFICATE GENERATION (Pre-Build Env) ---
$pfxPath = "$ProjectRoot/leyline.pfx"
$cerPath = "$ProjectRoot/leyline.cer"
$CertPassword = ($env:LEYLINE_CERT_PASS -or "REDACTED_CERT_PASS")

if (-not (Test-Path $pfxPath)) {
    Write-Host "[*] Generating Self-Signed Certificate..."
    $cert = New-SelfSignedCertificate -Subject "Leyline Audio" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
    $cert | Export-PfxCertificate -FilePath $pfxPath -Password (ConvertTo-SecureString -String $CertPassword -Force -AsPlainText)
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
    if ($LASTEXITCODE -ne 0) { Pop-Location; throw "Kernel build failed." }
    Pop-Location

    # Stage the package
    if (-not (Test-Path "$ProjectRoot/package")) { New-Item -ItemType Directory -Path "$ProjectRoot/package" | Out-Null }
    Copy-Item -Path "$ProjectRoot/target/release/leyline_package/*" -Destination "$ProjectRoot/package" -Recurse -Force


    # Sign the package
    $signArgs = @("sign", "/f", $pfxPath, "/p", $CertPassword, "/fd", "SHA256")
    & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot/package/leyline.sys" | Out-Null
    & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot/package/leyline.cat" | Out-Null

    # Ensure certificate and devcon.exe are copied to the package folder so we can push it to the VM
    Copy-Item -Path $cerPath -Destination "$ProjectRoot/package/leyline.cer" -Force
    
    $devconHostPath = "D:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe"
    if (Test-Path $devconHostPath) {
        Copy-Item -Path $devconHostPath -Destination "$ProjectRoot/package/devcon.exe" -Force
    }
}

# --- 4. VM DEPLOY & VERIFY ---
try {
    Write-Host "[*] [VM] Connecting and Installing..." -ForegroundColor Cyan
    $vmsess = New-PSSession -VMName $VMName -Credential $Credential

    # Capture local environment variables for use in the script block

    $sdkVersion = if ($env:LEYLINE_SDK_VERSION) { $env:LEYLINE_SDK_VERSION } else { "10.0.28000.0" }

    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path, $isUninstall)
        $ErrorActionPreference = "Continue"

        Write-Host "    (VM) Preparing environment..."
        if ($isUninstall) {
            pnputil /remove-device "ROOT\MEDIA\LeylineAudio" /force | Out-Null
            $drivers = pnputil /enum-drivers | Select-String "Original Name:\s+leyline.inf" -Context 3, 0
            foreach ($d in $drivers) {
                if ($d.Context.PreContext[0] -match "Published Name:\s+(oem\d+\.inf)") {
                    pnputil /delete-driver $matches[1] /uninstall /force
                }
            }
            return
        }

        if (Test-Path $path) { Remove-Item $path -Recurse -Force }
        New-Item -ItemType Directory -Path $path -Force | Out-Null
    } -ArgumentList $remotePath, $Uninstall

    if ($Uninstall) { Write-Host "[SUCCESS] Uninstalled."; return }

    Copy-Item -Path "$ProjectRoot/package\*" -Destination $remotePath -ToSession $vmsess -Recurse -Force

    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path, $sdkVersion)
        Set-Location $path
        certutil -addstore -f root leyline.cer | Out-Null
        certutil -addstore -f TrustedPublisher leyline.cer | Out-Null

        Write-Host "    (VM) Upgrading Driver Stack..."

        # 1. Add to Driver Store and INSTALL to matching devices
        pnputil /add-driver "leyline.inf" /install | Out-Null

        # 2. Force an update on the specific ROOT node if it exists
        $devcon = "C:\eWDK_28000\Program Files\Windows Kits\10\Tools\$sdkVersion\x64\devcon.exe"
        if (Test-Path $devcon) {
            & $devcon update "leyline.inf" "ROOT\MEDIA\LeylineAudio" | Out-Null
        }
        else {
            Write-Host "    (VM) [WARNING] devcon.exe not found in remote package path at $devcon. Falling back to devgen..." -ForegroundColor Yellow
            devgen /add /bus ROOT /hardwareid "ROOT\MEDIA\LeylineAudio" | Out-Null
        }

        Start-Sleep -Seconds 3
        $device = Get-PnpDevice | Where-Object { $_.HardwareId -match "ROOT\\MEDIA\\LeylineAudio" } | Select-Object -First 1
        if ($device) {
            Write-Host "    (VM) UPGRADE SUCCESSFUL: $($device.InstanceId)" -ForegroundColor Green
            Write-Host "    (VM) Status: $($device.Status)"
            Restart-Service "AudioEndpointBuilder" -Force -ErrorAction SilentlyContinue
            Restart-Service "Audiosrv" -Force -ErrorAction SilentlyContinue
        }
        else {
            Write-Host "    (VM) No existing device found. Performing fresh install..."
            if (Test-Path $devcon) {
                & $devcon install "leyline.inf" "ROOT\MEDIA\LeylineAudio" | Out-Null
            }
            else {
                # Fallback purely as safety
                devgen /add /bus ROOT /hardwareid "ROOT\MEDIA\LeylineAudio" | Out-Null
            }
        }

        # Final check
        Write-Host "    (VM) Searching for Endpoints..."
        $foundCount = 0
        $endpoints = Get-ChildItem "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render" -ErrorAction SilentlyContinue
        foreach ($e in $endpoints) {
            $prop = Join-Path $e.Name "Properties"
            if (Test-Path "Registry::$prop") {
                $val = (Get-ItemProperty "Registry::$prop")."{a45c254e-df1c-4efd-8020-67d146a850e0},2"
                if ($val -like "*Leyline*") {
                    Write-Host "    (VM) Found Endpoint: $val" -ForegroundColor Green
                    $foundCount++
                }
            }
        }
        Write-Host "    (VM) Total Leyline Audio Endpoints: $foundCount"
    } -ArgumentList $remotePath, $sdkVersion
}
catch {
    Write-Error "VM Operation failed: $_"
}
finally {
    if ($vmsess) { Remove-PSSession $vmsess }
    Write-Host "[*] Done."
}
