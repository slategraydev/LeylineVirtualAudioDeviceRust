# Leyline Audio: VM REMOTE INSTALLER
# Builds on host, installs on VM via PowerShell Direct.
# Requirements:
# 1. VM must be running.
# 2. Guest Services enabled in Hyper-V.
# 3. User must have permissions on VM.

param (
    [string]$VMName = "LeylineTestVM",
    [string]$UserName = "USER",
    [switch]$clean,
    [switch]$UseRootMedia
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

    # Force clean rebuild to ensure latest code is compiled
    # Note: Cargo workspace puts output in workspace root target/, not crate target/
    $kernelBinary = "$ProjectRoot/target/release/leyline.dll"
    $kernelSys = "$ProjectRoot/target/release/leyline.sys"

    # Get timestamp before build
    $preBuildTime = $null
    if (Test-Path $kernelBinary)
    {
        $preBuildTime = (Get-Item $kernelBinary).LastWriteTime
        Write-Host "[*] Previous build: $($preBuildTime.ToString('MM/dd/yyyy HH:mm:ss'))" -ForegroundColor Gray
    } elseif (Test-Path $kernelSys)
    {
        $preBuildTime = (Get-Item $kernelSys).LastWriteTime
        Write-Host "[*] Previous build: $($preBuildTime.ToString('MM/dd/yyyy HH:mm:ss'))" -ForegroundColor Gray
    }

    # Force clean to ensure cargo rebuilds everything
    Write-Host "[*] Cleaning old build artifacts..." -ForegroundColor Yellow
    # Clean from workspace root to ensure proper cleanup
    cargo clean --release
    if ($LASTEXITCODE -ne 0)
    {
        Pop-Location
        throw "cargo clean failed with exit code $LASTEXITCODE"
    }
    Pop-Location

    # Verify clean actually removed the files
    if (Test-Path $kernelBinary)
    {
        Remove-Item $kernelBinary -Force
        Write-Host "[*] Manually removed old DLL" -ForegroundColor Gray
    }
    if (Test-Path $kernelSys)
    {
        Remove-Item $kernelSys -Force
        Write-Host "[*] Manually removed old SYS" -ForegroundColor Gray
    }

    # Also clean stale files in crate target directory if they exist
    $staleBinary = "$ProjectRoot/crates/leyline-kernel/target/release/leyline.dll"
    $staleSys = "$ProjectRoot/crates/leyline-kernel/target/release/leyline.sys"
    if (Test-Path $staleBinary)
    {
        Remove-Item $staleBinary -Force
        Write-Host "[*] Removed stale DLL from crate target" -ForegroundColor Gray
    }
    if (Test-Path $staleSys)
    {
        Remove-Item $staleSys -Force
        Write-Host "[*] Removed stale SYS from crate target" -ForegroundColor Gray
    }

    # Run the build
    Write-Host "[*] Starting fresh build..." -ForegroundColor Cyan
    $buildOutput = & "$PSScriptRoot\Install.ps1" -clean:$clean -build -package -install:$false 2>&1
    $buildExitCode = $LASTEXITCODE

    # Show build output for debugging
    if ($buildOutput)
    {
        Write-Host "[*] Build output:" -ForegroundColor Gray
        $buildOutput | ForEach-Object { Write-Host "    $_" -ForegroundColor Gray }
    }

    if ($buildExitCode -ne 0)
    {
        throw "Build script failed with exit code $buildExitCode"
    }

    # Debug: Show what files exist in target/release
    Write-Host "[*] Listing build output directory..." -ForegroundColor Gray
    $targetDir = "$ProjectRoot/target/release"
    if (Test-Path $targetDir)
    {
        $files = Get-ChildItem $targetDir -File | Where-Object { $_.Name -match "\.(dll|sys|exe)$" } | Select-Object -First 10
        foreach ($file in $files)
        {
            Write-Host "    Found: $($file.Name) ($($file.LastWriteTime.ToString('HH:mm:ss')))" -ForegroundColor Gray
        }
    } else
    {
        Write-Host "    [WARNING] Target directory not found: $targetDir" -ForegroundColor Yellow
    }

    # PowerShell scripts don't set LASTEXITCODE properly, check if files exist instead
    if (-not (Test-Path $kernelBinary) -and -not (Test-Path $kernelSys))
    {
        Write-Host "[ERROR] Expected binaries not found:" -ForegroundColor Red
        Write-Host "  - $kernelBinary" -ForegroundColor Red
        Write-Host "  - $kernelSys" -ForegroundColor Red
        Write-Host "[HINT] Check if cargo wdk build produced output in $ProjectRoot/target/release/" -ForegroundColor Yellow
        throw "Build failed - no output binary found"
    }

    # Verify build actually produced new binary
    $postBuildTime = $null
    $actualBinary = $null
    if (Test-Path $kernelBinary)
    {
        $postBuildTime = (Get-Item $kernelBinary).LastWriteTime
        $actualBinary = $kernelBinary
    } elseif (Test-Path $kernelSys)
    {
        $postBuildTime = (Get-Item $kernelSys).LastWriteTime
        $actualBinary = $kernelSys
    }

    if ($preBuildTime -and $postBuildTime -le $preBuildTime)
    {
        throw "Build did not produce new binary! Timestamp unchanged: $postBuildTime"
    }

    # Verify the build actually produced a NEW file (not the old one)
    if ($preBuildTime -and $postBuildTime -eq $preBuildTime)
    {
        Write-Host "[ERROR] Build did not produce new binary!" -ForegroundColor Red
        Write-Host "[ERROR] Binary timestamp unchanged: $($postBuildTime.ToString('MM/dd/yyyy HH:mm:ss'))" -ForegroundColor Red
        Write-Host "[HINT] Try running manually:" -ForegroundColor Yellow
        Write-Host "      cd crates/leyline-kernel" -ForegroundColor Yellow
        Write-Host "      cargo clean --release" -ForegroundColor Yellow
        Write-Host "      cargo build --release" -ForegroundColor Yellow
        throw "Build timestamp did not update - cargo may not be recompiling changed files"
    }

    $currentTime = Get-Date
    $buildAge = ($currentTime - $postBuildTime).TotalMinutes
    if ($buildAge -gt 1)
    {
        Write-Host "[WARNING] New build is $([int]$buildAge) minutes old - build may have used cache" -ForegroundColor Yellow
    }

    Write-Host "[*] Build successful: $actualBinary" -ForegroundColor Green
    Write-Host "[*] NEW build created: $($postBuildTime.ToString('MM/dd/yyyy HH:mm:ss'))" -ForegroundColor Green

    if ($preBuildTime)
    {
        $timeDiff = ($postBuildTime - $preBuildTime).TotalMinutes
        Write-Host "[*] Time since previous build: $([int]$timeDiff) minutes" -ForegroundColor Gray
    }

    if (-not (Test-Path "$ProjectRoot/package"))
    { throw "Driver package not found. Run with -build."
    }

    # Verify package contains fresh binary
    $packagedSys = "$ProjectRoot/package/leyline.sys"
    if (Test-Path $packagedSys)
    {
        $packageTime = (Get-Item $packagedSys).LastWriteTime
        $sourceTime = (Get-Item $actualBinary).LastWriteTime

        Write-Host "[*] Source binary: $($sourceTime.ToString('MM/dd/yyyy HH:mm:ss'))" -ForegroundColor Gray
        Write-Host "[*] Packaged binary: $($packageTime.ToString('MM/dd/yyyy HH:mm:ss'))" -ForegroundColor Gray

        # Allow 2 second tolerance for file system timestamps
        $timeDiff = ($packageTime - $sourceTime).TotalSeconds
        if ($timeDiff -lt -2)
        {
            Write-Host "[ERROR] Package is older than source binary by $([int]$timeDiff) seconds!" -ForegroundColor Red
            throw "Package is stale - source: $sourceTime, package: $packageTime"
        }
        Write-Host "[*] Package verified fresh (timestamp diff: $([int]$timeDiff)s)" -ForegroundColor Green
    } else
    {
        throw "$ProjectRoot/package/leyline.sys not found - packaging failed"
    }

    # Locate DevGen and DevCon on host to bundle them (Ensuring we match the 28000 environment)
    $devgenHost = $null
    $devconHost = $null
    $possibleEwdk = @("D:\eWDK_28000", $env:eWDK_ROOT_DIR, "C:\Users\Slate\Downloads\EWDK_br_release_28000_251103-1709")
    foreach ($p in $possibleEwdk)
    {
        if ($p -and (Test-Path $p))
        {
            # Find devgen.exe
            if (-not $devgenHost)
            {
                $found = Get-ChildItem -Path $p -Filter "devgen.exe" -Recurse | Where-Object { $_.FullName -match "x64" } | Select-Object -First 1
                if ($found)
                { $devgenHost = $found
                }
            }
            # Find devcon.exe
            if (-not $devconHost)
            {
                $found = Get-ChildItem -Path $p -Filter "devcon.exe" -Recurse | Where-Object { $_.FullName -match "x64" } | Select-Object -First 1
                if ($found)
                { $devconHost = $found
                }
            }
            # Break if both found
            if ($devgenHost -and $devconHost)
            { break
            }
        }
    }

    if ($devgenHost)
    {
        Write-Host "[*] Bundling DevGen from: $($devgenHost.FullName)"
        Copy-Item $devgenHost.FullName "$ProjectRoot/package\devgen.exe" -Force
    }

    if ($devconHost)
    {
        Write-Host "[*] Bundling DevCon from: $($devconHost.FullName)"
        Copy-Item $devconHost.FullName "$ProjectRoot/package\devcon.exe" -Force
    } else
    {
        Write-Host "[WARNING] DevCon not found. Root\Media mode will not be available on VM." -ForegroundColor Yellow
    }

    # 3. Remote Provisioning
    Write-Host "--- [2/3] Deploying to VM: $VMName ---" -ForegroundColor Cyan

    # Create remote directory
    $remotePath = "C:\LeylineInstall"
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path, $UseRootMedia)
        if (Test-Path $path)
        { Remove-Item $path -Recurse -Force
        }
        New-Item -ItemType Directory -Path $path -Force | Out-Null
    } -ArgumentList $remotePath, $UseRootMedia

    # Copy package to VM
    Write-Host "[*] Copying driver files..."
    Copy-Item -Path "$ProjectRoot/package\*" -Destination $remotePath -ToSession $vmsess -Recurse -Force

    # 4. Remote Execution
    Write-Host "--- [3/3] Executing Remote Installation ---" -ForegroundColor Cyan
    Invoke-Command -Session $vmsess -ScriptBlock {
        param($path, $UseRootMedia)
        Set-Location $path
        $ErrorActionPreference = "Stop"

        Write-Host "[VM] Enabling Testsigning..."
        bcdedit /set testsigning on | Out-Null

        Write-Host "[VM] Enabling Kernel Debug Prints (DbgPrint)..."
        $regPath = "HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Debug Print Filter"
        if (-not (Test-Path $regPath))
        { New-Item -Path $regPath -Force | Out-Null
        }
        New-ItemProperty -Path $regPath -Name "DEFAULT" -Value 0xFFFFFFFF -PropertyType DWORD -Force | Out-Null

        Write-Host "[VM] Installing Certificates (Root and TrustedPublisher)..."
        # Using -f to force and -user for current user if machine store is restrictive
        certutil -addstore -f root leyline.cer | Out-Null
        certutil -addstore -f TrustedPublisher leyline.cer | Out-Null

        Write-Host "[VM] Cleaning old instances..."
        Get-PnpDevice -PresentOnly:$false | Where-Object { $_.HardwareID -contains "Root\Media\LeylineAudio" -or $_.HardwareID -contains "Root\LeylineAudio" } | ForEach-Object {
            pnputil /remove-device $_.InstanceId | Out-Null
        }

        Write-Host "[VM] Staging Driver..."
        $stageResult = pnputil /add-driver "leyline.inf" /install
        Write-Host "    -> $stageResult"

        # Session #42: Support both SWD\DEVGEN (default) and ROOT\MEDIA (experimental) enumeration modes
        if ($UseRootMedia)
        {
            Write-Host "[VM] [ROOT_MEDIA MODE] Creating device with devcon.exe install..."
            if (Test-Path "devcon.exe")
            {
                $devconResult = .\devcon.exe install "leyline.inf" "Root\Media\LeylineAudio" 2>&1
                Write-Host "    -> Devcon result: $devconResult"
            } else
            {
                Write-Host "    -> [ERROR] devcon.exe missing in package. Cannot use Root\Media mode." -ForegroundColor Red
            }
        } else
        {
            Write-Host "[VM] [SWD_DEVGEN MODE] Creating Device Node with DevGen (default)..."
            if (Test-Path "devgen.exe")
            {
                .\devgen.exe /add /hardwareid "Root\Media\LeylineAudio" | Out-Null
                Write-Host "    -> Device node created."
            } else
            {
                Write-Host "    -> [WARNING] devgen.exe missing in package. Device node not created." -ForegroundColor Yellow
            }
        }

        Write-Host "[VM] Success. Check Device Manager." -ForegroundColor Green
    } -ArgumentList $remotePath, $UseRootMedia

    Write-Host "`n[SUCCESS] Deployment to $VMName complete." -ForegroundColor Green

} finally
{
    if ($vmsess)
    { Remove-PSSession $vmsess
    }
    Set-Location $initialDir
}
