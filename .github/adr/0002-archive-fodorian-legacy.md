# ADR 0002 — Archive repository "fodorian-legacy"

- Date: 2026-08-15
- Author: Nicolás Jorquera
- Status: accepted

Context
-------
This repository contains an early experimental Rust + Tauri application developed as a personal learning project over ~6 weeks. The project served to learn Rust, Tauri and Linux, to prototype LLM-enabled agents running in ephemeral containers, and to validate operational strategies such as dynamic routing and fallback handling for API rate limits (HTTP 429). During development I used GCP credits (initial $300 trial and subsequent Gen AI App Builder credits) to run experiments involving Discovery Engine, Dialogflow CX, vector search, and ephemeral container orchestration. I also used the ADK to configure a reasoning engine that enabled session-scoped responses and the creation of ephemeral containers for agent experiments.

Rationale and Motivation
------------------------
The repository depends on native system libraries (glib, cairo, pkg-config, etc.) and requires ongoing maintenance of CI and runner configuration. Continuing active maintenance would consume time and operational resources without producing commensurate value for this prototype. From a FinOps perspective, it is preferable to reallocate remaining credits and effort to experiments that better align with current goals and available resources.

Decision
--------
Archive the repository "fodorian-legacy" and preserve its code and history as a read-only portfolio artifact. This ADR documents the rationale for archiving and the minimal operational steps taken to record the archive state.

Consequences
------------
- The repository will be set to read-only (archived) on GitHub: no new issues, pull requests, or pushes will be allowed while archived.
- The full history and source remain available for cloning or download.
- Resuming active development requires unarchiving and investment in runner configuration or containerization to handle native dependencies.

Operational steps (what will be committed)
-----------------------------------------
1. Add this ADR file at `.github/adr/0002-archive-fodorian-legacy.md`.
2. Add a visible archive banner at the top of `README.md` linking to this ADR.
3. Optionally create a tag `archived-2026-08-15` (if requested).
4. After the ADR and README changes are committed, archive the repository via GitHub Settings (or set `archived: true` via the API).

Record of key technical learnings (short)
----------------------------------------
- Implemented dynamic routing and fallback strategies to mitigate API 429 rate limits while running many simultaneous agents.
- Experimented with ephemeral container-based testing to validate runtime configurations.
- Used GCP credits to explore Discovery Engine, Dialogflow CX, and vector search; used ADK to configure a reasoning engine enabling session-scoped reasoning and control of ephemeral containers.
- Evaluated vector DBs (ChromaDB and LanceDB) and LLM approaches; prioritized ephemeral experimentation over long-running state.

FinOps Summary
--------------
Archiving aligns with a FinOps approach: it reduces ongoing operational costs, protects against unmanaged security debt, and lets you direct remaining credits and effort toward higher-impact experiments while preserving this work as evidence of learning.
