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

### Session #15: IOCTL Infrastructure & Loopback Allocation
- **Reviewer**: Antigravity (Gemini 3 Pro)
- **Date**: February 15, 2026
- **Summary**: Implemented the foundation for HSA communication and allocated shared memory for audio loopback.
- **Key Changes**:
    - Implemented Control Device Object (CDO) and `\DosDevices\LeylineAudio` symbolic link.
    - Added `DeviceExtension` to the FDO to store shared state.
    - Allocated a 64KB Loopback Buffer and MDL in `StartDevice`.
    - Hooked Major Function dispatchers (`IRP_MJ_CREATE`, `CLOSE`, `DEVICE_CONTROL`) to the CDO.
    - Modified `MiniportWaveRTStream` to consume the shared loopback buffer from the device extension.
    - Resolved several `wdk-sys` binding discrepancies regarding union field access and calling conventions.
- **Blockers**: 
    - Build currently FAILING due to `IRP` structure traversal issues (`CurrentStackLocation` field path).
    - Need to finalize traversal logic for nested anonymous unions in `Tail.Overlay`.
- **Verification**: 
    - Build: FAILED (1 Error remaining: `no field CurrentStackLocation on type _IRP__bindgen_ty_4__bindgen_ty_1__bindgen_ty_1`).

## PROJECT TODO (Next Session: #16)
- **[ ] Fix IRP Traversal**: Resolve the final build error by identifying the correct path to `CurrentStackLocation` in the generated bindings.
- **[ ] Capture Logic**: Verify the zero-copy loopback logic where Render and Capture share the same physical memory.
- **[ ] IOCTL Mapping**: Implement the User-Mode mapping of the shared parameters in `DispatchDeviceControl`.
- **[ ] Dynamic Formats**: Implement dynamic format negotiation in `DataRangeIntersection`.
