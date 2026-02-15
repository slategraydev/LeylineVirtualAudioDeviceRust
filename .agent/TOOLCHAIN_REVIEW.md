# Toolchain Review: Leyline Audio Driver

**Date**: February 14, 2026
**Status**: ACTIVE MANAGEMENT
**Reviewer**: Antigravity (Gemini 3 Pro (High))

## Required Toolchain Requirements

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.2.0+)
- **LLVM Version**: 17.0.6
- **Environment Variable**: `LIBCLANG_PATH` = `C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\Llvm\x64\bin`

### 2. Hardware Support App (WinUI 3 / C#)
- **Mandatory Framework**: .NET 8.0 SDK

### 3. APO Component (C++)
- **Mandatory Toolchain**: Visual Studio 2022 C++ Build Tools (cl.exe, nmake)
- **Note**: `package_driver.ps1` degrades gracefully if `cl.exe` is missing (uses pre-built DLL).

### 4. Driver Packaging
- **Script**: `scripts/package_driver.ps1`
- **Mandatory Tools**: `inf2cat.exe`, `signtool.exe`.
- **Status**: Verified fully functional in Session #08.

---

## Required Environment Variables & PATHs

```powershell
# Verification Script
$env:LIBCLANG_PATH = "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\Llvm\x64\bin"
Write-Host "Verifying Toolchains..."
if (Get-Command cargo -ErrorAction SilentlyContinue) { Write-Host "✅ Cargo: Found" } else { Write-Warning "❌ Cargo: Missing" }
if (Get-Command dotnet -ErrorAction SilentlyContinue) { Write-Host "✅ .NET: Found" } else { Write-Warning "❌ .NET: Missing" }
if (Test-Path "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x86\Inf2Cat.exe") { Write-Host "✅ Inf2Cat: Found" } else { Write-Warning "❌ Inf2Cat: Missing" }
```

---
*Last Updated: February 14, 2026*
