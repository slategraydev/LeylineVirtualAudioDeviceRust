# Leyline Audio Driver - Comment Audit & Update

## Tasks
- [x] Audit `crates/leyline-kernel/src/` for comment standards [11/11]
    - [x] `adapter.rs`
    - [x] `audio_bindings.rs`
    - [x] `audio_wrapper.h`
    - [x] `constants.rs`
    - [x] `descriptors.rs`
    - [x] `dispatch.rs`
    - [x] `lib.rs`
    - [x] `stream.rs`
    - [x] `topology.rs`
    - [x] `vtables.rs`
    - [x] `wavert.rs`
- [x] Audit `crates/leyline-shared/src/` [3/3]
    - [x] `buffer.rs`
    - [x] `lib.rs`
    - [x] `math.rs`
- [x] Audit `src/APO/` [6/6]
    - [x] `dllmain.cpp`
    - [x] `framework.h`
    - [x] `LeylineAPO.cpp`
    - [x] `LeylineAPO.h`
    - [x] `LeylineAPO.idl`
- [x] Audit `src/HSA/` [5/5]
    - [x] `App.xaml.cs`
    - [x] `DriverBridge.cs`
    - [x] `MainWindow.xaml.cs`
- [x] Reformat copyright headers to two-line style [1/1]
    - [x] Update all project source files
- [x] Refactor `descriptors.rs` [4/4]
    - [x] Create `descriptors/common.rs`
    - [x] Create `descriptors/render.rs`
    - [x] Create `descriptors/capture.rs`
    - [x] Create `descriptors/mod.rs` and update `lib.rs`
- [ ] Update `README.md` with detailed project narrative

## Review
- All file headers include Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.
- Section headers use tilde separators.
- Docstrings follow the "Perform X" imperative style.
- In-line comments explain the "Why" only.
