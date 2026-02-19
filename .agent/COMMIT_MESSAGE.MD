feat(driver,infra): Diagnosis and strict compliance updates for AEB support

Detailed changes:
- **Driver Core (`descriptors.rs`)**:
  - Updated `TOPO_RENDER_PINS` and `TOPO_CAPTURE_PINS` to use `core::ptr::null()` for `AutomationTable` on bridge pins.
  - Aligns with Microsoft SYSVAD reference behavior for standard property handling.

- **INF (`leyline.inx`)**:
  - Removed legacy WDM registration keys (`AssociatedFilters`, `wdmaud.drv`) to prevent modern audio stack conflicts.
  - Added explicit `PKEY_AudioEndpoint_FormFactor` definitions (Speakers/Microphone) to aid discovery.
  - Disabled `PKEY_AudioEndpoint_Supports_EventDriven_Mode` to mitigate potential Hyper-V WaveRT limitations.
  - Temporarily commented out all APO references (`LeylineAPO.dll`, FX properties) to isolate endpoint creation failures.

- **Build Infrastructure (`LaunchBuildEnv.ps1`)**:
  - Patched `LIB` environment variable to include User Mode (`um`) and Universal CRT (`ucrt`) library paths.
  - Resolves `LNK1181` and `LNK1104` errors for host-side `build.rs` compilation steps.

- **Outcome**:
  - Driver loads successfully.
  - Interfaces openable via `CreateFile` (Wave/Topo).
  - AEB failure diagnosed as environment-specific (Hyper-V headless state) with zero operational events.
