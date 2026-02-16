# Architectural Audit: Leyline Audio Driver

**Reviewer**: Antigravity (Kimi-k2.5)
**Date**: February 17, 2026

## Session #39 Focus: Topology Initialization Failure (0xC00002B9)

### Critical Issue: STATUS_REQUEST_NOT_ACCEPTED During Topology Port Init

**Error Code**: `0xC00002B9` (`STATUS_REQUEST_NOT_ACCEPTED`)  
**Location**: `adapter.rs` - `PcNewPort(CLSID_PortTopology)` or subsequent `Init()` call  
**Impact**: Topology miniport fails to initialize, preventing proper audio endpoint creation

---

## 1. Diagnostic Infrastructure Added (COMPLETED)

### Enhanced DbgPrint Instrumentation
Added comprehensive logging throughout the topology initialization path to isolate the failure:

**`topology.rs` - QueryInterface Diagnostics:**
- Logs every interface query with GUID identification
- Tracks acceptance/rejection of `IID_IMiniportTopology`, `IID_IUnknown`, `IID_IMiniport`
- Logs rejection of `IID_IPortTopology` and `IID_IPort` (if queried)
- Validates parameter null checks

**`topology.rs` - GetDescription Diagnostics:**
- Validates `this` and `out_description` parameters
- Logs descriptor pointer before return
- Tracks success/failure path

**`topology.rs` - Init Diagnostics:**
- Validates all parameters (`this`, `unknown_adapter`, `resource_list`, `port`)
- Logs entry and exit with status codes
- Tracks internal `MiniportTopology::init()` result

**`adapter.rs` - Port Creation Diagnostics:**
- Enhanced logging around `PcNewPort(CLSID_PortTopology)`
- Specific detection of `0xC00002B9` error code
- Descriptive error messages for common failure modes
- Validation of miniport pointer before `Init()` call

---

## 2. Missing Interface Added (COMPLETED)

**Issue**: Diagnostic code referenced `IID_IPort` which wasn't defined in `constants.rs`  
**Resolution**: Added `IID_IPort` from Windows SDK:
```rust
pub const IID_IPort: GUID = GUID {
    Data1: 0xB4C90A25,
    Data2: 0x5791,
    Data3: 0x11D0,
    Data4: [0x86, 0xF9, 0x00, 0xA0, 0xC9, 0x11, 0xB5, 0x44],
};
```

**Rationale**: PortCls may query for `IID_IPort` on the miniport during initialization. While the miniport currently rejects this (as expected), having the GUID defined enables proper diagnostics.

---

## 3. Root Cause Analysis: Potential Failure Points

### Hypothesis A: Descriptor Validation Failure (MOST LIKELY)
PortCls validates `PCFILTER_DESCRIPTOR` immediately after `GetDescription()`. Issues could include:

- **Null Pointers**: `TOPO_RENDER_PINS` or `TOPO_CAPTURE_PINS` may have null `Categories`, `DataRanges`, or `AutomationTable` pointers that PortCls rejects
- **Invalid Counts**: `PinCount`, `ConnectionCount`, or `CategoryCount` may be inconsistent with actual array sizes
- **Layout Mismatch**: `PCPIN_DESCRIPTOR` structure may not match PortCls expectations (field ordering, packing)

**Verification Steps:**
1. Check `DbgPrint` output for `LeylineTopo: GetDescription` - does it get called?
2. If `GetDescription` succeeds but `Init` fails, descriptor layout is likely valid
3. If `PcNewPort` fails immediately, the port creation itself is rejecting the miniport

### Hypothesis B: Interface Query Rejection
PortCls may query for interfaces the miniport doesn't support:

- `IID_IPort` - Should be rejected (miniport ≠ port), but rejection method matters
- `IID_IPortTopology` - Should be rejected (miniport ≠ port)
- `IID_IMiniport` - Should be accepted ✅

**Current Behavior**: Returns `STATUS_NOINTERFACE` for unsupported IIDs  
**Potential Issue**: PortCls may expect `STATUS_SUCCESS` with null pointer, or different behavior

### Hypothesis C: Vtable Layout Mismatch
The `IMiniportTopologyVTable` structure must exactly match PortCls expectations:

