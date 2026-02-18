# Fix: Audio Endpoint Visibility and AEB Handshake

Resolved major issues preventing audio endpoints from appearing in Windows Sound Control Panel.

## Changes:

### Kernel Driver (`leyline-kernel`)
- **Fixed Automation Tables**: `PIN_AUTOMATION_TABLE` now correctly points to `PIN_PROPERTIES`, allowing PortCls to handle Pin Name and Category queries.
- **Improved Handshake Support**: Added `KSPROPERTY_JACK_DESCRIPTION2` as a fallback to the same handler as `DESCRIPTION`.
- **Property Safety**: Added NULL pointer checks in `jack_description_handler` to prevent potential kernel crashes during AEB discovery.
- **Fixed GUID Stability**: Converted `const GUID` to `static GUID` for major KS property sets and categories to ensure stable memory addresses for FFI calls.
- **Component ID Support**: Corrected `KSCOMPONENTID` struct alignment and implemented the property handler to satisfy AEB requirements.
- **Pin Counting**: Implemented `IPinCount` for both Topology and WaveRT miniports to correctly report instance availability.

### Installation & Deployment (`scripts`)
- **Removed Debugger Bloat**: Dropped `kd.exe` integration from `Install.ps1` in favor of a faster, more reliable build-and-deploy cycle.
- **Improved Tool Pathing**: Refactored `LaunchBuildEnv.ps1` to robustly resolve WDK tools even when multiple versions or eWDK paths are present.
- **Enhanced Verification**: Added registry-level endpoint discovery checks to the installation script.
- **Versioning**: Implemented build-versioned kernel logging ("Leyline: DriverEntry v1.0.9") for easier deployment tracking.

### Configuration (`INF`)
- **Event-Driven Mode**: Explicitly added `PKEY_AudioEndpoint_Supports_EventDriven_Mode` to satisfy modern Windows audio engine requirements.
- **Default Enable**: Corrected `PKEY_AudioDevice_EnableEndpointByDefault` values.
- **Version Bumping**: Updated `DriverVer` to force PnP re-enumeration.

## Plain Text Commit Message [COPY-PASTE]
fix: resolve audio endpoint visibility and handshake stalls

- Fix automation table PropertyCount and linkage in descriptors.rs
- Convert GUIDs to static memory for stable FFI addresses
- Implement IPinCount for WaveRT and Topology miniports
- Remove kd.exe integration from Install.ps1 for faster build cycle
- Update INF version to 1.0.9.0 and fix PKEY_AudioDevice_EnableEndpointByDefault
- Robust tool path resolution in LaunchBuildEnv.ps1
