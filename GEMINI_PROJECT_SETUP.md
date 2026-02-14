# Gemini Project Setup Guide: Leyline Audio Driver

This document serves as the "Master Manual" for setting up the specialized development environment and continuity systems that allow Gemini agents to succeed on the Leyline project. It outlines all the living documents, workflows, and protocols established to preserve architectural soundness and session continuity.

## 1. Core Living Documents
These files are the "Memory" and "Constitution" of the project. Every session MUST begin with an audit of these.

- **`GEMINI.MD`**: The Technical Specification. Contains the high-level architecture, DCH requirements, and the definitive bibliography of WDK references.
- **`PROJECT_PROGRESS.MD`**: The session history. Every agent logs their work here, including a session number (e.g., SESSION #01), architectural suggestions, and updated TODO lists.
- **`CODE_REVIEW.md`**: The current health audit. This is overwritten every session with a fresh review of the code's soundness and professional standards.

## 2. Automation & Workflows
To maintain 100% alignment across sessions, the following automation has been configured:

- **`/status` Workflow**: Located at `.agent/workflows/status.md`.
    - **Purpose**: Automates the reading of all continuity files.
    - **Usage**: Agents should be instructed to run this command at the start of any new chat thread.

## 3. Critical Protocols
These mandates are codified in `GEMINI.MD` but are summarized here for setup purposes:

- **Reference Audit Mandate**: Agents must visit and audit external documentation for any technical decision.
- **Spec Evolution Protocol**: `GEMINI.MD` is never aggressively rewritten; changes are proposed in code reviews and implementation plans before being delicately applied.
- **Project Structure Sync**: Section 3 of `GEMINI.MD` (Project Structure) MUST be updated whenever the physical file or directory layout changes.

## 4. Setup Steps (For Future Infrastructure Recreation)
If this project were to be moved or recreated, the following steps were taken by the initial agent (Session #01):
1. Created the 4-crate Rust workspace (`leyline-kernel`, `leyline-shared`, `leyline-apo`, `leyline-hsa`).
2. Established `GEMINI.MD` with specialized "Agent Context" notes in the bibliography.
3. Created the `/status` workflow to force-sync agent state.
4. Formalized the `PROJECT_PROGRESS.MD` and `CODE_REVIEW.md` trackers in the project root.
5. Implemented professional ASCII/Block headers to ensure code readability for kernel-mode inspection.

## 5. Post-Completion Log (Future)
*This section will be populated upon final project delivery to outline the chronological evolution of the setup from boilerplate to production.*

---
*Last Updated: February 14, 2026*
