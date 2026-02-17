# Session #42 Commit Message: Audio Endpoint Infrastructure & Force-Enable

**Reviewer**: Antigravity (Kimi-k2.5)  
**Date**: February 18, 2026

## Plain Text Commit Message [COPY-PASTE]

```
feat: add RootMedia enumeration mode and force-enable endpoint visibility

Implement dual-mode enumeration infrastructure (-UseRootMedia) to test SWD\DEVGEN vs ROOT\MEDIA enumeration for virtual audio drivers. Add explicit IoRegisterDeviceInterface kernel API calls to bypass INF AddInterface processing. Add PKEY_AudioDevice_EnableEndpointByDefault INF property to force endpoint visibility per Microsoft documentation. Fix PSScriptAnalyzer warnings and add remote VM diagnostic tooling.

Forced both installation scripts to support SWD\DEVGEN (default via devgen.exe) and Root\Media (experimental via devcon.exe install) enumeration modes. Enhanced VM deployment with devcon.exe bundling and remote diagnostic capabilities. Updated INF with correct Microsoft GUID {F3E80BEF-1723-4FF2-BCC4-7F83DC5E46D4} for PKEY_AudioDevice_EnableEndpointByDefault with FLOW_MASK_RENDER|FLAG_ENABLE (0x00000101) and FLOW_MASK_CAPTURE|FLAG_ENABLE (0x00000201) values. Kernel build maintains zero-warning status; explicit interface registration implemented after WaveRT subdevice registration.

Changes:
- scripts/Install.ps1: Add -UseRootMedia switch, dual-mode enumeration logic
- scripts/Install-VM.ps1: Add -UseRootMedia switch, devcon.exe bundling, fix parameter propagation
- crates/leyline-kernel/src/adapter.rs: Add IoRegisterDeviceInterface and IoSetDeviceInterfaceState kernel API calls
- crates/leyline-kernel/leyline.inx: Add PKEY_AudioDevice_EnableEndpointByDefault property with correct Microsoft GUID
- scripts/Diagnose-VM.ps1: Create comprehensive remote VM diagnostic script

Verification:
- cargo build --release: 0 warnings, 0 errors
- cargo clippy --release: 0 warnings, 0 errors
- PSScriptAnalyzer: 0 warnings on modified scripts
- GUID verification: All KS GUIDs match Windows SDK headers (ks.h, ksmedia.h)
```

## Git Commit Payload [COMPRESSED]

```
feat: add RootMedia enumeration mode and force-enable endpoint visibility

Implement dual-mode enumeration infrastructure (-UseRootMedia) to test SWD\DEVGEN vs ROOT\MEDIA enumeration for virtual audio drivers. Add explicit IoRegisterDeviceInterface kernel API calls to bypass INF AddInterface processing. Add PKEY_AudioDevice_EnableEndpointByDefault INF property to force endpoint visibility per Microsoft documentation.

Changes:
- scripts/Install.ps1: Add -UseRootMedia switch, dual-mode enumeration logic
- scripts/Install-VM.ps1: Add -UseRootMedia switch, devcon.exe bundling, fix parameter propagation
- crates/leyline-kernel/src/adapter.rs: Add IoRegisterDeviceInterface and IoSetDeviceInterfaceState kernel API calls
- crates/leyline-kernel/leyline.inx: Add PKEY_AudioDevice_EnableEndpointByDefault property with correct Microsoft GUID
- scripts/Diagnose-VM.ps1: Create comprehensive remote VM diagnostic script

Verification:
- cargo build --release: 0 warnings, 0 errors
- cargo clippy --release: 0 warnings, 0 errors
- PSScriptAnalyzer: 0 warnings on modified scripts
- GUID verification: All KS GUIDs match Windows SDK headers
```

## Summary

Session #42 addressed the critical blocking issue preventing Leyline Audio Driver from appearing as audio endpoints in Windows. The session implemented two parallel approaches:

1. **Enumeration Mode Testing**: Added -UseRootMedia infrastructure to test if ROOT\MEDIA enumeration (vs SWD\DEVGEN) enables audio endpoint creation
2. **Force-Enable Property**: Added PKEY_AudioDevice_EnableEndpointByDefault per Microsoft documentation to force endpoints visible regardless of Audio Endpoint Builder heuristics
3. **Explicit Kernel API**: Implemented IoRegisterDeviceInterface calls in driver code to bypass INF AddInterface processing limitations

## Key Technical Decisions

- **Backward Compatibility**: SWD\DEVGEN remains default enumeration mode; -UseRootMedia switch enables testing of alternative
- **Zero-Warning Policy**: All code changes maintain zero-warning build status (verified with
 cargo build, cargo clippy, PSScriptAnalyzer)
- **Correct Microsoft GUIDs**: PKEY_AudioDevice_EnableEndpointByDefault uses official Microsoft GUID {F3E80BEF-1723-4FF2-BCC4-7F83DC5E46D4},4 per documentation
- **Kernel Code**: Explicit interface registration attempts after PcRegisterSubdevice with DbgPrint logging for success/failure tracking

## Testing Status

**Pending**: Root\Media enumeration with PKEY_AudioDevice_EnableEndpointByDefault force-enable property. Awaiting VM deployment verification in Session #43.

## Documentation Updates

- PROJECT_PROGRESS.MD: Session #42 log added with detailed technical changes and testing procedure
- CODE_REVIEW.md: Architecture audit with enumeration analysis and hypothesis validation
- TEST_REVIEW.md: Infrastructure test results and Session #43 test plan
- BUILD_REVIEW.MD: Build health verification with zero-warning proof
- TOOLCHAIN_REVIEW.md: Environment status with devcon.exe integration