```
Expected Layout:
[0]  IUnknown.QueryInterface
[1]  IUnknown.AddRef
[2]  IUnknown.Release
[3]  IMiniportTopology.GetDescription
[4]  IMiniportTopology.DataRangeIntersection
[5]  IMiniportTopology.Init
```

**Current Layout**: Matches expected layout ✅  
**Risk**: `#[repr(C)]` attribute ensures C layout, but field ordering must be exact

### Hypothesis D: COM Object Layout
The `MiniportTopologyCom` structure:

```rust
pub struct MiniportTopologyCom {
    pub vtable: *const IMiniportTopologyVTable,  // Must be first (this pointer)
    pub inner: MiniportTopology,
    pub ref_count: u32,
}
```

**Requirement**: The vtable pointer must be the first field (COM vtable pointer convention)  
**Current Status**: ✅ Correct - vtable is first field

### Hypothesis E: DataRangeIntersection Implementation
The `topology_data_range_intersection` callback may be called during `Init()`:

**Current Implementation**: 
- Validates `pin_id` (0 or 1)
- Checks `data_range.is_null()`
- Returns `STATUS_SUCCESS` even for minimal valid calls

**Potential Issue**: PortCls may expect more detailed validation or specific return codes for topology pins

---

## 4. Test Infrastructure Recommendations

### Immediate Diagnostic Actions

**1. DbgPrint Analysis (NEXT STEP)**
After building and loading the driver with new diagnostics:
```powershell
# In VM, run DebugView as Administrator
# Filter for "LeylineTopo:" and "Leyline:"
# Look for sequence:
#   - "QueryInterface" calls before failure
#   - "GetDescription" call (if any)
#   - "Init" call (if reached)
#   - Error code context
```

**2. Isolation Test**
Create a test build that **only** registers topology (disable WaveRT):
```rust
// In adapter.rs StartDevice, comment out WaveRender and WaveCapture registration
// Only register TopologyRender
```
This determines if the issue is:
- Topology-specific (fails in isolation)
- Interaction-related (works alone, fails with WaveRT)

**3. Descriptor Validation Test**
Add runtime descriptor validation in `topology_get_description`:
```rust
// Verify TOPO_RENDER_FILTER_DESCRIPTOR fields:
// - PinCount == 2
// - Pins pointer non-null
// - ConnectionCount == 1
// - Connections pointer non-null
// - CategoryCount == 2
// - Categories pointer non-null
```

### Long-term Test Infrastructure

**1. WDUTF (Windows Driver Unit Test Framework) Test**
Create user-mode test for topology miniport:
```cpp
// TestTopologyMiniport.cpp
// 1. CoCreateInstance or manual COM creation
// 2. QueryInterface(IID_IMiniportTopology)
// 3. Call GetDescription() - verify descriptor returned
// 4. Verify descriptor structure (pins, connections, categories)
```

**2. Kernel-mode Driver Test**
Create a test driver that:
- Instantiates `MiniportTopologyCom`
- Calls each COM method directly
- Validates return codes and behavior

**3. PortCls Validation Harness**
If available, use Windows Driver Kit validation tools to check:
- Descriptor layout compliance
- COM vtable alignment
- Interface implementation completeness

---

## 5. Potential Fixes to Attempt

### Fix 1: Add IPort Interface Support (If Queried)
If diagnostics show `IID_IPort` is being queried and rejected causes failure:
```rust
// In topology_query_interface, add:
else if crate::is_equal_guid(iid, &IID_IPort) {
    // Miniport doesn't implement IPort, but try different rejection
    *out = null_mut();
    return STATUS_SUCCESS; // Or different error code
}
```

### Fix 2: Return Interface Pointer for IMiniport
Ensure `IID_IMiniport` query returns valid pointer:
```rust
else if crate::is_equal_guid(iid, &IID_IMiniport) {
    (*com_obj).ref_count += 1;
    *out = this; // Return the miniport as IMiniport
    return STATUS_SUCCESS;
}
```

### Fix 3: Simplify DataRangeIntersection
Return more explicit status codes:
```rust
pub unsafe extern "system" fn topology_data_range_intersection(...) -> NTSTATUS {
    if pin_id > 1 {
        return STATUS_INVALID_PARAMETER;
    }
    
    // For topology, we may not need to support format intersection
    // Return NOT_IMPLEMENTED to let PortCls use defaults
    return STATUS_NOT_IMPLEMENTED;
}
```

