# NFT-Fungible Batch Distribution Contract

OpenZeppelin-based Soroban smart contract for batch minting and distribution of NFT tokens for StarShop merchants and creators.

## Architecture

- **Standards:** OpenZeppelin Stellar AccessControl (`DEFAULT_ADMIN_ROLE`, `MINTER_ROLE`).
- **Storage:** Isolated instance storage for metadata and persistent storage for token ownership, balances, and role assignments (`DataKey` enum).
- **Events:** Comprehensive events emitted for initialization, single mints, batch mints, transfers, and role administration.

## Key Methods

### Administrative
- `initialize(admin, name, symbol, base_uri)`: Initializes the contract and assigns admin and minter roles.
- `grant_role(caller, role, account)`: Grants a role (restricted to `DEFAULT_ADMIN_ROLE`).
- `revoke_role(caller, role, account)`: Revokes a role (restricted to `DEFAULT_ADMIN_ROLE`).
- `has_role(role, account)`: Checks whether an account has a given role.

### Minting & Distribution
- `mint_batch(caller, recipient, quantity)`: Mints `quantity` consecutive tokens to `recipient`. Restricted to `MINTER_ROLE`. Maximum 500 tokens per batch.

### Transfers & Ownership
- `transfer(from, to, token_id)`: Safely transfers token from caller to recipient. Caller must authorize and own the token.
- `owner_of(token_id)`: Returns current owner of the token.
- `balance_of(owner)`: Returns number of tokens owned by the account.
- `total_supply()`: Returns total number of tokens minted.
- `token_uri(token_id)`: Returns metadata URI for the token.

## Testing & Verification

Run tests:
```bash
cargo test -p nft-fungible-batch-contract
```

Linting:
```bash
cargo clippy -p nft-fungible-batch-contract -- -D warnings
```

WASM Build:
```bash
cargo build -p nft-fungible-batch-contract --release --target wasm32-unknown-unknown
```
