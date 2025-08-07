# Payment Escrow Contract

A Soroban smart contract that implements a secure payment escrow system with arbitrator dispute resolution.

## Overview

This contract provides a decentralized escrow service where:
- Buyers can create payments and lock funds in escrow
- Sellers can confirm delivery of goods/services
- Buyers can confirm receipt and release funds to sellers
- Disputes can be raised and resolved by authorized arbitrators
- Expired payments can be claimed back by buyers

## Key Features

### Payment Management
- **Create Payment**: Buyers can create payments with specified amounts, expiry periods, and descriptions
- **Confirm Delivery**: Sellers can confirm delivery, changing status to "Delivered"
- **Confirm Receipt**: Buyers can confirm receipt, completing the payment and releasing funds
- **Claim Expired**: Buyers can claim back funds from expired payments

### Dispute Resolution
- **Raise Dispute**: Buyers can dispute payments with reasons
- **Resolve Dispute**: Authorized arbitrators can resolve disputes in favor of buyer or seller
- **Dispute Deadlines**: Disputes can only be raised within a deadline based on payment expiry

### Arbitrator Management
- **Multiple Arbitrators**: Support for multiple arbitrators with vector storage
- **Add Arbitrator**: Existing arbitrators can add new arbitrators
- **Remove Arbitrator**: Arbitrators can be removed from the system
- **Transfer Rights**: Arbitrator rights can be transferred to new addresses

### Contract Upgrades
- **Upgrade Functionality**: Contract can be upgraded with new WASM
- **Authorization**: All arbitrators must authorize upgrades
- **State Preservation**: All existing state is preserved during upgrades

## Contract Structure

```
src/
├── lib.rs              # Main contract entry point
├── test.rs             # Comprehensive test suite
├── datatypes.rs        # Data structures and enums
├── interface.rs         # Contract interface definitions
├── implementations/
│   ├── arbitrator.rs   # Arbitrator management functions
│   ├── claim.rs        # Payment claim functionality
│   ├── create.rs       # Payment creation logic
│   ├── delivery.rs     # Delivery confirmation
    ├── mod.rs     
│   └── dispute.rs      # Dispute resolution
└── 
```

## Usage Examples

### Creating a Payment
```rust
// Buyer creates a payment
let payment_id = client.create_payment(
    &buyer,
    &seller,
    &amount,
    &token_contract_id,
    &expiry_days,
    &description
);
```

### Confirming Delivery
```rust
// Seller confirms delivery
client.seller_confirm_delivery(&payment_id, &seller);

// Buyer confirms receipt
client.buyer_confirm_delivery(&payment_id, &buyer);
```

### Raising a Dispute
```rust
// Buyer raises dispute
let dispute_reason = String::from_str(&env, "Item not as described");
client.dispute_payment(&payment_id, &buyer, &dispute_reason);
```

### Resolving a Dispute
```rust
// Arbitrator resolves dispute
client.resolve_dispute(
    &payment_id,
    &arbitrator,
    &DisputeDecision::PaySeller,
    &resolution_reason
);
```

### Claiming Expired Payment
```rust
// Buyer claims expired payment
client.claim_payment(&payment_id, &buyer);
```

## Testing

The contract includes comprehensive tests covering:
- Payment creation and management
- Delivery confirmation flows
- Dispute resolution scenarios
- Arbitrator management
- Contract upgrade functionality
- Authorization and security checks

Run tests with:
```bash
cargo test
```

## Security Features

- **Authorization Checks**: All functions require proper authorization
- **Dispute Deadlines**: Prevents disputes after payment expiry
- **Arbitrator Consensus**: Multiple arbitrators for dispute resolution
- **State Validation**: Comprehensive state checks and validations
- **Upgrade Safety**: Secure upgrade mechanism with authorization

## Deployment

The contract is designed for deployment on the Soroban network with:
- Proper initialization with arbitrator
- Token integration for payments
- Upgrade capability for future improvements
- Comprehensive error handling

## License

This project is licensed under the MIT License.