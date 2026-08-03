# Neutral Agent Pay

**Enterprise x402 payment rail and fiat settlement on Stellar.**

Built by [Vottun](https://vottun.com) with [Wirex](https://wirexapp.com) as lead partner and
[DFNS](https://dfns.co) as wallet infrastructure partner.

---

## What this is

AI agents can call APIs but cannot pay for them. The [x402 protocol](https://www.x402.org)
solves the protocol half of that problem by turning HTTP `402 Payment Required` into a
usable per-request payment challenge. On Stellar, x402 works through Soroban authorization:
a client pays by signing an authorization entry rather than a full transaction.

What does not exist yet is the operational layer a business needs around it — wallets that
can be provisioned and signed from programmatically, spending controls that bound what an
autonomous agent can do, reconciliation a finance team can audit, and a path from settled
stablecoin to spendable fiat.

Neutral Agent Pay is that layer.

```ts
import { NeutralAgentPayClient, DfnsSigner } from "@neutralagentpay/sdk";

const client = new NeutralAgentPayClient({
  network: "stellar:pubnet",
  rpcUrl: process.env.STELLAR_RPC_URL!,
  signer: new DfnsSigner({ /* ... */ }),
  maxAmountPerRequest: 1_000_000n,
});

// Pays automatically if the endpoint answers 402.
const res = await client.fetch("https://api.example.com/forecast");
```

On the merchant side, a route becomes paid by declaring a price:

```ts
import { paymentMiddleware } from "@neutralagentpay/middleware";

app.use(paymentMiddleware({
  network: "stellar:pubnet",
  payTo: MERCHANT_ADDRESS,
  defaultAsset: USDC_CONTRACT,
  facilitator: { url: FACILITATOR_URL, apiKey: FACILITATOR_API_KEY },
  routes: { "GET /forecast": { price: "0.001" } },
}));
```

## The gap this fills

Stellar's documentation lists the wallets that support the auth-entry signing x402 requires:
Freighter, Albedo, Hana, HOT, Klever, OneKey. Every one of them is a browser extension or a
mobile wallet — they all require a human to approve a prompt.

An autonomous agent making per-request micropayments cannot depend on a human clicking. The
DFNS-backed signer in this repository is the piece that closes that gap: programmatic
account provisioning, non-interactive signing, policy enforcement before the signature, and
MPC key custody where neither Vottun nor the merchant holds the customer's key.

## Architecture

Full technical architecture: [`docs/architecture.pdf`](./docs/architecture.pdf)
([markdown source](./docs/architecture.md)).

```
Agent ──► SDK ──► Merchant API + x402 middleware ──► Facilitator ──► Stellar
           │                                                            │
           └──► DFNS (MPC auth-entry signing)      Soroban controls ────┤
                                                                        │
                        Indexer ──► Dashboard ──► Wirex ──► fiat balance ┘
```

| Layer | What it does |
|---|---|
| **Client SDK** | Detects the 402 challenge, builds the SEP-41 `transfer` invocation, gets the auth entry signed, retries |
| **Middleware** | Declares paid routes and prices, issues challenges, verifies and settles |
| **Facilitator client** | x402 v2 `/verify`, `/settle`, `/supported` — provider-agnostic |
| **DFNS signer** | Programmatic Stellar accounts, hash signing of auth entries, policy-gated |
| **Soroban contracts** | Per-agent spending caps, allowlists, velocity limits, settlement registry |
| **Indexer & dashboard** | On-chain truth streamed into reconciliation and merchant operations |
| **Wirex adapter** | Settled stablecoin mapped to a fiat balance through regulated rails |

## Repository layout

```
contracts/
  payment-controls/       Soroban: spending caps, allowlists, velocity limits, pause
  settlement-registry/    Soroban: settled-payment records and indexer events
packages/
  sdk/                    Client SDK, including the DFNS signer
  middleware/             x402 payment middleware for merchant servers
  facilitator-client/     x402 v2 facilitator API client
examples/
  paid-api/               Minimal paid endpoint, end to end
docs/
  architecture.md/.pdf    Full technical architecture
  payer-model-spike.md    Open technical questions and how they get answered
  deployments.md          Contract addresses, published as each network goes live
```

## Status

This project is submitted to the Stellar Community Fund Build Award. Interfaces, contract
logic, storage design and tests are in place; the implementation is delivered across three
tranches.

| Tranche | Scope | Status |
|---|---|---|
| **T1 — MVP** | SDK, middleware, facilitator client, DFNS wallet integration, payer-model spike | Not started |
| **T2 — Testnet** | Soroban controls, settlement registry, indexer, dashboard, Wirex sandbox | Not started |
| **T3 — Mainnet** | Mainnet launch, neutral gateway, Wirex production, user testing | Not started |

Unimplemented functions throw `NotImplemented` with the deliverable that covers them, rather
than returning plausible-looking values. Nothing in this repository pretends to work.

## Open technical questions

Documented rather than hidden, with a validation spike in the first two weeks and an
architecture designed so that either answer still satisfies the deliverables. See
[`docs/payer-model-spike.md`](./docs/payer-model-spike.md).

1. Can a Soroban custom account act as the x402 payer through the facilitator?
2. Can the DFNS policy engine discriminate on hash-signing requests?
3. Is EURC among the facilitator's accepted asset contracts, or is a self-hosted instance needed?

## Build

```bash
# TypeScript packages
npm install
npm run build

# Soroban contracts
cd contracts
rustup target add wasm32-unknown-unknown
cargo test
cargo build --release --target wasm32-unknown-unknown
```

Copy `.env.example` to `.env` and fill in the facilitator and DFNS credentials.

## Prior work

Neutral Agent Pay builds on the **Wirex–Vottun–Stellar SDK**, delivered by Vottun under a
previous SCF award
([project page](https://communityfund.stellar.org/project/wirex-vottun-stellar-sdk-a-scalable-development-framework-icl)).
The wallet management, transaction lifecycle, configuration and streaming components of that
SDK are reused directly; the x402 layer, Soroban payment controls, gateway, and DFNS and
Wirex integrations are new work built on that foundation.

## License

MIT — see [`LICENSE`](./LICENSE). The Soroban contracts are open-sourced with verified
mainnet deployments, documentation and usage examples, reusable by any team building agentic
payment flows on Stellar.
