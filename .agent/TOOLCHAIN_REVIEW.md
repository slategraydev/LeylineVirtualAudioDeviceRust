# Toolchain Review: Leyline Audio Driver

**Date**: February 15, 2026
**Status**: SESSION #14 COMPLETE
**Reviewer**: Antigravity (Gemini 3 Pro)

## Required Toolchain Requirements

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.1.1+)
- **LLVM Version**: 17.0.6 (Contained in `D:\eWDK_28000\LLVM`)
- **Environment Variable**: `LIBCLANG_PATH` (Set to `D:\eWDK_28000\LLVM\bin`)
- **Environment Variable**: `WDK_ROOT` (Set via eWDK)
- **Linker Requirement**: `/NODEFAULTLIB:msvcrt` (Enforced in `build.rs`)

### 2. Build Automation
- **Master Script**: `scripts/LaunchBuildEnv.ps1`
- **Task Runner**: `cargo-make` (version 0.37.x+)

### 3. Hardware Support App (.NET)
- **SDK**: .NET 8.0
- **Workload**: `microsoft.net.sdk.maui` (for WinUI 3 dependencies if applicable, though using standard SDK here)
- **Tooling**: `Microsoft.WindowsAppSDK` (version 1.5+)

---

## Required Environment Variables & PATHs
- **eWDK Root**: `D:\eWDK_28000`
- **LLVM Bin**: `D:\eWDK_28000\LLVM\bin`
- **PATH**: Includes eWDK bin directories (sourced via `LaunchBuildEnv.ps1`).

---
*Last Updated: February 15, 2026*
