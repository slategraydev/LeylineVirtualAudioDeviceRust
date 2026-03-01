# Leyline Virtual Audio Device (Rust)

A high-performance, native Rust WaveRT virtual audio driver for Windows. This implementation leverages the `wdk-rs` framework to provide a memory-safe alternative to traditional C++ kernel development.

## Current Status

The core adapter logic is stable, and the driver registers with `PortCls` as expected. However, the `Audio Endpoint Builder` is currently not enumerating the endpoints. This is pretty weird as my C++ implementation initializes correctly in the same environment. Currently looking into this.

## Project Layout

```
LeylineVirtualAudioDeviceRust/
├── crates/
│   ├── leyline-kernel/         # Kernel-mode driver (Rust, no_std, WDM)
│   └── leyline-shared/         # Common logic shared across kernel/user boundary
├── src/
│   ├── APO/                    # Audio Processing Object (C++, COM)
│   └── HSA/                    # Hardware Support App (C#, WinUI 3)
├── scripts/
│   ├── LaunchBuildEnv.ps1      # eWDK & Rust environment initializer
│   ├── Install.ps1             # Automated build, package, and install pipeline
│   └── Uninstall.ps1           # Driver removal and cleanup script
├── test/
│   └── EndpointTester/         # C# utility to verify audio endpoint enumeration
├── package/                    # Staged build artifacts (gitignored)
├── Makefile.toml               # cargo-make task definitions
└── README.md
```

## Building & Deployment

### Prerequisites

- **eWDK** (Enterprise WDK) at `D:\eWDK_28000` or set `LEYLINE_EWDK_ROOT`.
- Windows 10 / 11 SDK `10.0.28000.0` or set `LEYLINE_SDK_VERSION`.
- **Rust Toolchain** 1.75+ (no_std) with `cargo-make` installed (`cargo install cargo-make`).
- A Hyper-V VM named `LeylineTestVM` (for automated deployment testing).

### 1. Initialize Environment
Before building, you must initialize the eWDK environment in your current PowerShell session:
```powershell
.\scripts\LaunchBuildEnv.ps1
```

### 2. Building All Components
The project uses `cargo-make` to orchestrate the multi-language build (Rust, C++, C#).

| Command | Description |
|---------|-------------|
| `cargo make build-all` | Builds Kernel, APO, and HSA individually. |
| `cargo make build`     | Builds and stages all components into the `package/` folder. |
| `cargo make clean`     | Purges all build artifacts and target directories. |

### 3. Automated Deployment (VM)
To build, sign, and deploy the entire stack to a target Hyper-V VM:
```powershell
# Full build and install to VM
cargo make install

# Clean build and install
.\scripts\Install.ps1 -clean
```

### 4. Individual Component Builds
If you need to work on a specific layer, you can use these targeted tasks:
```powershell
cargo make build-kernel  # Rust Driver
cargo make build-apo     # C++ APO
cargo make build-hsa     # C# WinUI 3 App
```

## Key Design Decisions

- **Unified Build System**: Uses `cargo-make` to bridge the gap between Rust's `cargo`, C++'s `nmake`, and C#'s `dotnet`.
- **Automatic Staging**: All successful builds are staged in the `package/` folder and signed automatically for kernel-mode compatibility.
- **Software Component Integration**: The INF automatically registers the HSA as a Software Component, allowing for modern hardware-app associations.
- **APO Multi-Effect Registration**: Registers the Leyline APO for Stream, Mode, and Endpoint effects during installation.

---
**Last Updated**: February 28, 2026
