// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

use std::env;
use std::path::PathBuf;

fn main() {
    let wdk_root = env::var("WDKContentRoot")
        .unwrap_or_else(|_| "C:\\Program Files (x86)\\Windows Kits\\10".to_string());
    let wdk_version =
        env::var("WindowsTargetPlatformVersion").unwrap_or_else(|_| "10.0.26100.0".to_string());

    let wdk_root_trimmed = wdk_root.trim_end_matches('\\');
    let wdk_version_trimmed = wdk_version.trim_end_matches('\\');
    let wdk_root_path = PathBuf::from(wdk_root_trimmed);

    println!(r"cargo:rustc-link-search=native={wdk_root_trimmed}\Lib\{wdk_version_trimmed}\km\x64");

    let inc_km = wdk_root_path
        .join("Include")
        .join(wdk_version_trimmed)
        .join("km");
    let inc_shared = wdk_root_path
        .join("Include")
        .join(wdk_version_trimmed)
        .join("shared");
    let inc_crt = wdk_root_path
        .join("Include")
        .join(wdk_version_trimmed)
        .join("km")
        .join("crt");

    let mut bindings = bindgen::Builder::default()
        .header("src/audio_wrapper.h")
        .use_core()
        .ctypes_prefix("core::ffi")
        .clang_arg("-D_AMD64_")
        .clang_arg("-D_KERNEL_MODE")
        .blocklist_type("GUID")
        .blocklist_type("_GUID")
        .blocklist_type("ULONG")
        .blocklist_type("LONGLONG")
        .raw_line("use wdk_sys::GUID;")
        .raw_line("pub type _GUID = GUID;")
        .raw_line("pub type ULONG = core::ffi::c_ulong;")
        .raw_line("pub type LONGLONG = core::ffi::c_longlong;")
        .raw_line("#[repr(C)] #[derive(Copy, Clone)] pub struct KSDATAFORMAT { pub FormatSize: ULONG, pub Flags: ULONG, pub SampleSize: ULONG, pub Reserved: ULONG, pub MajorFormat: GUID, pub SubFormat: GUID, pub Specifier: GUID, }")
        .raw_line("pub type KSDATARANGE = KSDATAFORMAT;")
        .raw_line("pub type PKSDATARANGE = *mut KSDATAFORMAT;")
        .raw_line("#[repr(C)] #[derive(Copy, Clone)] pub struct PCCONNECTION_DESCRIPTOR { pub FromNode: ULONG, pub FromNodePin: ULONG, pub ToNode: ULONG, pub ToNodePin: ULONG, }")
        .raw_line("pub type PPCCONNECTION_DESCRIPTOR = *mut PCCONNECTION_DESCRIPTOR;")
        .raw_line("#[repr(C)] #[derive(Copy, Clone)] pub struct KSPIN_DESCRIPTOR { pub InterfacesCount: ULONG, pub Interfaces: *const core::ffi::c_void, pub MediumsCount: ULONG, pub Mediums: *const core::ffi::c_void, pub DataRangesCount: ULONG, pub DataRanges: *const *mut KSDATAFORMAT, pub DataFlow: i32, pub Communication: i32, pub Category: *const GUID, pub Name: *const GUID, pub Reserved: LONGLONG, pub Reserved2: LONGLONG, }");

    let s_km = inc_km.to_str().unwrap();
    let s_shared = inc_shared.to_str().unwrap();
    let s_crt = inc_crt.to_str().unwrap();

    bindings = bindings.clang_arg("-I".to_owned() + s_km);
    bindings = bindings.clang_arg("-I".to_owned() + s_shared);
    bindings = bindings.clang_arg("-I".to_owned() + s_crt);

    let generated = bindings
        .allowlist_type(".*WAVEFORMAT.*")
        .allowlist_type(".*KSSTATE.*")
        .allowlist_type(".*PCPIN_DESCRIPTOR.*")
        .allowlist_type(".*PCFILTER_DESCRIPTOR.*")
        .blocklist_type("KSDATAFORMAT")
        .blocklist_type("KSDATARANGE")
        .blocklist_type("PKSDATARANGE")
        .blocklist_type("_KSDATAFORMAT")
        .blocklist_type("KSTOPOLOGY_CONNECTION")
        .blocklist_type("PCCONNECTION_DESCRIPTOR")
        .blocklist_type("PPCCONNECTION_DESCRIPTOR")
        .blocklist_type("KSPIN_DESCRIPTOR")
        .blocklist_type("_KSPIN_DESCRIPTOR")
        .allowlist_var("KSSTATE_.*")
        .allowlist_var("KSDATAFORMAT_.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_file = out_path.join("audio_bindings.rs");
    generated
        .write_to_file(&bindings_file)
        .expect("Couldn't write bindings!");

    if env::var("CARGO_CFG_TEST").is_err() {
        println!("cargo:rustc-link-arg=/subsystem:native");
        println!("cargo:rustc-link-arg=/driver");
        println!("cargo:rustc-link-arg=/entry:DriverEntry");
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:msvcrt");
        println!("cargo:rustc-link-lib=ntoskrnl");
        println!("cargo:rustc-link-lib=hal");
        println!("cargo:rustc-link-lib=wmilib");
        println!("cargo:rustc-link-lib=portcls");
    }

    println!("cargo:rerun-if-env-changed=WDKContentRoot");
    println!("cargo:rerun-if-env-changed=WindowsTargetPlatformVersion");
    println!("cargo:rerun-if-changed=src/audio_wrapper.h");
}
