```text
# Architectural Audit: Leyline Audio Driver

**Reviewer**: Antigravity (Kimi-k2.5)
**Date**: February 17, 2026

## Session #41: Audio Endpoint Investigation - IN PROGRESS

### Executive Summary

**Status**: Driver achieves full kernel-level initialization but **audio endpoints do not appear in Windows**. This is the critical blocking issue for the "Product North Star" (Two-endpoint virtual driver).

The driver successfully:
- ✅ Loads without errors
- ✅ Registers 4 subdevices (WaveRender, WaveCapture, TopologyRender, TopologyCapture)
- ✅ Establishes physical connections between WaveRT and Topology
- ✅ Topology miniport initializes correctly
- ✅ HardwareID matches INF (`Root\LeylineAudio`)

But fails at:
- ❌ Audio endpoint enumeration (0 endpoints in MMDevices registry)
- ❌ Windows Audio Service recognition

---

## 1. Architecture Validation: What Works

### 1.1 Driver Initialization Chain ✅

```
DriverEntry → PcInitializeAdapterDriver → SUCCESS
AddDevice → PcAddAdapterDevice → SUCCESS  
StartDevice → All Subdevices Registered → SUCCESS
```

**DbgPrint Evidence**:
```
Leyline: StartDevice COMPLETED SUCCESSFULLY
Leyline: Registered Subdevices:
  - WaveRender (Output)
  - WaveCapture (Input)
  - TopologyRender
  - TopologyCapture
Leyline: Physical Connections:
  - WaveRender -> TopologyRender
  - TopologyCapture -> WaveCapture
