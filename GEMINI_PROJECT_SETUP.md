# Agent Framework Meta-Manual: Continuity & Workflow Automation

This document outlines the project-agnostic framework designed to "reign in" AI agents and ensure 100% architectural continuity across development sessions. This system uses a combination of "Living Documents," structured hand-offs, and automated status workflows to preserve context and standard compliance.

## 1. The Strategy: Automated Alignment
The core of this framework is the **Sync-on-Entry** pattern. Every agent session begins with a mandatory execution of a status workflow that forces the agent to read and acknowledge the project's current state, technical constraints, and previous architectural decisions.

### 1.1 Core Artifacts (The "Memory Bank")
The framework relies on a hierarchical stack of markdown artifacts in the project root:
- **`GEMINI.MD`**: The Technical Specification and "Constitution." Defines the product, architecture, and developer protocols.
- **`PROJECT_PROGRESS.MD`**: session history and chronological work log.
- **`CODE_REVIEW.md`**: Architectural audit (Overwritten every session to prevent bloat).
- **`TEST_REVIEW.md`**: verification and test status.
- **`BUILD_REVIEW.MD`**: Build health and high-level toolchain constraints.
- **`TOOLCHAIN_REVIEW.md`**: [NEW] Granular environment management (PATHs, EnvVars, Binaries). Agents MUST use this to verify and CONFIGURE their build environment.

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
1. **Log Progress**: Append work to the session log.
2. **Audit Architecture**: Overwrite the code review with a fresh perspective.
3. **Verify Health**: Update test and build reviews.
4. **Harden Environment**: Update `TOOLCHAIN_REVIEW.md` with current path status and any newly required binaries. [CRITICAL]
5. **Zero-Warning Enforcement**: Resolve ALL errors and warnings before updating logs or artifacts. No "temporary" warnings allowed.
6. **Warning Literacy**: Agents MUST NOT trust the summary output of build tools. They MUST proactively search build logs using `grep` or `Select-String` for "warning" to ensure 100% cleanliness. [NEW]
7. **Goal-Oriented Development**: Every action MUST progress the project toward the "Product North Star" defined in `GEMINI.MD` Section 0. Aimless development or feature-creep is a protocol violation. [NEW]
8. **Sacrosanct Testing**: Tests represent the 1:1 physical manifestation of the technical spec. Diluting tests to make code pass is strictly prohibited. If code fails a test, refactor the code, not the test. [NEW]
9. **Delicate Spec Evolution**: Update the "Constitution" (`GEMINI.MD`) only for deliberate architectural decisions or structure changes.

## 4. Why This Works
- **Context Preservation**: Prevents agents from "hallucinating" the project state or ignoring established patterns.
- **Project Agnostic**: This structure can be applied to any codebase (Rust, C++, Python, etc.) by simply adjusting the technical specification.
- **Developer Reference**: Provides a clear, chronological history for human developers to audit agent performance and project evolution.

---
*Framework Version: 1.1.0 (Toolchain-Hardened)*
*Last Refined: February 14, 2026*
