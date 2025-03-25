# Grindurus Grinder - ICP Rust Implementation

This is a Rust implementation of the Grindurus Grinder for the Internet Computer Platform (ICP). It provides automated pool rebalancing and management services.

## Features

- Ethereum integration via ic-web3
- Automated pool rebalancing
- Intent NFT management
- Gas optimization
- Price monitoring via CoinGecko API
- Stable storage using ic-stable-structures

## Setup

1. Install the DFINITY Canister SDK (dfx):
```bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```

2. Start the local network:
```bash
dfx start --background
```

3. Deploy the canister:
```bash
dfx deploy
```

## Configuration

Create a `.env` file with your configuration:

```env
INTENT_NFT_ADDRESS=your_intent_nft_address
POOLS_NFT_ADDRESS=your_pools_nft_address
GRINDER_AI_ADDRESS=your_grinder_ai_address
RPC_URL=your_ethereum_rpc_url
```

## Architecture

- `lib.rs`: Core canister functionality and state management
- `ethereum.rs`: Ethereum interface and smart contract interactions
- `grinder.rs`: Pool management and rebalancing logic

## Key Differences from JS Version

1. Persistent Storage: Uses ic-stable-structures instead of JSON files
2. Concurrency: Async/await patterns adapted for ICP
3. Memory Management: Efficient memory usage with stable storage
4. Type Safety: Strong type system with Rust's safety guarantees

## Testing

Run the tests:
```bash
cargo test
```

## Security

- All transaction costs are verified before execution
- Gas price optimization with 1.4x multiplier
- Maximum transaction cost limits
- Secure key management via ICP
