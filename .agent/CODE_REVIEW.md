# Architectural Audit: Leyline Audio Driver

**Reviewer**: Antigravity (Gemini 2.0 Pro)
**Date**: February 18, 2026

## Session #47: Format Correction & Build Verification
*(Previous session summaries preserved)*

...

## Session #113: Automation Table Fixes & ABI Stability

### Executive Summary
Session #113 identified a critical bug in `descriptors.rs` where the `PIN_AUTOMATION_TABLE` (used by WaveRT and some Topology pins) was returning `STATUS_NOT_IMPLEMENTED` for Pin Name and Category queries. This blocked PortCls's default handlers and stalled the Audio Endpoint Builder (AEB). Additionally, all GUID constants were converted to `static` to ensure stable memory addresses for FFI calls.

### 6.1 Automation Table Restoration ✅
- **Issue**: Attempting to implement Pin Name/Category in custom automation tables via `STATUS_NOT_IMPLEMENTED` caused the AEB to stall. If a custom table is provided, it must either fully implement the property or be removed to let PortCls handle it.
- **Fix**: Removed Pin Name and Category from custom `PIN_AUTOMATION_TABLE` and `TOPO_PIN_PROPERTIES`. Kept only `JACK_DESCRIPTION` as it requires physical connectivity simulation.
- **Impact**: PortCls now correctly responds to Pin Name/Category using internal metadata from `KSPIN_DESCRIPTOR`, satisfying AEB requirements.

### 6.2 GUID Stability (FFI ABI) ✅
- **Issue**: `const GUID` in Rust can lead to transient addresses when using the `&` operator, which is highly unstable for long-lived FFI pointers in PortCls automation tables.
- **Fix**: Converted all core GUIDs (`KSPROPSETID_Jack`, `KSCATEGORY_AUDIO`, etc.) to `static GUID`.
- **Impact**: Provides a single, stable address in the `.rdata` section, ensuring PortCls property lookup functions correctly.

### 6.3 Diagnostic Optimization
- **Debugger Removal**: The kernel debugger (`kd.exe`) was removed from the automated `Install.ps1` cycle to improve stability. Verification now relies on on-VM health checks and build-versioned (v1.0.9) kernel logging.

**Status**: 🟢 **PROTOCOL FULLY IMPLEMENTED** - All known AEB property requirements are now correctly exposed via stable automation tables or PortCls defaults.

### 6.4 Jack Description Handler Safety & Compliance ⚠️ -> ✅
-   **Issue**: The `jack_description_handler` was attempting to read the Property ID from `(*property_request).Instance`. In PortCls `PCPROPERTY_REQUEST`, the `Instance` field points to the *Pin Instance* (or is NULL), not the property definition. Accessing it as `KSPROPERTY` was unsafe and likely returning garbage, causing the handler to fail or behave randomly.
-   **Fix**: Updated to retrieve ID from `(*(*property_request).PropertyItem).Id`.
-   **Issue**: The handler lacked a `BasicSupport` check. Windows AEB often queries `BasicSupport` before asking for the value.
-   **Fix**: Added standard `KSPROPERTY_TYPE_BASICSUPPORT` handling, returning `AccessFlags` (Get | BasicSupport).
-   **Impact**: Ensures stable, compliant responses to AEB queries, preventing the "Silent Failure" state.
-   **Descriptors Compliance:** Updated `TOPO_RENDER_PINS` and `TOPO_CAPTURE_PINS` to use `core::ptr::null()` for `AutomationTable` on bridge pins (id=1). Matches `sysvad` reference; avoids potential conflicts with PortCls default handlers.
-   **INF Modernization:** Removed legacy WDM registration keys (`AssociatedFilters`, `wdmaud.drv`) from `leyline.inx`. Added explicit `PKEY_AudioEndpoint_FormFactor` and disabled unsupported `EventDriven_Mode`.
-   **Build Robustness:** Patched `LaunchBuildEnv.ps1` to support host-side Rust build scripts by linking User Mode libraries.
-   **Safety:** Ensured `unsafe` blocks in `adapter.rs` are minimized and documented.
