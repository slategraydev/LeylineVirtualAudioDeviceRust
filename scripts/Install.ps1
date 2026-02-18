# Leyline Audio: ROBUST VM INSTALLER & DEBUGGER
# Logic: Revert (VM) -> Build (Host) -> Deploy (VM) -> Install (VM) -> Monitor (Host)
# Target: LeylineTestVM ONLY

param (
    [switch]$clean,          # Full clean build on HOST
    [switch]$fast,           # Skip reverting VM (previously noRevert)
    [switch]$Uninstall,      # Only perform uninstallation/scrub on VM
    [string]$VMName = "LeylineTestVM",
    [string]$SnapshotName = "LeylineSnapshot",
    [string]$DebuggerPath = "C:\eWDK_28000\Program Files\Windows Kits\10\Debuggers\x64\kd.exe",
    [int]$DebugPort = 50000,
    [string]$DebugKey = "1.2.3.4"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue" # Suppress "Removed X of Y files" progress bars
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
$remotePath = "C:\LeylineInstall"
$DebuggerLog = Join-Path $ProjectRoot "Debugger_Log.txt"

# Host Credentials for VM
$secPassword = ConvertTo-SecureString "REDACTED_VM_PASS" -AsPlainText -Force
$cred = New-Object System.Management.Automation.PSCredential ("USER", $secPassword)

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

    Write-Host "    Waiting for VM network/heartbeat..." -NoNewline
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
    if ($timer.Elapsed.TotalSeconds -ge $timeout) {
        Write-Warning "`nVM did not report IP within timeout. Proceeding, but connection might fail."
    }
    Start-Sleep -Seconds 5 # stabilizing buffer
}

