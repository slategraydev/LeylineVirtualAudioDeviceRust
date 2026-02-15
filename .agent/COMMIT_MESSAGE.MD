# Session Commit Message: PortCls Remediation & Debugging Strategy
**Reviewer**: Gemini CLI
**Date**: February 15, 2026

## Session Summary
Successfully corrected the `PORT_CLASS_DEVICE_EXTENSION_SIZE` constant to match x64 requirements (512 bytes), ensuring PortCls has sufficient memory for its internal state. Despite this fix, `PcAddAdapterDevice` continues to return `STATUS_INVALID_PARAMETER` (0xC000000D). The session concluded by updating project artifacts to pivot towards a WinDBG-led investigation in the next session to isolate the specific invalid parameter.

## Changes
- **Kernel**: Corrected `PORT_CLASS_DEVICE_EXTENSION_SIZE` to 512 bytes in `crates/leyline-kernel/src/lib.rs`.
- **Kernel**: Updated `AddDevice` extension size calculation to be strictly compliant with PortCls specifications.
- **Documentation**: Updated `.agent/PROJECT_PROGRESS.MD` with Session #22 results and adjusted Session #23 TODOs for WinDBG debugging.
- **Cleanup**: Removed redundant `PROJECT_PROGRESS.MD` from the root directory.

## Zero-Warning Verification Proof
- `cargo wdk build` (Release) completed with 0 errors and 0 warnings.
- `Select-String -Pattern "warning" -Path build.log` (Internal validation) returned no results.

## Git Commit Payload
```text
fix: align PortCls device extension size for x64

- Corrected PORT_CLASS_DEVICE_EXTENSION_SIZE to 512 bytes (64 * sizeof(ULONG_PTR)).
- Updated AddDevice extension size calculation for PortCls compliance.
- Pivot to WinDBG investigation for persistent 0xC000000D error.
```
