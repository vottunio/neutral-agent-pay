# Soroban contracts

| Contract | Purpose |
|---|---|
| `payment-controls` | Per-agent spending caps, destination and asset allowlists, velocity limits, emergency pause |
| `settlement-registry` | Records settled payments and emits the events consumed by the indexer |

## Status

Interfaces, storage layout, control logic and unit tests are defined. Hardening, gas
profiling, the custom-account (`__check_auth`) variant and mainnet deployment land in
Tranches 2 and 3 of the SCF Build Award, informed by the payer-model spike documented in
[`../docs/payer-model-spike.md`](../docs/payer-model-spike.md).

## Design notes

**Enforcement point.** `authorize_payment` runs during transaction simulation, which the
x402 facilitator performs before submitting anything on-chain. A policy violation is
therefore a free, deterministic rejection rather than a failed transaction.

**Window accounting.** Rolling-window counters live in temporary storage with a TTL matched
to the policy window, so expiry is self-cleaning and rent stays bounded. Policies live in
persistent storage.

**Empty allowlist semantics.** An empty `allowed_destinations` or `allowed_assets` vector
means "unrestricted". This makes the common case cheap and avoids forcing every merchant to
enumerate destinations on day one.

**Idempotency.** `settlement_registry.record` rejects a duplicate `payment_id` rather than
overwriting, so a retry storm cannot corrupt the reconciliation record.

## Build and test

```bash
rustup target add wasm32-unknown-unknown
cargo test
cargo build --release --target wasm32-unknown-unknown
stellar contract optimize --wasm target/wasm32-unknown-unknown/release/payment_controls.wasm
```

Deployed contract addresses are published in [`../docs/deployments.md`](../docs/deployments.md)
as each network goes live.
