# Leyline Audio: UBER INSTALLER (Build + Install)
# MUST be run as Administrator.

param (
    [switch]$clean,
    [switch]$build,
    [switch]$package,
    [switch]$install
)

# Default behavior: If no switches provided, do everything
if (-not ($build -or $package -or $install)) {
    $build = $true; $package = $true; $install = $true
}

$ErrorActionPreference = "Stop"
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
Push-Location $ProjectRoot

try {
    # ... (Admin and Testsigning checks remain) ...
    # 0. Administrator Guard
    $currentPrincipal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
    if (-not $currentPrincipal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw "This script MUST be run as Administrator."
    }

    # 0.1 Check Testsigning Status
    $testSigning = bcdedit /enum "{current}" | Select-String "testsigning\s+Yes"
    if (-not $testSigning) {
        Write-Host "[!] Test-signing is NOT enabled. Enabling now..." -ForegroundColor Yellow
        bcdedit /set testsigning on
        Write-Host "[CRITICAL] Test-signing enabled. YOU MUST REBOOT YOUR PC before this script can install drivers." -ForegroundColor Red
        return
    }

    Write-Host "`n--- [1/5] Initializing eWDK Environment ---" -ForegroundColor Cyan
    # ... (rest of environment setup remains the same) ...
    if (-not $env:WDK_ROOT) {
        $possiblePaths = @(
            "D:\eWDK_28000",
            "C:\Users\Slate\Downloads\EWDK_br_release_28000_251103-1709",
            "D:\"
        )
        
        $ewdkRoot = $null
        foreach ($p in $possiblePaths) {
            if (Test-Path (Join-Path $p "BuildEnv\SetupBuildEnv.cmd")) {
                $ewdkRoot = $p
                break
            }
        }
        
        if (-not $ewdkRoot) {
            Write-Host "[!] eWDK build environment (SetupBuildEnv.cmd) not found in expected locations." -ForegroundColor Red
            throw "eWDK not found. Checked: $($possiblePaths -join ', ')"
        }
        
        $env:eWDK_ROOT_DIR = $ewdkRoot
        Write-Host "[*] Using eWDK at: $ewdkRoot" -ForegroundColor Gray
        $cmd = "`"$ewdkRoot\BuildEnv\SetupBuildEnv.cmd`" amd64 10.0.28000.0 && set"
        $envVars = cmd.exe /c $cmd
        foreach ($line in $envVars) {
            if ($line -match "^([^=]+)=(.*)$") {
                $name = $matches[1]; $value = $matches[2]
                if ($name -eq "PATH") { $env:PATH = $value + ";" + $env:PATH }
                else { [System.Environment]::SetEnvironmentVariable($name, $value, "Process") }
            }
        }

        # SDK Path Injection (Rust Fix)
        $sdkLibRoot = "$ewdkRoot\Program Files\Windows Kits\10\Lib\$env:WindowsTargetPlatformVersion"
        $sdkIncRoot = "$ewdkRoot\Program Files\Windows Kits\10\Include\$env:WindowsTargetPlatformVersion"
        $env:LIB += ";$sdkLibRoot\um\x64;$sdkLibRoot\km\x64;$sdkLibRoot\ucrt\x64"
        $env:INCLUDE += ";$sdkIncRoot\um;$sdkIncRoot\km;$sdkIncRoot\ucrt;$sdkIncRoot\shared"
        
        $llvmPath = "$ewdkRoot\LLVM\bin"
        if (Test-Path $llvmPath) { $env:LIBCLANG_PATH = $llvmPath; $env:PATH = "$llvmPath;" + $env:PATH }

        $env:WDK_ROOT = $env:WDKContentRoot
        $binRoot = Join-Path $env:WDK_ROOT "bin\$env:WindowsTargetPlatformVersion"
        $env:INF2CAT_EXE = Join-Path $binRoot "x86\Inf2Cat.exe"
        $env:SIGNTOOL_EXE = Join-Path $binRoot "x64\signtool.exe"
        
        # Fixed Devcon Path for eWDK 28000
        $env:DEVCON_EXE = Join-Path $env:eWDK_ROOT_DIR "Program Files\Windows Kits\10\Tools\$env:WindowsTargetPlatformVersion\x64\devcon.exe"
        
        # Add DevGen Path (Modern replacement for devcon install)
        $env:DEVGEN_EXE = Join-Path $env:eWDK_ROOT_DIR "Program Files\Windows Kits\10\Tools\$env:WindowsTargetPlatformVersion\x64\devgen.exe"
    }

    Write-Host "--- [2/5] Executing Compilations ---" -ForegroundColor Cyan

    if ($clean) {
        Write-Host "[!] Performing Deep Clean of all build artifacts and legacy devices..." -ForegroundColor Yellow
        # Kernel Clean
        Push-Location "crates/leyline-kernel"; cargo clean; Pop-Location
        # APO Clean
        Push-Location "src/APO"; nmake /f Makefile clean; Pop-Location
        # HSA Clean
        dotnet clean src/HSA/LeylineHSA.csproj -c Release | Out-Null
        # Package Purge
        if (Test-Path "package") { Remove-Item "package" -Recurse -Force }
        
        # System State Purge: Remove existing and legacy devices to start from a truly blank slate
        $legacyIds = @("Root\LeylineAudio", "Root\simpleaudiosample", "Root\SimpleAudioDriver")
        Get-PnpDevice -PresentOnly:$false | Where-Object { 
            $hwid = $_.HardwareID
            $match = $false
            foreach ($id in $legacyIds) { if ($hwid -contains $id) { $match = $true; break } }
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

    if ($build) {
        Write-Host "--- [2/5] Executing Compilations ---" -ForegroundColor Cyan
        # Kernel
        Write-Host "[*] Building Kernel..."
        Push-Location "crates/leyline-kernel"; cargo wdk build --profile release || throw "Kernel Build Failed"; Pop-Location
        # HSA
        Write-Host "[*] Building HSA..."
        dotnet build src/HSA/LeylineHSA.csproj -c Release /p:Version=$Version || throw "HSA Build Failed"
        # APO
        Write-Host "[*] Building APO..."
        Push-Location "src/APO"; nmake /f Makefile || throw "APO Build Failed"; Pop-Location
    }

    if ($package) {
        Write-Host "--- [3/5] Packaging & Signing ---" -ForegroundColor Cyan
        if (Test-Path "package") { Remove-Item "package" -Recurse -Force }
        New-Item -ItemType Directory -Path "package/HSA" -Force | Out-Null
        
        # Kernel: The Rust build produces a .dll (cdylib), but Windows drivers MUST be .sys
        $kernelOutput = "crates/leyline-kernel/target/release/leyline.dll"
        if (-not (Test-Path $kernelOutput)) {
            # Fallback check for .sys just in case environment handles it
            $kernelOutput = "crates/leyline-kernel/target/release/leyline.sys"
        }
        
        if (-not (Test-Path $kernelOutput)) { throw "Kernel build artifact NOT FOUND at $kernelOutput" }
        
        Write-Host "[*] Packaging Kernel: $kernelOutput -> package/leyline.sys"
        Copy-Item $kernelOutput "package/leyline.sys"
        Copy-Item "crates/leyline-kernel/leyline.inx" "package/leyline.inf"
        Copy-Item "src/APO/LeylineAPO.dll" "package/"
        dotnet publish src/HSA/LeylineHSA.csproj -c Release -r win-x64 --self-contained false -o "package/HSA" | Out-Null

        # Generate Catalog and Sign
        & $env:INF2CAT_EXE /driver:package /os:10_X64,Server2016_X64
        if (-not (Test-Path "package\leyline.pfx")) {
            $cert = New-SelfSignedCertificate -Subject "Leyline Audio Driver" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
            $cert | Export-PfxCertificate -FilePath package\leyline.pfx -Password (ConvertTo-SecureString -String "REDACTED_CERT_PASS" -Force -AsPlainText)
            $cert | Export-Certificate -FilePath package\leyline.cer
        }
        foreach ($f in @("package\leyline.sys", "package\leyline.cat", "package\LeylineAPO.dll", "package\HSA\LeylineHSA.exe")) {
            if (Test-Path $f) { & $env:SIGNTOOL_EXE sign /f package\leyline.pfx /p password /fd SHA256 /t http://timestamp.digicert.com $f }
        }
    }

    if ($install) {
        Write-Host "--- [4/5] System Provisioning ---" -ForegroundColor Cyan
        certutil -addstore root package\leyline.cer | Out-Null
        certutil -addstore TrustedPublisher package\leyline.cer | Out-Null

        Write-Host "--- [5/5] PnP Driver Installation & Verification ---" -ForegroundColor Cyan
        
        # 1. Clean up any existing instances to avoid duplicates
        Write-Host "[*] Checking for existing Leyline instances..."
        $existing = Get-PnpDevice -PresentOnly:$false | Where-Object { $_.HardwareID -contains "Root\LeylineAudio" }
        foreach ($dev in $existing) {
            Write-Host "    -> Removing old instance: $($dev.InstanceId)"
            pnputil /remove-device $dev.InstanceId | Out-Null
        }

        # 2. Create the software device node using DevGen (Modern way)
        Write-Host "[*] Creating software device node with DevGen..."
        if (Test-Path $env:DEVGEN_EXE) {
            & $env:DEVGEN_EXE /add /hardwareid "Root\LeylineAudio" | Out-Null
        } else {
            throw "devgen.exe NOT FOUND at $env:DEVGEN_EXE"
        }

        # 3. Install the driver package and associate it with the node
        Write-Host "[*] Installing driver package and associating with device..."
        $stageResult = pnputil /add-driver "package\leyline.inf" /install
        Write-Host "    -> $stageResult"

        # Final Verification
        Start-Sleep -Seconds 2
        $finalDevices = Get-PnpDevice -PresentOnly:$true | Where-Object { $_.HardwareID -contains "Root\LeylineAudio" }
        
        if ($finalDevices) {
            foreach ($dev in $finalDevices) {
                Write-Host "`n[SUCCESS] Found active Leyline device: $($dev.FriendlyName)" -ForegroundColor Green
                Write-Host "          Instance ID: $($dev.InstanceId)"
                Write-Host "          Status: $($dev.Status)"
                
                if ($dev.Status -ne "OK") {
                    Write-Host "`n################################################################" -ForegroundColor Red
                    Write-Host "# [CRITICAL] DEVICE ERROR: $($dev.Status)                      #" -ForegroundColor Red
                    Write-Host "# The driver is installed but failed to start.                 #" -ForegroundColor Red
                    Write-Host "# Check Event Viewer or run 'pnputil /enum-devices /problem'.  #" -ForegroundColor Red
                    Write-Host "################################################################`n" -ForegroundColor Red
                }
            }
        } else {
            Write-Host "`n[ERROR] Driver installed, but no active device node was found!" -ForegroundColor Red
        }

        Write-Host "`n[SUCCESS] Leyline Audio $Version Built & Installed." -ForegroundColor Green
    }
} finally {
    Pop-Location
}
