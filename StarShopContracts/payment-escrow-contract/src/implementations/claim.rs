use crate::{
    datatypes::{Payment, PaymentEscrowError, PaymentStatus},
    interface::ClaimInterface,
    PaymentEscrowContract, PaymentEscrowContractClient, PaymentEscrowContractArgs
};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{contractimpl, symbol_short, Address, Env};

/// Implementation of the ClaimInterface trait for PaymentEscrowContract
/// This module handles payment claiming functionality, allowing buyers to 
/// claim refunds for expired payments that haven't been completed or disputed.
#[contractimpl]
impl ClaimInterface for PaymentEscrowContract {

    /// Allows the buyer to claim a refund for an expired payment
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `payment_id` - The unique identifier of the payment to claim
    /// * `claimer` - The address attempting to claim the payment (must be the buyer)
    /// 
    /// # Returns
    /// * `Result<(), PaymentEscrowError>` - Success or error
    /// 
    /// # Business Logic
    /// * Only expired payments can be claimed
    /// * Only the buyer can claim expired payments
    /// * Disputed payments cannot be claimed (must be resolved first)
    /// * Claims transfer funds back to the buyer and mark payment as Refunded
    /// 
    /// # Security
    /// * Requires authentication from the claimer
    /// * Validates payment expiration
    /// * Prevents claiming of disputed payments
    /// * Ensures only the buyer can claim
    fn claim_payment(env: Env, payment_id: u128, claimer: Address) -> Result<(), PaymentEscrowError> {
        // Authentication - claimer must authorize this transaction
        // This ensures only the intended claimer can execute the claim operation
        claimer.require_auth();

        // Retrieve the payment details from persistent storage
        // This contains all payment information including status, amounts, and timestamps
        let payment: Payment = env
            .storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)?;

        // Expiration check: verify that the payment has actually expired
        // This prevents premature claims and ensures the escrow period has passed
        let current_timestamp = env.ledger().timestamp();
        if current_timestamp <= payment.expiry {
            return Err(PaymentEscrowError::NotExpired);
        }

        // Dispute check: prevent claiming of payments that are currently disputed
        // Disputed payments must be resolved by an arbitrator before any claims can be made
        if payment.status == PaymentStatus::Disputed {
            return Err(PaymentEscrowError::PaymentDisputed);
        }

        // Authorization check: ensure only the buyer can claim expired payments
        // This prevents unauthorized parties from claiming funds
        if payment.buyer != claimer {
            return Err(PaymentEscrowError::UnauthorizedAccess);
        }

        // Create token client for fund transfer operations
        // This enables interaction with the token contract to transfer funds
        let token_client = TokenClient::new(&env, &payment.token);

        // Transfer funds from the escrow contract back to the buyer
        // This executes the actual refund of the escrowed amount
        token_client.transfer(&env.current_contract_address(), &payment.buyer,
            &payment.amount);

        // Update payment status to Refunded to reflect the completed claim
        // This prevents double-claiming and provides clear payment state
        let updated_payment = Payment {
            status: PaymentStatus::Refunded,
            ..payment
        };
        
        // Persist the updated payment status to storage
        // This ensures the claim is permanent and the payment state is updated
        env.storage()
            .persistent()
            .set(&payment_id, &updated_payment);

        // Emit an event for transparency and off-chain tracking
        // This allows external systems to track successful claims
        env.events().publish((symbol_short!("claimed"), payment_id), payment_id);

        Ok(())
    }
}