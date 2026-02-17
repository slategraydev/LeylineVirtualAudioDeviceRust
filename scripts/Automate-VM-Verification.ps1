# Leyline Audio: END-TO-END VM VERIFIER (WinDbg kd.exe Mode)
# Reverts VM, configures kernel debugging, and streams logs via kd.exe.

param (
    [string]$VMName = "LeylineTestVM",
    [string]$SnapshotName = "Snapshot",
    [string]$UserName = "USER",
    [string]$DebuggerPath = "D:\eWDK_28000\Program Files\Windows Kits\10\Debuggers\x64\kd.exe",
    [string]$HostIP = "172.21.176.1",
    [int]$DebugPort = 50000,
    [string]$DebugKey = "1.2.3.4",
    [switch]$UseRootMedia,
    [switch]$Fast
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
$LogFile = "$ProjectRoot\LeylineKernelLogs.txt"

function Wait-VMSession {
    param($vname, $uname, $timeoutSec = 120)
    $start = Get-Date
    Write-Host "    -> Waiting for VM session ($vname)..." -ForegroundColor Gray
    while (((Get-Date) - $start).TotalSeconds -lt $timeoutSec) {
        try {
            $pw = ConvertTo-SecureString "REDACTED_VM_PASS" -AsPlainText -Force
            $cr = New-Object System.Management.Automation.PSCredential ($uname, $pw)
            $s = New-PSSession -VMName $vname -Credential $cr -ErrorAction SilentlyContinue
            if ($s) { return $s }
        }
        catch { }
        Start-Sleep -Seconds 5
    }
    throw "Timeout waiting for VM session on $vname."
}

try {
    Write-Host "`n[0/6] PREPARING HOST..." -ForegroundColor Cyan
    Get-Process -Name kd, cargo, rustc, leyline*, background*, "rust-analyzer" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    if (-not $Fast) {
        if (Test-Path "$ProjectRoot\target") { Remove-Item -Path "$ProjectRoot\target" -Recurse -Force -ErrorAction SilentlyContinue }
    }
    if (Test-Path $LogFile) { Remove-Item $LogFile -Force }

    if ($Fast) {
        Write-Host "[1/6] FAST MODE: Skipping VM Revert & Reboot..." -ForegroundColor Yellow
        $vm = Get-VM -Name $VMName
        if ($vm.State -ne 'Running') {
            Write-Host "    -> VM is not running. Starting..." -ForegroundColor Cyan
            Start-VM -Name $VMName | Out-Null
        }
        $vmsess = Wait-VMSession $VMName $UserName
    }
    else {
        Write-Host "[1/6] REVERTING VM: $VMName to $SnapshotName..." -ForegroundColor Cyan
        $vm = Get-VM -Name $VMName
        if ($vm.State -ne 'Off') { Stop-VM -Name $VMName -Force -ErrorAction SilentlyContinue | Out-Null }
        try {
            Restore-VMSnapshot -VMName $VMName -Name $SnapshotName -Confirm:$false -ErrorAction Stop | Out-Null
            Write-Host "    -> Snapshot restored." -ForegroundColor Green
        }
        catch {
            Write-Host "    -> Note: Snapshot restore skipped or already at target state ($($_.Exception.Message))." -ForegroundColor Gray
        }
        
        Write-Host "[2/6] STARTING VM & CONFIGURING KERNEL DEBUG..." -ForegroundColor Cyan
        Start-VM -Name $VMName | Out-Null
        $vmsess = Wait-VMSession $VMName $UserName

        Invoke-Command -Session $vmsess -ScriptBlock {
            param($ip, $port, $key)
            bcdedit /debug on | Out-Null
            bcdedit /set { current } edebug on | Out-Null
            bcdedit /dbgsettings net hostip:$ip port:$port key:$key | Out-Null
            Write-Host "    -> [VM] Kernel & Early Debug configured (Net: $ip : $port)"
        } -ArgumentList $HostIP, $DebugPort, $DebugKey
        Remove-PSSession $vmsess

        Write-Host "[3/6] POWER CYCLING VM TO ENABLE DEBUGGING..." -ForegroundColor Cyan
        Stop-VM -Name $VMName -Force | Out-Null
        Start-VM -Name $VMName | Out-Null
        Start-Sleep -Seconds 5
    }

    Write-Host "[4/6] STARTING KD.EXE CAPTURE & WAITING FOR VM..." -ForegroundColor Cyan
    if (-not (Test-Path $DebuggerPath)) { throw "kd.exe not found at $DebuggerPath" }
    
    # Start kd.exe in background
    $kdArgs = "-k net:port=$DebugPort,key=$DebugKey -v"
    $kdProc = Start-Process cmd -ArgumentList "/c `"`"$DebuggerPath`" $kdArgs > `"$LogFile`"`"" -WindowStyle Hidden -PassThru
    Write-Host "    -> kd.exe initialized and waiting for connection..." -ForegroundColor Green

    # Wait for VM to finish rebooting
    $vmsess = Wait-VMSession $VMName $UserName

    Write-Host "[5/6] INSTALLING DRIVER..." -ForegroundColor Cyan
    $cleanFlag = if ($Fast) { $false } else { $true }
    
    if ($UseRootMedia) {
        & "$PSScriptRoot\Install-VM.ps1" -VMName $VMName -UserName $UserName -clean:$cleanFlag -UseRootMedia
    }
    else {
        & "$PSScriptRoot\Install-VM.ps1" -VMName $VMName -UserName $UserName -clean:$cleanFlag
    }
    
    Write-Host "[VM] Enumerating Audio Endpoints..." -ForegroundColor Cyan
    if ($vmsess) {
        $endpointOutput = Invoke-Command -Session $vmsess -ScriptBlock {
            $endpoints = Get-PnpDevice -Class AudioEndpoint -Status OK -ErrorAction SilentlyContinue
            if ($endpoints) {
                return ($endpoints | Select-Object FriendlyName, Status, InstanceId | Format-Table -AutoSize | Out-String)
            }
            else {
                return "No active Audio Endpoints found."
            }
        }
        $endpointOutput | Set-Content -Path "$ProjectRoot\endpoints_host.txt" -Force
        Write-Host "    -> Endpoints saved to endpoints_host.txt"
        Write-Host $endpointOutput -ForegroundColor Gray
    }

    Write-Host "[VM] Enumerating Leyline Devices (Device Manager)..." -ForegroundColor Cyan
    if ($vmsess) {
        $devOutput = Invoke-Command -Session $vmsess -ScriptBlock {
            $devs = Get-PnpDevice | Where-Object { $_.FriendlyName -match "Leyline" -or $_.HardwareID -match "Leyline" }
            if ($devs) {
                return ($devs | Select-Object FriendlyName, Status, Class, InstanceId, Problem | Format-Table -AutoSize | Out-String)
            }
            else {
                return "No Leyline devices found in Device Manager."
            }
        }
        $devOutput | Set-Content -Path "$ProjectRoot\devices_host.txt" -Force
        Write-Host "    -> Device list saved to devices_host.txt"
        Write-Host $devOutput -ForegroundColor Gray
    }

    Write-Host "[6/6] FINALIZING LOGS..." -ForegroundColor Cyan
    Start-Sleep -Seconds 30
    $kdProc | Stop-Process -Force -ErrorAction SilentlyContinue
    
    if (Test-Path $LogFile) {
        $content = Get-Content $LogFile
        Write-Host "    -> Captured $([int]($content.Count)) lines of kernel data." -ForegroundColor Green
        $leylineLines = $content | Select-String "Leyline"
        if ($leylineLines) {
            Write-Host "`n--- [WINDBG / KD.EXE KERNEL LOGS] ---" -ForegroundColor Yellow
            $leylineLines | Select-Object -Last 20
        }
        else {
            Write-Host "    -> [!] No 'Leyline' strings found. Handshake might be failing or trace level too low." -ForegroundColor Yellow
            $content | Select-Object -Last 10 | ForEach-Object { Write-Host "        $_" -ForegroundColor Gray }
        }
    }

}
catch {
    Write-Host "`n[CRITICAL ERROR] Automation Failed!" -ForegroundColor Red
    Write-Host "Details: $_" -ForegroundColor Red
    exit 1
}
finally {
    if ($vmsess) { Remove-PSSession $vmsess }
    Get-Process -Name kd -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
}
