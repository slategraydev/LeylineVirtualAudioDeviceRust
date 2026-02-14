GEMINI.MD: Leyline Audio Driver Technical Specification

Product Name: Leyline Audio Driver

Version: 1.0.0-PRO

Author: Randall Rosas (Slategray)

Platform: Windows 10/11 (x64, ARM64)

Framework: Rust-native WaveRT (Wave Real-Time)



## Current Status: Building Phase 1 (Core Foundations)

### Completed ✅
- **[x] Project Archetype**: All directory "slots" established (APO, HSA, Test, Kernel).
- **[x] IP Protection**: Mandatory professional headers applied to all source files.
- **[x] Kernel Core**: `DriverEntry` and `DispatchDeviceControl` (IOCTL) foundations implemented.
- **[x] WaveRT Scaffolding**: `MiniportWaveRT` and `MiniportWaveRTStream` structures with interface placeholders.
- **[x] Shared Logic**: Ring Buffer implementation and cross-crate GUID/IOCTL definitions.
- **[x] Build Tooling**: Comprehensive `Makefile.toml` for `cargo-make` workflow.

### In Progress 🏗️
- **[/] Trait Implementation**: Mapping foundational structures to `wdk` traits.
- **[/] Shared Constants**: Expanding `leyline-shared` with driver-specific GUIDs.

### Upcoming 📅
- **[ ] APO Component**: Implementation of C++ Audio Processing Objects for effects.
- **[ ] HSA Integration**: Development of the WinUI 3 Hardware Support App.
- **[ ] Advanced WaveRT**: MMCSS coordination and low-latency periodicity tuning.

1. Professional Licensing & README Standards

As a solo developer project published on GitHub, the licensing must balance public transparency with your ownership rights.



1.1 Ownership \& Attribution

Every source file must include the following professional header to preserve your IP:



Rust





// Copyright (c) 2026 Randall Rosas (Slategray).

// All rights reserved.

//

// This source code is provided for educational and review purposes. 

// Redistribution and use in binary form without express permission is prohibited.

// See LICENSE file in the project root for full terms.





1.2 Recommended License: Polyform Shield 1.0.0

Since you want to avoid third-party SDK licensing (like Steinberg) and maintain a professional solo-commercial posture:

Polyform Shield: Allows people to read, study, and modify the code, but prohibits them from using it to compete with your product or offer it as a service. It is a modern, "Source-Available" standard that feels professional without being "corporate."

Alternative (Permissive): If you prefer standard Open Source, use Apache License 2.0. It provides explicit patent grants, which is highly recommended for kernel-level work.



2\. Core Architecture: Rust-Powered WaveRT

The Leyline Audio Driver is a software-only virtual adapter utilizing the WaveRT (Wave Real-Time) port-miniport architecture. This is the gold standard for modern, glitch-resilient Windows audio.



2.1 The Rust Kernel Stack

To eliminate memory safety vulnerabilities, the core is implemented in Rust using the official Microsoft driver crates.

Framework: windows-drivers-rs using wdk and wdk-sys.

Configuration:

cdylib crate type for the .sys binary.

panic = "abort" to prevent kernel-mode unwinding.

\#!\[no\_std] environment (mandatory for kernel).

Safety Pattern: Every unsafe block must be preceded by a // SAFETY: comment justifying the invariants (e.g., pointer validity or IRQL level).



2.2 Native Low-Latency (No ASIO SDK Needed)

Instead of relying on the Steinberg ASIO SDK, Leyline implements a native WASAPI Exclusive Mode path.

IAudioClient3 Integration: The driver implements small-buffer DDIs to support 1ms to 2ms periodicity, allowing pro-audio applications to achieve ASIO-level latency using native Windows APIs.

MMCSS Coordination: All streaming threads are registered with the Multimedia Class Scheduler Service to prevent CPU preemption during high-load sessions.

Clock Synchronization: Uses the system's high-precision timestamp to maintain a consistent master clock across the Leyline Output and Leyline Input endpoints. 28



/LeylineAudioDriver

├── /crates

│   ├── /leyline-kernel      # Core Rust Driver (cdylib)

│   │   ├── src/lib.rs       # IMiniportWaveRT \& DriverEntry

│   │   ├── src/stream.rs    # IMiniportWaveRTStream 

│   │   ├── src/buffer.rs    # Shared Ring Buffer logic 

│   │   └── leyline.inx      # Driver installation template

│   └── /leyline-shared      # Shared GUIDs and IOCTL codes

├── /src

│   ├── /APO                 # C++ Audio Processing Objects (SFX/MFX/EFX) 

│   └── /HSA                 # WinUI 3 Hardware Support App (MSIX)

├── /test

│   ├── /WDUTF               # Unit tests for kernel code

│   └── /Latency             # Automated RTL Utility scripts

├── CODE_REVIEW.md           # Architectural audit
├── TEST_REVIEW.md           # Testing audit
├── BUILD_REVIEW.MD          # Build/Environment audit
├──.gitignore               # Windows/Rust ignore rules

├── LICENSE                  # Polyform or Apache 2.0 text

├── Makefile.toml            # cargo-make task definitions

