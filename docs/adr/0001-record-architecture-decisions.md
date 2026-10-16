# ADR-0001: Record architecture decisions

- **Status:** Accepted
- **Date:** 2026-10-16

## Context

The project's rule is that every architectural and stack decision is recorded
in the roadmap set, and that a decision existing only in code or conversation is
not decided. Roadmaps track progress well, but they are a poor place to keep the
reasoning behind a choice that was contested or that rules alternatives out.

## Decision

Record each architecturally significant decision as a numbered Architecture
Decision Record in `docs/adr/`.

- One decision per file, named `NNNN-short-title.md`.
- Use `template.md`; keep the `Context`, `Decision`, and `Consequences` sections.
- An accepted ADR is immutable. Do not edit its decision; supersede it with a
  new ADR and set the old one's status to `Superseded by ADR-XXXX`.
- A change that alters an architecture rule, a stack choice, or a contract
  policy requires an ADR before the code lands.

## Consequences

The roadmap set stays about what to build and in what order; ADRs hold why.
Reviewers can see the alternatives that were rejected. The cost is one more
document to write for significant changes, which is deliberate.
