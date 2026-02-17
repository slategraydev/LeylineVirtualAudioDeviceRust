# Leyline Audio: UBER INSTALLER (Build + Install)
# MUST be run as Administrator.

param (
    [switch]$clean,
    [switch]$build,
    [switch]$package,
    [switch]$install,
    [switch]$UseRootMedia
)

# Default behavior: If no switches provided, do everything
if (-not ($build -or $package -or $install))
{
    $build = $true; $package = $true; $install = $true
}

# Session #42: Root\Media Enumeration Mode
# By default, use DevGen (SWD\DEVGEN). Use -UseRootMedia to test alternative enumeration.


$ErrorActionPreference = "Stop"
$initialDir = Get-Location
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
Push-Location $ProjectRoot

try
{
    # ... (Admin and Testsigning checks remain) ...
    # 0. Administrator Guard
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator))
    {
        throw "This script MUST be run as Administrator."
    }

    # 0.1 Check Testsigning Status
    $testSigning = bcdedit /enum "{current}" | Select-String "testsigning\s+Yes"
    if (-not $testSigning)
    {
        Write-Host "[!] Test-signing is NOT enabled. Enabling now..." -ForegroundColor Yellow
        bcdedit /set testsigning on
        Write-Host "[CRITICAL] Test-signing enabled. YOU MUST REBOOT YOUR PC before this script can install drivers." -ForegroundColor Red
        return
    }

    Write-Host "`n--- [1/5] Initializing eWDK Environment ---" -ForegroundColor Cyan

    # Use the robust LaunchBuildEnv.ps1 to set up the environment
    . "$PSScriptRoot\LaunchBuildEnv.ps1"

    if ($env:WDK_ROOT)
    {
        $binRoot = Join-Path $env:WDK_ROOT "bin\$env:WindowsTargetPlatformVersion"
        $env:INF2CAT_EXE = Join-Path $binRoot "x86\Inf2Cat.exe"
        $env:SIGNTOOL_EXE = Join-Path $binRoot "x64\signtool.exe"

        # Fixed Devcon Path for eWDK 28000
        $env:DEVCON_EXE = Join-Path $env:eWDK_ROOT_DIR "Program Files\Windows Kits\10\Tools\$env:WindowsTargetPlatformVersion\x64\devcon.exe"

        # Add DevGen Path (Modern replacement for devcon install)
        $env:DEVGEN_EXE = Join-Path $env:eWDK_ROOT_DIR "Program Files\Windows Kits\10\Tools\$env:WindowsTargetPlatformVersion\x64\devgen.exe"
    }


    Write-Host "--- [2/5] Executing Compilations ---" -ForegroundColor Cyan

    if ($clean)
    {
        Write-Host "[!] Performing Deep Clean of all build artifacts and legacy devices..." -ForegroundColor Yellow
        # Kernel Clean (Release Only)
        Push-Location "crates/leyline-kernel"; cargo clean --release; Pop-Location
        # APO Clean
        Push-Location "src/APO"; nmake /f Makefile clean; Pop-Location
        # HSA Clean
        dotnet clean src/HSA/LeylineHSA.csproj -c Release | Out-Null
        # Package Purge
        if (Test-Path "$ProjectRoot/package")
        { Remove-Item "$ProjectRoot/package" -Recurse -Force
        }

        # System State Purge: Remove existing and legacy devices to start from a truly blank slate
        $legacyIds = @("Root\LeylineAudio", "Root\simpleaudiosample", "Root\SimpleAudioDriver")
        Get-PnpDevice -PresentOnly:$false | Where-Object {
            $hwid = $_.HardwareID
            $match = $false
            foreach ($id in $legacyIds)
            { if ($hwid -contains $id)
                { $match = $true; break
                }
            }
            $match
        } | ForEach-Object {
            Write-Host "    -> Scrubbing Device Instance: $($_.InstanceId) ($($_.FriendlyName))"
            pnputil /remove-device $_.InstanceId | Out-Null
        }
    }

    # Version Update (Increments every build to force Windows to accept the update)
    $Version = "1.0.1.$( (Get-Date).Hour * 100 + (Get-Date).Minute )"
    Write-Host "[*] Building Version: $Version" -ForegroundColor Cyan
    (Get-Content "crates/leyline-kernel/leyline.inx") -replace "DriverVer\s*=.*", "DriverVer   = $(Get-Date -Format 'MM/dd/yyyy'),$Version" | Set-Content "crates/leyline-kernel/leyline.inx"

    if ($build)
    {
        Write-Host "--- [2/5] Executing Compilations ---" -ForegroundColor Cyan

        # 1. Kernel Build
        Write-Host "[*] Building Kernel..."
        Push-Location "crates/leyline-kernel"
        cargo wdk build --profile release
        if ($LASTEXITCODE -ne 0)
        { throw "Kernel Build Failed"
        }
        Pop-Location

        # 2. HSA Build
        Write-Host "[*] Building HSA..."
        dotnet build src/HSA/LeylineHSA.csproj -c Release /p:Version=$Version
        if ($LASTEXITCODE -ne 0)
        { throw "HSA Build Failed"
        }

        # 3. APO Build
        Write-Host "[*] Building APO..."
        Push-Location "src/APO"
        # Environment is already set by LaunchBuildEnv.ps1, so we can run nmake directly
        nmake /f Makefile
        if ($LASTEXITCODE -ne 0)
        { throw "APO Build Failed with code $LASTEXITCODE"
        }
        Pop-Location
    }

    if ($package)
    {
        Write-Host "--- [3/5] Packaging & Signing ---" -ForegroundColor Cyan
        if (Test-Path "$ProjectRoot/package")
        { Remove-Item "$ProjectRoot/package" -Recurse -Force
        }
        New-Item -ItemType Directory -Path "$ProjectRoot/package/HSA" -Force | Out-Null

        # Kernel: The Rust build produces a .dll (cdylib), but Windows drivers MUST be .sys
        # Note: In a workspace, cargo puts output in workspace root target/, not crate target/
        $kernelOutput = "target/release/leyline.dll"
        if (-not (Test-Path $kernelOutput))
        {
            # Fallback check for .sys just in case environment handles it
            $kernelOutput = "target/release/leyline.sys"
        }

        if (-not (Test-Path $kernelOutput))
        { throw "Kernel build artifact NOT FOUND at $kernelOutput"
        }

        Write-Host "[*] Packaging Kernel: $kernelOutput -> $ProjectRoot/package/leyline.sys"
        Copy-Item $kernelOutput "$ProjectRoot/package/leyline.sys"
        Copy-Item "$ProjectRoot/crates/leyline-kernel/leyline.inx" "$ProjectRoot/package/leyline.inf"
        Copy-Item "$ProjectRoot/src/APO/LeylineAPO.dll" "$ProjectRoot/package/"
        dotnet publish "$ProjectRoot/src/HSA/LeylineHSA.csproj" -c Release -r win-x64 --self-contained false -o "$ProjectRoot/package/HSA" | Out-Null

        # Generate Catalog and Sign
        & $env:INF2CAT_EXE /driver:"$ProjectRoot/package" /os:10_X64,Server2016_X64
        if (-not (Test-Path "$ProjectRoot/package\leyline.pfx"))
        {
            $cert = New-SelfSignedCertificate -Subject "Leyline Audio Driver" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
            $cert | Export-PfxCertificate -FilePath "$ProjectRoot/package\leyline.pfx" -Password (ConvertTo-SecureString -String "REDACTED_CERT_PASS" -Force -AsPlainText)
            $cert | Export-Certificate -FilePath "$ProjectRoot/package\leyline.cer"
        }
        foreach ($f in @("$ProjectRoot/package\leyline.sys", "$ProjectRoot/package\leyline.cat", "$ProjectRoot/package\LeylineAPO.dll", "$ProjectRoot/package\HSA\LeylineHSA.exe"))
        {
            if (Test-Path $f)
            { & $env:SIGNTOOL_EXE sign /f "$ProjectRoot/package\leyline.pfx" /p password /fd SHA256 /t http://timestamp.digicert.com $f
            }
        }
    }

    if ($install)
    {
        Write-Host "--- [4/5] System Provisioning ---" -ForegroundColor Cyan
        certutil -addstore root "$ProjectRoot/package\leyline.cer" | Out-Null
        certutil -addstore TrustedPublisher "$ProjectRoot/package\leyline.cer" | Out-Null

        Write-Host "--- [5/5] PnP Driver Installation & Verification ---" -ForegroundColor Cyan

        # 1. Clean up any existing instances to avoid duplicates
        Write-Host "[*] Checking for existing Leyline instances..."
        $existing = Get-PnpDevice -PresentOnly:$false | Where-Object { $_.HardwareID -contains "Root\LeylineAudio" }
        foreach ($dev in $existing)
        {
            Write-Host "    -> Removing old instance: $($dev.InstanceId)"
            pnputil /remove-device $dev.InstanceId | Out-Null
        }

        # Session #42: Enumerator Selection
        # SWD\DEVGEN (default) vs ROOT\MEDIA (experimental for audio endpoint support)
        if ($UseRootMedia)
        {
            Write-Host "[*] [ROOT_MEDIA MODE] Creating device with devcon.exe install..." -ForegroundColor Cyan
            Write-Host "    This mode uses Root\\Media enumerator which may support audio endpoints." -ForegroundColor Gray
            if (Test-Path $env:DEVCON_EXE)
            {
                # Use devcon install to create a traditional ROOT\MEDIA enumerated device
                # This creates Instance ID like ROOT\MEDIA\0000 instead of SWD\DEVGEN\{GUID}
                $devconResult = & $env:DEVCON_EXE install "$ProjectRoot/package/leyline.inf" "Root\LeylineAudio" 2>&1
                Write-Host "    -> Devcon result: $devconResult" -ForegroundColor Gray
            } else
            {
                throw "devcon.exe NOT FOUND at $env:DEVCON_EXE. Cannot use Root\\Media mode."
            }
        } else
        {
            Write-Host "[*] [SWD_DEVGEN MODE] Creating software device node with DevGen (default)..." -ForegroundColor Cyan
            Write-Host "    Note: If endpoints don't appear, try -UseRootMedia switch." -ForegroundColor Gray
            if (Test-Path $env:DEVGEN_EXE)
            {
                & $env:DEVGEN_EXE /add /hardwareid "Root\LeylineAudio" | Out-Null
            } else
            {
                throw "devgen.exe NOT FOUND at $env:DEVGEN_EXE"
            }
        }

        # 3. Install the driver package (skip for Root\Media mode as devcon install already does this)
        if (-not $UseRootMedia)
        {
            Write-Host "[*] Installing driver package and associating with device..."
            $stageResult = pnputil /add-driver "$ProjectRoot/package/leyline.inf" /install
            Write-Host "    -> $stageResult"
        } else
        {
            Write-Host "[*] [ROOT_MEDIA MODE] Driver already staged by devcon install." -ForegroundColor Gray
            # Still need to ensure driver is in driver store
            $stageResult = pnputil /add-driver "$ProjectRoot/package/leyline.inf" 2>&1 | Out-Null
        }

        # Final Verification
        Start-Sleep -Seconds 2
        $finalDevices = Get-PnpDevice -PresentOnly:$true | Where-Object { $_.HardwareID -contains "Root\LeylineAudio" }

        if ($finalDevices)
        {
            foreach ($dev in $finalDevices)
            {
                Write-Host "`n[SUCCESS] Found active Leyline device: $($dev.FriendlyName)" -ForegroundColor Green
                Write-Host "          Instance ID: $($dev.InstanceId)"
                Write-Host "          Status: $($dev.Status)"

                if ($dev.Status -ne "OK")
                {
                    Write-Host "`n################################################################" -ForegroundColor Red
                    Write-Host "# [CRITICAL] DEVICE ERROR: $($dev.Status)                      #" -ForegroundColor Red
                    Write-Host "# The driver is installed but failed to start.                 #" -ForegroundColor Red
                    Write-Host "# Check Event Viewer or run 'pnputil /enum-devices /problem'.  #" -ForegroundColor Red
                    Write-Host "################################################################`n" -ForegroundColor Red
                }
            }
        } else
        {
            Write-Host "`n[ERROR] Driver installed, but no active device node was found!" -ForegroundColor Red
        }

        Write-Host "`n[SUCCESS] Leyline Audio $Version Built & Installed." -ForegroundColor Green
    }
} finally
{
    Set-Location $initialDir
}
