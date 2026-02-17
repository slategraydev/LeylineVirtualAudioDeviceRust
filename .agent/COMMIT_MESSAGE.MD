# Commit Message: Session #41

**Reviewer**: Antigravity (Kimi-k2.5)
**Date**: February 17, 2026
**Session**: #41

## Summary
Driver achieves full kernel-level initialization with all 4 subdevices registered and physical connections established. Audio endpoints do not appear in Windows - suspected SWD\DEVGEN enumeration incompatibility with Windows Audio Service. Uninstall script hardened for complete system cleanup.

## Changes Made

### Code Changes
- **adapter.rs**: Added TopologyCapture miniport registration (previously only TopologyRender)
- **adapter.rs**: Enabled PcRegisterPhysicalConnection for both render and capture chains
- **adapter.rs**: Added import for PcRegisterPhysicalConnection from lib.rs
- **adapter.rs**: Enhanced DbgPrint output showing all registered subdevices and connections

### Script Hardening
- **Uninstall.ps1**: Added SWD\DEVGEN device detection and removal
- **Uninstall.ps1**: Added orphaned "Generic software device" cleanup
- **Uninstall.ps1**: Enhanced driver store purge with /force /uninstall flags
- **Uninstall.ps1**: Added WMI force-removal fallback for stubborn devices
- **Uninstall.ps1**: Fixed PSScriptAnalyzer warnings (removed unused variables)
- **Uninstall.ps1**: Added certificate cleanup with certutil for Root stores

### Diagnostics
- **Diagnose-Endpoints.ps1**: Created comprehensive endpoint diagnostic script
- **Diagnose-VMAudio.ps1**: Created VM-specific audio diagnostic
- **Test-HostReady.ps1**: Created pre-flight system readiness checker

## Verification

### Build Status
- **Kernel**: ✅ Compiles with 0 warnings, 0 errors
- **APO**: ✅ Build successful
- **HSA**: ✅ Build successful

### Driver Initialization
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

### Blocking Issue
- ❌ Audio Endpoints: 0 found in Windows Audio subsystem
- ❌ MMDevices Registry: No Leyline entries
- 🔴 **CRITICAL**: Driver works at kernel level but Windows Audio Service doesn't enumerate endpoints

## Zero-Warning Proof
```
Compiling leyline-kernel v0.1.0 (C:\LeylineAudioDriver\crates\leyline-kernel)
    Finished `release` profile [optimized] target(s) in 0.80s
```

## Plain Text Commit Message [COPY-PASTE]
```
feat: Complete WaveRT topology registration, harden uninstall script

- Add TopologyCapture miniport registration (was missing)
- Enable physical connections between WaveRT and Topology
- Harden Uninstall.ps1 with SWD\DEVGEN device cleanup
- Add driver store purge with /force /uninstall flags
- Create comprehensive endpoint diagnostics
- Fix PSScriptAnalyzer warnings

Driver initializes all 4 subdevices successfully but audio endpoints
do not appear in Windows. Suspected SWD\DEVGEN enumeration issue.
Requires investigation in Session #42.
```

## Next Session (#42) TODO
- [ ] CRITICAL: Investigate SWD\DEVGEN vs Root\Media enumeration for audio endpoints
- [ ] Test clean install with reboot to verify INF AddInterface processing
- [ ] Check MMDevices registry after install
- [ ] Try explicit IoRegisterDeviceInterface if INF method fails
- [ ] Verify descriptor categories match INF exactly