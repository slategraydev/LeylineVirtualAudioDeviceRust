# Toolchain Review: Leyline Audio Driver

**Date**: February 14, 2026
**Status**: SESSION #10 COMPLETE
**Reviewer**: Antigravity (Gemini 3 Pro (High))

## Required Toolchain Requirements

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.1.1+)
- **LLVM Version**: 17.0.6 (Mandatory for `bindgen`)
- **Environment Variable**: `LIBCLANG_PATH` (Set via `scripts/LaunchBuildEnv.ps1`)
- **Environment Variable**: `WDK_ROOT` (Set via `scripts/LaunchBuildEnv.ps1`)

### 2. Build Automation
- **Master Script**: `scripts/LaunchBuildEnv.ps1` (Locked to `D:\eWDK_28000`)
- **Package Script**: `scripts/package_driver.ps1`

### 3. Verification Tools
- **PE Audit**: Built-in verification in `package_driver.ps1` to ensure Subsystem 1 (Native).

---

## Required Environment Variables & PATHs

```powershell
# Centralized Setup
& ".\scripts\LaunchBuildEnv.ps1"
```

---
*Last Updated: February 14, 2026*
