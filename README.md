# Leyline Audio Driver

A native Rust WaveRT virtual audio device. The code is technically sound (in my opinion), but the Windows Audio stack is being difficult.

## Current Status

The core adapter logic is stable. `DriverEntry` and `PcInitializeAdapterDriver` are functional. The filters and pins register with `PortCls` as expected. Everything is technically "Working Properly" according to Device Manager.

However, the `Audio Endpoint Builder` is not enumerating endpoints on `Hyper-V`. My `MMDeviceEnumerator` calls return zero devices. This is pretty weird because my C++ reference implementation works fine in the same environment.

I am currently investigating why the builder is ignoring the topology. The focus is on:
-   **Subdevice Registration**: Ensuring the naming used in `PcRegisterSubdevice` mirrors the C++ reference exactly.
-   **Physical Connections**: Verifying that the internal routing between the Wave and Topology filters is correctly wired via `PcRegisterPhysicalConnection`.
-   **INF Policy**: Auditing registry keys for missing `PKEY` properties that the AEB requires to recognize the device as a valid endpoint.


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
