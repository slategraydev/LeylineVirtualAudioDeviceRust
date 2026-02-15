$ErrorActionPreference = "Stop"

# Configuration
$ProjectRoot = Resolve-Path "$PSScriptRoot\.."
Set-Location $ProjectRoot

# Tools
$Inf2Cat = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x86\Inf2Cat.exe"
$SignTool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64\signtool.exe"

# 1. Build Kernel
Write-Host "Building Kernel..."
$env:LIBCLANG_PATH = "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\Llvm\x64\bin"
cargo wdk build
if ($LASTEXITCODE -ne 0) { throw "Kernel build failed" }

# 2. Build HSA
Write-Host "Building HSA..."
dotnet build src/HSA/LeylineHSA.csproj -c Debug
if ($LASTEXITCODE -ne 0) { throw "HSA build failed" }

# 3. Build APO
Write-Host "Building APO..."
if (Get-Command "cl.exe" -ErrorAction SilentlyContinue) {
    Set-Location "src/APO"
    nmake
    if ($LASTEXITCODE -ne 0) { throw "APO build failed" }
    Set-Location $ProjectRoot
} else {
    Write-Warning "cl.exe not found in PATH. Skipping APO build (Requires VS Dev Prompt)."
    # Fail if DLL is missing, otherwise assume previous build
    if (-not (Test-Path "src/APO/LeylineAPO.dll")) {
        throw "APO DLL missing and build tools not found."
    }
}

# 4. Package
Write-Host "Packaging..."
if (Test-Path "package") { Remove-Item "package" -Recurse -Force }
New-Item -ItemType Directory -Path "package/HSA" -Force | Out-Null

Copy-Item "crates/leyline-kernel/target/debug/leyline.dll" "package/leyline.sys"
Copy-Item "crates/leyline-kernel/leyline.inx" "package/leyline.inf"
Copy-Item "src/APO/LeylineAPO.dll" "package/"

Write-Host "Publishing HSA..."
dotnet publish src/HSA/LeylineHSA.csproj -c Debug -r win-x64 --self-contained false -o "package/HSA" | Out-Null

# 5. Inf2Cat
Write-Host "Running Inf2Cat..."
& $Inf2Cat /driver:package /os:10_X64,Server2016_X64

# 6. Signing
Write-Host "Signing..."
# Check for PFX
if (-not (Test-Path "package/leyline.pfx")) {
    Write-Host "Generating Self-Signed Cert..."
    $cert = New-SelfSignedCertificate -Subject "Leyline Audio Driver" -Type CodeSigningCert -CertStoreLocation "Cert:\CurrentUser\My"
    $cert | Export-Certificate -FilePath package/leyline.cer
    $cert | Export-PfxCertificate -FilePath package/leyline.pfx -Password (ConvertTo-SecureString -String "REDACTED_CERT_PASS" -Force -AsPlainText)
}

$FilesToSign = @("package/leyline.sys", "package/leyline.cat", "package/LeylineAPO.dll", "package/HSA/LeylineHSA.exe")
foreach ($file in $FilesToSign) {
    if (Test-Path $file) {
        & $SignTool sign /f package/leyline.pfx /p password /fd SHA256 /t http://timestamp.digicert.com $file
    }
}

Write-Host "Build and Package Complete!"
