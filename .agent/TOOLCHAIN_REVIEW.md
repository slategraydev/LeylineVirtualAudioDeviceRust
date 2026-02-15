# Toolchain Review: Leyline Audio Driver

**Date**: February 14, 2026  
**Status**: ACTIVE MANAGEMENT  
**Reviewer**: Antigravity (Gemini 3 Pro (High))
**Date**: February 14, 2026

## Required Toolchain Requirements

This document tracks the mandatory toolchains for the Leyline Audio Driver project. Agents MUST verify this file at the start of every session and ensure the environment is correctly configured before attempting any build or verification tasks.

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.2.0+)
- **Mandatory Tool**: `cargo-make` (version 0.37.0+)
- **LLVM Version**: 17.0.6 (Must be matched to WDK bindings)
- **PATH Requirement**: `C:\Program Files\Microsoft Visual Studio\2022\Professional\VC\Tools\Llvm\x64\bin`
- **Environment Variable**: `LIBCLANG_PATH` pointing to the above LLVM bin directory.

### 2. Hardware Support App (WinUI 3 / C#)
- **Mandatory Framework**: .NET 8.0 SDK
- **Mandatory Workload**: Windows App SDK (WinUI 3 templates)
- **PATH Requirement**: `dotnet.exe` must be accessible.

### 3. APO Component (C++)
- **Mandatory Toolchain**: Visual Studio 2022 C++ Build Tools
- **Mandatory SDK**: Windows 11 SDK / eWDK
- **Automation**: `scripts/package_driver.ps1` handles the build if run from a Developer Prompt.

### 4. Driver Packaging (Automated)
- **Script**: `scripts/package_driver.ps1`
- **Mandatory Tools**: `inf2cat.exe`, `signtool.exe`, `stampinf.exe`
- **Action**: The script locates and invokes these tools automatically.

---

## Toolchain Health Audit

| Tool | Category | Status | Path Configured? | Action Required |
| :--- | :--- | :---: | :---: | :--- |
| **cargo-wdk** | Rust/Kernel | ✅ | Yes | Verified in Session #05 (Build Success) |
| **cargo-make** | Workspace | ✅ | Yes | Verified in Session #05 |
| **LLVM 17.0.6** | Rust/Kernel | ✅ | Yes | Verified in Session #05 |
| **.NET SDK 8.0** | WinUI 3/HSA | ✅ | Yes | Verified in Session #03 |
| **MSVC (C++)** | APO | ✅ | Yes | Accessible via VS Professional |
| **Inf2Cat** | Packaging | ✅ | Auto | Script uses hardcoded path (checked) |
| **SignTool** | Packaging | ✅ | Auto | Script uses hardcoded path (checked) |

---

## Required Environment Variables & PATHs

Agents must run the following check in PowerShell to verify the session is "Build Ready":

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
