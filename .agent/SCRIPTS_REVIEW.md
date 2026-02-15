# Professional Script Review: Leyline Audio Driver

**Date**: February 14, 2026
**Status**: SESSION #10 COMPLETE
**Reviewer**: Antigravity (Gemini 3 Pro (High))

## Script Inventory

| Script | Purpose | Language | Dependency |
| :--- | :--- | :--- | :--- |
| **`scripts/LaunchBuildEnv.ps1`** | Unified eWDK/LLVM environment setup. Supports permanent path `D:\eWDK_28000`. | PowerShell 7+ | Drive D: (eWDK) |
| **`scripts/package_driver.ps1`** | Full pipeline: Build (Kernel, HSA, APO), Package, Inf2Cat, Sign. | PowerShell 7+ | `cargo-wdk`, `dotnet`, `signtool` |
| **`scripts/install_driver.ps1`** | Installs certificate and driver package. | PowerShell 7+ | `certutil`, `pnputil` |
| **`scripts/uninstall_driver.ps1`** | Uninstalls driver and cleans up device nodes. | PowerShell 7+ | `devcon`, `pnputil` |

## Automation Logic Audit

### `LaunchBuildEnv.ps1`
-   **Discovery**: Automatically detects eWDK on `D:\eWDK_28000` or mounted drives.
-   **Capture**: Dynamically sources environment variables from the eWDK's internal CMD setup.

### `package_driver.ps1`
-   **Integrity**: Includes a binary subsystem check to prevent invalid driver compilation.
-   **Cleanliness**: Performs a full build of both Kernel and HSA components in Release mode.

---
*Last Updated: February 14, 2026*
