# Professional Script Review: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 1.5 Pro)
**Date**: February 15, 2026
**Status**: SESSION #25 COMPLETE

## Script Inventory

| Script | Purpose | Language | Dependency |
| :--- | :--- | :--- | :--- |
| **`scripts/Install.ps1`** | **[UBER]** One-click build, sign, and install. Hardened for eWDK 28000. | PowerShell 7+ | Drive D: (eWDK) |
| **`scripts/Uninstall.ps1`** | **[UBER]** Complete system purge of all artifacts (including legacy samples). | PowerShell 7+ | None |
| **`scripts/LaunchBuildEnv.ps1`** | Environment initialization. | PowerShell 7+ | Drive D: (eWDK) |
| **`scripts/Update-Version.ps1`** | Automated INF version stamping. | PowerShell 7+ | None |
| **`scripts/build_apo.ps1`** | Robust C++ APO build (Manual Paths). | PowerShell 7+ | eWDK + VC Tools |
| **`scripts/package_driver.ps1`** | Full pipeline: Build, Package, Sign. | PowerShell 7+ | `cargo-wdk`, `dotnet`, `build_apo.ps1` |

## Automation Logic Audit
- **Install.ps1**: Fixed `DEVCON_EXE` path for eWDK 28000. Added aggressive cleanup for `Root\simpleaudiosample` and `Root\SimpleAudioDriver` instances. Implemented fixed instance ID `0000` for consistent device enumeration.
- **Uninstall.ps1**: Hardened to purge all legacy `simpleaudiosample` and `SimpleAudioDriver` artifacts from the Driver Store and PnP tree.
- **Makefile.toml**: Orchestrates the pipeline; verified task mapping for `clean`, `install`, and `uninstall`.

---
*Last Updated: February 15, 2026*
