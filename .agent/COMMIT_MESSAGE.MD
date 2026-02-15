feat: refactor shared logic and implement host-side unit testing

Decoupled core math and buffer logic into `leyline-shared` and implemented host-side unit testing.

Changes:
- Moved `RingBuffer` and `WaveRTMath` to `leyline-shared` crate.
- Added 4 unit tests covering buffer wrapping and WaveRT position arithmetic.
- Refactored `leyline-kernel` to use `leyline-shared` exports.
- Modified `build.rs` to allow `cargo test` to run on the kernel crate without kernel linker flags.
- Confirmed zero-warning build for both crates.

Verification Proof:
- `cargo test -p leyline-shared`: 4 tests passed.
- `cargo wdk build`: Success (0 warnings).

Git Commit Payload:
feat: shared logic refactor and unit testing
- Decoupled RingBuffer and WaveRTMath to leyline-shared.
- Implemented unit tests for audio buffer and math primitives.
- Updated leyline-kernel for shared logic consumption.
- Refined build script for user-space testing compatibility.
