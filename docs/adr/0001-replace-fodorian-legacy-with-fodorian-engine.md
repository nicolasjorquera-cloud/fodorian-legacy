# ADR-0001: Replace fodorian-legacy with fodorian engine

- **Status:** Accepted
- **Date:** 2026-08-05

## Context

`fodorian-legacy` is a terminal AI copilot (Tauri + React + Rust) whose backend depends on **Google Vertex AI (Reasoning Engine)**. Continuing on that stack is no longer viable: running out of Vertex AI credits exposed that the original architecture did not account for **fin-ops** — the cost and allocation of cloud AI credits were treated as an afterthought rather than as a strategic resource. Credits for **Google Discovery Engine** remain available and are better suited to the product's needs.

## Decision

Create **`fodorian engine`**, a rewrite of the same concept (an AI copilot for Linux) built on top of the learnings from `fodorian-legacy`, that:

- Migrates from Vertex AI Reasoning Engine to the **Discovery Engine** API, refocusing the product toward a more specific use case.
- Treats **credit usage as a first-class concern (fin-ops strategy)**: the architecture deliberately routes work toward the provider where credits remain and minimizes cost per operation.
- Includes a **deterministic indexing pipeline** for a data store: scraping → conversion to Markdown → **LLM-based classification** of information prior to indexing.
- Starts from scratch; `fodorian-legacy` is kept only as history and its code is not ported.

## Consequences

**Positive:**
- Clean start with better structure.
- Leverages the available Discovery Engine credits.
- Builds on the technical and architectural lessons of the first project.
- Cost-conscious design (fin-ops) from day one, rather than retrofitting it later.
- Clear separation between the indexing pipeline and the Linux tool.

**Negative:**
- Cost of a full rewrite.
- The indexing pipeline is required work before the Linux tool can be built.

## Alternatives considered

1. **Continue on `fodorian-legacy`** — rejected: architecture needs reworking and depends on an exhausted credit source.
2. **Port the existing code** — rejected: would carry over the legacy architecture and its lack of fin-ops consideration.
3. **Stay on Vertex AI** — rejected: no credits available on that stack.