└── README.md                # Project documentation



4\. Engineering Standards \& Documentation



4.1 Technical Documentation

Rustdoc: All public kernel interfaces must document safety contracts under a # Safety section.

Doxygen: Use JAVADOC-style comments in C++ (APO components) to generate technical manuals.

SAL 2.0: Use \_In\_, \_Out\_, and \_IRQL\_requires\_max\_ annotations for all DDI interactions.



4.2 Git Hygiene (.gitignore)

A specialized .gitignore to prevent leaking private certificates or polluting the repo with build artifacts.

4.3 Coding and Commenting Standards

To maintain professional, enterprise-grade quality, the following standards apply:

Block Headers: Every major section of functionality (traits, implementation blocks, dispatch routines) must be preceded by a distinct block header with a summary of its purpose.

Inline Comments: Focus on explaining the "why" and handling of complex logic or safety invariants. Do not state the obvious "what."

Variable Management:
- No hardcoded values; use named constants defined at the top of the file or in `leyline-shared`.
- Define all local variables at the top of their respective functions (C-style) for maximum clarity in kernel-mode inspection.

Safety: Every `unsafe` block must be preceded by a `// SAFETY:` comment justifying the logic.

4.4 Environment & Build Workflow

The Leyline development environment is standardized to ensure parity across sessions.

### Environment Setup
- **Compiler**: Mount the Enterprise WDK (eWDK) ISO for a self-contained build environment.
- **LLVM**: `winget install LLVM.LLVM --version 17.0.6` (LLVM 18+ has known ARM64 stability issues).
- **Rust Tooling**: `cargo-wdk` for packaging and `cargo-make` for task automation.
- **Command**: `.\LaunchBuildEnv.cmd` must be run before any build task.

4.5 Project Health & Continuity

Continuity of project standards, architectural integrity, and the development roadmap is preserved via a strict hand-off procedure.

### Hand-off & Maintenance Mandate
Every agent participating in this project MUST adhere to the following maintenance routine:
1. **Session Start**: Execute the `/status` workflow (`.agent/workflows/status.md`) immediately.
2. **PROJECT_PROGRESS.MD**: Append a new Session Log with work completed and architectural suggestions.
3. **CODE_REVIEW.md**: Overwrite with a fresh technical audit of the current state.
4. **TEST_REVIEW.md / BUILD_REVIEW.MD**: Update with verification and build status.
5. **GEMINI.MD Sync**: Update Section 3 (Project Structure) if physical files have changed.

### Reference Audit Mandate
When starting a task involving a Windows API, DDI, or WaveRT concept, the agent MUST:
- **Audit References**: Visit external documentation links mentioned in Section 7.
- **Verify Assumptions**: Cross-reference current logic with official Microsoft WDK documentation.

4.6 Spec Evolution Protocol (The "Constitution")

`GEMINI.MD` is the "Living Constitution" of the project. It MUST be evolved with extreme care.
- **Delicate Updates**: The spec is updated ONLY for deliberate architectural decisions, protocol shifts, or structure changes.
- **No Mass Refactors**: Agents are strictly prohibited from performing massive specification or codebase refactors without explicit user approval.
- **Proposals First**: Identify gaps or deficiencies in `CODE_REVIEW.md` before proposing a delicate update to the spec.



Code snippet





\# Rust/C++ Build Artifacts

target/

bin/

obj/

\*.pdb

\*.log



\# Driver Signing (CRITICAL: DO NOT COMMIT)

\*.cer

\*.pfx

\*.pvk



\# Deployment Metadata

\*.cat

\*.sys

\*.inf





5\. Pain-Free Build \& Deployment Workflow



5.1 Environment Setup

Compiler: Mount the Enterprise WDK (eWDK) ISO for a self-contained build environment.

Command: .\\LaunchBuildEnv.cmd

LLVM: winget install LLVM.LLVM --version 17.0.6 (LLVM 18+ has ARM64 issues).

Rust Tooling:

cargo install cargo-wdk (Automates the build/package pipeline).

cargo install --locked cargo-make --features tls-native.



5.2 Build \& Signing Commands

Local Build: cargo wdk build --profile release

This command invokes stampinf, inf2cat, and generates a local WDRLocalTestCert.cer.

Machine Setup: On the test computer, run:

bcdedit /set testsigning on and reboot.

Certificate Deployment:

certutil -addstore root WDRLocalTestCert.cer

certutil -addstore TrustedPublisher WDRLocalTestCert.cer

Installation: pnputil /add-driver leyline.inf /install 1



6\. Testing Architecture



6.1 WDUTF Unit Testing

Uses the Windows Driver Unit Test Framework to execute kernel-logic tests in user-space. This allows you to verify ring buffer wrap-around and synchronization logic without risk of BSOD.



6.2 HLK Certification (Production Path)

To achieve the "Certified for Windows" signature for enterprise deployment: 22

Audio Fidelity: Validates bit-perfect loopback across the shared buffer.

HVCI Compliance: Automated check to ensure the driver is compatible with Memory Integrity.

Unreported Latency: Validates that GetPosition and GetHWLatency accurately reflect real-world delay.



7. Sources and Technical References

