feat: Implement APO format negotiation and HSA real-time graph

**Session #06 Summary**:
This session focused on hardening the APO's format negotiation logic and enhancing the HSA with real-time visualization of the shared audio state. It also enforced strict cleanup protocols for verification logs and resolved a pending kernel warning.

**Changes**:
-   **APO**: Implemented `IsInputFormatSupported` in `LeylineAPO.cpp` to strictly enforce `IEEE_FLOAT` stereo format, replacing the placeholder stub.
-   **HSA**: Added a `Polyline` graph to `MainWindow.xaml` and implemented history tracking in `MainWindow.xaml.cs` to visualize the peak levels from shared memory.
-   **Kernel**: Fixed `static_mut_refs` warning in `lib.rs` by using `&raw mut` for the `SHARED_PARAMS` access.
-   **Project**: Updated `GEMINI_PROJECT_SETUP.md` to require recursive cleanup of `*.txt` and `*.log` files to keep the repo clean.
-   **Cleanup**: Removed accumulated log files from `crates/leyline-kernel`.

**Verification**:
-   **Kernel**: `cargo wdk build` -> SUCCESS (0 Errors, 0 Warnings).
-   **HSA**: `dotnet build` -> SUCCESS (0 Errors, 0 Warnings).
-   **APO**: Logic verified via code review (Build environment pending).
-   **Zero-Warning Proof**: Manually verified build logs for "warning" string sufficiency.

**Git Commit Payload**:
```
feat: Implement APO format negotiation and HSA real-time graph

- Implement `IsInputFormatSupported` in `LeylineAPO.cpp` (Float32 Stereo strict).
- Add `Polyline` to `MainWindow.xaml` for audio monitoring.
- Implement graph update logic in `MainWindow.xaml.cs`.
- Fix `static_mut_refs` warning in kernel `lib.rs` using `&raw mut`.
- Update `GEMINI_PROJECT_SETUP.md` with recursive cleanup mandate.
- Clean up verification logs in `crates/leyline-kernel`.
```
