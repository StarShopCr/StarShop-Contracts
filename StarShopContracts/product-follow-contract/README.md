# 🔔 Product Follow System Contract

## 📌 Overview

The Product Follow System Contract is a Stellar-based smart contract that enables users to follow products and receive notifications about various updates. This system is built using the Soroban SDK and provides a robust way to manage product subscriptions and notifications.

### Key Features

- **Product Following**: Users can follow/unfollow specific products
- **Category-based Notifications**: Support for different notification types:
  - Price changes
  - Restock alerts
  - Special offers
- **Notification Preferences**: Users can customize their notification settings
- **Follow Management**: Efficient handling of follow relationships
- **Rate Limiting**: Built-in protection against notification spam
- **Authorization**: Secure user authentication for all operations

## 🏗 Contract Structure

| File | Description |
|------|-------------|
| `lib.rs` | Core contract implementation and trait definitions |
| `follow.rs` | Follow management system implementation |
| `alerts.rs` | Alert system for handling notifications |
| `notification.rs` | Notification delivery and preferences management |
| `interface.rs` | Trait definitions for contract interfaces |
| `datatype.rs` | Data structures and type definitions |
| `test.rs` | Comprehensive test suite |

## 🛠 Setup and Installation

### Prerequisites
- Rust toolchain (latest stable version)
- Soroban CLI
- Stellar development environment
- Git

### Build Steps

1. Build for local development & testing:
```bash
cargo build -p product-follow-contract
```

2. Build WASM bytecode for deployment:
```bash
cargo build -p product-follow-contract --release --target wasm32-unknown-unknown
```

3. Run tests:
```bash
cargo test -p product-follow-contract
```

## 🚀 Testnet Deployment (Test-Only)

This contract is deployed and configured on the **Stellar Testnet** for testing and staging integration verification.

### Deployment Details
- **Network:** Stellar Testnet
- **Network Passphrase:** `Test SDF Network ; September 2015`
- **RPC URL:** `https://soroban-testnet.stellar.org`
- **Contract ID:** `CCS7XKR3UV76MGC2XQ4ABLMTMGOUBTG2AQ4EGAL4L5CD2OQKP4X7A66G`
- **Status:** Test-Only (Staging Verification)
- **Explorer:** [StellarExpert Testnet Contract](https://stellar.expert/explorer/testnet/contract/CCS7XKR3UV76MGC2XQ4ABLMTMGOUBTG2AQ4EGAL4L5CD2OQKP4X7A66G)

### Deployment Script & Dry-Run

The deployment artifact and script are available in `deploy_testnet.sh`:

```bash
# Verify WASM artifact and testnet parameters
./StarShopContracts/product-follow-contract/deploy_testnet.sh --dry-run
```

### Contract Initialization

To initialize the deployed contract on Testnet:

```bash
stellar contract invoke \
    --id CCS7XKR3UV76MGC2XQ4ABLMTMGOUBTG2AQ4EGAL4L5CD2OQKP4X7A66G \
    --source <ADMIN_SECRET_OR_IDENTITY> \
    --network testnet \
    -- initialize \
    --admin <ADMIN_ADDRESS>
```

## 💻 Usage

### Core Functions

1. **Following a Product**
```rust
fn follow_product(
    env: Env,
    user: Address,
    product_id: u32,
    categories: Vec<FollowCategory>
) -> Result<(), Error>
```

2. **Unfollowing a Product**
```rust
fn unfollow_product(
    env: Env,
    user: Address,
    product_id: u32
) -> Result<(), Error>
```

3. **Checking Follow Status**
```rust
fn is_following(
    env: Env,
    user: Address,
    product_id: u32
) -> bool
```

## ⚙️ Configuration and Settings

### Notification Categories

The contract supports multiple notification categories that can be configured per follow:

```rust
pub enum FollowCategory {
    PriceChange,   
    Restock,      
    SpecialOffer   
}
```

### Follow Preferences

Users can customize their follow settings through the `NotificationPreferences` structure:

```rust
pub struct NotificationPreferences {
    pub user: Address,                 
    pub categories: Vec<FollowCategory>,
    pub mute_notifications: bool,       
    pub priority: NotificationPriority  
}
```

## 🔐 Security Features

### Authorization
- All follow/unfollow operations require user authorization
- Contract methods validate caller identity
- Built-in protection against unauthorized modifications

### Rate Limiting
- Maximum of 100 followers per product
- Cooldown period between follow operations
- Protection against spam and abuse

## 🧪 Testing

The contract includes comprehensive tests covering:

1. Follow/Unfollow operations
2. Notification delivery
3. Authorization checks
4. Rate limiting
5. Edge cases and error handling

Run the test suite:
```bash
cargo test --package product-follow-contract
```

## 🚨 Error Handling

### Common Error Types

```rust
pub enum FollowError {
    FollowLimitExceeded = 1,  
    AlreadyFollowing = 2,     
    NotFollowing = 3,      
    InvalidCategory = 4,      
    Unauthorized = 5,          
    InvalidProductId = 6      
}
```

## 📝 Usage Examples

### Following a Product

```rust
use soroban_sdk::{Address, Env};
use product_follow_contract::{FollowCategory, ProductFollowClient};


let env = Env::default();
let client = ProductFollowClient::new(&env, &contract_id);


let categories = vec![
    &env,
    FollowCategory::PriceChange,
    FollowCategory::Restock
];

client.follow_product(
    &user_address,
    &product_id,
    &categories
)?;
```

### Managing Notification Preferences

```rust

let preferences = NotificationPreferences {
    user: user_address,
    categories: vec![&env, FollowCategory::PriceChange],
    mute_notifications: false,
    priority: NotificationPriority::High
};

client.update_preferences(&preferences)?;
```

## 📚 Best Practices

1. **Rate Limiting**
   - Implement cooldown periods between follow operations
   - Monitor follower counts to prevent spam
   - Use batch operations for multiple follows when possible

2. **Error Handling**
   - Always check return values for errors
   - Implement proper error recovery mechanisms
   - Log important events for debugging

3. **Security**
   - Validate all user inputs
   - Implement proper authorization checks
   - Regular security audits of follow operations

## 🤝 Contributing

We welcome contributions to improve the Product Follow System Contract! Please:

1. Fork the repository
2. Create a feature branch
3. Submit a pull request with detailed description
4. Ensure all tests pass
5. Follow our coding standards

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
