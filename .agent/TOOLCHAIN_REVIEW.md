# Toolchain Review: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 3 Pro)
**Date**: February 15, 2026
**Status**: SESSION #16 COMPLETE

## Required Toolchain Requirements

### 1. Kernel Driver (Rust)
- **Mandatory Tool**: `cargo-wdk` (version 0.1.1+)
- **LLVM Version**: 17.0.6 (Contained in `D:\eWDK_28000\LLVM`)
- **Environment Variable**: `LIBCLANG_PATH` (Set to `D:\eWDK_28000\LLVM\bin`)
- **Environment Variable**: `WDK_ROOT` (Set via eWDK - Note: `cargo-wdk` likely infers this from `eWDK_ROOT_DIR` or PATH)
- **Linker Requirement**: `/NODEFAULTLIB:msvcrt` (Enforced in `build.rs`)
- **Compiler**: Rust 1.88.0+ (Supports `&raw mut`)

### 2. Build Automation
- **Master Script**: `scripts/LaunchBuildEnv.ps1`
- **Task Runner**: `cargo-make` (version 0.37.x+)

### 3. Hardware Support App (.NET)
- **SDK**: .NET 8.0
- **Workload**: `microsoft.net.sdk.maui` (if applicable)
- **Tooling**: `Microsoft.WindowsAppSDK` (version 1.5+)

## Recent Issues & Considerations
- **Bindgen Complexity**: Resolved by manual `get_current_irp_stack_location` helper targeting `__bindgen_anon_2`.
- **Rust Unsafe**: `PHYSICAL_ADDRESS` fields (like `QuadPart`) are safe to access in this environment, but accessing `LARGE_INTEGER` union fields is unsafe. The fix was removing an unnecessary `unsafe` block which implied `QuadPart` was accessible safely, likely due to struct definition or other factors.
