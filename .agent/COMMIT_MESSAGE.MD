feat: Implement HSA Bridge and Real-Time Visualization

Implemented the zero-copy communication channel between the kernel driver and the Hardware Support App (HSA).

*   **Bridge**: Implemented `IOCTL_LEYLINE_MAP_BUFFER` to map kernel loopback pages to user-mode.
*   **Sync**: Added QPC (QueryPerformanceCounter) timing and buffer metadata to `SharedParameters`.
*   **HSA**: Implemented `DriverBridge` and `MainWindow` oscilloscope using direct memory access.
*   **Cleanup**: Added automatic unmapping logic in `dispatch_close` to prevent memory leaks.
*   **Fix**: Standardized IOCTL codes using `CTL_CODE` macro logic.

**Verification:**
*   `leyline-kernel`: Build Success (0 warnings).
*   `LeylineHSA`: Build Success (0 warnings).
