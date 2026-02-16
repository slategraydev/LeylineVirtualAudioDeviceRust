# Toolchain Review: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 1.5 Pro)
**Date**: February 15, 2026
**Status**: SESSION #25 COMPLETE

## Required Toolchain Requirements

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.1.1+)
- **LLVM Version**: 17.0.6 (Contained in `D:\eWDK_28000\LLVM`)
- **Environment Variable**: `LIBCLANG_PATH` (Set to `D:\eWDK_28000\LLVM\bin`)
- **Linker Requirement**: `/NODEFAULTLIB:msvcrt` (Enforced in `build.rs`)
- **ABI Note**: `extern "C"` mandated for PortCls FFI blocks.

### 2. Audio Processing Object (C++)
- **Compiler**: `cl.exe` (via eWDK / VC Tools)
- **Make Tool**: `nmake.exe`

### 3. Build Automation
- **Master Script**: `scripts/LaunchBuildEnv.ps1` (Kernel Env)
- **Task Runner**: `cargo-make` (version 0.37.x+)
- **PnP Management**: `devcon.exe` & `devgen.exe` (Mandatory eWDK 28000 paths). [FIXED]
    - `devcon.exe`: `D:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devcon.exe`
    - `devgen.exe`: `D:\eWDK_28000\Program Files\Windows Kits\10\Tools\10.0.28000.0\x64\devgen.exe`

### 4. Hardware Support App (.NET)
- **SDK**: .NET 8.0
- **Runtime Strategy**: Self-Contained (`WindowsAppSDKSelfContained=true`).
