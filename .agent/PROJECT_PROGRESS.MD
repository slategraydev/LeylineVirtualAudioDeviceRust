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
    - **Sanity Check**: Audited project state; fixed GUID discrepancies in `leyline-shared` and identified missing IOCTL/Loopback logic.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Build: `LeylineHSA` Success (0 warnings).
    - GUID Alignment: Verified 1:1 match between shared and kernel crates.

### Session #15: Dispatch Infrastructure & Loopback Logic
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented `DriverEntry` hook overrides, Control Device Object (CDO) creation, and Shared Buffer allocation logic.
- **Key Changes**:
    - Hooked `IRP_MJ_CREATE`, `CLOSE`, and `DEVICE_CONTROL` in `DriverEntry`.
    - Implemented CDO creation and Symbolic Link `\DosDevices\LeylineAudio`.
    - Implemented `dispatch_device_control` skeleton.
    - Implemented 64KB loopback buffer allocation in `StartDevice`.
    - Updated `MiniportWaveRTStream` to reuse `loopback_mdl` if available (Zero-Copy logic).
    - **Blocker**: Kernel build failed due to `bindgen` complex union traversal issue (IRP Stack Location).
- **Verification**: 
    - Build: `leyline-kernel` FAILED (1 Error).
    - Tests: `leyline-shared` Passed.

### Session #16: IRP Fix & Zero-Copy Verification
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Resolved the critical IRP structure traversal error and achieved a zero-warning build for the kernel.
- **Key Changes**:
    - Implemented `get_current_irp_stack_location` helper function to traverse `bindgen`'s nested anonymous unions (`Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation`).
    - Refactored `dispatch_device_control` to use the helper and clean up status logic.
    - Fixed `unsafe` block warnings in `stream.rs` (removed unnecessary blocks).
    - Fixed mutable static reference warning in `lib.rs` using `&raw mut`.
    - Verified Zero-Copy logic: `allocate_audio_buffer` correctly shares `loopback_mdl` between Render/Capture streams.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Build: `LeylineHSA` Success (0 warnings).
    - Tests: `leyline-shared` Passed (4 tests).

### Session #17: HSA Bridge & Visualization
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented the kernel-to-user bridge and real-time waveform visualization in the HSA.
- **Key Changes**:
    - **Shared Protocol**: Updated `SharedParameters` to include QPC timing and buffer metadata; fixed IOCTL codes to use standard `CTL_CODE` macros.
    - **Kernel Mapping**: Implemented `IOCTL_LEYLINE_MAP_BUFFER` in `dispatch_device_control` using `MmMapLockedPagesSpecifyCache` (UserMode).
    - **Kernel Sync**: Updated `MiniportWaveRTStream` to publish `render_start_qpc`, `frequency`, and `byte_rate` to shared memory.
    - **HSA Implementation**: Implemented `DriverBridge` connection and `MainWindow` visualization loop (60Hz oscilloscope).
    - **Cleanup Logic**: Added `user_mapping` tracking in `DeviceExtension` and auto-unmap in `dispatch_close`.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Build: `LeylineHSA` Success (0 warnings).
    - Integration: Validated `SharedParameters` struct alignment and IOCTL values.

### Session #18: APO Integration & INF Wiring
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Successfully integrated the C++ Audio Processing Object (APO) into the build pipeline and registered it in the INF for both Render and Capture endpoints.
- **Key Changes**:
    - Created `scripts/build_apo.ps1` to robustly build the C++ APO in the eWDK environment (using `vcvarsall.bat` for x64).
    - Updated `package_driver.ps1` to use the new build script.
    - Updated `leyline.inx` to register the APO CLSID for "WaveRender" and "WaveCapture" interfaces (correcting reference strings).
    - Fixed `C4100` warnings in `dllmain.cpp` (unreferenced parameters).
    - Verified full package build (Kernel + APO + HSA) with **Zero Warnings**.
- **Verification**: 
    - Build: `LeylineAPO` Success (0 warnings).
    - Package: `leyline.inf` correctly references `LeylineAPO.dll` and registers `{C8D3E4F5...}`.

### Session #19: Dynamic Formats & Endpoint Naming
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented true dynamic format negotiation in the kernel and distinct endpoint naming in the INF.
- **Key Changes**:
    - **Dynamic Formats**: Refactored `miniport_data_range_intersection` to intersect requested ranges with driver capabilities (44.1k-192k, 16/24/32-bit). It now prefers the highest quality supported.
    - **Endpoint Naming**: Updated `leyline.inx` to split registry keys, assigning "Leyline Output" to the Render interface and "Leyline Input" to the Capture interface.
    - **Toolchain Hardening**: Updated `scripts/build_apo.ps1` to manually enforce eWDK SDK/MSVC paths, bypassing `vcvarsall.bat` fragility.
- **Verification**: 
    - Build: Full Package Success (0 warnings in Kernel/HSA/APO/Inf2Cat).
    - INF: Verified distinct `FriendlyName` registry keys for Render/Capture.

## PROJECT TODO (Next Session: #20)
- **[ ] Driver Installation**: Perform a full test installation on a target machine to verify APO loading and end-to-end audio flow.
- **[ ] Feature**: Implement `GetPosition` and `GetHWLatency` with high-precision QPC logic to support the new dynamic rates.
- **[ ] Feature**: Implement `SetState` transitions properly (currently stubs) to handle Pause/Run correctly.
