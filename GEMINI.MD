GEMINI.MD: Leyline Audio Driver Technical Specification

Product Name: Leyline Audio Driver

Version: 1.0.0-PRO

Author: Randall Rosas (Slategray)

Platform: Windows 10/11 (x64, ARM64)

Framework: Rust-native WaveRT (Wave Real-Time)



1\. Professional Licensing \& README Standards

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



7\. Sources and Technical References

1 Microsoft Learn - DCH Design Principles \& Best Practices

2 Microsoft Learn - APIValidator for Universal Drivers

3 Microsoft Learn - Introduction to Port Class Drivers

4 Microsoft Learn - Introduction to WaveRT Port Driver

5 Microsoft Learn - WaveRT Buffer Management \& Direct Access

6 Microsoft Learn - Driver Store Isolation (InfVerif /h)

7 LevelUp - Implementing Ring Buffers in WDM Drivers

8 Microsoft Learn - IMiniportWaveRTStream Interface

9 Microsoft Learn - WaveRT Miniport Best Practices

10 Microsoft Q\&A - Fixing Unreported Latency Failures in HLK

11 Sonarworks - WASAPI Exclusive Mode vs ASIO Architecture

12 Microsoft Learn - Low Latency Audio DDI Standards (Windows 10/11)

13 Microsoft Learn - FSLogix Profile Persistence for Multi-session

14 Microsoft Learn - Windows Enterprise Multi-session FAQ

15 Microsoft Learn - Managing Drivers via Intune \& SCCM

16 Microsoft Learn - HVCI Driver Compatibility Requirements

17 Microsoft Tech Community - InfVerif /h Rule Enforcement (25H2)

18 Microsoft Learn - Driver Package Isolation Validation

19 Steinberg SDK - Legacy Licensing Constraints

20 Microsoft Learn - Glitch Detection via FIFO Interrupts

21 Wikipedia - Audio Fidelity Testing for Windows Logo

22 NicSRS - WHQL Certification Requirements 2025

23 DevelopEx - Optimizing WHQL Workflow for Virtual Drivers

24 Microsoft Learn - eWDK and VS Build Tool Requirements

25 LevelUp - Cross-Pin Synchronization in WaveRT Drivers

26 Microsoft GitHub - windows-drivers-rs (Official Platform)

27 Microsoft Learn - The Test-Signing Boot Configuration

12 Microsoft Learn - WASAPI Exclusive Mode Low Latency API

HydrogenAudio - Bit Transparency in WASAPI Exclusive

Ferrocene Developers - Professional Rust Copyright Header Standards

Linux Kernel Docs - Rust Safety Documentation Standards

Works cited

Universal Windows Drivers for Audio - Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/audio-universal-drivers

Universal Windows Drivers for Audio - GitHub, accessed February 14, 2026, https://github.com/MicrosoftDocs/windows-driver-docs/blob/staging/windows-driver-docs-pr/audio/audio-universal-drivers.md

Introduction to Port Class - Windows drivers | Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/introduction-to-port-class

Introducing the WaveRT Port Driver - Windows - Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/introducing-the-wavert-port-driver

Understanding the WaveRT Port Driver - Windows - Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/understanding-the-wavert-port-driver

Sample Audio Drivers - Windows drivers | Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/sample-audio-drivers

A Guide to Kernel Mode Audio Device Driver Development in ..., accessed February 14, 2026, https://levelup.gitconnected.com/building-a-virtual-audio-cable-with-a-custom-wdm-driver-a-guide-to-kernel-mode-audio-device-25931fac999d

WaveRT Miniport Driver - Windows - Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/wavert-miniport-driver

Developing a WaveRT Miniport Driver - Windows drivers | Microsoft ..., accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/developing-a-wavert-miniport-driver

window development - Microsoft Q\&A, accessed February 14, 2026, https://learn.microsoft.com/en-us/answers/questions/5665323/window-development

\[WIN] Virtual ASIO to ASIO driver (ASIO to ASIO) - Sonarworks Support, accessed February 14, 2026, https://support.sonarworks.com/hc/en-us/articles/4404011605906--WIN-Virtual-ASIO-to-ASIO-driver-ASIO-to-ASIO

Low Latency Audio - Windows drivers | Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/low-latency-audio

Merging Audio Device - MAD, accessed February 14, 2026, https://www.merging.com/products/merging\_audio\_device

Windows Enterprise multi-session FAQ - Azure | Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/azure/virtual-desktop/windows-multisession-faq

Allowing more than 2 users to connect to Windows Enterprise Multi Session through AVD, accessed February 14, 2026, https://learn.microsoft.com/en-au/answers/questions/5642423/allowing-more-than-2-users-to-connect-to-windows-e

Memory integrity and VBS enablement - Microsoft, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/oem-hvci-enablement

Driver Isolation Ruleset Changes for WHCP and Attestation Submissions, accessed February 14, 2026, https://techcommunity.microsoft.com/blog/hardware-dev-center/driver-isolation-ruleset-changes-for-whcp-and-attestation-submissions/4461580

Microsoft commits to making Windows drivers better through strict WHCP policies - Neowin, accessed February 14, 2026, https://www.neowin.net/news/microsoft-commits-to-making-windows-drivers-better-through-strict-whcp-policies/

Microsoft Ignite: Windows driver development news - NTDEV, accessed February 14, 2026, https://community.osr.com/t/microsoft-ignite-windows-driver-development-news/59923

ACX Streaming - Windows drivers | Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/acx-streaming

WHQL Testing - Wikipedia, accessed February 14, 2026, https://en.wikipedia.org/wiki/WHQL\_Testing

What Is WHQL Certification and How to Obtain It?, accessed February 14, 2026, https://www.nicsrs.com/blog/what-is-whql-certification-and-how-to-obtain-it

WHQL Certification: Workflow \& Optimization (Part 2) - Developex, accessed February 14, 2026, https://developex.com/blog/whql-certification-workflow-optimization/

Download the Windows Driver Kit (WDK) - Microsoft Learn, accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/download-the-wdk

A list of open-source C++ libraries - cppreference.com, accessed February 14, 2026, https://en.cppreference.com/w/cpp/links/libs.html

microsoft/windows-drivers-rs: Platform that enables ... - GitHub, accessed February 14, 2026, https://github.com/microsoft/windows-drivers-rs

microsoft/Windows-rust-driver-samples: Rust port of the ... - GitHub, accessed February 14, 2026, https://github.com/microsoft/Windows-rust-driver-samples

Virtual Audio Driver SDK for Windows 10 and 11 - Virtual Audio Cable, accessed February 14, 2026, https://www.thesycon.de/eng/virtual\_audiodriver.shtml

Audio Processing Object Architecture - Windows drivers | Microsoft ..., accessed February 14, 2026, https://learn.microsoft.com/en-us/windows-hardware/drivers/audio/audio-processing-object-architecture

WHQL Certification: Benefits \& Test Lab Setup (Part 1) - Developex, accessed February 14, 2026, https://developex.com/blog/whql-certification-setup/



