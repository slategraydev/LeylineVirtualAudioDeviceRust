feat: Integrate APO pipeline and register endpoints

- Created `scripts/build_apo.ps1` to robustly build the C++ APO (using `vcvarsall.bat` and `nmake`).
- Updated `leyline.inx` to register the APO CLSID for "WaveRender" and "WaveCapture" interfaces.
- Corrected INF reference strings to match `PcRegisterSubdevice` names.
- Resolved all C++ warnings (`C4100`) in `dllmain.cpp`.
- Verified full package build (Kernel, HSA, APO) with zero warnings.

```text
feat: Integrate APO pipeline and register endpoints

- Created `scripts/build_apo.ps1` to robustly build the C++ APO.
- Updated `leyline.inx` to register the APO CLSID for "WaveRender" and "WaveCapture".
- Resolved all C++ warnings in `dllmain.cpp`.
- Verified full package build with zero warnings.
```