This section provides the exhaustive foundational knowledge required for this project. Agents must prioritize these resources for all technical decisions.

### 7.1 Core WaveRT Architecture (Microsoft Learn)
- [Introduction to Port Class](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/introduction-to-port-class) - **Agent Context**: Understanding the relationship between PortCls and the Miniport.
- [Introducing the WaveRT Port Driver](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/introducing-the-wavert-port-driver) - **Agent Context**: Core logic for real-time audio streaming.
- [Understanding the WaveRT Port Driver](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/understanding-the-wavert-port-driver) - **Agent Context**: Deep dive into shared buffer management.
- [Developing a WaveRT Miniport Driver](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/developing-a-wavert-miniport-driver) - **Agent Context**: Tactical implementation guide for `IMiniportWaveRT`.
- [WaveRT Miniport Driver Interfaces](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/wavert-miniport-driver) - **Agent Context**: Reference for mandatory COM-like interfaces.
- [IMiniportWaveRTStream Interface](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/portcls/nn-portcls-iminiportwavertstream) - **Agent Context**: Direct DDI reference for stream control.

### 7.2 Windows Audio Engineering (Low Latency & Universal)
- [DCH Design Principles](https://learn.microsoft.com/en-us/windows-hardware/drivers/develop/dch-compliant-driver-package-example) - **Agent Context**: Ensuring the driver is Declarative, Componentized, and HSA-ready.
- [Universal Windows Drivers for Audio](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/audio-universal-drivers) - **Agent Context**: Requirements for DCH and modern driver packaging.
- [ApiValidator Tool](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/apivalidator) - **Agent Context**: Verifying that all DDI calls are valid for Universal Drivers.
- [Low Latency Audio Guidelines](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/low-latency-audio) - **Agent Context**: Implementing `IAudioClient3` small-buffer paths (1ms-2ms).
- [Audio Processing Object (APO) Architecture](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/audio-processing-object-architecture) - **Agent Context**: Foundation for the `leyline-apo` component.
- [Glitch Reporting for Audio Streams](https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/glitch-reporting-for-offloaded-audio-streams) - **Agent Context**: Using ETW events to report FIFA/buffer glitches back to the OS.

### 7.3 Rust Driver Framework & Standards
- [Windows Drivers in Rust (Official Platform)](https://github.com/microsoft/windows-drivers-rs) - **Agent Context**: Primary repo for `wdk` and `wdk-sys` crates.
- [Rust Driver Samples](https://github.com/microsoft/Windows-rust-driver-samples) - **Agent Context**: Reference implementations for `no_std` kernel logic.
- [Ferrocene Licensing & Headers](https://public-docs.ferrocene.dev/main/licensing.html) - **Agent Context**: Standards for professional copyright headers in safety-critical Rust.
- [Linux Kernel Rust Safety Standards](https://docs.kernel.org/rust/general-information.html) - **Agent Context**: Best practices for `// SAFETY:` comments and `unsafe` documentation.

### 7.4 Certification & Hardware Quality (WHQL/HLK)
- [WHQL Certification Requirements (NicSRS)](https://www.nicsrs.com/blog/what-is-whql-certification-and-how-to-obtain-it) - **Agent Context**: High-level overview of the certification process.
- [Optimizing WHQL Workflow (DevelopEx)](https://developex.com/blog/whql-certification-workflow-optimization/) - **Agent Context**: Specific strategies for virtual/software-only drivers.
- [Fixing Unreported Latency in HLK](https://learn.microsoft.com/en-us/windows-hardware/test/hlk/testref/audio-reliability-test-for-wavert-drivers) - **Agent Context**: Accurate `GetPosition` reporting to prevent HLK failure.
- [HVCI Compatibility Requirements](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/oem-hvci-enablement) - **Agent Context**: Memory integrity and Virtualization-Based Security (VBS) compliance.

### 7.5 Advanced Pro-Audio & Comparison
- [WASAPI Exclusive Mode (HydrogenAudio)](https://wiki.hydrogenaudio.org/index.php?title=WASAPI) - **Agent Context**: Technical breakdown of bit-transparent playback paths.
- [Virtual ASIO to ASIO Architecture (Sonarworks)](https://support.sonarworks.com/hc/en-us/articles/4404011605906--WIN-Virtual-ASIO-to-ASIO-driver-ASIO-to-ASIO) - **Agent Context**: Competitive analysis for bridging WASAPI and ASIO.
- [Building a Virtual Audio Cable (LevelUp)](https://levelup.gitconnected.com/building-a-virtual-audio-cable-with-a-custom-wdm-driver-a-guide-to-kernel-mode-audio-device-25931fac999d) - **Agent Context**: Practical implementation logic for ring buffers and WaveRT pins.

### 7.6 Environment & Build Tools
- [eWDK and WDK Downloads](https://learn.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk) - **Agent Context**: Required toolchains for building the `.sys` and `.inf` packages.
- [Test-Signing Boot Configuration](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/the-test-signing-boot-configuration) - **Agent Context**: Setting up targets for unverified driver execution.

---
*Comprehensive Audit & Reference Mapping Complete: February 14, 2026*



