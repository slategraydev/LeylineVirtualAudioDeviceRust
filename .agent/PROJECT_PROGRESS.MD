# Project Progress: Leyline Audio Driver

## Session Log

... (previous sessions 10-34 remain unchanged) ...

### Session #35: Script Hardening & PowerShell 5.1 Compatibility
**Reviewer**: Antigravity (Gemini 2.0 Flash)
**Date**: February 16, 2026
**Summary**: Resolved a critical parser error in the deployment scripts caused by PowerShell 7 specific syntax. Hardened the build pipeline for compatibility with Windows PowerShell 5.1 and extended the VM deployment script.
**Key Changes**:
- **Syntax Compatibility**: Replaced `||` operators with PowerShell 5.1 compatible error handling in `Install.ps1`.
- **APO Build Refinement**: Refactored the APO build block to use a robust single-process eWDK environment sourcing and build execution.
- **Install-VM Extension**: Added `-clean` parameter to `Install-VM.ps1` to support deep cleaning on the host before remote deployment.
- **Verification**: Confirmed script syntax is now valid in both PowerShell 5.1 and 7.
- **Status**: Deployment Scripts Fixed & Hardened.

### Session #36: Build Pipeline Repair & Warning Resolution
**Reviewer**: Antigravity (Gemini 2.0 Flash)
**Date**: February 16, 2026
**Summary**: Fixed critical build failures in `Install.ps1` (Kernel & APO) caused by environment propagation issues. Resolved persistent Rust warnings and C++ compilation errors.
**Key Changes**:
- **Environment Fix**: Refactored `Install.ps1` to use `LaunchBuildEnv.ps1` for robust environment setup, fixing `bindgen` errors.
- **APO Repair**: Simplified APO build logic in `Install.ps1` to avoid `cmd.exe` line length limits.
- **C++ Fixes**: Resolved ambiguous `CLSID_LeylineAPO` symbol in `dllmain.cpp` and fixed invalid string escape sequences in `LeylineAPO.cpp`.
- **Warning Suppression**: Applied targeted `#[allow(unnecessary_transmutes)]` to `lib.rs` to generated code.
- **Architecture Cleanup**: Removed duplicate `bindgen` inclusion in `stream.rs`, enforcing a single source of truth.
- **Verification**: Confirmed successful Kernel and APO builds with **0 Warnings**.
**Status**: Build Pipeline (Kernel + APO) 100% Green.

### Session #37: Script Directory Safety Refactor
**Reviewer**: Antigravity (Gemini 2.0 Flash)
**Date**: February 16, 2026
**Summary**: Refactored all deployment scripts to robustly handle directory state. Ensured that scripts always return the user to their original working directory upon completion or failure, preventing "directory drift" during iterative development.
**Key Changes**:
- **Pattern Standardization**: Applied `try...finally { Set-Location $initialDir }` pattern to `Install.ps1`, `Uninstall.ps1`, and `Install-VM.ps1`.
- **Stack Safety**: Removed reliance on potentially unbalanced `Push-Location`/`Pop-Location` calls for global state restoration.
- **Verification**: Validated that `Install.ps1` (failure), `Uninstall.ps1` (success/failure), and `Install-VM.ps1` (failure) all correctly restore the shell to the originating directory.
- **Issue Tracking**: Documented persistent `0xC00002B9` error during Topology initialization. Topology registration remains disabled in `adapter.rs` pending investigation.
**Status**: Scripts Hardened & Developer-Friendly.

### Session #38: GUID Verification & Constants Hardening
**Reviewer**: Antigravity (kimi-k2.5)
**Date**: February 17, 2026
**Summary**: Completed comprehensive GUID audit across the entire driver. Added missing critical GUIDs required for proper KS property handling and standardized interface declarations. Verified all GUIDs match Windows SDK headers.
**Key Changes**:
- **GUID Verification**: Cross-referenced all 20+ GUIDs in `constants.rs` against Windows SDK headers (`ks.h`, `ksmedia.h`, `portcls.h`).
- **Missing GUIDs Added**:
  - `KSINTERFACESETID_STANDARD` - Required for pin interface declarations (was hardcoded)
  - `KSPROPSETID_PIN` - Required for pin property handling
  - `KSPROPSETID_CONNECTION` - Required for connection state management
  - `GUID_NULL` - Standard null GUID for defaults
