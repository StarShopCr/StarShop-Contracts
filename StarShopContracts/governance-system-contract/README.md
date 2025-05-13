# Governance System Contract

A robust, flexible on-chain governance system built on Soroban for the Stellar blockchain. This contract enables decentralized decision-making through a comprehensive proposal and voting mechanism with configurable parameters.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Technical Implementation](#technical-implementation)
- [Security Features](#security-features)
- [Building and Testing](#building-and-testing)
- [Usage Examples](#usage-examples)
- [Contributing](#contributing)

## Overview

The Governance System Contract provides a framework for decentralized governance, allowing token holders to create, vote on, and execute proposals that affect the system. The contract supports various proposal types, voting configurations, and execution mechanisms.

### Key Features

- **Flexible Proposal System**: Support for multiple proposal types (governance, technical, economic)
- **Dual Voting Mechanisms**: Choose between token-weighted voting or one-address-one-vote
- **Delegation**: Allow users to delegate their voting power to trusted representatives
- **Moderation Controls**: Admin and moderator roles to maintain system integrity
- **Execution Engine**: Automated execution of approved proposals
- **Configurable Parameters**: Customizable quorum, thresholds, and voting periods

### Use Cases

- DAO governance for treasury management
- Protocol parameter adjustments
- Smart contract upgrades
- Community-driven product decisions
- Economic policy changes

## Architecture

The governance system is built with a modular architecture, separating concerns into specialized components:

### Core Components

1. **Governance Contract (`governance.rs`)**: The main entry point that orchestrates all governance operations
2. **Proposal Manager (`proposals.rs`)**: Handles proposal lifecycle management
3. **Voting System (`voting.rs`)**: Manages vote casting and tallying
4. **Execution Engine (`execution.rs`)**: Executes approved proposal actions
5. **Weight Calculator (`weights.rs`)**: Determines voting power and handles delegation
6. **Types (`types.rs`)**: Defines data structures and constants
7. **Utils (`utils.rs`)**: Creates Symbol key for fetching proposals, statuses, votes and delegations

### Data Flow

```txt
User → Governance Contract → Specific Module → Storage/Events
```

### Contract Interactions

The governance contract interacts with several external contracts:

- **Token Contract**: For voting weight calculation based on token holdings
- **Referral Contract**: For user verification and level checks
- **Auction Contract**: For economic proposals that modify auction parameters

## Technical Implementation

### Proposal Lifecycle

Proposals follow a well-defined lifecycle:

1. **Draft**: Initial state when a proposal is created
2. **Active**: When a moderator activates the proposal, starting the voting period
3. **Passed/Rejected**: Based on voting outcome
4. **Executed**: After successful execution of the proposal's actions
5. **Vetoed**: If vetoed by a moderator after passing
6. **Canceled**: If canceled by the proposer, admin, or moderator

The state transitions are strictly controlled through permission checks and status validations.

### Voting Mechanisms

The contract supports two voting mechanisms:

#### Token-Weighted Voting

- Voting power proportional to token holdings
- Support for delegation of voting power
- Configurable maximum voting power cap

#### One-Address-One-Vote

- Equal voting power for all eligible voters
- Simple majority voting

Both mechanisms have configurable parameters:

- **Quorum**: Minimum participation required (percentage * 10000)
- **Threshold**: Support required for approval (percentage * 10000)
- **Voting Duration**: Length of the voting period in seconds

### Delegation System

The delegation system allows token holders to delegate their voting power:

- Users can delegate their entire voting power to a single address
- Delegated power is automatically included in the delegatee's vote
- Circular delegation is prevented through validation checks
- Delegations can be changed at any time

Implementation details:

- Delegations are stored using hashed keys for space efficiency
- The system tracks both delegations and delegators through bidirectional mappings

### Execution Engine

The execution engine handles the implementation of approved proposals:

- Supports multiple action types in a single proposal
- Enforces execution delay periods
- Handles interaction with external contracts
- Provides transaction atomicity (all actions succeed or all fail)

Supported action types:

- Updating proposal requirements
- Appointing/removing moderators
- Updating reward rates
- Updating level requirements
- Updating auction conditions (requires function implementation in auction contract)

### Storage Model

The contract uses Soroban's storage capabilities:

- Instance storage for contract state
- Key-based storage for proposals, votes, and delegations
- Efficient data structures to minimize storage costs

## Security Features

The governance system implements several security measures:

1. **Authorization Checks**: All sensitive operations require appropriate authorization
2. **Role-Based Access Control**: Admin and moderator roles with specific privileges
3. **Status Validations**: Strict state machine to prevent invalid operations
4. **Economic Security**: Required stake for proposal creation
5. **Cooldown Periods**: Prevent spam proposals
6. **Proposal Limits**: Cap on active proposals per address
7. **Referral Verification**: KYC/verification requirements for participation
8. **Level Checks**: Referral level requirements for economic changes

## Building and Testing

### Prerequisites

- Rust toolchain (1.68.0+)
- Soroban CLI (latest version)
- Stellar network access (testnet/pubnet)

### Building

1. Clone the repository:

```bash
git clone https://github.com/onlydust/starshop-contracts.git
cd starshop-contracts
```

2. Build the contract:

```bash
cd StarShopContracts/governance-system-contract
cargo build --release
```

3. Generate the WASM file:

```bash
soroban contract build
```

The compiled WASM will be available in the `target/wasm32-unknown-unknown/release/` directory.

### Testing

Run the tests with:

```bash
cargo test
```

For verbose test output:

```bash
cargo test -- --nocapture
```

To run specific tests:

```bash
cargo test test_create_proposal -- --nocapture
```

### Deploying

1. Install the Soroban CLI if you haven't already:

```bash
cargo install soroban-cli
```

2. Deploy to the testnet:

```bash
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/governance_system_contract.wasm \
  --source <source-account> \
  --network testnet
```

3. Initialize the contract (make sure to replace
 placeholder values):

```bash
soroban contract invoke \
  --id <contract-id> \
  --source <admin-account> \
  --network testnet \
  -- \
  initialize \
  --admin <admin-address> \
  --token <token-address> \
  --referral_contract <referral-contract-address> \
  --auction_contract <auction-contract-address> \
  --config <voting-config-json>
```

## Usage Examples

### Creating a Proposal

```bash
soroban contract invoke \
  --id <contract-id> \
  --source <proposer-account> \
  --network testnet \
  -- \
  create_proposal \
  --proposer <proposer-address> \
  --title "Example Proposal" \
  --description "This is an example proposal" \
  --metadata_hash "0x1234567890abcdef" \
  --proposal_type "GovernanceChange" \
  --actions '[{"UpdateProposalRequirements": {"cooldown_period": 86400, "required_stake": 1000, "proposal_limit": 5, "max_voting_power": 10000}}]' \
  --voting_config '{"duration": 604800, "quorum": 3000, "threshold": 5000, "execution_delay": 86400, "one_address_one_vote": false}'
```

### Activating a Proposal

```bash
soroban contract invoke \
  --id <contract-id> \
  --source <moderator-account> \
  --network testnet \
  -- \
  activate_proposal \
  --caller <moderator-address> \
  --proposal_id 1
```

### Casting a Vote

```bash
soroban contract invoke \
  --id <contract-id> \
  --source <voter-account> \
  --network testnet \
  -- \
  cast_vote \
  --voter <voter-address> \
  --proposal_id 1 \
  --support true
```

### Executing a Proposal

```bash
soroban contract invoke \
  --id <contract-id> \
  --source <executor-account> \
  --network testnet \
  -- \
  execute_proposal \
  --executor <executor-address> \
  --proposal_id 1
```

## Contributing

Contributions to the Governance System Contract are welcome! Please see our [Contributing Guide](../../GUIDE_CONTRIBUTING.md) for more information.