feat: Implement dynamic formats and fix installer

- Implemented `DataRangeIntersection` in `leyline-kernel` for 44.1k-192k support.
- Updated `leyline.inx` to name endpoints "Leyline Output" and "Leyline Input".
- Updated `install_driver.ps1` to use `devcon` for virtual device creation.
- Hardened `build_apo.ps1` toolchain paths.
- **Known Issue**: HSA silent launch failure; installer creates duplicates.
