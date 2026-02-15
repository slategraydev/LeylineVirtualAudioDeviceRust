# Toolchain Review: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 3 Pro)
**Date**: February 15, 2026
**Status**: SESSION #18 COMPLETE

## Required Toolchain Requirements

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.1.1+)
- **LLVM Version**: 17.0.6 (Contained in `D:\eWDK_28000\LLVM`)
- **Environment Variable**: `LIBCLANG_PATH` (Set to `D:\eWDK_28000\LLVM\bin`)
- **Linker Requirement**: `/NODEFAULTLIB:msvcrt` (Enforced in `build.rs`)

### 2. Audio Processing Object (C++)
- **Compiler**: `cl.exe` (via eWDK / VC Tools)
- **Make Tool**: `nmake.exe`
- **Environment**: Must be initialized via `vcvarsall.bat x64` to set `INCLUDE` and `LIB` paths for User Mode headers (`windows.h`, `audioenginebaseapo.h`).
- **Verified Path**: `D:\eWDK_28000\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvarsall.bat`

### 3. Build Automation
- **Master Script**: `scripts/LaunchBuildEnv.ps1` (Kernel Env)
- **APO Script**: `scripts/build_apo.ps1` (C++ Env)
- **Task Runner**: `cargo-make` (version 0.37.x+)

### 4. Hardware Support App (.NET)
- **SDK**: .NET 8.0
- **Tooling**: `Microsoft.WindowsAppSDK` (version 1.5+)
