# Professional Script Review: Leyline Audio Driver

**Date**: February 14, 2026
**Status**: ACTIVE MANAGEMENT
**Reviewer**: Antigravity (Gemini 3 Pro (High))

## Script Inventory

| Script | Purpose | Language | Dependency |
| :--- | :--- | :--- | :--- |
| **`scripts/package_driver.ps1`** | Full pipeline: Build (Kernel, HSA, APO), Package, Inf2Cat, Sign. | PowerShell 7+ | `cargo-wdk`, `dotnet`, `nmake`, `inf2cat`, `signtool` |
| **`scripts/install_driver.ps1`** | Installs self-signed certificate and driver package. | PowerShell 7+ | `certutil`, `pnputil` |
| **`scripts/uninstall_driver.ps1`** | Uninstalls driver and provides cleanup guidance. | PowerShell 7+ | `pnputil` |

## Automation Logic Audit

### Standardized Helpers
-   **`Test-IsAdministrator`**: Mandatory check for elevated privileges in all deployment scripts.

### `package_driver.ps1`
-   **Kernel Build**: Invokes `cargo wdk build`. Sets `LIBCLANG_PATH` locally if needed.
-   **HSA Build**: Invokes `dotnet build` and `dotnet publish` for `win-x64`.
-   **APO Build**: 
    -   **Conditional Logic**: Checks for `cl.exe` in `$env:PATH`.
    -   **Behavior**: If found, runs `nmake` in `src/APO`. If not found, prints a warning and skips compilation (relying on pre-built DLL if present).
    -   **Gap**: Requires execution from a **Visual Studio Developer Prompt** to succeed cleanly.
-   **Packaging**:
    -   Cleans `package/` directory.
    -   Aggregates `.sys`, `.inf`, `.dll`, and HSA executable.
-   **Validation**: Runs `Inf2Cat` to verify driver signability.
-   **Signing**:
    -   Generates a self-signed `leyline.pfx` if missing.
    -   Signs all binaries and the catalog file.

## Developer Workflow
To execute a full production build:
1.  Open **x64 Native Tools Command Prompt for VS 2022**.
2.  Run: `powershell -ExecutionPolicy Bypass -File scripts/package_driver.ps1`

---
*Last Updated: February 14, 2026*