- **Code Quality**: Replaced hardcoded GUID in `descriptors.rs` with named constant `KSINTERFACESETID_STANDARD`.
- **Bug Fixes**: Confirmed `KSNODETYPE_SPEAKER` and `KSNODETYPE_MICROPHONE` now have correct values from `ksmedia.h`.
- **Verification**: Build completed with **0 Warnings**, **0 Errors**.
**Status**: GUID Architecture Hardened & Complete.

### Session #39: Topology Initialization Diagnostics
**Reviewer**: Antigravity (kimi-k2.5)
**Date**: February 17, 2026
**Summary**: Added comprehensive diagnostic instrumentation to isolate the `0xC00002B9` (STATUS_REQUEST_NOT_ACCEPTED) topology initialization failure. Enhanced both the topology miniport and adapter layer with detailed DbgPrint logging.
**Key Changes**:
- **Diagnostic Instrumentation**:
  - Added detailed DbgPrint to `topology_query_interface` logging every interface query (IID_IMiniportTopology, IID_IUnknown, IID_IMiniport, IID_IPortTopology, IID_IPort)
  - Added NULL parameter validation with logging in topology callbacks
  - Enhanced `adapter.rs` with step-by-step topology initialization logging
- **Missing Interface Added**:
  - Added `IID_IPort` GUID to `constants.rs` (was missing from diagnostic checks)
- **Build Fixes**:
  - Fixed NTSTATUS literal type issues (`0xC00002B9u32 as i32`)
  - Fixed unused variable warnings
- **Diagnostic Purpose**: The enhanced logging will identify exactly which interface query or initialization step fails, helping isolate whether the issue is:
  - Missing interface support in QueryInterface
  - Invalid descriptor returned by GetDescription
  - Parameter validation failure in Init
- **Debug Infrastructure**: Created `scripts/Enable-KernelDebug.ps1` helper script to configure Windows for kernel DbgPrint capture
**Status**: Diagnostic Infrastructure Deployed. Ready for VM testing.

## PROJECT TODO (Next Session: #40)
### Session #38: GUID Verification & Constants Hardening
**Reviewer**: Antigravity (kimi-k2.5)
**Date**: February 17, 2026
**Summary**: Completed comprehensive GUID audit across the entire driver. Added missing critical GUIDs required for proper KS property handling and standardized interface declarations. Verified all GUIDs match Windows SDK headers.
**Key Changes**:
- **GUID Verification**: Cross-referenced all 20+ GUIDs in `constants.rs` against Windows SDK headers (`ks.h`, `ksmedia.h`, `portcls.h`).
- **Missing GUIDs Added**:
  - `KSINTERFACESETID_STANDARD` - Required for pin interface declarations (was hardcoded)
  - `KSPROPSETID_PIN` - Required for pin property handling
  - `KSPROPSETID_CONNECTION` - Required for connection state management
  - `GUID_NULL` - Standard null GUID for defaults
- **Code Quality**: Replaced hardcoded GUID in `descriptors.rs` with named constant `KSINTERFACESETID_STANDARD`.
- **Bug Fixes**: Confirmed `KSNODETYPE_SPEAKER` and `KSNODETYPE_MICROPHONE` now have correct values from `ksmedia.h`.
- **Verification**: Build completed with **0 Warnings**, **0 Errors**.
**Status**: GUID Architecture Hardened & Complete.

## PROJECT TODO (Next Session: #39)
- [ ] **Baseline Load Test**: Deploy the compiled `.sys` and `.inf` to the VM using the hardened `Install-VM.ps1`.
- [ ] **Investigate Topology Init**: Analyze the `0xC00002B9` failure in `PcNewPort`/`Init` for the Topology miniport.
- [ ] **HSA Verification**: Run the WinUI 3 app alongside the driver to verify IOCTL connectivity.
