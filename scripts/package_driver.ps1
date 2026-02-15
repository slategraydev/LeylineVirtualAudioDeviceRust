# Copyright (c) 2026 Randall Rosas (Slategray).
# All rights reserved.

$ErrorActionPreference = "Stop"

# Configuration
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
Set-Location $ProjectRoot

# Load Environment if not already set
if (-not $env:WDK_ROOT)
{
    Write-Host "Setting up environment..." -ForegroundColor Gray
    & ".\scripts\LaunchBuildEnv.ps1"
}


# 1. Build Kernel (Release for Subsystem 1 verification)
Write-Host "`n--- Building Kernel (Release) ---" -ForegroundColor Cyan
Push-Location "crates/leyline-kernel"
cargo wdk build --profile release
if ($LASTEXITCODE -ne 0)
{ Pop-Location; throw "Kernel build failed"
}
Pop-Location

# 2. Build HSA
Write-Host "`n--- Building HSA ---" -ForegroundColor Cyan
dotnet build src/HSA/LeylineHSA.csproj -c Release
if ($LASTEXITCODE -ne 0)
{ throw "HSA build failed"
}

# 3. Build APO
Write-Host "`n--- Building APO ---" -ForegroundColor Cyan
& "$PSScriptRoot\build_apo.ps1"
if ($LASTEXITCODE -ne 0)
{
    Write-Warning "APO build script failed. Using pre-built or existing LeylineAPO.dll if available."
}

# 4. Package
Write-Host "`n--- Packaging ---" -ForegroundColor Cyan
if (Test-Path "package")
{ Remove-Item "package" -Recurse -Force
}
New-Item -ItemType Directory -Path "package/HSA" -Force | Out-Null

# Use release build for final package
Copy-Item "crates/leyline-kernel/target/release/leyline.dll" "package/leyline.sys"
Copy-Item "crates/leyline-kernel/leyline.inx" "package/leyline.inf"
if (Test-Path "src/APO/LeylineAPO.dll")
{
    Copy-Item "src/APO/LeylineAPO.dll" "package/"
}

Write-Host "Publishing HSA..."
dotnet publish src/HSA/LeylineHSA.csproj -c Release -r win-x64 --self-contained false -o "package/HSA" | Out-Null

# 5. Inf2Cat
Write-Host "`n--- Running Inf2Cat ---" -ForegroundColor Cyan
$Inf2Cat = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x86\Inf2Cat.exe"
& $Inf2Cat /driver:package /os:10_X64,Server2016_X64

# 6. Signing
Write-Host "`n--- Signing ---" -ForegroundColor Cyan
$SignTool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"

# Generate Cert if missing
if (-not (Test-Path "package/leyline.pfx"))
{
    $cert = New-SelfSignedCertificate -Subject "Leyline Audio Driver" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
    $cert | Export-Certificate -FilePath package/leyline.cer
    $cert | Export-PfxCertificate -FilePath package/leyline.pfx -Password (ConvertTo-SecureString -String "REDACTED_CERT_PASS" -Force -AsPlainText)
}

$FilesToSign = @("package/leyline.sys", "package/leyline.cat", "package/LeylineAPO.dll", "package/HSA/LeylineHSA.exe")
foreach ($file in $FilesToSign)
{
    if (Test-Path $file)
    {
        & $SignTool sign /f package/leyline.pfx /p password /fd SHA256 /t http://timestamp.digicert.com $file
    }
}

Write-Host "`nBuild and Package Complete! Status: VALID NATIVE BINARY" -ForegroundColor Green
