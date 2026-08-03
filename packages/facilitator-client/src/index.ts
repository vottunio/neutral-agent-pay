/**
 * Client for the x402 v2 facilitator API.
 *
 * Written against the specification rather than a specific provider, so the same client
 * works with the hosted Build-on-Stellar facilitator, the Coinbase facilitator, or a
 * self-hosted OpenZeppelin relayer plugin. Switching provider is configuration only —
 * which is the mitigation for both the EURC asset question and facilitator availability
 * risk (architecture §4.3, §8, R2/R5).
 */

export type StellarNetwork = "stellar:testnet" | "stellar:pubnet";

export interface PaymentRequirements {
  scheme: "exact";
  network: StellarNetwork;
  /** Asset SEP-41 contract address. */
  asset: string;
  /** Amount in the asset's smallest unit, as a decimal string. */
  maxAmountRequired: string;
  /** Destination (merchant) address. */
  payTo: string;
  resource: string;
  description?: string;
  mimeType?: string;
}

export interface PaymentPayload {
  x402Version: 2;
  scheme: "exact";
  network: StellarNetwork;
  /** Transaction XDR with signed auth entries and an unsigned envelope. */
  payload: { transaction: string };
}

export interface VerifyResult {
  isValid: boolean;
  invalidReason?: string;
  payer?: string;
}

export interface SettleResult {
  success: boolean;
  errorReason?: string;
  transaction?: string;
  network?: StellarNetwork;
  payer?: string;
}

export interface SupportedKind {
  x402Version: number;
  scheme: string;
  network: StellarNetwork;
  /** Asset contract addresses this facilitator will settle. */
  assets?: string[];
}

export interface FacilitatorConfig {
  url: string;
  apiKey?: string;
  timeoutMs?: number;
}

export class FacilitatorClient {
  constructor(private readonly config: FacilitatorConfig) {}

  /** POST /verify — validates a payment payload against the stated requirements. */
  async verify(
    _payload: PaymentPayload,
    _requirements: PaymentRequirements,
  ): Promise<VerifyResult> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.2");
  }

  /** POST /settle — submits the payment on-chain via the facilitator's relayer. */
  async settle(
    _payload: PaymentPayload,
    _requirements: PaymentRequirements,
  ): Promise<SettleResult> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.2");
  }

  /**
   * GET /supported — discovery of accepted schemes, networks and assets.
   *
   * Called during the Tranche 1 spike to determine whether EURC is accepted by the hosted
   * facilitator, which decides whether a self-hosted instance is required.
   */
  async supported(): Promise<SupportedKind[]> {
    throw new Error("NotImplemented: Tranche 1, deliverable D1.2");
  }
}
