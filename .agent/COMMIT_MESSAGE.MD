feat: add comprehensive topology initialization diagnostics for 0xC00002B9 error

- Added extensive DbgPrint instrumentation to topology miniport:
  - topology_query_interface: Logs all interface queries (IID_IMiniportTopology, IID_IUnknown, IID_IMiniport, IID_IPortTopology, IID_IPort)
  - topology_get_description: Added parameter validation and logging
  - topology_init: Added entry/exit logging with status tracking
- Enhanced adapter.rs diagnostics:
  - Detailed logging around PcNewPort(CLSID_PortTopology)
  - Specific detection and logging of 0xC00002B9 (STATUS_REQUEST_NOT_ACCEPTED)
  - Enhanced error messages with potential causes
  - Step-by-step topology initialization tracking
- Added IID_IPort GUID to constants.rs for complete interface diagnostics
- Fixed NTSTATUS literal type issues (0xC00002B9u32 as i32) for proper comparison
- Fixed unused variable warnings in topology.rs

## Plain Text Commit Message [COPY-PASTE]
feat: add comprehensive topology initialization diagnostics for 0xC00002B9 error

- Added extensive DbgPrint instrumentation to topology miniport (QueryInterface, GetDescription, Init)
- Enhanced adapter.rs with detailed topology initialization logging
- Added IID_IPort GUID for complete interface diagnostic support
- Fixed NTSTATUS type issues for proper error code comparison
- Build: 0 warnings, 0 errors - ready for VM diagnostic testing