# Project Progress: Leyline Audio Driver

## Session Log

... (previous sessions 10-27 remain unchanged) ...

### Session #28: Reference Hardening & BSOD Resolution
- **Date**: February 16, 2026
- **Reviewer**: Gemini CLI
- **Summary**: Resolved a critical `IRQL_NOT_LESS_OR_EQUAL` BSOD and established a robust local header reference library.
- **Key Changes**:
    - **BSOD Fix (Dispatch Hooks)**: Removed dangerous `MajorFunction` table hijacking in `lib.rs`. This eliminated the risk of illegal paging and race conditions during PortCls operation.
    - **BSOD Fix (GetPosition)**: Corrected `MiniportWaveRTStream::get_position` in `stream.rs` to use a proper `KsAudioPosition` struct, ensuring safe/aligned memory access at `DISPATCH_LEVEL`.
    - **Reference Library**: Populated `.agent/references/` with core WDK headers (`portcls.h`, `ksmedia.h`, `avrt.h`, etc.) and updated `GEMINI.MD` to map them for future agents.
    - **Warning Hygiene**: Achieved 100% clean build by removing dead code/imports after refactoring.
- **Persistent Issue**: `StartDevice` still fails at `Port Init Topology` with `0xC00002B9` (STATUS_INVALID_PARAMETER_MIX).

### Session #29: Horizontal Refactor & Stability Hardening
- **Date**: February 16, 2026
- **Reviewer**: Antigravity (Gemini 2.0 Flash)
- **Summary**: Initiated a massive "Horizontal Refactor" to split the 1,700-line `lib.rs` into logical modules. Implemented stability hardening for non-paged memory.
- **Key Changes**:
    - **Horizontal Refactor**: Split `lib.rs` into logical modules (`constants.rs`, `vtables.rs`, `descriptors.rs`, `dispatch.rs`, `wavert.rs`, `topology.rs`, `adapter.rs`) to improve maintainability.
    - **Memory Hardening**: Forced all VTables and descriptors into the `.rdata` (non-paged) section to prevent `DISPATCH_LEVEL` page faults.
    - **Isolation Strategy**: Simplified `StartDevice` to a pure WaveRT registration baseline to isolate errors.
- **Current Problems**:
    - **Build Failure**: The project is currently non-compiling due to `wdk-sys` binding naming mismatches and visibility issues introduced by the split.
- **Status**: Non-compiling (In Refactor).

### Session #30: Build Restoration & Binding Hardening
- **Date**: February 16, 2026
- **Reviewer**: Antigravity (Gemini 2.0 Flash)
- **Summary**: Restored the build system after the horizontal refactor by implementing custom binding generation.
- **Key Changes**:
    - **Custom Bindings**: Created `audio_wrapper.h` and updated `build.rs` to generate audio-specific Rust bindings via `bindgen`.
    - **Namespace Cleanup**: Resolved massive ambiguity errors by namespacing the `audio` module and using explicit type aliases in `stream.rs`.
    - **Shared Library Updates**: Added `ticks_to_bytes` and `get_base_address` to `leyline-shared` to satisfy kernel-mode requirements.
    - **Surgical Code Updates**: Updated `wavert.rs` and `descriptors.rs` to support `bindgen`'s nested union field layouts.
- **Current Problems**:
    - **Type Mismatches**: Final remaining errors relate to `audio::GUID` vs `wdk_sys::GUID` distinction.
- **Status**: Near-compiling (Finalizing type unification).

### Session #31: Type Unification & Descriptor Hardening
- **Date**: February 16, 2026
- **Reviewer**: Antigravity (Gemini 2.0 Flash)
- **Summary**: Resolved the critical GUID type mismatch and standardized PortCls descriptor naming.
- **Key Changes**:
    - **GUID Unification**: Blocked bindgen's `GUID` and injected `wdk_sys::GUID` into `audio_bindings.rs` to ensure 100% type parity across the crate.
    - **Descriptor Hardening**: Blocked `KSDATAFORMAT` and `PCCONNECTION_DESCRIPTOR` in bindgen and provided static-friendly manual definitions to support `descriptors.rs` initialization.
    - **Naming Correction**: Updated `PCCONNECTION` to `PCCONNECTION_DESCRIPTOR` and corrected field names (`FromNodePin`, `ToNodePin`) to match WDK headers.
    - **Visibility Fix**: Made the `audio` module public in `stream.rs` to resolve re-export errors in `lib.rs`.
- **Current Problems**:
    - **KSPIN_DESCRIPTOR Union**: Final remaining errors relate to initializing the `KSPIN_DESCRIPTOR` union in `descriptors.rs`.
- **Status**: Near-compiling (Surgical fix phase).

## PROJECT TODO (Next Session: #32)
- [ ] **KSPIN_DESCRIPTOR Fix**: Align `descriptors.rs` with the `KSPIN_DESCRIPTOR` union layout or block/manually define it.
- [ ] **Zero-Warning Proof**: Achieve 100% clean build.
- [ ] **Baseline Load Test**: Verify driver stability on a test target.

