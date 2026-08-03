# Example: a paid API endpoint

A minimal Express server with one x402-protected route, and a client that pays for it
automatically. This is the reference integration an external developer reproduces as part of
the Tranche 2 completion criteria.

## Run

```bash
cp ../../.env.example .env    # fill in FACILITATOR_API_KEY and DFNS credentials
npm install
npm run server                # merchant side, port 4021
npm run client                # agent side, pays and prints the response
```

## What happens

1. The client requests `/forecast` and receives `402` with the payment requirements.
2. The client builds a SEP-41 `transfer` invocation and has the authorization entry signed
   by its DFNS-managed Stellar account.
3. The request is retried with the payment payload attached.
4. The server verifies and settles through the facilitator, which submits the transaction
   and sponsors the network fee.
5. The server returns the forecast; the transaction is verifiable on Stellar Expert.

Delivered in Tranche 1 (testnet) and hardened in Tranche 3 (mainnet, with a tagged SDK
release and quickstarts).
