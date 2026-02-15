# Toolchain Review: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 3 Pro)
**Date**: February 15, 2026
**Status**: SESSION #19 COMPLETE

## Required Toolchain Requirements

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.1.1+)
- **LLVM Version**: 17.0.6 (Contained in `D:\eWDK_28000\LLVM`)
- **Environment Variable**: `LIBCLANG_PATH` (Set to `D:\eWDK_28000\LLVM\bin`)
- **Linker Requirement**: `/NODEFAULTLIB:msvcrt` (Enforced in `build.rs`)

### 2. Audio Processing Object (C++)
- **Compiler**: `cl.exe` (via eWDK / VC Tools)
- **Make Tool**: `nmake.exe`
- **Environment**: 
    - **Preferred**: `vcvarsall.bat x64`
    - **Fallback**: Manual `INCLUDE` and `LIB` paths for SDK 10.0.28000.0 and MSVC 14.44.35207 (Implemented in `build_apo.ps1`).

### 3. Build Automation
- **Master Script**: `scripts/LaunchBuildEnv.ps1` (Kernel Env)
- **APO Script**: `scripts/build_apo.ps1` (C++ Env)
- **Task Runner**: `cargo-make` (version 0.37.x+)

### 4. Hardware Support App (.NET)
- **SDK**: .NET 8.0
- **Tooling**: `Microsoft.WindowsAppSDK` (version 1.5+)
