# Toolchain Review: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 3 Pro)
**Date**: February 15, 2026
**Status**: SESSION #15 COMPLETE

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
- **Workload**: `microsoft.net.sdk.maui` (if applicable)
- **Tooling**: `Microsoft.WindowsAppSDK` (version 1.5+)

---

## Required Environment Variables & PATHs
- **eWDK Root**: `D:\eWDK_28000`
- **LLVM Bin**: `D:\eWDK_28000\LLVM\bin`
- **PATH**: Includes eWDK bin directories (sourced via `LaunchBuildEnv.ps1`).

## Recent Issues & Considerations
- **Bindgen Complexity**: Nested anonymous unions in `IRP` struct are proving difficult to traverse with standard field paths. Using a direct `*mut _IO_STACK_LOCATION` calculation based on the `CurrentStackLocation` offset may be more reliable than field-by-field access.
