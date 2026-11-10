# Contributing to Velapor

Thank you for wanting to help. Bug reports, ideas and pull requests are all welcome.

## Before you start

For anything bigger than a small fix, please open an issue first, so we can agree on the idea
before you spend time on the code.

## The one rule

**Velapor only reads.** It may look at anything the system will tell it, but it never writes,
changes, stops or deletes anything: no settings, no files, no processes. A finding can *advise*
the person to do something; Velapor itself never does it.

`core/tests/read_only.rs` scans the core for write calls and fails the build if it finds one.
If a fact can only come from one of the system's own tools, add that tool to the fixed list in
`core/src/system_query.rs`, with fixed arguments, and only if the tool itself only reports.

## Adding a rule

1. Pick the area's file in `core/src/rules/` (hardware, software, network, performance, integrity).
2. Add a small struct that implements `Rule`: an `id`, a `domain`, a `name` and `check`.
   `check` looks at the `Context` (the latest readings, the history and the baseline) and returns
   `Outcome::Clear`, `Outcome::Found(findings)` or `Outcome::NotApplicable`.
3. Every finding explains itself in plain words: what is happening, the likely cause, a
   recommendation, and the evidence (the readings it is based on).
4. Add it to the `rules()` list at the top of that file (the `RuleBook` collects them all) and
   add a test.

## Code style

- Small objects with one job each, and a short comment on every function.
- Plain words in everything a person reads: no jargon where a simple word works.
- Before a pull request:

```sh
cargo fmt --all
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

## Licence

By contributing, you agree that your contribution is released under the [MIT licence](LICENSE).
