feat: register topology filter and expand data ranges

Registered the Topology subdevice and implemented multi-format data negotiation to support professional audio streaming.

- Implemented `IMiniportTopology` COM wrapper and registration logic.
- Defined `PCFILTER_DESCRIPTOR` for the Wave filter with PCM and IEEE Float support.
- Implemented `DataRangeIntersection` for dynamic format negotiation.
- Created `scripts/Update-Version.ps1` for automated INF stamping.
- Verified zero-warning build across the entire kernel crate.

**Zero-Warning Proof**: Verified via `Select-String -Pattern "warning"` on build logs (No matches).

**Git Commit Payload**:
```
feat: register topology filter and expand data ranges

- Implement IMiniportTopology COM wrapper and registration.
- Define static Wave filter descriptors for PCM/Float.
- Implement DataRangeIntersection negotiation logic.
- Automate INF version stamping in build pipeline.
```
