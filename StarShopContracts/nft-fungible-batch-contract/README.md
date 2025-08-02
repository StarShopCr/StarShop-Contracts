# NFT-Fungible Batch Distribution Contract

A Soroban smart contract for Starshop that enables batch minting and distribution of NFTs with role-based access control.

## Features

- **Batch Minting**: Sellers can mint multiple NFTs in a single transaction
- **Role-Based Access Control**: Admin and minter roles with OpenZeppelin patterns
- **NFT Transfers**: Full transfer functionality for minted NFTs
- **Event Emission**: Comprehensive events for all contract actions
- **Storage Isolation**: Well-structured storage with key prefixes

## Contract Functions

### Initialization
- `initialize(admin)` - Initialize contract and set first admin

### Minting
- `mint_batch(recipient, quantity, sender)` - Mint batch of NFTs (MINTER_ROLE only)

### Transfers
- `transfer(from, to, token_id, sender)` - Transfer NFT between addresses

### Role Management
- `grant_role(role, account, sender)` - Grant role to account (ADMIN only)
- `revoke_role(role, account, sender)` - Revoke role from account (ADMIN only)

### Queries
- `owner_of(token_id)` - Get token owner
- `balance_of(owner)` - Get NFT balance of address
- `total_supply()` - Get total NFTs minted
- `has_role(role, account)` - Check if address has role

## Events

- `ContractInitialized(admin)` - Contract initialization
- `NFTMinted(token_id, recipient)` - Single NFT minted
- `NFTBatchMinted(quantity, recipient)` - Batch minting completed
- `Transfer(from, to, token_id)` - NFT transfer
- `RoleGranted(role, account, sender)` - Role granted
- `RoleRevoked(role, account, sender)` - Role revoked

## Roles

- **DEFAULT_ADMIN_ROLE**: Can grant/revoke any role
- **MINTER_ROLE**: Can mint NFTs in batches

## Storage Structure

- `init` - Initialization guard
- `owner:{token_id}` - Token ownership mapping
- `balance:{address}` - Address NFT balance
- `supply` - Total supply counter
- `token_id` - Next token ID counter
- `role:{role}:{address}` - Role assignments

## Integration with Starshop

This contract is designed to integrate with the Starshop marketplace:
- Sellers can be granted MINTER_ROLE to mint NFTs for buyers
- Batch minting reflects product purchase quantities
- Transfer functionality enables secondary trading
- Events support frontend integration and analytics

## Building

```bash
cargo build --release
```

## Testing

```bash
cargo test
```

## Deployment

Deploy to Stellar using Soroban CLI:

```bash
soroban contract deploy target/wasm32-unknown-unknown/release/nft_fungible_batch_contract.wasm
```

## Security

- Role-based access control prevents unauthorized minting
- Ownership checks prevent unauthorized transfers
- Initialization guard prevents re-initialization
- Storage isolation prevents key collisions 