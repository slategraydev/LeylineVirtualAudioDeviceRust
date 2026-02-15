feat: finalize portable and isolated build environment

Transitioned the project to a 100% contained eWDK toolchain with a portable build script.

Changes:
- Refactored uild.rs to dynamically detect WDK and SDK versions from environment variables.
- Fixed environment capture logic in LaunchBuildEnv.ps1.
- Implemented PortCls initialization in DriverEntry.
- Confirmed Native Subsystem (1) for leyline.sys.
- Organized all build scripts into the scripts/ directory.

Verification Proof:
- Build: Success (0 warnings) using D:\eWDK_28000.
- Device: Successfully loaded as a PortCls adapter.

Git Commit Payload:
feat: portable build script and environment containment
