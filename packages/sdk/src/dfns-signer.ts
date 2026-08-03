/**
 * DFNS-backed signer.
 *
 * Uses the DFNS Keys API to sign the 32-byte Soroban authorization payload hash
 * (`POST /keys/{keyId}/signatures` with `kind: "Hash"`), which DFNS documents as supported
 * across all cryptographic schemes. This is what allows a policy-governed, non-interactive
 * wallet to participate in x402 on Stellar.
 *
 * Two integration constraints, documented in architecture §6.3:
 *   1. Signature generation is a change-inducing request and requires a User Action
 *      Signature in addition to the bearer token. For a non-interactive agent this must be
 *      configured as a service account or delegated signing arrangement.
 *   2. Whether DFNS policies can discriminate by signature kind is unconfirmed. To be
 *      settled with DFNS during the 30-day sandbox, before D1.3 closes.
 */

import type { Signer } from "./index.js";

export interface DfnsSignerConfig {
  apiUrl: string;
  /** DFNS key id — obtained from a wallet as `wallet.signingKey.id`. */
  keyId: string;
  /** `Stellar` for mainnet, `StellarTestnet` for testnet. */
  network: "Stellar" | "StellarTestnet";
  serviceAccountToken: string;
}

export class DfnsSigner implements Signer {
  constructor(private readonly config: DfnsSignerConfig) {}

  async getAddress(): Promise<string> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.3");
  }

  async signAuthEntryHash(_hash: Uint8Array): Promise<Uint8Array> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.3");
  }

  /** Provision a Stellar account and establish trustlines for the configured assets. */
  async provisionAccount(_assets: string[]): Promise<string> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.3");
  }
}
