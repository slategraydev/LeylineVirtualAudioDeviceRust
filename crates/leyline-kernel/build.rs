// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

use std::env;
use std::path::PathBuf;

fn main() -> Result<(), wdk_build::ConfigError> {
    wdk_build::Config::from_env_auto()?.configure_binary_build()?;

    println!("cargo:rustc-link-lib=Acx01000");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/audio_wrapper.h");

    let wdk_root = env::var("WDKContentRoot")
        .or_else(|_| env::var("LEYLINE_EWDK_ROOT").map(|r| format!("{}\\Program Files\\Windows Kits\\10", r.trim_end_matches('\\'))))
        .unwrap_or_else(|_| "C:\\Program Files (x86)\\Windows Kits\\10".to_string());
    let wdk_version = env::var("WindowsTargetPlatformVersion")
        .or_else(|_| env::var("LEYLINE_SDK_VERSION"))
        .unwrap_or_else(|_| "10.0.28000.0".to_string());

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
    // WDF headers are NOT under the versioned Include directory.
    // They live at: Include\wdf\kmdf\1.33
    let inc_wdf = wdk_root_path
        .join("Include")
        .join("wdf")
        .join("kmdf")
        .join("1.33");
    // ACX headers live at: Include\{version}\km\acx\km\1.1
    let inc_acx = wdk_root_path
        .join("Include")
        .join(wdk_version_trimmed)
        .join("km")
        .join("acx")
        .join("km")
        .join("1.1");

    let mut bindings = bindgen::Builder::default()
        .header("src/audio_wrapper.h")
        .use_core()
        .ctypes_prefix("core::ffi")
        .clang_arg("-D_AMD64_")
        .clang_arg("-D_KERNEL_MODE")
        // KMDF version defines (required by wdf.h function table)
        .clang_arg("-DKMDF_VERSION_MAJOR=1")
        .clang_arg("-DKMDF_VERSION_MINOR=33")
        // ACX version defines (required by acx.h headers)
        .clang_arg("-DACX_VERSION_MAJOR=1")
        .clang_arg("-DACX_VERSION_MINOR=1")
        .blocklist_type("GUID")
        .blocklist_type("_GUID")
        .blocklist_type("ULONG")
        .blocklist_type("LONGLONG")
        .raw_line("use wdk_sys::*;")
        .raw_line("pub type _GUID = GUID;")
        .raw_line("pub type ULONG = core::ffi::c_ulong;")
        .raw_line("pub type LONGLONG = core::ffi::c_longlong;");

    let s_km = inc_km.to_str().unwrap();
    let s_shared = inc_shared.to_str().unwrap();
    let s_crt = inc_crt.to_str().unwrap();
    let s_wdf = inc_wdf.to_str().unwrap();
    let s_acx = inc_acx.to_str().unwrap();

    bindings = bindings.clang_arg("-I".to_owned() + s_km);
    bindings = bindings.clang_arg("-I".to_owned() + s_shared);
    bindings = bindings.clang_arg("-I".to_owned() + s_crt);
    bindings = bindings.clang_arg("-I".to_owned() + s_wdf);
    bindings = bindings.clang_arg("-I".to_owned() + s_acx);

    let generated = bindings
        .allowlist_type(".*WDF.*")
        .allowlist_type(".*ACX.*")
        .allowlist_type(".*WAVEFORMAT.*")
        .allowlist_var(".*WDF.*")
        .allowlist_var(".*ACX.*")
        .allowlist_var(".*Acx.*")
        .allowlist_function(".*Wdf.*")
        .allowlist_function(".*Acx.*")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_file = out_path.join("audio_bindings.rs");
    generated
        .write_to_file(&bindings_file)
        .expect("Couldn't write bindings!");

    Ok(())
}
