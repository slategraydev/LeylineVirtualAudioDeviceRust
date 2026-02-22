# COMMIT_MESSAGE.MD

## Plain Text Commit Message [COPY-PASTE]
feat: unblock endpoint enumeration with modern discovery interfaces

This change resolves the "silent stall" in the Audio Endpoint Builder by implementing mandatory discovery protocols.

- Implemented IPinName interface in the Wave miniport to provide subdevice identity.
- Enhanced proposed_format_handler to support extensible format negotiation (48kHz Stereo).
- Added redundant Jack Description properties to the Wave filter for discovery robustness.
- Standardized GetPinName vtable signatures and fixed COM type mismatches.
- Verified build cleanliness with 0 warnings.

## Git Commit Payload
feat: unblock endpoint enumeration with modern discovery interfaces

- Forced IPinName acceptance in wavert.rs QueryInterface.
- Implemented standard 48kHz Stereo KSDATAFORMAT_WAVEFORMATEXTENSIBLE return in descriptors.rs.
- Promoted JACK_DESCRIPTION to WAVE_FILTER_PROPERTIES for redundant path support.
- Fixed duplication and type mismatches in miniport implementation.
- Zero-warning verification complete.
