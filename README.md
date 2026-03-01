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

## Building & Deployment

### Prerequisites

- **eWDK** (Enterprise WDK) at `D:\eWDK_28000` or set `LEYLINE_EWDK_ROOT`.
- Windows 10 / 11 SDK `10.0.28000.0` or set `LEYLINE_SDK_VERSION`.
- **Rust Toolchain** 1.75+ (no_std) with `cargo-make` installed.
- A Hyper-V VM named `LeylineTestVM` (for automated deployment testing).

### Quick Start (Scripts)

The quickest way to build and deploy is using the provided PowerShell scripts. These handle environment initialization, certificate generation, and VM orchestration.

```powershell
# 1. Initialize the build environment (Run once per session)
.\scripts\LaunchBuildEnv.ps1

# 2. Build, package, and install to the target VM
.\scripts\Install.ps1

# 3. Clean build (force re-build of all components)
.\scripts\Install.ps1 -clean

# 4. Uninstall from target VM
.\scripts\Uninstall.ps1
```

### Manual Build (No Scripts)

For granular control or debugging, you can build and deploy components individually without using the helper scripts.

#### 1. Environment Setup
You must manually set up the eWDK environment variables to use `cl.exe`, `link.exe`, and `cargo-wdk`.
```powershell
# Note: LaunchBuildEnv.ps1 is still recommended for this step
.\scripts\LaunchBuildEnv.ps1
```

#### 2. Component Builds

**Kernel-Mode Driver (Rust):**
```powershell
cd crates/leyline-kernel
cargo wdk build --profile release
```
*Artifacts are located in `target/release/leyline_package/`.*

**Audio Processing Object (APO - C++):**
```powershell
cd src/APO
midl /nologo /header LeylineAPO_h.h LeylineAPO.idl
cl /nologo /W4 /Zi /O2 /EHa /D_USRDLL /D_WINDLL /c LeylineAPO.cpp dllmain.cpp
link /nologo /dll /def:LeylineAPO.def /out:LeylineAPO.dll LeylineAPO.obj dllmain.obj user32.lib ole32.lib oleaut32.lib
```

**Hardware Support App (HSA - C#):**
```powershell
dotnet build src/HSA/LeylineHSA.csproj -c Release
```

#### 3. Packaging & Signing
The driver requires a valid signature to load.
```powershell
# Sign the driver binary and catalog
signtool sign /f leyline.pfx /p "leyline" /fd SHA256 package/leyline.sys
signtool sign /f leyline.pfx /p "leyline" /fd SHA256 package/leyline.cat
```

#### 4. Manual Installation
On the target machine, run the following as Administrator:
```powershell
# 1. Trust the test certificate
certutil -addstore -f root leyline.cer
certutil -addstore -f TrustedPublisher leyline.cer

# 2. Install the driver
pnputil /add-driver leyline.inf /install
devcon update leyline.inf "ROOT\MEDIA\LeylineAudio"

# 3. Restart audio services
Restart-Service AudioEndpointBuilder
Restart-Service Audiosrv
```

## Environment Variables


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
