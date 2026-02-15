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

### 4. Driver Packaging & Deployment
- **Scripts**: `scripts/package_driver.ps1`, `scripts/install_driver.ps1`, `scripts/uninstall_driver.ps1`.
- **Mandatory Tools**: `inf2cat.exe`, `signtool.exe`, `certutil.exe`, `pnputil.exe`.
- **Status**: Verified fully functional.

---

## Required Environment Variables & PATHs

```powershell
# Verification Script
$env:LIBCLANG_PATH = "C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\Llvm\x64\bin"
Write-Host "Verifying Toolchains..."
# [Output from Session #09: ✅ Cargo: Found, ✅ .NET: Found, ✅ Inf2Cat: Found, ✅ SignTool: Found]
```

---
*Last Updated: February 14, 2026*