### Fix 4: Validate Descriptor in GetDescription
Add runtime checks before returning descriptor:
```rust
pub unsafe extern "system" fn topology_get_description(...) -> NTSTATUS {
    // ... existing code ...
    
    // Validate descriptor before returning
    let descriptor = if is_capture {
        &TOPO_CAPTURE_FILTER_DESCRIPTOR
    } else {
        &TOPO_RENDER_FILTER_DESCRIPTOR
    };
    
    // Check for null pointers in descriptor
    if descriptor.Pins.is_null() || descriptor.Categories.is_null() {
        DbgPrint(c"LeylineTopo: ERROR - Descriptor has null pointers!\n".as_ptr());
        return STATUS_INVALID_PARAMETER;
    }
    
    *description = descriptor;
    STATUS_SUCCESS
}
```

---

## 6. Next Steps for Session #40

### Priority 1: Diagnostic Analysis

**Prerequisite: Enable Kernel Debug Output**

Before capturing diagnostics, you must enable kernel debug output on your test VM:

**Option A: Use the Helper Script (Recommended)**
```powershell
# On your test VM, run:
.\scripts\Enable-KernelDebug.ps1
# Then reboot the VM
```

**Option B: Manual Configuration**
```cmd
# Run as Administrator:
bcdedit /debug on
reg add "HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Debug Print Filter" /v DEFAULT /t REG_DWORD /d 0xffffffff /f
# Then reboot
```

**Capture and Analysis Steps:**

1. Build driver with new diagnostics
2. Deploy to VM using `Install-VM.ps1`
3. On the VM, run **DebugView** (from Sysinternals) as Administrator:
   - Enable **Capture** → **Capture Kernel**
   - Set filter to: `Leyline*;LeylineTopo*`
4. Load the driver and capture output in real-time
5. Analyze `DbgPrint` sequence to identify exact failure point

**Reference**: See detailed DebugView instructions in `TEST_REVIEW.md` → "DebugView Setup" section.

### Priority 2: Isolation Testing
1. Create topology-only test build
2. Verify if `PcNewPort` succeeds in isolation
3. Determine if WaveRT coexistence is the issue

### Priority 3: Descriptor Audit
1. Add descriptor validation code
2. Check all pointers in `TOPO_RENDER_FILTER_DESCRIPTOR`
3. Verify structure sizes match PortCls expectations

### Priority 4: Interface Experimentation
If diagnostics indicate interface query issues:
1. Modify `topology_query_interface` behavior
2. Test different return codes for unsupported interfaces
3. Consider supporting `IID_IPort` if consistently queried

---

## Build Verification
- **Status**: ✅ SUCCESS
- **Warnings**: 0
- **Errors**: 0
- **New Constants**: `IID_IPort` added
- **Diagnostics**: Enhanced throughout topology initialization path

---

### How to View DbgPrint Output

To see the driver's diagnostic messages, use **Sysinternals DebugView**:

1. **Download**: https://docs.microsoft.com/en-us/sysinternals/downloads/debugview
2. **Run as Administrator** (required for kernel capture)
3. **Enable Kernel Capture**: Click **Capture** → **Capture Kernel**
4. **Filter Output**: Edit → Filter/Highlight → Include: `Leyline*;LeylineTopo*`
5. **Load Driver**: Run `Install.ps1` and watch output in real-time

**Note**: You must first enable kernel debug output (see "Prerequisite" section above) or most DbgPrint messages will be suppressed by Windows.

---

### Knowledge Base: STATUS_REQUEST_NOT_ACCEPTED (0xC00002B9)

This unusual NTSTATUS indicates the request was valid but cannot be fulfilled. In PortCls context, this typically means:

1. **Resource Conflict**: Another driver/component owns required resources
2. **Validation Failure**: Parameters passed checks but are semantically invalid
3. **State Conflict**: Operation not valid in current driver state
4. **Interface Rejection**: COM interface query rejected in a way the caller cannot handle

Unlike `STATUS_INVALID_PARAMETER` (bad input) or `STATUS_NOINTERFACE` (IID not supported), `STATUS_REQUEST_NOT_ACCEPTED` suggests the request was understood but refused at a higher level.