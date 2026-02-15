feat: PortCls initialization and 100% isolated eWDK build environment

Resolved critical load failures and established production-grade build containment.

- Forced Subsystem 1 (Native) via build.rs to fix 0xc0000034 load error.
- Implemented PcInitializeAdapterDriver and PcAddAdapterDevice.
- Created LaunchBuildEnv.ps1 for eWDK path isolation (D:\eWDK_28000).
- Fixed boxed trait object type mismatches in WaveRT stream initialization.
- Updated GEMINI.MD and GEMINI_PROJECT_SETUP.MD with new architecture mandates.

Verification Proof:
- PE Header Audit: Subsystem 1 (Native) verified.
- Service: Leyline service is RUNNING.
- Build: 0 warnings across all crates.
