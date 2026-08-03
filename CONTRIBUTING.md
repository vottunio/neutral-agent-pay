# Contributing

Issues and pull requests are welcome, particularly from teams building agentic payment flows
on Stellar.

- **Contracts** (`contracts/`): Rust, Soroban SDK. Run `cargo test` and `cargo fmt` before
  opening a PR. New control logic needs a test covering both the allow and the deny path.
- **Packages** (`packages/`): TypeScript, strict mode, ES modules. Run `npm run build`.
- **Documentation**: architecture changes go in `docs/architecture.md`; regenerate
  `docs/architecture.pdf` in the same PR.

Anything that changes an on-chain interface or the settlement path should explain the effect
on reconciliation in the PR description.
