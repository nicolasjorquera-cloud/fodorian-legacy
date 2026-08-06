# ADR-0001: Replace fodorian-legacy with fodorian-os

- **Status:** Accepted
- **Date:** 2026-08-05

## Context

`fodorian-legacy` is a terminal AI copilot (Tauri + React + Rust) whose backend depends on **Google Vertex AI (Reasoning Engine)**. Active Google Cloud credits for Vertex AI are exhausted, while credits for **Google Discovery Engine** remain available. The project needs to be reoriented technically to align with the infrastructure currently available.

## Decision

Create **`fodorian-os`**, a rewrite of the same concept (an AI copilot for Linux) that:

- Migrates from Vertex AI Reasoning Engine to the **Discovery Engine** API, refocusing the product toward a more specific use case.
- Includes a **deterministic indexing pipeline** for a data store: scraping → conversion to Markdown → **LLM-based classification** of information prior to indexing.
- Starts from scratch; `fodorian-legacy` is kept only as history and its code is not ported.

## Consequences

**Positive:**
- Clean start with better structure.
- Leverages the available Discovery Engine credits.
- Clear separation between the indexing pipeline and the Linux tool.

**Negative:**
- Cost of a full rewrite.
- The indexing pipeline is required work before the Linux tool can be built.

## Alternatives considered

1. **Continue on `fodorian-legacy`** — rejected: Vertex AI credits exhausted, architecture needs reworking.
2. **Port the existing code** — rejected: would carry over the legacy architecture.
3. **Stay on Vertex AI** — rejected: no credits available.
