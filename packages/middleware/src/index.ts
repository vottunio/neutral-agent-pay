/**
 * x402 payment middleware for merchant servers.
 *
 * A developer declares which routes are paid and at what price; the middleware issues the
 * 402 challenge, verifies and settles through the facilitator, and serves the resource.
 */

import type { StellarNetwork } from "@neutralagentpay/facilitator-client";

export interface RoutePricing {
  /** Price as a decimal string in the asset's display unit, e.g. "0.001". */
  price: string;
  /** SEP-41 asset contract address. Defaults to the config-level asset. */
  asset?: string;
  description?: string;
  /**
   * Serve the resource once `/verify` succeeds, settling asynchronously.
   *
   * Removes settlement latency from the request path at the cost of a small settlement
   * risk window. Off by default; opting in is an explicit, per-route merchant decision.
   */
  settleAsync?: boolean;
}

export interface MiddlewareConfig {
  network: StellarNetwork;
  /** Merchant address receiving payment. */
  payTo: string;
  defaultAsset: string;
  facilitator: { url: string; apiKey?: string };
  routes: Record<string, RoutePricing>;
}

/** Express-compatible middleware factory. */
export function paymentMiddleware(_config: MiddlewareConfig): unknown {
  throw new Error("NotImplemented: Tranche 1, deliverable D1.1");
}
