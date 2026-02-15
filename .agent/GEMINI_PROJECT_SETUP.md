# Agent Framework Meta-Manual: Continuity & Workflow Automation

This document outlines the project-agnostic framework designed to "reign in" AI agents and ensure 100% architectural continuity across development sessions. This system uses a combination of "Living Documents," structured hand-offs, and automated status workflows to preserve context and standard compliance.

## 1. The Strategy: Automated Alignment
The core of this framework is the **Sync-on-Entry** pattern. Every agent session begins with a mandatory execution of a status workflow that forces the agent to read and acknowledge the project's current state, technical constraints, and previous architectural decisions.

### 1.0 Environment Initialization [CRITICAL]
Before any build, test, or audit action, agents MUST execute:
`.\scripts\LaunchBuildEnv.ps1`
This ensures the eWDK toolchain is correctly sourced and `LIBCLANG_PATH` is set.

### 1.1 Core Artifacts (The "Memory Bank")
The framework relies on a hierarchical stack of markdown artifacts in the project root:
- **`GEMINI.MD`**: The Technical Specification and "Constitution." Defines the product, architecture, and developer protocols. (Located in `.agent/`)
- **`PROJECT_PROGRESS.MD`**: session history and chronological work log. (Located in `.agent/`)
- **`CODE_REVIEW.md`**: Architectural audit. (Located in `.agent/`)
- **`TEST_REVIEW.md`**: verification and test status. (Located in `.agent/`)
- **`BUILD_REVIEW.MD`**: Build health. (Located in `.agent/`)
- **`SCRIPTS_REVIEW.md`**: Automation script audit. (Located in `.agent/`)
- **`TOOLCHAIN_REVIEW.md`**: Granular environment management. (Located in `.agent/`)
- **`COMMIT_MESSAGE.MD`**: Session commit message template. (Located in `.agent/`)

## 2. Implementation: The Status Workflow
The primary automation tool is the `/status` workflow, located at `.agent/workflows/status.md`. This file serves as the agent's "Standard Operating Procedure" (SOP) for session alignment.

### 2.1 Workflow File Structure
```markdown
---
description: Synchronize with the latest project status, standards, and TODOs.
---
1. Read [Core Docs in Hierarchical Order]
2. Perform Reference Audit (Verify Assumptions)
3. Synthesize Summary (Current Session #, Top TODOs)
4. Acknowledge Hand-off Mandates
```

## 3. The Hand-off Mandate
To maintain the integrity of this framework, agents are bound by a strict maintenance protocol at the end of every session:
1. **Log Progress**: Append work to the session log in `.agent/PROJECT_PROGRESS.MD`.
2. **Draft Commit Message**: Update `.agent/COMMIT_MESSAGE.MD` using the **Impact-First Template**:
    - **Header**: Conventional Commits (e.g., `feat:`, `fix:`) + punchy summary.
    - **Summary**: One sentence on the "Why" (the primary problem solved).
    - **Impact Bullets**: 3-5 short, direct bullets starting with an action verb (e.g., "Forced...", "Implemented...", "Fixed...").
    - **NO CATEGORIES**: Avoid bolded category headers (e.g., **Binary Integrity:**). Focus on the action.
3. **Audit Architecture**: Update `.agent/CODE_REVIEW.md` with a fresh perspective.
4. **Verify Tests**: Update `.agent/TEST_REVIEW.md` with current coverage and results.
5. **Verify Build**: Update `.agent/BUILD_REVIEW.MD` with current build status.
6. **Harden Environment**: Update `.agent/TOOLCHAIN_REVIEW.md` with current path status and any new tool requirements. [CRITICAL]
7. **Session Cleanup**: Delete all ephemeral verification logs (e.g., `*.txt`, `*.log`) from the project root **and all subdirectories** (e.g., `crates/leyline-kernel`).
8. **Build Sanitation**: Execute a "clean" command on all build targets to ensure no cross-pollination between sessions. [CRITICAL]
9. **Zero-Warning Enforcement**: Resolve ALL errors and warnings before updating logs or artifacts. No "temporary" warnings allowed.
10. **Warning Literacy**: Agents MUST NOT trust the summary output of build tools. They MUST proactively search build logs using `grep` or `Select-String` for "warning" to ensure 100% cleanliness. [NEW]
11. **Goal-Oriented Development**: Every action MUST progress the project toward the "Product North Star" defined in `GEMINI.MD` Section 0. Aimless development or feature-creep is a protocol violation. [NEW]
12. **Sacrosanct Testing**: Tests represent the 1:1 physical manifestation of the technical spec. Diluting tests to make code pass is strictly prohibited. If code fails a test, refactor the code, not the test. [NEW]
13. **Delicate Spec Evolution**: Update the "Constitution" (`GEMINI.MD`) only for deliberate architectural decisions or structure changes.
14. **No Auto-Commit**: Agents MUST NOT execute `git commit`. The session ends with changes staged or in the working directory for user review.

### 3.1 Task List Protocol [CRITICAL]
The session `task.md` is the operational checklist for the agent. To ensure no mandatory file is skipped, agents MUST list each review file as a separate checklist item in the "Session Verification" section.
- **PROHIBITED**: `[ ] Update Session Artifacts` (Too generic)
- **REQUIRED**:
    - `[ ] Update PROJECT_PROGRESS.MD`
    - `[ ] Update CODE_REVIEW.md`
    - `[ ] Update TEST_REVIEW.md`
    - `[ ] Update BUILD_REVIEW.MD`
    - `[ ] Update SCRIPTS_REVIEW.md`
    - `[ ] Update TOOLCHAIN_REVIEW.md`
    - `[ ] Update COMMIT_MESSAGE.MD`

## 4. Why This Works
- **Context Preservation**: Prevents agents from "hallucinating" the project state or ignoring established patterns.
- **Project Agnostic**: This structure can be applied to any codebase (Rust, C++, Python, etc.) by simply adjusting the technical specification.
- **Developer Reference**: Provides a clear, chronological history for human developers to audit agent performance and project evolution.

---
*Framework Version: 1.1.0 (Toolchain-Hardened)*
*Last Refined: February 14, 2026*
