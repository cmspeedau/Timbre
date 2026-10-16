# Contributing to Timbre

The roadmap set is the source of truth. Read the [Master Roadmap](Master%20Roadmap.md)
before changing anything, and record every architectural or stack decision in a
roadmap or an ADR before it appears in code.

## Development environment

- **Toolchain:** pinned in `rust-toolchain.toml`. With `rustup`, `cargo` uses it
  automatically; you do not need to `rustup override` anything.
- **Editor:** `.editorconfig` and `rustfmt.toml` define formatting. Format on
  save, or run `cargo fmt` before committing.
- **No non-Rust tooling is required to build, test, or run the project.** This is
  a hard rule (Master Roadmap, non-goals). Do not add a build step that needs
  Node, Python, a JVM, or a web engine.

## Commands

```sh
cargo fmt --all                     # format
cargo clippy --all-targets --all-features -- -D warnings   # lint (warnings are errors)
cargo test --workspace --all-features                      # test
cargo build --workspace                                    # build
```

CI runs exactly these. A change is not ready until all four pass locally.

## Coding conventions

- **Edition and MSRV** come from the workspace `Cargo.toml`; do not raise either
  without an ADR.
- **Lints** live in `[workspace.lints]`. Every crate opts in with
  `[lints] workspace = true`; do not disable a lint crate-locally without a
  comment explaining why.
- **Unsafe** is denied workspace-wide. It may only be allowed in a platform
  integration crate, with a `// SAFETY:` comment on every block.
- **Public items are documented.** `missing_docs` is an error under `-D warnings`.
- **Dependencies** are declared once in `[workspace.dependencies]` and referenced
  as `dep = { workspace = true }`. Respect the dependency rules in
  [ADR-0003](docs/adr/0003-workspace-layout-and-dependency-rules.md).
- **Serde contracts** live in `timbre-api-contracts` or `timbre-shared-types`,
  never duplicated in a subsystem crate.

## Branches and commits

- Branch names: `feat/…`, `fix/…`, `docs/…`, `refactor/…`, `test/…`, `chore/…`.
- Commit messages follow Conventional Commits:
  `type(scope): imperative summary`, then a body explaining why.
- One logical change per commit. Keep the tree green at every commit.

## Decisions

Architectural decisions are recorded as ADRs in `docs/adr/`, one file per
decision, using `docs/adr/template.md`. An accepted ADR is not edited to change
its decision; supersede it with a new one.

## Progress

Phases, tasks, and exit gates are tracked as checkboxes in the roadmaps. Tick an
item in the same change that lands it. Do not tick a milestone because code was
merged; tick it when its exit gate is demonstrated.
