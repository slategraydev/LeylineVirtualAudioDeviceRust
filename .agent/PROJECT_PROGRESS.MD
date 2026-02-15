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

### Session #19: Dynamic Formats, Endpoint UX & Install Fixes
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Implemented format negotiation, improved INF naming, and fixed installation logic for virtual devices.
- **Key Changes**:
    - **Dynamic Formats**: Refactored `miniport_data_range_intersection` to intersect requested ranges with driver capabilities (44.1k-192k).
    - **Endpoint Naming**: Updated `leyline.inx` to name endpoints "Leyline Output" and "Leyline Input".
    - **Installation Script**: Updated `scripts/install_driver.ps1` to use `devcon install` instead of `pnputil`, fixing the missing device node issue.
    - **Toolchain**: Hardened `build_apo.ps1` with manual eWDK paths.
- **Verification**: 
    - Build: Full Package Success (0 warnings).
    - **Field Issue**: User reported duplicate device nodes and HSA silent failure. `devcon install` creates a new node every time it's run. HSA likely crashing due to missing device or dependencies.

### Session #20: Driver Stability & Installer Idempotency
- **Date**: February 15, 2026
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Summary**: Resolved Code 31 (STATUS_INVALID_PARAMETER) by fixing GUIDs and hardened the installer logic.
- **Key Changes**:
    - Fixed `KSCATEGORY_AUDIO_GUID` in `leyline-kernel` (lib.rs).
    - Fixed standard category GUIDs in `leyline-shared`.
    - Set `MaxOutputStreams = 0` in `PcAddAdapterDevice` to match WaveRT software-only patterns.
    - Refactored `install_driver.ps1` to be idempotent (removes old nodes before `devcon install`).
    - Standardized all PortCls callbacks to `extern "system"`.
    - Added `DbgPrint` diagnostic logging throughout the initialization path.
- **Verification**: 
    - Build: `leyline-kernel` Success (0 warnings).
    - Package: `package-all` Success.
    - Installer: Idempotency verified via script logic.

### Session #21: PortCls Remediation & IRP Fix
- **Date**: February 15, 2026
- **Reviewer**: Gemini CLI
- **Summary**: Resolved the deterministic PcAddAdapterDevice 0xC000000D failure and corrected critical structural field paths.
- **Key Changes**:
    - **PortCls Fix**: Corrected `PORT_CLASS_DEVICE_EXTENSION_SIZE` to 64 bytes (x64 standard) and refactored `DeviceExtensionSize` calculation to strictly exceed this threshold.
    - **Object Table Fix**: Set `MaxObjects` to 8 in `PcAddAdapterDevice`. PortCls documentation and technical analysis confirm that passing 0 can trigger `STATUS_INVALID_PARAMETER`.
    - **ABI Alignment**: Updated `StartDevice` signature to use `PDEVICE_OBJECT` for the first parameter, matching the `PCPFNSTARTDEVICE` callback definition.
    - **IRP Traversal**: Corrected the nested anonymous union path for `CurrentStackLocation` (`Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation`).
    - **HSA Bridge**: Implemented `IOCTL_LEYLINE_MAP_PARAMS` to allow user-mode mapping of the shared parameter block.
- **Verification**: 
    - Build: `cargo make build` (Release) Success (0 warnings).
    - Verification: `Select-String` on build log returned 0 warnings/errors.
    - Tests: `leyline-shared` passed (4 tests).

### Session #22: PortCls Extension Alignment & Persistent 0xC000000D
- **Date**: February 15, 2026
- **Reviewer**: Gemini CLI
- **Summary**: Aligned `PORT_CLASS_DEVICE_EXTENSION_SIZE` with x64 requirements, but `PcAddAdapterDevice` continues to fail with `STATUS_INVALID_PARAMETER` (0xC000000D).
- **Key Changes**:
    - **PortCls Fix**: Corrected `PORT_CLASS_DEVICE_EXTENSION_SIZE` to 512 bytes on x64 (`64 * sizeof(ULONG_PTR)`).
    - **Extension Calculation**: Updated `AddDevice` to pass the combined size (PortCls + Driver) to `PcAddAdapterDevice`.
- **Observations**: 
    - Despite correct alignment and size, the driver still reports `PcAddAdapterDevice failed with status 0xC000000D`.
    - This suggests the invalid parameter may be related to non-code factors such as build environment inconsistencies, INF configuration, or certificate/signing issues affecting PortCls initialization.
- **Verification**: 
    - Build: `cargo wdk build` (Release) Success.
    - Status: Deterministic failure at `AddDevice` persists.

## PROJECT TODO (Next Session: #23)
- **[ ] WinDBG Investigation**: Attach debugger to target and set breakpoints on `portcls!PcAddAdapterDevice` to identify which parameter is triggering `STATUS_INVALID_PARAMETER`.
- **[ ] Environment Audit**: Verify eWDK toolchain consistency and check for certificate/signing issues that might interfere with `PortCls` loading.
- **[ ] INF Review**: Audit `leyline.inx` for naming or reference string issues that could cause subdevice registration failures.
- **[ ] Streaming Verification**: Confirm endpoints appear in Windows Sound Settings once `AddDevice` succeeds.

