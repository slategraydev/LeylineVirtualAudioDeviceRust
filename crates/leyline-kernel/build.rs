// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

fn main() {
    // 1. Pull paths from eWDK environment variables set by LaunchBuildEnv.ps1
    let wdk_root = std::env::var("WDKContentRoot")
        .unwrap_or_else(|_| "C:\\Program Files (x86)\\Windows Kits\\10".to_string());

    let wdk_version = std::env::var("WindowsTargetPlatformVersion")
        .unwrap_or_else(|_| "10.0.26100.0".to_string());

    // Path to Kernel Mode libraries (using detected version)
    println!(
        "cargo:rustc-link-search=native={}\\Lib\\{}\\km\\x64",
        wdk_root.trim_end_matches('\\'),
        wdk_version.trim_end_matches('\\')
    );

    // Mandatory Kernel Linker Flags
    println!("cargo:rustc-link-arg=/subsystem:native");
    println!("cargo:rustc-link-arg=/driver");
    println!("cargo:rustc-link-arg=/entry:DriverEntry");

    // Core Kernel Libraries
    println!("cargo:rustc-link-lib=ntoskrnl");
    println!("cargo:rustc-link-lib=hal");
    println!("cargo:rustc-link-lib=wmilib");
    println!("cargo:rustc-link-lib=portcls");

    // Re-run if environment changes
    println!("cargo:rerun-if-env-changed=WDKContentRoot");
    println!("cargo:rerun-if-env-changed=WindowsTargetPlatformVersion");
}
