# Session Commit: Build Restoration & Custom Bindings

**Reviewer**: Antigravity (Gemini 2.0 Flash)
**Date**: February 16, 2026

## Summary
Restored the build toolchain following the horizontal refactor by implementing a custom bindgen pipeline for audio-specific WDK headers.

## Changes
- **feat(kernel)**: Added `audio_wrapper.h` and `bindgen` logic to `build.rs`.
- **fix(kernel)**: Resolved type ambiguities in `stream.rs` using explicit namespacing.
- **feat(shared)**: Enhanced `leyline-shared` with position tracking and buffer management methods.
- **chore(kernel)**: Updated `wavert.rs` and `descriptors.rs` for `bindgen` union compatibility.

## Git Commit Payload
```
feat: implement custom bindgen pipeline for audio headers

- Added audio_wrapper.h to include PortCls and KS headers.
- Updated build.rs to generate Rust bindings for missing audio types.
- Resolved type collisions between wdk-sys and custom bindings.
- Updated stream and wavert modules to handle nested union fields.
- Enhanced leyline-shared with math and buffer interface methods.
```
