/**
 * Neutral Agent Pay client SDK.
 *
 * Wraps `fetch` so that an AI agent or application transparently pays for x402-protected
 * endpoints: it detects the 402 challenge, builds the SEP-41 `transfer` invocation, has the
 * authorization entry signed by the configured wallet, and retries the original request.
 *
 * The transaction envelope is deliberately left unsigned — the facilitator rebuilds and
 * submits it, and rejects payloads whose envelope already carries signatures.
 */

import type { PaymentRequirements, StellarNetwork } from "@neutralagentpay/facilitator-client";

export interface Signer {
  /** Stellar address (or contract address, in the custom-account path) of the payer. */
  getAddress(): Promise<string>;
  /**
   * Sign a 32-byte Soroban authorization payload hash.
   *
   * This is the whole reason the wallet layer is pluggable: x402 on Stellar requires
   * auth-entry signing, and the listed compatible wallets are all human-driven. The DFNS
   * signer implements this through hash signing, without human interaction.
   */
  signAuthEntryHash(hash: Uint8Array): Promise<Uint8Array>;
}

export interface ClientConfig {
  network: StellarNetwork;
  rpcUrl: string;
  signer: Signer;
  /** Hard ceiling enforced client-side, independent of on-chain controls. */
  maxAmountPerRequest?: bigint;
  /** Assets the client is willing to pay in, as SEP-41 contract addresses. */
  acceptedAssets?: string[];
}

export interface PaymentResult {
  transactionHash: string;
  amount: string;
  asset: string;
  payer: string;
  paymentId: string;
}

export class NeutralAgentPayClient {
  constructor(private readonly config: ClientConfig) {}

  /**
   * `fetch` with automatic x402 payment. On a 402 response the challenge is parsed, the
   * payment is authorized and signed, and the request is retried once.
   */
  async fetch(_input: string | URL, _init?: RequestInit): Promise<Response> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.1");
  }

  /** Parse the payment requirements from a 402 response. */
  parseChallenge(_response: Response): Promise<PaymentRequirements> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.1");
  }

  /**
   * Build the SEP-41 `transfer` invocation, compute the authorization entry payload hash,
   * obtain a signature from the signer, and assemble the x402 payment payload.
   */
  async authorizePayment(_requirements: PaymentRequirements): Promise<PaymentResult> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.1");
  }
}

export { DfnsSigner } from "./dfns-signer.js";
