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

## PROJECT TODO (Next Session: #30)
- **[ ] Build Restoration**: Resolve `wdk-sys` import errors and cross-module visibility in the new modular structure.
- **[ ] Baseline Load Test**: Once building, verify the "pure WaveRT" driver loads without BSOD.
- **[ ] Incremental Topology**: Gradually re-add Topology filters to pinpoint `0xC00002B9`.
