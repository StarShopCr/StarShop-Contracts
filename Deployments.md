# Stellar Testnet Deployments - StarShop Contracts

> ⚠️ Test Environment Only - Stellar Testnet
> Source Account: `alice`
> Network: testnet

This file tracks the deployment of all smart contracts under `StarShopContracts/` to Stellar Testnet.

## Deployment Process

Each contract is built and deployed with:

```bash
stellar contract build
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/<contract_name>.wasm \
  --source alice \
  --network testnet
```

## Deployed Contracts

| Contract | Path | Contract ID | Deployed At | Status |
|----------|------|-------------|-------------|--------|
| starshop-core | StarShopContracts/starshop-core | `CA...` | - | Pending |
| starshop-marketplace | StarShopContracts/starshop-marketplace | `CA...` | - | Pending |
| starshop-token | StarShopContracts/starshop-token | `CA...` | - | Pending |
| starshop-nft | StarShopContracts/starshop-nft | `CA...` | - | Pending |
| starshop-escrow | StarShopContracts/starshop-escrow | `CA...` | - | Pending |

### Notes
- All deployments are for testing purposes only, not production.
- Contract IDs must be updated after each successful deployment.
- Each contract has its own sub-issue for tracking.
