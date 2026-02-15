# Toolchain Review: Leyline Audio Driver

**Date**: February 15, 2026
**Status**: SESSION #13 COMPLETE
**Reviewer**: Antigravity (Gemini 3 Pro)

## Required Toolchain Requirements

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.1.1+)
- **LLVM Version**: 17.0.6 (Contained in `D:\eWDK_28000\LLVM`)
- **Environment Variable**: `LIBCLANG_PATH` (Set to `D:\eWDK_28000\LLVM\bin`)
- **Environment Variable**: `WDK_ROOT` (Set via eWDK)

### 2. Build Automation
- **Master Script**: `scripts/LaunchBuildEnv.ps1`
- **Task Runner**: `cargo-make` (version 0.37.x+)

---

## Required Environment Variables & PATHs
- **eWDK Root**: `D:\eWDK_28000`
- **LLVM Bin**: `D:\eWDK_28000\LLVM\bin`
- **PATH**: Must include the eWDK bin directories (set via `LaunchBuildEnv.ps1`).

---
*Last Updated: February 15, 2026*
