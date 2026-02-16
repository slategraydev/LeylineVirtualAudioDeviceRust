# Leyline Audio: VM REMOTE INSTALLER
# Builds on host, installs on VM via PowerShell Direct.
# Requirements:
# 1. VM must be running.
# 2. Guest Services enabled in Hyper-V.
# 3. User must have permissions on VM.

param (
    [string]$VMName = "LeylineTestVM",
    [string]$UserName = "USER",
    [switch]$clean
)

$ErrorActionPreference = "Stop"
$initialDir = Get-Location
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."

# Create credential object for blank password
$secPassword = ConvertTo-SecureString "REDACTED_VM_PASS" -AsPlainText -Force
$cred = New-Object System.Management.Automation.PSCredential ($UserName, $secPassword)

$vmsess = $null

try
{
    Push-Location $ProjectRoot

    # 1. Check VM Status
    Write-Host "[*] Checking VM: $VMName..." -ForegroundColor Cyan
    $vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
    if (-not $vm)
    { throw "VM '$VMName' not found."
    }
    if ($vm.State -ne 'Running')
    { throw "VM '$VMName' is not running. Current state: $($vm.State)"
    }

    # Create Session
    $vmsess = New-PSSession -VMName $VMName -Credential $cred

    # 2. Build on Host
    Write-Host "--- [1/3] Building Driver Package on Host ---" -ForegroundColor Cyan
    & "$PSScriptRoot\Install.ps1" -clean:$clean -build -package -install:$false


    if (-not (Test-Path "package"))
    { throw "Driver package not found. Run with -build."
    }

    # Locate DevGen on host to bundle it (Ensuring we match the 28000 environment)
    $devgenHost = $null
    $possibleEwdk = @("D:\eWDK_28000", $env:eWDK_ROOT_DIR, "C:\Users\Slate\Downloads\EWDK_br_release_28000_251103-1709")
    foreach ($p in $possibleEwdk)
    {
        if ($p -and (Test-Path $p))
        {
            $found = Get-ChildItem -Path $p -Filter "devgen.exe" -Recurse | Where-Object { $_.FullName -match "x64" } | Select-Object -First 1
            if ($found)
            { $devgenHost = $found; break
            }
        }
    }

    if ($devgenHost)
    {
        Write-Host "[*] Bundling DevGen from: $($devgenHost.FullName)"
        Copy-Item $devgenHost.FullName "package\devgen.exe" -Force
    }

    # 3. Remote Provisioning
    Write-Host "--- [2/3] Deploying to VM: $VMName ---" -ForegroundColor Cyan

    # Create remote directory
    $remotePath = "C:\LeylineInstall"
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path)
        if (Test-Path $path)
        { Remove-Item $path -Recurse -Force
        }
        New-Item -ItemType Directory -Path $path -Force | Out-Null
    } -ArgumentList $remotePath

    # Copy package to VM
    Write-Host "[*] Copying driver files..."
    Copy-Item -Path "package\*" -Destination $remotePath -ToSession $vmsess -Recurse -Force

    # 4. Remote Execution
    Write-Host "--- [3/3] Executing Remote Installation ---" -ForegroundColor Cyan
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path)
        Set-Location $path
        $ErrorActionPreference = "Stop"

        Write-Host "[VM] Enabling Testsigning..."
        bcdedit /set testsigning on | Out-Null

        Write-Host "[VM] Enabling Kernel Debug Prints (DbgPrint)..."
        $regPath = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Debug Print Filter"
        if (-not (Test-Path $regPath))
        { New-Item -Path $regPath -Force | Out-Null
        }
        New-ItemProperty -Path $regPath -Name "DEFAULT" -Value 0xF -PropertyType DWORD -Force | Out-Null

        Write-Host "[VM] Installing Certificates (Root and TrustedPublisher)..."
        # Using -f to force and -user for current user if machine store is restrictive
        certutil -addstore -f root leyline.cer | Out-Null
        certutil -addstore -f TrustedPublisher leyline.cer | Out-Null

        Write-Host "[VM] Cleaning old instances..."
        Get-PnpDevice -PresentOnly:$false | Where-Object { $_.HardwareID -contains "Root\LeylineAudio" } | ForEach-Object {
            pnputil /remove-device $_.InstanceId | Out-Null
        }

        Write-Host "[VM] Staging Driver..."
        $stageResult = pnputil /add-driver "leyline.inf" /install
        Write-Host "    -> $stageResult"

        Write-Host "[VM] Creating Device Node with DevGen..."
        if (Test-Path "devgen.exe")
        {
            .\devgen.exe /add /hardwareid "Root\LeylineAudio" | Out-Null
            Write-Host "    -> Device node created."
        } else
        {
            Write-Host "    -> [WARNING] devgen.exe missing in package. Device node not created." -ForegroundColor Yellow
        }

        Write-Host "[VM] Success. Check Device Manager." -ForegroundColor Green
    } -ArgumentList $remotePath

    Write-Host "`n[SUCCESS] Deployment to $VMName complete." -ForegroundColor Green

} finally
{
    if ($vmsess)
    { Remove-PSSession $vmsess
    }
    Set-Location $initialDir
}
