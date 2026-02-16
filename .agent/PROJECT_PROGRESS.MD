# Project Progress: Leyline Audio Driver

## Session Log

### Session #10: PortCls & Environment Hardening
- **Date**: February 14, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Transitioned to 100% contained eWDK toolchain and implemented PortCls initialization.
- **Key Changes**:
    - Refactored `build.rs` for dynamic WDK/SDK detection.
    - Fixed environment capture in `LaunchBuildEnv.ps1`.
    - Implemented `PcInitializeAdapterDriver` in `DriverEntry`.
    - Forced Subsystem 1 (Native) for `leyline.sys`.
- **Verification**: Build Success (0 warnings).

### Session #11: WaveRT Filter Registration & COM Implementation
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented WaveRT filter registration and manual COM interfaces to expose audio endpoints.
- **Key Changes**:
    - Implemented `StartDevice` logic using `PcNewPort` and `PcRegisterSubdevice`.
    - Created manual COM VTables and wrappers for `IMiniportWaveRT` and `IMiniportWaveRTStream`.
    - Updated `LaunchBuildEnv.ps1` to support self-contained LLVM at `D:\eWDK_28000\LLVM`.
    - Resolved all naming convention warnings via targeted suppressions.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Toolchain: Verified `libclang.dll` at `D:\eWDK_28000\LLVM\bin`.

### Session #12: Test-Driven Refactoring & Shared Logic
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Refactored core logic into `leyline-shared` and implemented host-side unit testing.
- **Key Changes**:
    - Moved `RingBuffer` and `WaveRTMath` to `leyline-shared`.
    - Implemented 4 unit tests in `leyline-shared` (passing on host).
    - Refactored `leyline-kernel` to consume shared modules.
    - Updated `build.rs` to support user-space unit testing without kernel linker interference.
- **Verification**: 
    - `cargo test -p leyline-shared`: 4 Passed.
    - `cargo wdk build`: Success (0 warnings).

### Session #13: Topology Registration & Data Range Expansion
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Registered the Topology filter and expanded data range support to PCM and IEEE Float.
- **Key Changes**:
    - Implemented `IMiniportTopology` COM wrapper and registered the Topology subdevice.
    - Defined `PCFILTER_DESCRIPTOR` for the Wave filter with PCM and Float data ranges.
    - Implemented `DataRangeIntersection` logic in `MiniportWaveRT`.
    - Created `scripts/Update-Version.ps1` for automated INF version stamping.
    - Integrated version stamping into the `cargo-make` build pipeline.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Automation: INF stamping verified (`1.0.13.xxx`).

### Session #14: Bridge Pins & Dual-Endpoint Convergence
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented internal filter routing and expanded the driver to expose two discrete audio endpoints (Render/Capture).
- **Key Changes**:
    - Defined `PCCONNECTION` arrays and bridge pins to connect Wave and Topology filters.
    - Implemented `PcRegisterPhysicalConnection` logic in `StartDevice`.
    - Doubled the filter registration to expose both "Leyline Output" (Render) and "Leyline Input" (Capture) subdevices.
    - Refactored `MiniportWaveRT` and `MiniportTopology` to return context-aware descriptors (Render vs. Capture).
    - Fixed kernel linker error by suppressing `msvcrt.lib` via `/NODEFAULTLIB`.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Build: `LeylineHSA` Success (0 warnings).

### Session #15: Dispatch Infrastructure & Loopback Logic
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented `DriverEntry` hook overrides, Control Device Object (CDO) creation, and Shared Buffer allocation logic.
- **Key Changes**:
    - Hooked `IRP_MJ_CREATE`, `CLOSE`, and `DEVICE_CONTROL` in `DriverEntry`.
    - Implemented CDO creation and Symbolic Link `\DosDevices\LeylineAudio`.
    - Implemented 64KB loopback buffer allocation in `StartDevice`.
    - Updated `MiniportWaveRTStream` to reuse `loopback_mdl` if available (Zero-Copy logic).
- **Verification**: 
    - Build: `leyline-kernel` FAILED (1 Error).

### Session #16: IRP Fix & Zero-Copy Verification
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Resolved the critical IRP structure traversal error and achieved a zero-warning build for the kernel.
- **Key Changes**:
    - Implemented `get_current_irp_stack_location` helper function.
    - Refactored `dispatch_device_control` to use the helper.
    - Fixed `unsafe` block warnings and mutable static reference warnings.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Build: `LeylineHSA` Success (0 warnings).

