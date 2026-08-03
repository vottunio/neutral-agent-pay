# Payer-model validation spike

**Scheduled:** weeks 1–2 of Tranche 1 · **Deliverable:** D1.2 · **Output:** findings documented in this file

## Why this spike exists

x402 on Stellar requires the payer's wallet to sign Soroban **authorization entries**. Two
questions are unresolved in public documentation, and both affect where spending controls
can be enforced. Rather than assume an answer and discover the truth in week eight, the
project resolves them in the first two weeks, before the contract design in Tranche 2 is
closed.

The architecture is designed so that **either answer to either question still satisfies the
tranche deliverables** — see [`architecture.md`](./architecture.md) §7.4.

## Question 1 — Can a Soroban custom account act as the x402 payer?

**Why it matters.** If yes, the spending controls live in `__check_auth` and are
cryptographically unbypassable: no valid authorization can exist without passing them. If
no, enforcement moves to a layered model (DFNS policy engine before signing, plus
settlement-contract validation), which delivers the same product behaviour under a weaker
trust model.

**What is known.** Stellar documents auth-entry signing as the requirement for an
x402-compatible wallet. The facilitator validates that auth entries are "signed by the
payer" — unambiguous for a classic account, undocumented for a contract account.
OpenZeppelin publishes a Smart Account implementation for Stellar, so the primitive exists;
its acceptance by the facilitator's verification path is unverified.

**Method.**

1. Deploy a minimal Soroban custom account on testnet whose `__check_auth` accepts a single
   Ed25519 signer.
2. Build an x402 payment where the payer is that contract address, and submit it to the
   facilitator's `/verify`.
3. If verification succeeds, proceed to `/settle` and confirm the transfer on-chain.
4. Repeat against a self-hosted instance of the facilitator plugin, to distinguish a
   protocol limitation from a hosted-configuration limitation.

**Exit criteria.** A recorded testnet transaction settling from a contract payer (Path A
confirmed), or a documented rejection with the facilitator's reason (Path B adopted).

## Question 2 — Can DFNS policies discriminate on hash-signing requests?

**Why it matters.** In Path B, the DFNS policy engine is the primary pre-signature control.
If policies cannot inspect or classify a hash-signing request, that enforcement layer is
coarser than assumed and more weight falls on the settlement-contract validation.

**What is known.** DFNS supports `kind: "Hash"` signing across all cryptographic schemes,
exposes Stellar as a first-class network, and gates signature requests through a policy
engine (signature requests carry an `approvalId` and a policy-resolution timestamp). The
public documentation does not state whether a policy can discriminate by signature kind or
reason about the contents of an opaque hash.

**Method.** Confirm directly with DFNS during the free 30-day sandbox, and empirically test
a policy configured to restrict hash signing on a sandbox key.

**Exit criteria.** A written answer from DFNS, plus a sandbox test result, before the D1.3
design is closed.

## Question 3 — Is EURC accepted by the hosted facilitator?

**Why it matters.** Wirex settles in both USDC and EURC and the merchant base is European,
so EURC is not optional. The facilitator is configured with an explicit list of accepted
asset contract addresses per network.

**Method.** Query `/supported` on both testnet and mainnet endpoints and enumerate the
accepted asset contracts.

**Exit criteria.** Either EURC is accepted (no further work), or a self-hosted facilitator
plugin instance is deployed with both USDC and EURC configured. The plugin is open source
and published on npm with a documented Docker Compose example, so this is a deployment
exercise rather than new protocol work.

## Findings

_To be completed during Tranche 1. Each question is answered here with the evidence
(transaction hashes, API responses, written confirmations) that supports the answer._

| Question | Status | Answer | Evidence |
|---|---|---|---|
| 1 — Custom account as payer | Open | — | — |
| 2 — DFNS policy granularity | Open | — | — |
| 3 — EURC facilitator support | Open | — | — |