# --- 2. HOST BUILD ---
if (-not $Uninstall) {
    Write-Host "[*] [HOST] Initializing Build Environment..." -ForegroundColor Cyan
    . "$PSScriptRoot\LaunchBuildEnv.ps1"

    if ($clean) {
        Write-Host "    -> Performing CLEAN build (User Requested)..." -ForegroundColor Yellow
    
        # Kernel
        if (Test-Path "$ProjectRoot/crates/leyline-kernel/target") { Remove-Item "$ProjectRoot/crates/leyline-kernel/target" -Recurse -Force -ErrorAction SilentlyContinue }
        Push-Location "$ProjectRoot/crates/leyline-kernel"; cargo clean; Pop-Location
    
        # HSA
        if (Test-Path "$ProjectRoot/src/HSA/bin") { Remove-Item "$ProjectRoot/src/HSA/bin" -Recurse -Force -ErrorAction SilentlyContinue }
        if (Test-Path "$ProjectRoot/src/HSA/obj") { Remove-Item "$ProjectRoot/src/HSA/obj" -Recurse -Force -ErrorAction SilentlyContinue }
    
        # Package
        if (Test-Path "$ProjectRoot/package") { Remove-Item "$ProjectRoot/package" -Recurse -Force -ErrorAction SilentlyContinue }
    }

    Write-Host "[*] [HOST] Compiling & Packaging..." -ForegroundColor Cyan
    # 1. Kernel Build
    Write-Host "    -> Building Kernel (release)..."
    Push-Location "$ProjectRoot/crates/leyline-kernel"
    cargo wdk build --profile release
    if ($LASTEXITCODE -ne 0) { throw "Kernel build failed." }
    Pop-Location

    # 2. HSA Build
    Write-Host "    -> Building HSA..."
    dotnet build "$ProjectRoot/src/HSA/LeylineHSA.csproj" -c Release /p:Version=1.0.1.VM | Out-Null

    # 3. APO Build
    Write-Host "    -> Building APO..."
    Push-Location "$ProjectRoot/src/APO"
    if (Test-Path "Makefile") { nmake /f Makefile | Out-Null }
    Pop-Location

    # Package Aggregation
    Write-Host "    -> Aggregating Artifacts..."
    if (Test-Path "$ProjectRoot/package") { Remove-Item "$ProjectRoot/package" -Recurse -Force }
    New-Item -ItemType Directory -Path "$ProjectRoot/package/HSA" -Force | Out-Null

    $kernelPath = "$ProjectRoot/target/release/leyline.dll"
    if (-not (Test-Path $kernelPath)) { throw "Kernel binary not found at $kernelPath" }

    Copy-Item $kernelPath "$ProjectRoot/package/leyline.sys"
    Copy-Item "$ProjectRoot/crates/leyline-kernel/leyline.inx" "$ProjectRoot/package/leyline.inf"
    Copy-Item "$ProjectRoot/src/APO/LeylineAPO.dll" "$ProjectRoot/package/"
    dotnet publish "$ProjectRoot/src/HSA/LeylineHSA.csproj" -c Release -r win-x64 --self-contained false -o "$ProjectRoot/package/HSA" | Out-Null

    # Signing
    Write-Host "    -> Signing artifacts..."
    if (-not (Test-Path "$ProjectRoot/package\leyline.cer")) {
        $cert = New-SelfSignedCertificate -Subject "Leyline Audio" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
        $cert | Export-PfxCertificate -FilePath "$ProjectRoot/package\leyline.pfx" -Password (ConvertTo-SecureString -String "REDACTED_CERT_PASS" -Force -AsPlainText)
        $cert | Export-Certificate -FilePath "$ProjectRoot/package\leyline.cer"
    }
    if (-not (Test-Path "$ProjectRoot/package\leyline.pfx")) {
        Write-Host "Re-exporting PFX..."
        $cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq "CN=Leyline Audio" } | Select-Object -First 1
        if ($cert) {
            $cert | Export-PfxCertificate -FilePath "$ProjectRoot/package\leyline.pfx" -Password (ConvertTo-SecureString -String "REDACTED_CERT_PASS" -Force -AsPlainText)
        }
    }
    & $env:INF2CAT_EXE /driver:"$ProjectRoot/package" /os:10_X64 | Out-Null

    $pfxPath = "$ProjectRoot\package\leyline.pfx"
    $signArgs = @("sign", "/f", $pfxPath, "/p", "REDACTED_CERT_PASS", "/fd", "SHA256")

    Write-Host "    -> Signing leyline.sys..."
    & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot/package\leyline.sys" | Out-Null
    & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot/package\leyline.cat" | Out-Null
    & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot/package\LeylineAPO.dll" | Out-Null
    & $env:SIGNTOOL_EXE $signArgs "$ProjectRoot/package\HSA\LeylineHSA.exe" | Out-Null

    # --- 3. START DEBUGGER & LOG STREAM ---
    Write-Host "[*] [HOST] Starting Debugger (kd.exe)..." -ForegroundColor Cyan

    # Attempt to resolve KD path from environment if available
    if ($env:eWDK_ROOT_DIR) {
        $DebuggerPath = Join-Path $env:eWDK_ROOT_DIR "Program Files\Windows Kits\10\Debuggers\x64\kd.exe"
    }
    if (-not (Test-Path $DebuggerPath)) {
        # Fallback paths
        $DebuggerPath = "C:\eWDK_28000\Program Files\Windows Kits\10\Debuggers\x64\kd.exe"
        if (-not (Test-Path $DebuggerPath)) { 
            $DebuggerPath = "D:\eWDK_28000\Program Files\Windows Kits\10\Debuggers\x64\kd.exe" 
        }
    }

    if (-not (Test-Path $DebuggerPath)) {
        Write-Error "Debugger (kd.exe) not found at: $DebuggerPath"
        throw "Debugger not found."
    }

    # Stop any existing KD processes to be safe
    Get-Process -Name kd -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1

    if (Test-Path $DebuggerLog) { Remove-Item $DebuggerLog -Force -ErrorAction SilentlyContinue }

    Write-Host "    -> Starting KD job..."
    
    $logJob = Start-Job -ScriptBlock {
        param($DebuggerPath, $DebugPort, $DebugKey, $LogPath)
        
        $kdArgs = @(
            "-k", "net:port=$DebugPort,key=$DebugKey",
            "-v",
            "-y", "$PSScriptRoot\..\package", # Search local package first
            "-c", "ed nt!Kd_DEFAULT_Mask 0xFFFFFFFF; g",
            "-logo", $LogPath
        )
        $env:_NT_SYMBOL_PATH = "SRV*C:\Symbols*http://msdl.microsoft.com/download/symbols" # Reset to default
        & $DebuggerPath @kdArgs
    } -ArgumentList $DebuggerPath, $DebugPort, $DebugKey, $DebuggerLog, $PSScriptRoot

    Write-Host "    -> Debugger logging to: $DebuggerLog"
    Write-Host "    -> Waiting for debugger to initialize (2s)..."
    Start-Sleep -Seconds 2
    Write-Host "    -> Streaming events to console..." -ForegroundColor Green
    if (-not (Test-Path $DebuggerLog)) {
        New-Item -ItemType File -Path $DebuggerLog -Force | Out-Null
    }
}

