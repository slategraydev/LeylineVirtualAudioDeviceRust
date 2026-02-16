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

## PROJECT TODO (Next Session: #29)
- **[ ] Topology Audit**: Deep-dive into `TOPO_RENDER_FILTER_DESCRIPTOR` using local `portcls.h` reference to resolve `0xC00002B9`.
- **[ ] CDO Restoration**: Re-implement the Hardware Support App (HSA) communication line using the standard/safe WDM separate Device Object method.
- **[ ] Streaming Verification**: Test loopback and position reporting.
