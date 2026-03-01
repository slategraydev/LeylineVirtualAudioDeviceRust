# Leyline Audio Driver

A native Rust WaveRT virtual audio device. It is currently in Phase 2. The code is technically sound, but the Windows Audio stack is being difficult.

## Current Status

The core adapter logic is stable. `DriverEntry` and `PcInitializeAdapterDriver` are functional. The filters and pins register with `PortCls` as expected. Everything is technically "Working Properly" according to Device Manager.

However, the `Audio Endpoint Builder` is not enumerating endpoints on `Hyper-V`. My `MMDeviceEnumerator` calls return zero devices. This is particularly annoying because the C++ reference implementation works fine in the same environment. I am currently checking if the headless VM state is keeping the service idle, or if I missed a `PKEY` in the `INF` policy. The logs are silent. I am auditing registry keys to see why the builder is ignoring the topology.

## Architecture

The project is split to minimize kernel-mode complexity.

`leyline-kernel` handles the `no_std` environment and the `COM` vtables required for `WaveRT`. It is designed to stay out of the way of the hardware-agnostic streaming logic.

`leyline-shared` contains the ring buffer implementation and the math for `QPC` to byte-offset conversions. It avoids drift without requiring float logic in the hot path.

`src/HSA` is the Hardware Support App bridge. It uses `WinUI 3` and will eventually handle routing via `IOCTL` calls. It is currently a work in progress.

## Environment

This requires the Enterprise `WDK` (26H1/28000) and `Rust` 1.75 or newer. `LLVM` 17.0.6 is used for consistent `bindgen` output.

## Workflow

```powershell
# Setup the environment.
.\scripts\LaunchBuildEnv.ps1

# Build the system and setup files.
cargo make build

# Install to the local machine. Requires testsigning.
cargo make install
```
