# Leyline Audio: BUILD ENVIRONMENT SETUP
# Does not build, just sets environment variables.

$ErrorActionPreference = "Stop"

if (-not $env:WDK_ROOT) {
    $possiblePaths = @(
        "D:\eWDK_28000",
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
        throw "eWDK not found. Checked: $($possiblePaths -join ', ')"
    }

    $env:eWDK_ROOT_DIR = $ewdkRoot
    Write-Host "[*] Using eWDK at: $ewdkRoot" -ForegroundColor Gray
    $cmd = "`"$ewdkRoot\BuildEnv\SetupBuildEnv.cmd`" amd64 10.0.28000.0 && set"
    $envVars = cmd.exe /c $cmd
    foreach ($line in $envVars) {
        if ($line -match "^([^=]+)=(.*)$") {
            $name = $matches[1]; $value = $matches[2]
            if ($name -eq "PATH") {
                $env:PATH = $value + ";" + $env:PATH 
            }
            else {
                [System.Environment]::SetEnvironmentVariable($name, $value, "Process") 
            }
        }
    }

    # SDK Path Injection
    $sdkLibRoot = "$ewdkRoot\Program Files\Windows Kits\10\Lib\$env:WindowsTargetPlatformVersion"
    $sdkIncRoot = "$ewdkRoot\Program Files\Windows Kits\10\Include\$env:WindowsTargetPlatformVersion"
    $env:LIB += ";$sdkLibRoot\um\x64;$sdkLibRoot\km\x64;$sdkLibRoot\ucrt\x64"
    $env:INCLUDE += ";$sdkIncRoot\um;$sdkIncRoot\km;$sdkIncRoot\ucrt;$sdkIncRoot\shared"

    $llvmPath = "$ewdkRoot\LLVM\bin"
    if (Test-Path $llvmPath) {
        $env:LIBCLANG_PATH = $llvmPath; $env:PATH = "$llvmPath;" + $env:PATH 
    }

    $env:WDK_ROOT = $env:WDKContentRoot
    
    # Tool Paths
    $kitsRoot = "$ewdkRoot\Program Files\Windows Kits\10"
    $env:INF2CAT_EXE = "$kitsRoot\bin\$env:WindowsTargetPlatformVersion\x86\Inf2Cat.exe"
    $env:SIGNTOOL_EXE = "$kitsRoot\bin\$env:WindowsTargetPlatformVersion\x64\signtool.exe"

    if (-not (Test-Path $env:INF2CAT_EXE)) { Write-Warning "Inf2Cat not found at $env:INF2CAT_EXE" }
    if (-not (Test-Path $env:SIGNTOOL_EXE)) { Write-Warning "SignTool not found at $env:SIGNTOOL_EXE" }

    Write-Host "[SUCCESS] Environment Variables Set." -ForegroundColor Green
}
