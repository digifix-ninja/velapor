# Contributing to Velapor

Thank you for wanting to help keep watch. Velapor is at the very beginning, so ideas, questions and
discussions in the [issues](../../issues) are the most valuable contribution right now. Code
contributions will open up once the foundation is in place.

## Reporting a bug or suggesting a feature

Open an issue that describes the problem you would like solved, not only the solution. Include your
operating system and Velapor's version. Leave out anything personal you would rather not make public:
hostnames, user names, IP addresses.

## Pull requests

1. Fork the repository and create a branch from `main`.
2. Follow the code style: small objects with one job each, a short comment on every function, and
   names that read like plain English. See [ARCHITECTURE.md](ARCHITECTURE.md) for how the project is
   put together.
3. Run the checks:
   ```sh
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
4. Describe what changes for the person using Velapor, before and after.

## Watchman's oath

Velapor observes and recommends. A change must never let it modify the system, send data off the
machine, or widen what runs with elevated rights, without that being discussed in an issue first and
clearly visible to the user. Every finding must show the evidence it is based on.
