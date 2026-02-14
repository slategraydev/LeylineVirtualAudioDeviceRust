---
description: Synchronize with the latest project status, standards, and TODOs.
---

This workflow ensures the agent is fully aligned with the Leyline Audio Driver's technical specification, current progress, and architectural status.

1.  **Sync with Technical Specification**:
    *   Read `c:\LeylineAudioDriver\GEMINI.MD` to internalize project-wide standards, DCH requirements, and coding guidelines.

2.  **Sync with Development Progress**:
    *   Read `c:\LeylineAudioDriver\PROJECT_PROGRESS.MD` to see the session history, completed work, and the current `PROJECT TODO` list.

3.  **Sync with Architectural Health**:
    *   Read `c:\LeylineAudioDriver\CODE_REVIEW.md` to understand the current audit status and any architectural pitfalls identified by the previous agent.

4.  **Sync with Testing Status**:
    *   Read `c:\LeylineAudioDriver\TEST_REVIEW.md` to review the current verification state and any identified testing gaps.

5.  **Sync with Build Health**:
    *   Read `c:\LeylineAudioDriver\BUILD_REVIEW.MD` to verify the stability of the multi-crate toolchain and environment constraints.

6.  **Sync with Toolchain Status**: [CRITICAL]
    *   Read `c:\LeylineAudioDriver\TOOLCHAIN_REVIEW.md` to verify project toolchains and ensure PATHs/EnvVars are configured.
    *   **Action**: Proactively set `$env:LIBCLANG_PATH` and update `$env:PATH` based on the review.

7.  **Sync with Project Setup Guide**:
    *   Read `c:\LeylineAudioDriver\GEMINI_PROJECT_SETUP.md`.
    *   **Action**: This is the LAST document to review. Evaluate if developer protocols or project structures require updates based on recent changes.

8.  **Reference Audit & Deep Dive**:
    *   Scan `GEMINI.MD` for external links or Microsoft WDK references relevant to the current task.
    *   Perform a proactive deep-dive into these references to ensure the implementation plan is technically sound.

98.  **Summary Output**:
    *   Synthesize the information from these SIX files.
    *   **Goal Alignment**: Explicitly state how the current task progresses the "Product North Star" (Two-endpoint virtual driver). [NEW]
    *   State the current session number (Previous Session + 1).
    *   Summarize the topmost items in the `PROJECT TODO` list.
    *   Acknowledge any high-priority architectural suggestions from `CODE_REVIEW.md`.

9.  **Hand-off & Maintenance Mandate**:
    *   The agent is now bound by the protocols in `GEMINI.MD` Section 4.5, 4.6, and 4.7.
    *   **Goal Persistence**: MUST verify that the session's work directly serves the "Product North Star."
    *   **Sacrosanct Testing**: MUST NOT modify tests to allow code to pass. Code must meet the spec defined by the tests. [NEW]
    *   **CODE_REVIEW.md**: MUST be overwritten with a fresh audit at the end of the session.
    *   **PROJECT_PROGRESS.MD**: MUST be appended with a new Session Log.
    *   **TEST_REVIEW.md & BUILD_REVIEW.MD**: MUST be updated with verification results.
    *   **TOOLCHAIN_REVIEW.md**: MUST be updated with current environment status and any new tool requirements. [NEW]
    *   **Zero-Warning Proof**: MUST resolve ALL errors and warnings. PROOF is required by running `Select-String -Pattern "warning"` on the most recent build logs and confirming no matches. [CRITICAL]
    *   **GEMINI.MD**: MUST be updated for deliberate architectural shifts or structure changes. **NO MASS REFACTORS.**
    *   **GEMINI_PROJECT_SETUP.md**: Update ONLY if the agent framework's meta-manual needs refinement for future reference.

Use this workflow at the start of every session to maintain 100% project continuity.
