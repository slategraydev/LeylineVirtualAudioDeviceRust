# Leyline Audio Driver

Native Rust WaveRT implementation for Windows. Built to automate virtual audio routing.

## Current Status
- **Kernel Core**: DriverEntry and PortCls filter registration are stable.
- **Enumeration**: Currently failing. The Audio Endpoint Builder (AEB) does not enumerate endpoints on Hyper-V. No events are present in operational logs. This is likely a headless VM or INF constraint.
- **HSA**: WinUI 3 bridge is in progress.

## Specification
Architectural requirements are defined in [GEMINI.MD](GEMINI.MD). Review this documentation before modifying the driver core.

## Environment
- **Platform**: Windows 10/11 (x64).
- **Toolchain**: eWDK (26H1/28000), Rust 1.75+ (no_std), LLVM 17.0.6.
- **Workflow**: cargo-wdk for packaging, cargo-make for automation.

## Workflow
```powershell
# Initialize eWDK environment.
.\scripts\LaunchBuildEnv.ps1

# Build sys and inf artifacts.
cargo make build

# Install on target (requires testsigning).
cargo make install
```
