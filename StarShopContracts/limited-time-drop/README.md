# Limited-Time Drop Contract

A Soroban smart contract for managing limited-time product drops with access control and purchase tracking.

## Features

- **Access Control**
  - Whitelist-based access
  - User level verification (Standard, Premium, Verified)
  - Admin-only functions for managing access

- **Drop Management**
  - Create time-limited product drops
  - Set supply limits and per-user purchase limits
  - Track drop status (Pending, Active, Completed, Cancelled)
  - Manage drop metadata (title, image, price)

- **Purchase System**
  - Enforce whitelist and user level requirements
  - Track purchase history per user
  - Monitor total purchases per drop
  - Maintain buyer lists

## Contract Structure

The contract is organized into several modules:

- `access.rs`: Handles whitelist and user level management
- `drop.rs`: Manages drop creation and status
- `tracking.rs`: Tracks purchases and buyer information
- `types.rs`: Defines data structures and enums
- `interface.rs`: Documents the contract's public interface

## Usage

### Initialization

```rust
// Initialize contract with admin
contract.initialize(admin_address);
```

### Creating a Drop

```rust
// Create a new drop
let drop_id = contract.create_drop(
    creator_address,
    title,
    product_id,
    max_supply,
    start_time,
    end_time,
    price,
    per_user_limit,
    image_uri
);
```

### Managing Access

```rust
// Add user to whitelist (admin only)
contract.add_to_whitelist(admin_address, user_address);

// Set user level (admin only)
contract.set_user_level(admin_address, user_address, UserLevel::Premium);
```

### Making Purchases

```rust
// Purchase from a drop
contract.purchase(buyer_address, drop_id, quantity);
```

### Querying Information

```rust
// Get drop details
let drop = contract.get_drop(drop_id);

// Get purchase history
let history = contract.get_purchase_history(user_address, drop_id);

// Get total purchases
let total = contract.get_drop_purchases(drop_id);

// Get buyer list
let buyers = contract.get_buyer_list(drop_id);
```

## Access Control

The contract implements a two-tier access control system:

1. **Whitelist Requirement**
   - Users must be added to the whitelist by an admin
   - Only whitelisted users can make purchases

2. **User Level Requirement**
   - Users must have at least Premium level to make purchases
   - Standard level users cannot make purchases
   - Verified level provides additional privileges

## Error Handling

The contract uses a comprehensive error system:

```rust
pub enum Error {
    NotInitialized,     // Contract not initialized
    AlreadyInitialized, // Contract already setup
    Unauthorized,       // Caller lacks permission
    DropNotFound,       // Drop doesn't exist
    DropNotActive,      // Drop is not active
    DropEnded,          // Drop has ended
    DropNotStarted,     // Drop hasn't started yet
    InsufficientSupply, // Not enough items left
    UserLimitExceeded,  // User purchase limit reached
    InvalidQuantity,    // Invalid purchase quantity
    InvalidTime,        // Invalid time window
    InvalidPrice,       // Invalid price
    NotWhitelisted,     // User not whitelisted
    InsufficientLevel,  // User level too low
    InvalidUserLevel,   // Invalid user level
    PurchaseFailed,     // Purchase transaction failed
}
```

## Events

The contract emits events for important state changes:

- `init`: Contract initialization
- `drop_created`: New drop creation
- `purchase`: Successful purchase
- `status_update`: Drop status changes

## Security Considerations

1. **Access Control**
   - All admin functions require authentication
   - Purchase functions verify whitelist and user level
   - Status updates are admin-only

2. **Data Validation**
   - Time windows are validated
   - Prices must be positive
   - Supply limits are enforced
   - User purchase limits are tracked

3. **State Management**
   - Contract state is properly initialized
   - Storage keys are properly namespaced
   - Data consistency is maintained

## Testing

The contract includes comprehensive tests covering:

- Drop creation and management
- Purchase functionality
- Access control
- Error conditions
- Edge cases

Run tests with:
```bash
cargo test
```