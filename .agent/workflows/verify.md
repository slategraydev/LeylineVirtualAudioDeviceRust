---
description: Run the automated end-to-end VM verification pipeline.
---

This workflow executes the full verification cycle: reverting the VM, building the driver, installing it, and validating kernel initialization.

1.  **Execute Automation Script**
    *   Runs `scripts/Automate-VM-Verification.ps1`.
    *   **Parameters**: Defaults to `LeylineTestVM`.
    
// turbo
2.  Run the verification command:
    ```powershell
    .\scripts\Automate-VM-Verification.ps1
    ```