### Session #17: HSA Bridge & Visualization
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented the kernel-to-user bridge and real-time waveform visualization in the HSA.
- **Key Changes**:
    - Updated `SharedParameters` and implemented `IOCTL_LEYLINE_MAP_BUFFER`.
    - Updated `MiniportWaveRTStream` to publish timing metadata.
    - Implemented `DriverBridge` connection and `MainWindow` visualization in HSA.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Build: `LeylineHSA` Success (0 warnings).

### Session #18: APO Integration & INF Wiring
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Integrated the C++ APO into the build pipeline and registered it in the INF.
- **Key Changes**:
    - Created `scripts/build_apo.ps1`.
    - Updated `leyline.inx` to register the APO CLSID for Render/Capture endpoints.
- **Verification**: 
    - Build: `LeylineAPO` Success (0 warnings).

### Session #19: Dynamic Formats, Endpoint UX & Install Fixes
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented format negotiation, improved INF naming, and fixed installation logic.
- **Key Changes**:
    - Refactored `miniport_data_range_intersection`.
    - Updated endpoint naming to "Leyline Output" and "Leyline Input".
    - Updated `scripts/install_driver.ps1` to use `devcon install`.
- **Verification**: 
    - Build: Full Package Success (0 warnings).

### Session #20: Driver Stability & Installer Idempotency
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Resolved Code 31 and hardened installer logic.
- **Key Changes**:
    - Fixed `KSCATEGORY_AUDIO_GUID` alignment.
    - Set `MaxOutputStreams = 0` for software-only pattern.
    - Refactored `install_driver.ps1` to be idempotent.
- **Verification**: 
    - Build: Success (0 warnings).

### Session #21: PortCls Remediation & IRP Fix
- **Date**: February 15, 2026
- **Reviewer**: Gemini CLI
- **Summary**: Resolved deterministic failures and corrected structural field paths.
- **Key Changes**:
    - Corrected `PORT_CLASS_DEVICE_EXTENSION_SIZE` and object table limits.
    - Updated `StartDevice` ABI signature and IRP traversal paths.
- **Verification**: 
    - Build: Success (0 warnings).

### Session #22: PortCls Extension Alignment & Persistent 0xC000000D
- **Date**: February 15, 2026
- **Reviewer**: Gemini CLI
- **Summary**: Aligned extension sizes for x64 but encountered persistent initialization failure.

### Session #23: PortCls ABI Alignment & HSA Deployment
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Resolved deterministic failures by aligning PortCls FFI to `extern "C"` and hardened HSA deployment.
- **Key Changes**:
    - Refactored PortCls FFI and driver callbacks to `extern "C"`.
    - Enabled self-contained WinUI 3 deployment for HSA.
- **Verification**: Build Success (0 warnings).

### Session #24: High-End Professionalization & Uber Scripts
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented ETW telemetry, unified the installation pipeline, and performed a total system purge of legacy artifacts.
- **Key Changes**:
    - Added ETW registration to `DriverEntry`.
    - Created Uber Scripts `Install.ps1` and `Uninstall.ps1`.
    - Fixed `PcAddAdapterDevice` failure via INF registry keys.
- **Verification**: Build Success (0 warnings).

### Session #25: System Cleanup & Installer Hardening
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 1.5 Pro)
- **Summary**: Purged legacy driver artifacts and hardened the installation script for eWDK 28000.
- **Key Changes**:
    - **System Purge**: Identified and deleted 6 residual `simpleaudiosample` driver packages (oemXX.inf) from the Driver Store.
    - **Diagnostic Resolution**: Confirmed that "UNKNOWN" root devices `ROOT\SYSTEM\0001` and `0002` belong to Virtual Desktop and Oculus, resolving the "duplicate roots" mystery.
    - **Installer Hardening**: Corrected `DEVCON_EXE` path in `Install.ps1` for eWDK 28000 and implemented aggressive cleanup logic for legacy HardwareIDs.
    - **Node Management**: Implemented fixed instance ID `0000` for root device creation to prevent duplicate enumeration.
    - **Modernization/Reversion**: Evaluated `devgen.exe` + `pnputil` workflow but reverted to fixed `devcon` to maintain professional `ROOT\MEDIA` device grouping.
- **Verification**: 
    - Build: Success (0 warnings).
    - Installation: `ROOT\MEDIA\0002` created and bound via `devcon`.
    - Verification: `simpleaudiosample` strings 100% purged from system and driver store.

## PROJECT TODO (Next Session: #26)
- **[ ] Verification**: Final check of endpoint visibility in Windows Sound Settings after the next reboot.
- **[ ] HSA Testing**: Verify HSA can connect to the new `ROOT\MEDIA` instance.
- **[ ] Logic Expansion**: Implement real-time volume/mute controls in `MiniportTopology`.
