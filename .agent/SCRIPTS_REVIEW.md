# Automation Scripts Audit: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 2.0 Pro)
**Date**: February 16, 2026

## Script Inventory & Health

| Script | Purpose | Status | Notes |
| :--- | :--- | :---: | :--- |
| **`LaunchBuildEnv.ps1`** | **eWDK Setup** | ✅ | **CRITICAL:** Patched with UM/UCRT libs for `build.rs` compilation. |
| **`Install.ps1`** | Build & Deploy | ✅ | Uses `devcon.exe`. Supports incremental `-fast`. |
| **`Uninstall.ps1`** | Cleanup | ✅ | Removes driver & device node. |
| **`Automate-VM-Verification.ps1`** | End-to-End Test | ✅ | Default: Fast (no revert). `-Full`: Reverts VM. |

## Observations
- Scripts successfully handle the `Root\Media\LeylineAudio` identity.
- **Default Behavior**: `Install-VM.ps1` now uses `devcon.exe` (Root\Media) by default. Use `-UseSwd` for legacy SWD enumeration.
- **Action for Next Session**: Ensure `Uninstall.ps1` is run between every test to purge the interface cache in the registry, as AEB is sensitive to stale interface metadata.
