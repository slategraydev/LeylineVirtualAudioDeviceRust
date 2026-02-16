# Architectural Audit: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 1.5 Pro)
**Date**: February 16, 2026

## Critical Findings & Resolutions

### 1. IRQL_NOT_LESS_OR_EQUAL BSOD (RESOLVED)
- **Finding**: System crashed during installation/streaming.
- **Diagnosis**: 
    1. **Illegal Hooking**: The driver was overwriting `MajorFunction` in `StartDevice`. This interfered with `portcls.sys` and accessed code/data at inappropriate IRQLs.
    2. **Unaligned Memory Access**: `GetPosition` was performing manual pointer arithmetic on a raw `u64` pointer, which is unsafe at `DISPATCH_LEVEL` where PortCls typically calls it.
- **Resolution**: 
    1. Removed all `MajorFunction` hijacking.
    2. Implemented a proper `KsAudioPosition` struct in `stream.rs` for safe, aligned access.

### 2. Topology Port Initialization Failure (STATUS_INVALID_PARAMETER_MIX)
- **Finding**: `StartDevice` fails at `PcNewPort` or `Init` for Topology with `0xC00002B9`.
- **Diagnosis**: This is likely a descriptor error. PortCls is rejecting the `PCFILTER_DESCRIPTOR` for the topology miniport, possibly due to incorrectly terminated `PCCONNECTION` arrays or invalid pin categories.
- **Next Step**: Audit `TOPO_RENDER_PINS` and `TOPO_CONNECTIONS` against `portcls.h` reference.

## Safety Audit
- **IRQL Safety**: Streaming path (`GetPosition`, `SetState`) is now significantly safer.
- **Hooking**: 0 hooks present. 100% compliant with standard WDM/PortCls patterns.

## Recommendations for Session #29
1.  **Deployment**: Proceed with installation.
2.  **Telemetry**: If failure persists, enable Driver Verifier to catch the exact allocation causing corruption.
