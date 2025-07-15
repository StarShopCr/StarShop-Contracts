use crate::{
    datatypes::{DataKey, Payment, PaymentEscrowError, PaymentStatus},
    interface::PaymentInterface,
    PaymentEscrowContract, PaymentEscrowContractArgs, PaymentEscrowContractClient,
};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{contractimpl, symbol_short, Address, Env, String};

/// Implementation of the PaymentInterface trait for PaymentEscrowContract
/// This module handles payment creation and retrieval functionality, including
/// escrow setup, fund transfers, and payment lifecycle management.
#[contractimpl]
impl PaymentInterface for PaymentEscrowContract {

    /// Retrieves the total count of payments created in the system
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment reference
    /// 
    /// # Returns
    /// * `u128` - The total number of payments created
    /// 
    /// # Purpose
    /// * Provides system statistics for monitoring and analytics
    /// * Used internally for generating unique payment IDs
    /// * Enables external systems to track payment volume
    fn get_payment_count(env: &Env) -> u128 {
        // Retrieve the payment counter from persistent storage
        // This counter is incremented with each new payment creation
        env.storage()
            .persistent()
            .get(&DataKey::PaymentCounter)
            .unwrap_or(0) // Default to 0 if no payments have been created yet
    }

    /// Creates a new escrow payment between a buyer and seller
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `buyer` - The address of the buyer (payer)
    /// * `seller` - The address of the seller (recipient)
    /// * `amount` - The payment amount in token units
    /// * `token` - The token contract address for the payment
    /// * `expiry_days` - Number of days until payment expires (0 = 30 days default)
    /// * `description` - Human-readable description of the payment
    /// 
    /// # Returns
    /// * `Result<u128, PaymentEscrowError>` - Payment ID on success, error on failure
    /// 
    /// # Business Logic
    /// * Transfers funds from buyer to escrow contract
    /// * Creates unique payment ID and stores payment details
    /// * Sets up dispute deadline based on expiry period
    /// * Validates all inputs and buyer authorization
    /// 
    /// # Security
    /// * Requires buyer authentication
    /// * Validates buyer has sufficient funds
    /// * Prevents self-payment (buyer != seller)
    /// * Ensures positive payment amounts
    fn create_payment(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: i128,
        token: Address,
        expiry_days: u32,
        description: String,
    ) -> Result<u128, PaymentEscrowError> {
        // Authentication - buyer must authorize this transaction
        // This ensures only the intended buyer can create payments
        buyer.require_auth();

        // Input validation: ensure payment amount is positive
        // This prevents creating payments with zero or negative amounts
        if amount <= 0 {
            return Err(PaymentEscrowError::InvalidAmount);
        }

        // Self-payment prevention: buyer and seller must be different addresses
        // This prevents users from creating payments to themselves
        if buyer == seller {
            return Err(PaymentEscrowError::CannotPaySelf);
        }

        // Generate unique payment ID by incrementing the payment counter
        // This ensures each payment has a unique identifier for tracking
        let mut payment_counter = Self::get_payment_count(&env);
        payment_counter += 1;
        let payment_id = payment_counter;

        // Calculate payment timestamps for lifecycle management
        let current_ledger = env.ledger().timestamp();
        
        // Set expiry period with default fallback
        // If no expiry is specified (0), default to 30 days for safety
        let expiry_days = if expiry_days == 0 { 30 } else { expiry_days };
        
        // Calculate expiry timestamp by adding days to current time
        // Convert days to seconds for precise timestamp calculation
        let expiry_timestamp = current_ledger + (expiry_days as u64 * 24 * 60 * 60);
        
        // Calculate dispute deadline based on payment duration
        // For long-term payments (7+ days): dispute deadline = expiry - 7 days
        // For short-term payments (<7 days): dispute deadline = expiry time
        // This ensures disputes are possible throughout the payment period
        let dispute_deadline = if expiry_days >= 7 {
            expiry_timestamp - (7 * 24 * 60 * 60) // 7 days before expiry for long payments
        } else {
            expiry_timestamp // Full payment period for short payments
        };

        // Create token client for fund transfer operations
        // This enables interaction with the specified token contract
        let token_client = TokenClient::new(&env, &token);

        // Validate buyer has sufficient funds before transfer
        // This prevents failed transfers and ensures payment feasibility
        let buyer_balance = token_client.balance(&buyer);
        if buyer_balance < amount {
            return Err(PaymentEscrowError::InsufficientFunds);
        }

        // Transfer funds from buyer to escrow contract
        // This locks the funds in escrow until payment completion or expiry
        token_client.transfer(&buyer, &env.current_contract_address(), &amount);

        // Create payment struct with all relevant details
        // This contains all information needed for payment lifecycle management
        let payment = Payment {
            id: payment_id,
            buyer,
            seller,
            amount,
            token,
            status: PaymentStatus::Pending, // Initial status: funds held in escrow
            created_at: current_ledger,
            expiry: expiry_timestamp,
            dispute_deadline,
            description,
        };

        // Update the payment counter in persistent storage
        // This ensures the next payment gets a unique ID
        env.storage()
            .persistent()
            .set(&DataKey::PaymentCounter, &payment_id);

        // Store the complete payment details in persistent storage
        // This enables retrieval and management of the payment throughout its lifecycle
        env.storage().persistent().set(&payment_id, &payment);

        // Emit an event for transparency and off-chain tracking
        // This allows external systems to monitor payment creation
        env.events().publish(
            (DataKey::PaymentCounter, symbol_short!("payment")),
            payment_id,
        );

        Ok(payment_id as u128)
    }

    /// Retrieves a specific payment by its unique identifier
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `payment_id` - The unique identifier of the payment to retrieve
    /// 
    /// # Returns
    /// * `Result<Payment, PaymentEscrowError>` - Payment details or error if not found
    /// 
    /// # Purpose
    /// * Enables payment status checking and details retrieval
    /// * Supports dispute resolution and claim verification
    /// * Provides transparency for all payment participants
    fn get_a_payment(env: Env, payment_id: u128) -> Result<Payment, PaymentEscrowError> {
        // Retrieve payment details from persistent storage
        // Returns the complete payment struct or NotFound error
        env.storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)
    }
}
