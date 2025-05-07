# Subscription System Contract

This repository contains the smart contract for managing subscriptions in the StarShop ecosystem. The contract is designed to handle subscription plans, user subscriptions, and related operations in a decentralized and secure manner.

This contract is built in Rust using the [Soroban](https://soroban.stellar.org/) framework for the Stellar network.

## Features

- **Subscription Plans**: Create and manage subscription plans with different durations and prices.
- **User Subscriptions**: Allow users to subscribe to plans and track their subscription status.
- **Renewals**: Enable automatic or manual subscription renewals.
- **Cancellations**: Allow users to cancel their subscriptions.
- **Events**: Emit events for key actions like subscription creation, renewal, and cancellation.

## Contract Overview

The contract is written in Rust and includes the following key components:

### 1. Subscription Plan Management

- **Create Plan**: Admins can create subscription plans with a unique ID, price, and duration.
- **Update Plan**: Admins can update the details of existing plans.
- **Delete Plan**: Admins can remove plans that are no longer needed.

### 2. User Subscription Management

- **Subscribe**: Users can subscribe to a plan by paying the required amount.
- **Renew**: Users can renew their subscriptions before they expire.
- **Cancel**: Users can cancel their subscriptions at any time.

### 3. Events

The contract emits the following events:

- `PlanCreated`: Emitted when a new subscription plan is created.
- `Subscribed`: Emitted when a user subscribes to a plan.
- `Renewed`: Emitted when a user renews their subscription.
- `Cancelled`: Emitted when a user cancels their subscription.

## Project Files

- **contracts/**: Contains the Rust smart contract code built with Soroban.
- **scripts/**: Houses any helper scripts for automated tasks or interactions.
- **test/**: Includes test cases verifying the contract logic.
- **Cargo.toml**: Defines Rust dependencies and project configuration.
- **README.md**: This documentation file providing a high-level overview of the contract.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