# --- 4. VM OPERATION ---
try {
    Write-Host "[*] Connecting to VM..." -ForegroundColor Cyan
    $vmsess = New-PSSession -VMName $VMName -Credential $cred


    # --- SCRUB / UNINSTALL ---
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path)
        Write-Host "--- [VM] PRE-INSTALL SCRUB INITIATED ---" -ForegroundColor Magenta
        Write-Host "    (This ensures a clean environment before installation)" -ForegroundColor Gray
        $toolPath = "C:\eWDK\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64"
        $devcon = Join-Path $toolPath "devcon.exe"

        # 1. Kill duplicate IDs
        if (Test-Path $devcon) {
            & $devcon remove "Root\Media\LeylineAudio" | Out-Null
            & $devcon remove "Root\LeylineAudio" | Out-Null
            & $devcon remove "SWC\Leyline*" | Out-Null
        }

        # 2. PnP Check (PowerShell native)
        Get-PnpDevice -PresentOnly:$false | Where-Object { $_.FriendlyName -like "*Leyline*" -or $_.HardwareID -contains "Root\Media\LeylineAudio" } | ForEach-Object {
            Write-Host "     -> Removing Instance: $($_.InstanceId)"
            pnputil /remove-device $_.InstanceId /force | Out-Null
        }
        
        # 3. Service/Store Cleanup
        $svc = Get-Service "Leyline*" -ErrorAction SilentlyContinue
        if ($svc) { 
            Write-Host "     -> Stopping/Deleting Service..."
            sc.exe stop LeylineAudio | Out-Null
            sc.exe delete LeylineAudio | Out-Null
        }
        
        # 4. Driver Store - Aggressive
        $drivers = pnputil /enum-drivers | Select-String "Original Name:\s+leyline\.inf" -Context 1
        foreach ($d in $drivers) {
            if ($d.Context.PreContext[0] -match "(oem\d+\.inf)") { 
                $inf = $matches[1]
                Write-Host "     -> Deleting driver package: $inf"
                pnputil /delete-driver $inf /force | Out-Null 
            }
        }

        # 5. File Cleanup (System32)
        $sysFiles = @("C:\Windows\System32\drivers\leyline.sys", "C:\Windows\System32\LeylineAPO.dll")
        foreach ($f in $sysFiles) {
            if (Test-Path $f) { 
                Write-Host "     -> Deleting system file: $f"
                Remove-Item $f -Force -ErrorAction SilentlyContinue 
            }
        }

        if (Test-Path $path) { Remove-Item $path -Recurse -Force }
        New-Item -ItemType Directory -Path $path -Force | Out-Null
        
    } -ArgumentList $remotePath

    if ($Uninstall) {
        Write-Host "[SUCCESS] VM scrubbed/uninstalled." -ForegroundColor Green
        return
    }

    # --- DEPLOY & INSTALL ---
    Write-Host "    -> Copying files..."
    Copy-Item -Path "$ProjectRoot/package\*" -Destination $remotePath -ToSession $vmsess -Recurse -Force

    Write-Host "    -> Installing Driver on VM..."
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path)
        $ErrorActionPreference = "Stop"
        Set-Location $path
        certutil -addstore -f root leyline.cer | Out-Null
        certutil -addstore -f TrustedPublisher leyline.cer | Out-Null
        
        $devcon = "C:\eWDK\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe"
        Write-Host "       (VM) Installing driver..."
        pnputil /add-driver "leyline.inf" /install | Out-Null
        $res = & $devcon install "leyline.inf" "Root\Media\LeylineAudio" 2>&1
        if ($res -match "failed") { Write-Warning "Devcon install warning/error: $res" }
        pnputil /scan-devices | Out-Null
    } -ArgumentList $remotePath
}
catch {
    Write-Error "VM Operation failed: $_"
}

# --- 5. MONITOR LOOP ---
Write-Host "`n[*] Installation triggering done. Monitoring logs..." -ForegroundColor Cyan
Write-Host "    - Press Ctrl+C to stop debugger and exit."
Write-Host "    - Waiting for 'END_OF_DEBUG' signal...`n"

try {
    $reader = New-Object System.IO.StreamReader([System.IO.File]::Open($DebuggerLog, [System.IO.FileMode]::Open, [System.IO.FileAccess]::Read, [System.IO.FileShare]::ReadWrite))
    $foundEnd = $false
    
    while (-not $foundEnd) {
        $line = $reader.ReadLine()
        if ($line -ne $null) {
            $trimmed = $line.Trim()
            
            # Show connection status
            if ($trimmed -match "Waiting to reconnect" -or $trimmed -match "Connected to") {
                Write-Host "    [DBGENG] $trimmed" -ForegroundColor Yellow
            }

            if ($trimmed -match "Leyline" -or $trimmed -match "ERROR" -or $trimmed -match "FAILURE") {
                Write-Host $trimmed -ForegroundColor Gray
            }
            if ($trimmed -match "END_OF_DEBUG") {
                Write-Host "`n[SUCCESS] 'END_OF_DEBUG' signal received. Stopping." -ForegroundColor Green
                $foundEnd = $true
            }
        }
        else {
            if ($logJob.State -ne 'Running') {
                Write-Warning "Debugger job stopped unexpectedly."
                Receive-Job -Job $logJob | Write-Warning
                break
            }
            Start-Sleep -Milliseconds 200
        }
    }
    $reader.Close()
}
finally {
    Write-Host "`n[*] Cleaning up..." -ForegroundColor Cyan
    if ($logJob) {
        Stop-Job $logJob -ErrorAction SilentlyContinue
        Remove-Job $logJob -ErrorAction SilentlyContinue
    }
    Get-Process -Name kd -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    if ($vmsess) { Remove-PSSession $vmsess }
    Write-Host "    -> Log file saved at: $DebuggerLog"
    Write-Host "[*] Done."
}
