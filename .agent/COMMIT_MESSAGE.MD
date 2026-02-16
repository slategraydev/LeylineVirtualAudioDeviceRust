# Session Commit Message Template

## Plain Text Commit Message [COPY-PASTE]
feat: restore HSA IOCTL bridge and FDO linkage

Restored the communication bridge between the Hardware Support App and the modularized kernel driver.

- Implemented IOCTL handling for status retrieval and buffer mapping in dispatch.rs.
- Established a global Functional Device Object (FDO) reference for cross-dispatch access.
- Updated StartDevice to capture the FDO reference for stable IOCTL handling.
- Verified 100% warning-free build for the kernel and shared crates.

## Detailed Impact Bullets
- **HSA Integration**: Restored functionality for GET_STATUS and MAP_BUFFER IOCTLs to support real-time monitoring.
- **Architectural Linkage**: Implemented FDO-CDO linkage to allow standalone IOCTL routines to access adapter state.
- **Verification**: Achieved zero-warning baseline proof via cargo-wdk release build.

**Reviewer**: Gemini CLI (Gemini 2.0 Flash)
**Date**: February 16, 2026
