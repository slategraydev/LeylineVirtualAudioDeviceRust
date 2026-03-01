# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
# LEYLINE INSTALLER
# Performs host-side builds and orchestrated deployment to the test VM.
# ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

param (
    [switch]$clean,          # Full clean build on HOST
    [switch]$fast,           # Skip reverting VM
    [switch]$Uninstall,      # Only perform uninstallation/scrub on VM
    [string]$VMName = $(if ($env:LEYLINE_VM_NAME) { $env:LEYLINE_VM_NAME } else { "TestVM" }),
    [string]$SnapshotName = $(if ($env:LEYLINE_VM_SNAPSHOT) { $env:LEYLINE_VM_SNAPSHOT } else { "Leyline" }),
    [PSCredential]$Credential
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
$remotePath = "C:\LeylineInstall"
$BuildVersion = "0.1.0"

# Fallback for VM Credentials if not provided
if (-not $PSBoundParameters.ContainsKey('Credential')) {
    $VMUser = $(if ($env:LEYLINE_VM_USER) { $env:LEYLINE_VM_USER } else { "USER" })
    $VMPassword = $(if ($env:LEYLINE_VM_PASS) { $env:LEYLINE_VM_PASS } else { "rd" })
    $secPassword = ConvertTo-SecureString $VMPassword -AsPlainText -Force
    $Credential = New-Object System.Management.Automation.PSCredential ($VMUser, $secPassword)
}

# --- 0. CERTIFICATE GENERATION ---
$pfxPath = "$ProjectRoot\leyline.pfx"
$cerPath = "$ProjectRoot\leyline.cer"
$CertPassword = $(if ($env:LEYLINE_CERT_PASS) { $env:LEYLINE_CERT_PASS } else { "leyline" })

if (-not (Test-Path $pfxPath)) {
    Write-Host "[*] Generating Self-Signed Certificate..."
    $cert = New-SelfSignedCertificate -Subject "Leyline Audio" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
    $secCertPass = ConvertTo-SecureString -String $CertPassword -Force -AsPlainText
    $cert | Export-PfxCertificate -FilePath $pfxPath -Password $secCertPass
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
        $targets = @("$ProjectRoot\crates\leyline-kernel\target", "$ProjectRoot\src\HSA\bin", "$ProjectRoot\package")
        foreach ($t in $targets) { if (Test-Path $t) { Remove-Item $t -Recurse -Force } }
    }

    # Build Kernel (Rust)
    Write-Host "    -> Building Kernel (Rust)..."
    Push-Location "$ProjectRoot\crates\leyline-kernel"
    cargo wdk build --profile release
    if ($LASTEXITCODE -ne 0) { Pop-Location; throw "Kernel build failed." }
    Pop-Location

    # Build APO (C++)
    Write-Host "    -> Building APO (C++)..."
    Push-Location "$ProjectRoot\src\APO"
    nmake /nologo
    if ($LASTEXITCODE -ne 0) { Pop-Location; throw "APO build failed." }
    Pop-Location

    # Build HSA (C#)
    Write-Host "    -> Building HSA (C#)..."
    dotnet build "$ProjectRoot\src\HSA\LeylineHSA.csproj" -c Release
    if ($LASTEXITCODE -ne 0) { throw "HSA build failed." }

    # Stage the package
    if (-not (Test-Path "$ProjectRoot\package")) { New-Item -ItemType Directory -Path "$ProjectRoot\package" | Out-Null }
    Copy-Item -Path "$ProjectRoot\target\release\leyline_package\*" -Destination "$ProjectRoot\package" -Recurse -Force

    # Stage APO and HSA
    Copy-Item -Path "$ProjectRoot\src\APO\LeylineAPO.dll" -Destination "$ProjectRoot\package" -Force
    $hsaOutDir = "$ProjectRoot\src\HSA\bin\x64\Release\net8.0-windows10.0.19041.0\win-x64"
    if (Test-Path "$hsaOutDir\LeylineHSA.exe") {
        Copy-Item -Path "$hsaOutDir\LeylineHSA.exe" -Destination "$ProjectRoot\package" -Force
    }

    # Sign the binaries
    $signArgs = @("sign", "/f", $pfxPath, "/p", $CertPassword, "/fd", "SHA256")
    if ($env:SIGNTOOL_EXE) {
        & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot\package\leyline.sys" | Out-Null
        & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot\package\leyline.cat" | Out-Null
        & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot\package\LeylineAPO.dll" | Out-Null
    }

    Copy-Item -Path $cerPath -Destination "$ProjectRoot\package\leyline.cer" -Force
    $sdkVersion = if ($env:LEYLINE_SDK_VERSION) { $env:LEYLINE_SDK_VERSION } else { "10.0.28000.0" }
    $devconHostPath = Join-Path $env:WDK_ROOT "Tools\$sdkVersion\x64\devcon.exe"
    if (Test-Path $devconHostPath) {
        Copy-Item -Path $devconHostPath -Destination "$ProjectRoot\package\devcon.exe" -Force
    }
}

# --- 4. VM DEPLOY & VERIFY ---
try {
    Write-Host "[*] [VM] Connecting and Installing..." -ForegroundColor Cyan
    $vmsess = New-PSSession -VMName $VMName -Credential $Credential
    $sdkVersion = if ($env:LEYLINE_SDK_VERSION) { $env:LEYLINE_SDK_VERSION } else { "10.0.28000.0" }

    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path, $isUninstall)
        if ($isUninstall) {
            pnputil /remove-device "ROOT\MEDIA\LeylineAudio" /force | Out-Null
            return
        }
        if (Test-Path $path) { Remove-Item $path -Recurse -Force }
        New-Item -ItemType Directory -Path $path -Force | Out-Null
    } -ArgumentList $remotePath, $Uninstall

    if ($Uninstall) { return }

    Copy-Item -Path "$ProjectRoot\package\*" -Destination $remotePath -ToSession $vmsess -Recurse -Force

    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path, $sdkVersion)
        Set-Location $path
        certutil -addstore -f root leyline.cer | Out-Null
        certutil -addstore -f TrustedPublisher leyline.cer | Out-Null
        pnputil /add-driver "leyline.inf" /install | Out-Null
        Restart-Service "AudioEndpointBuilder" -Force
        Restart-Service "Audiosrv" -Force
    } -ArgumentList $remotePath, $sdkVersion
}
finally {
    if ($vmsess) { Remove-PSSession $vmsess }
    Write-Host "[*] Done."
}