```

### 1.2 Subdevice Registration ✅

All 4 required subdevices are properly registered:

| Subdevice | Port Type | Miniport | Status |
|-----------|-----------|----------|--------|
| WaveRender | PortWaveRT | MiniportWaveRT | ✅ Registered |
| WaveCapture | PortWaveRT | MiniportWaveRT | ✅ Registered |
| TopologyRender | PortTopology | MiniportTopology | ✅ Registered |
| TopologyCapture | PortTopology | MiniportTopology | ✅ Registered |

### 1.3 COM Interface Handling ✅

Both miniports correctly handle `QueryInterface`:
- ✅ Accept `IID_IMiniportWaveRT`, `IID_IMiniportTopology`, `IID_IUnknown`
- ✅ Reject unknown IIDs with `STATUS_NOINTERFACE` (expected behavior)
- ✅ Reference counting works correctly

### 1.4 Physical Connections ✅

Both render and capture chains connected:
- WaveRender Pin 1 → TopologyRender Pin 0
- TopologyCapture Pin 1 → WaveCapture Pin 0

---

## 2. Critical Issue: Audio Endpoint Enumeration FAILURE

### 2.1 Diagnostic Results

| Check | Expected | Actual | Status |
|-------|----------|--------|--------|
| Driver Device Status | OK | OK | ✅ |
| Windows Audio Service | Running | Running | ✅ |
| Audio Endpoints (Render) | 1+ | 0 | ❌ |
| Audio Endpoints (Capture) | 1+ | 0 | ❌ |
| MMDevices Registry | Leyline entries | None | ❌ |
| KS Audio Properties | Present | "No Audio Properties" | ❌ |

### 2.2 Root Cause Hypotheses

**PRIMARY HYPOTHESIS: SWD\DEVGEN Enumeration Incompatibility**

The driver uses `devgen.exe` to create software devices with hardware ID `Root\LeylineAudio`, but the Instance ID is `SWD\DEVGEN\{GUID}`.

**Evidence**:
- Device Instance ID: `SWD\DEVGEN\{54ECF7D1-030B-5D43-8A57-84AFF9159297}`
- Hardware ID: `Root\LeylineAudio` (correct)
- Windows Audio Service may not process INF `AddInterface` entries for SWD-enumerated devices

**Comparison**:
| Method | Instance ID Prefix | Audio Endpoint Support |
|--------|-------------------|------------------------|
| SWD\DEVGEN | `SWD\DEVGEN\{GUID}` | ❌ Unknown/Problematic |
| Root\Media | `ROOT\MEDIA\0000` | ✅ Standard for audio |
| Traditional PnP | Hardware-specific | ✅ Well-tested |

**Why This Matters**:
Windows Audio Endpoint Builder service scans for audio devices based on enumeration method. SWD (Software Device) enumeration may not trigger the same registration paths as traditional audio enumeration.

---

## 3. Secondary Hypotheses

### 3.1 INF AddInterface Processing

The INF registers audio interfaces:
```ini
AddInterface = %KSCATEGORY_AUDIO%, "WaveRender", Leyline_WaveRenderInterface
AddInterface = %KSCATEGORY_AUDIO%, "WaveCapture", Leyline_WaveCaptureInterface
```

**Potential Issues**:
- Windows may skip AddInterface processing on driver update (vs fresh install)
- Reference string "WaveRender" must match subdevice name exactly
- Category GUIDs may need adjustment

### 3.2 Missing Explicit Interface Registration

Currently relying on INF `AddInterface`. May need explicit code registration:
```rust
// Potential missing code:
IoRegisterDeviceInterface(device_object, &KSCATEGORY_AUDIO, NULL, &interface_string);
```

### 3.3 Descriptor Category Configuration

Current categories in descriptors:
- `KSCATEGORY_AUDIO_GUID` + `KSCATEGORY_RENDER_GUID` (for render)
- `KSCATEGORY_AUDIO_GUID` + `KSCATEGORY_CAPTURE_GUID` (for capture)

**Questions**:
- Should we use `KSCATEGORY_REALTIME` for WaveRT?
- Are the category combinations correct for Windows Audio Service recognition?

---

## 4. Uninstall Script Hardening (Completed)

### 4.1 New Capabilities

The `Uninstall.ps1` script now handles:

| Scenario | Method | Status |
|----------|--------|--------|
| Multiple device instances | devcon + pnputil + WMI | ✅ |
| SWD\DEVGEN devices | InstanceId matching | ✅ |
| Orphaned "Generic software device" | FriendlyName check | ✅ |
| Legacy simpleaudiosample | HardwareID matching | ✅ |
| Driver store purge | `/force /uninstall` flags | ✅ |
| Corrupted services | sc.exe fallback | ✅ |
| Certificate cleanup | certutil for Root stores | ✅ |

### 4.2 Complete Cleanup Verification

The script now performs 7-step cleanup:
1. Stop audio services
2. Remove all device instances (3 methods)
3. Purge driver store (with /uninstall)
4. Delete services (detect "marked for deletion")
5. Registry cleanup (8 paths)
6. System file cleanup (6 files)
7. Certificate cleanup (6 stores)

---

## 5. Recommendations for Session #42

### 5.1 Priority 1: Enumerator Investigation

**TEST**: Try Root\Media enumeration instead of SWD\DEVGEN

**Options**:
1. **DevGen with different parameters**: Check if devgen can use Root\Media
2. **Manual device creation**: Use `IoCreateDevice` with `FILE_DEVICE_UNKNOWN` and manual PnP registration
3. **Alternative**: Use `SwDeviceCreate` API with different properties

**Expected Outcome**:
If Instance ID changes to `ROOT\MEDIA\0000` and endpoints appear, confirms SWD\DEVGEN is the issue.

### 5.2 Priority 2: AddInterface Verification

**TEST**: Force AddInterface re-processing

**Steps**:
1. Run hardened `Uninstall.ps1` (deletes driver package completely)
2. Reboot (ensures clean driver store state)
3. Run `Install.ps1 -clean` (fresh INF processing)
4. Check MMDevices registry immediately after install

**Verification**:
```powershell
Get-ChildItem "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Render"
Get-ChildItem "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\Capture"
```

### 5.3 Priority 3: Explicit Interface Registration

**TEST**: Add `IoRegisterDeviceInterface` calls in `StartDevice`

**Implementation**:
In `adapter.rs`, after `PcRegisterSubdevice` calls, add:
```rust
// Register audio device interface explicitly
IoRegisterDeviceInterface(
    device_object,
    &KSCATEGORY_AUDIO,
    null_mut(),
    &mut interface_string,
);
```

**Risk**: May conflict with PortCls internal registration.

### 5.4 Priority 4: Category/GUID Audit

**TEST**: Verify descriptor categories match INF categories exactly

**Check**:
- Descriptors use `KSCATEGORY_RENDER_GUID` from constants
- INF uses `%KSCATEGORY_RENDER%` from strings section
- Both must resolve to `{65E8773E-8F56-11D0-A3B9-00A0C9223196}`

---

## 6. Knowledge Base: Lessons Learned

### 6.1 SWD vs Root Enumeration

**SWD (Software Device) Enumeration**:
- Created by `devgen.exe` or `SwDeviceCreate`
- Instance ID: `SWD\DEVGEN\{GUID}`
- Good for generic software devices
- **Unknown**: Audio endpoint support

**Root Enumeration**:
- Created by traditional INF-based installation
- Instance ID: `ROOT\MEDIA\0000` (or similar)
- Standard for audio drivers
- **Proven**: Works with Windows Audio Service

### 6.2 INF AddInterface Behavior

Windows processes `AddInterface` entries:
- During driver package installation (pnputil /add-driver /install)
- When device is first enumerated
- **NOT** automatically on driver update (requires /force reinstall)

### 6.3 PortCls Registration Requirements

For audio endpoints to appear:
1. Driver must register with PortCls (`PcAddAdapterDevice`)
2. WaveRT miniports must be registered (`PcRegisterSubdevice`)
3. INF must have matching `AddInterface` entries
4. Windows Audio Endpoint Builder must enumerate the device
5. MMDevices registry must be populated with endpoint properties

---

## 7. Build Verification

**Session #41 Build Status**: ✅ SUCCESS
- Warnings: 0 (after removing unused variables)
- Errors: 0
- Driver loads: ✅
- All subdevices register: ✅

**Zero-Warning Proof**:
```
Compiling leyline-kernel v0.1.0
    Finished `release` profile [optimized] target(s) in 0.80s
```

---

## 8. Next Steps Summary

| Priority | Action | Goal |
|----------|--------|------|
| P0 | Test Root\Media enumeration | Confirm SWD\DEVGEN issue |
| P1 | Clean install with reboot | Verify AddInterface processing |
| P2 | Check MMDevices registry | Confirm endpoint registration |
| P3 | Try explicit interface registration | Bypass INF dependency |
| P4 | Category/GUID audit | Ensure descriptor/INF match |

**Success Criteria for Session #42**:
- Audio endpoints appear in `mmsys.cpl`
- `Get-PnpDevice` shows "Leyline Output" and "Leyline Input"
- MMDevices registry populated with Leyline entries
- Windows Audio Service recognizes endpoints

---

**Status**: 🔴 **BLOCKING ISSUE** - Audio endpoints required for Product North Star  
**Estimated Effort**: 1-2 sessions to resolve enumeration method  
**Risk**: May require significant architecture change if SWD\DEVGEN is incompatible with audio endpoints