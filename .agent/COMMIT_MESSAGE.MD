feat: Implement dynamic format negotiation and robust APO build

- Implemented `DataRangeIntersection` in `leyline-kernel` to support 44.1kHz-192kHz.
- Updated `leyline.inx` to name endpoints "Leyline Output" and "Leyline Input".
- Hardened `scripts/build_apo.ps1` with manual eWDK paths to bypass shell errors.
- Verified Zero-Warning build across Kernel, HSA, and APO.
