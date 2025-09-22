# Contributing to QuantumDesk

Thank you for helping build the world's most intelligent open-source financial terminal. This guide explains how to get set up, propose changes, and keep the repo healthy as it scales.

## Ground Rules

- Be respectful and inclusive. Treat other contributors as you would teammates on a trading desk.
- Favor reproducibility. Every feature should be testable with `cargo test` or an equivalent script.
- Prefer incremental, reviewable pull requests over massive drops.

## Getting Started

1. Fork the repository and clone your fork.
2. Install the Rust toolchain with [`rustup`](https://rustup.rs/) (Rust 1.75+).
3. Run the basics to ensure everything compiles:
   ```bash
   cargo fmt
   cargo clippy --all-targets
   cargo test
   ```
4. Run `cargo run` and press `q` to exit the TUI to confirm the terminal UI launches correctly.

## Branching & Pull Requests

- Create a feature branch off `main` (`feature/exchange-bitfinex`, `fix/ui-resize`, etc.).
- Keep commits focused; squash or rebase before opening a PR if it clarifies history.
- Include screenshots or terminal recordings (e.g., `asciinema`) for UI-facing changes.
- Reference related issues in your PR description (e.g., `Closes #42`).
- Fill out the PR template (coming soon) so reviewers understand the change scope and testing.

## Coding Guidelines

- Follow `rustfmt` defaults. CI will check formatting.
- Use idiomatic Rust error handling (`anyhow`, `thiserror`) rather than panicking.
- Add tests when touching business logic: data parsers, metrics calculations, alert triggers.
- Keep modules focused (`data`, `metrics`, `alerts`, `ui`, `ai`, `config`). Introduce new modules only when the domain grows.
- Prefer `tracing` for observability once logging lands; avoid ad-hoc `println!` calls.

## Documentation

- Update relevant docs when behavior changes (`README.md`, `ROADMAP.md`, docs/ pages).
- For new integrations, add a short playbook under `docs/integrations/` (create if missing).
- Explain configuration changes and required secrets in `docs/configuration.md` (planned).

## Issues & Discussions

- Use the **Roadmap** milestones when filing issues so work maps to a release.
- Mark first-timer-friendly tickets with `good first issue` and include context/resources.
- Use GitHub Discussions for strategic topics: architecture, AI agent ideas, plugin protocols.

## Release Process (Draft)

1. Create a release branch (e.g., `release/v0.1.0`).
2. Update changelog (`docs/CHANGELOG.md`, planned) and version in `Cargo.toml`.
3. Tag the release and attach binaries/build instructions.
4. Publish crates/binaries once automated packaging is in place.

## Community

- Join our community Discord for contributor syncs and market desk show-and-tell.
- Participate in periodic hackathons focused on new exchanges, MCP agents, and macro data feeds.
- Share success stories, dashboards, and scripts with the community—we will highlight standout contributions in release notes.

Welcome aboard! 🎛️
