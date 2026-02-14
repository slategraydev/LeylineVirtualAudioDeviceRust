# Leyline Audio Driver

Native Rust WaveRT Audio Driver for Windows.

## Technical Specification
Refer to [GEMINI.MD](GEMINI.MD) for the full architectural specification.

## Development Environment
- Windows 10/11 x64/ARM64
- eWDK (Enterprise Windows Driver Kit)
- Rust with `cargo-wdk` and `cargo-make`
- LLVM 17.0.6

## Build Commands
```powershell
# Build the driver
cargo make build

# Install the driver (requires testsigning on)
cargo make install
```
