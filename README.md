# Leyline Audio Driver (Rust)

A high-performance, native Rust WaveRT virtual audio driver for Windows. This implementation leverages the `wdk-rs` framework to provide a memory-safe alternative to traditional C++ kernel development.

## Current Status

The core adapter logic is stable, and the driver registers with `PortCls` as expected. However, the `Audio Endpoint Builder` is currently not enumerating the endpoints. This is pretty weird as my C++ implementation initializes correctly in the same environment. Currently looking into this.

## Project Layout

```
LeylineVirtualAudioDeviceRust/
├── crates/
│   ├── leyline-kernel/         # Kernel-mode driver (Rust, no_std, WDM)
│   │   ├── src/
│   │   │   ├── lib.rs          # DriverEntry, DriverUnload, global state
│   │   │   ├── adapter.rs      # AddDevice, StartDevice, DeviceExtension
│   │   │   ├── wavert.rs       # MiniportWaveRT implementation & PortCls callbacks
│   │   │   ├── topology.rs     # MiniportTopology implementation & routing logic
│   │   │   ├── stream.rs       # MiniportWaveRTStream & buffer management
│   │   │   ├── descriptors/    # KS filter, pin, and property descriptors
│   │   │   ├── constants.rs    # PortCls GUIDs and driver-wide constants
│   │   │   ├── dispatch.rs     # IRP dispatch routines for the Control Device Object
│   │   │   └── vtables.rs      # Manual COM vtable construction for no_std
│   │   ├── leyline.inx         # INF template for driver installation
│   │   └── Cargo.toml          # Kernel crate configuration & WDK dependencies
│   └── leyline-shared/         # Common logic shared across kernel/user boundary
│       ├── src/
│       │   ├── lib.rs          # Shared identity GUIDs and IOCTL codes
│       │   ├── buffer.rs       # Cross-process RingBuffer implementation
│       │   └── math.rs         # Precision calculations for WaveRT position
│       └── Cargo.toml          # Shared crate configuration
├── src/
│   ├── APO/                    # Audio Processing Object (C++, COM)
│   └── HSA/                    # Hardware Support App (C#, WinUI 3)
├── scripts/
│   ├── LaunchBuildEnv.ps1      # eWDK & Rust environment initializer
│   ├── Install.ps1             # Automated build, deploy, and verify pipeline
│   └── Uninstall.ps1           # Driver removal and cleanup script
├── test/
│   └── EndpointTester/         # C# utility to verify audio endpoint enumeration
├── package/                    # Staged build artifacts (gitignored)
├── Makefile.toml               # cargo-make task aliases
└── README.md
```

## Architecture

The driver exposes four PortCls subdevices:

| Name              | Class     | Role                                |
|-------------------|-----------|-------------------------------------|
| `WaveRender`      | WaveRT    | Renders audio from client apps      |
| `WaveCapture`     | WaveRT    | Captures audio to client apps       |
| `TopologyRender`  | Topology  | Volume/mute nodes for render path   |
| `TopologyCapture` | Topology  | Bridging for the capture path       |

Physical connections are defined to establish valid signal paths:
- `WaveRender (pin 1)` → `TopologyRender (pin 0)`
- `TopologyCapture (pin 1)` → `WaveCapture (pin 1)`

## Building

### Prerequisites

- **eWDK** (Enterprise WDK) at `D:\eWDK_28000` or set `LEYLINE_EWDK_ROOT`.
- Windows 10 / 11 SDK `10.0.28000.0` or set `LEYLINE_SDK_VERSION`.
- **Rust Toolchain** 1.75+ (no_std) with `cargo-make` installed.
- A Hyper-V VM named `LeylineTestVM` (for automated deployment testing).

### Quick Start

```powershell
# Setup the environment (eWDK + Rust)
.\scripts\LaunchBuildEnv.ps1

# Full build + deploy to local machine
cargo make install

# Build only (all components)
cargo make build

# Clean rebuild
cargo make clean

# Run endpoint verification
cargo make test-endpoints
```

### Environment Variables

| Variable              | Default            | Description                        |
|-----------------------|--------------------|------------------------------------|
| `LEYLINE_EWDK_ROOT`   | `D:\eWDK_28000`   | Root of the eWDK installation      |
| `LEYLINE_SDK_VERSION` | `10.0.28000.0`    | WDK / SDK version string           |
| `LEYLINE_VM_NAME`     | `TestVM`          | Hyper-V VM name                    |
| `LEYLINE_VM_PASS`     | *(required)*      | VM administrator password          |
| `LEYLINE_CERT_PASS`   | *(required)*      | Code-signing certificate password  |

## Key Design Decisions vs. C++

- **Manual COM Vtables**: Unlike the C++ version which uses `CUnknown` and `DECLARE_STD_UNKNOWN()`, the Rust driver manually constructs COM-compliant vtables to interface with `PortCls` in a `no_std` environment.
- **Memory Safety**: Leverages Rust's ownership and borrowing rules to prevent common kernel-mode crashes (e.g., use-after-free) without the overhead of a garbage collector.
- **`no_std` Integration**: The entire kernel component is built without the Rust standard library, relying on `wdk-alloc` for pool-tagged memory management.
- **Binary Integrity**: Enforces `Subsystem 1` (Native) for all kernel binaries via custom linker arguments, ensuring strict adherence to Windows Driver requirements.
- **Descriptor Tables**: Statically initialized in `.rdata` using `#[link_section]`, achieving the same performance characteristics as the C++ implementation.
