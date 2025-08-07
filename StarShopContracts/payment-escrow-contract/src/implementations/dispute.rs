use crate::{
    datatypes::{DataKey, Payment, PaymentEscrowError, PaymentStatus, DisputeEvent, DisputeResolvedEvent, DisputeDecision},
    interface::DisputeInterface,
    PaymentEscrowContract, PaymentEscrowContractClient, PaymentEscrowContractArgs
};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{symbol_short, String, Address, Env, Vec, contractimpl};

/// Implementation of the DisputeInterface trait for PaymentEscrowContract
/// This module handles dispute creation and resolution for escrow payments,
/// including dispute initiation, arbitrator resolution, and fund distribution.
#[contractimpl]
impl DisputeInterface for PaymentEscrowContract {

    /// Allows buyers or sellers to initiate a dispute for an escrow payment
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `payment_id` - The unique identifier of the payment to dispute
    /// * `disputer` - The address initiating the dispute (must be buyer or seller)
    /// * `reason` - Human-readable reason for the dispute
    /// 
    /// # Returns
    /// * `Result<(), PaymentEscrowError>` - Success or error
    /// 
    /// # Business Logic
    /// * Locks funds in escrow until arbitrator resolution
    /// * Records dispute details for transparency
    /// * Prevents further payment actions until resolved
    /// * Only works within dispute deadline period
    /// 
    /// # Security
    /// * Requires disputer authentication
    /// * Validates disputer is buyer or seller
    /// * Prevents duplicate disputes
    /// * Enforces dispute deadline
    /// * Prevents disputes on completed/expired payments
    fn dispute_payment(env: Env, payment_id: u128, disputer: Address, reason: String) -> Result<(), PaymentEscrowError> {
        // Authentication - disputer must authorize this transaction
        // This ensures only the intended disputer can initiate disputes
        disputer.require_auth();

        // Retrieve the payment details from persistent storage
        // This contains all payment information including status, participants, and timestamps
        let payment: Payment = env
            .storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)?;

        // Authorization check: verify that the disputer is either the buyer or seller
        // Only payment participants can initiate disputes, preventing unauthorized disputes
        if payment.buyer != disputer && payment.seller != disputer {
            return Err(PaymentEscrowError::UnauthorizedAccess);
        }

        // Duplicate prevention: check if payment has already been disputed
        // This prevents multiple disputes on the same payment
        if payment.status == PaymentStatus::Disputed {
            return Err(PaymentEscrowError::AlreadyDisputed);
        }

        // Status validation: ensure payment is in a disputable state
        // Completed or expired payments cannot be disputed
        if payment.status == PaymentStatus::Completed || payment.status == PaymentStatus::Expired {
            return Err(PaymentEscrowError::NotValid);
        }

        // Deadline check: verify that the dispute period has not expired
        // This ensures disputes are only possible within the allowed timeframe
        let current_timestamp = env.ledger().timestamp();
        if current_timestamp > payment.dispute_deadline {
            return Err(PaymentEscrowError::DisputePeriodExpired);
        }

        // Update payment status to Disputed to lock the funds
        // This prevents further actions until an arbitrator resolves the dispute
        let updated_payment = Payment {
            status: PaymentStatus::Disputed,
            ..payment
        };

        // Persist the updated payment status to storage
        // This ensures the dispute is permanent and funds are locked
        env.storage()
            .persistent()
            .set(&payment_id, &updated_payment);

        // Create dispute event for record keeping
        // This provides transparency and enables off-chain dispute tracking
        let dispute_event = DisputeEvent {
            order_id: payment_id,
            initiator: disputer,
            reason,
        };

        // Store dispute event in persistent storage
        // This maintains a permanent record of the dispute for future reference
        env.storage()
            .persistent()
            .set(&(DataKey::DisputedPayments, payment_id), &dispute_event);

        // Emit an event for transparency and off-chain tracking
        // This allows external systems to monitor dispute creation
        env.events().publish((symbol_short!("disputed"), payment_id), payment_id);

        Ok(())
    }

    /// Allows authorized arbitrators to resolve disputes and distribute funds
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `payment_id` - The unique identifier of the disputed payment
    /// * `arbitrator` - The address of the arbitrator resolving the dispute
    /// * `decision` - The arbitrator's decision (PaySeller or RefundBuyer)
    /// * `reason` - Human-readable reason for the decision
    /// 
    /// # Returns
    /// * `Result<(), PaymentEscrowError>` - Success or error
    /// 
    /// # Business Logic
    /// * Transfers funds based on arbitrator decision
    /// * Marks payment as Completed (seller wins) or Refunded (buyer wins)
    /// * Records resolution details for transparency
    /// * Finalizes the dispute resolution process
    /// 
    /// # Security
    /// * Requires arbitrator authentication
    /// * Validates arbitrator authorization
    /// * Ensures payment is in disputed status
    /// * Requires non-empty resolution reason
    /// * Prevents unauthorized dispute resolution
    fn resolve_dispute(env: Env, payment_id: u128, arbitrator: Address, decision: DisputeDecision, reason: String) -> Result<(), PaymentEscrowError> {
        // Authentication - arbitrator must authorize this transaction
        // This ensures only authorized arbitrators can resolve disputes
        arbitrator.require_auth();

        // Retrieve the payment details from persistent storage
        // This contains all payment information needed for resolution
        let payment: Payment = env
            .storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)?;

        // Status validation: ensure payment is in disputed status
        // Only disputed payments can be resolved by arbitrators
        if payment.status != PaymentStatus::Disputed {
            return Err(PaymentEscrowError::NotValid);
        }

        // Authorization check: verify that the arbitrator is authorized
        // Retrieve the list of authorized arbitrators from storage
        let authorized_arbitrators: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Arbitrator)
            .ok_or(PaymentEscrowError::UnauthorizedAccess)?;

        // Verify the arbitrator is in the authorized list
        // This prevents unauthorized parties from resolving disputes
        if !authorized_arbitrators.contains(&arbitrator) {
            return Err(PaymentEscrowError::NotArbitrator);
        }

        // Input validation: ensure resolution reason is not empty
        // This ensures arbitrators provide meaningful resolution explanations
        if reason.is_empty() {
            return Err(PaymentEscrowError::NotValid);
        }

        // Create token client for fund transfer operations
        // This enables interaction with the token contract to distribute funds
        let token_client = TokenClient::new(&env, &payment.token);

        // Execute the arbitrator's decision using pattern matching
        // This handles both possible outcomes: PaySeller or RefundBuyer
        match decision {
            DisputeDecision::PaySeller => {
                // Transfer funds from escrow to the seller
                // This awards the payment to the seller as decided by the arbitrator
                let transfer_result = token_client.transfer(
                    &env.current_contract_address(),
                    &payment.seller,
                    &payment.amount,
                );
                
                // Validate the transfer was successful
                // This ensures the fund distribution actually occurred
                if transfer_result != () {
                    return Err(PaymentEscrowError::TransferFailed);
                }

                // Update payment status to Completed to reflect seller victory
                // This marks the payment as successfully completed
                let updated_payment = Payment {
                    status: PaymentStatus::Completed,
                    ..payment
                };
                
                // Persist the updated payment status to storage
                // This ensures the resolution is permanent
                env.storage()
                    .persistent()
                    .set(&payment_id, &updated_payment);
            },
            DisputeDecision::RefundBuyer => {
                // Transfer funds from escrow back to the buyer
                // This refunds the payment to the buyer as decided by the arbitrator
                let transfer_result = token_client.transfer(
                    &env.current_contract_address(),
                    &payment.buyer,
                    &payment.amount,
                );
                
                // Validate the transfer was successful
                // This ensures the refund actually occurred
                if transfer_result != () {
                    return Err(PaymentEscrowError::TransferFailed);
                }

                // Update payment status to Refunded to reflect buyer victory
                // This marks the payment as refunded to the buyer
                let updated_payment = Payment {
                    status: PaymentStatus::Refunded,
                    ..payment
                };
                
                // Persist the updated payment status to storage
                // This ensures the resolution is permanent
                env.storage()
                    .persistent()
                    .set(&payment_id, &updated_payment);
            }
        }

        // Create dispute resolved event for record keeping
        // This provides transparency and enables off-chain resolution tracking
        let dispute_resolved_event = DisputeResolvedEvent {
            order_id: payment_id,
            resolution: decision,
            admin: arbitrator,
        };

        // Store dispute resolved event in persistent storage
        // This maintains a permanent record of the resolution for future reference
        env.storage()
            .persistent()
            .set(&(DataKey::ResolvedDisputes, payment_id), &dispute_resolved_event);

        // Emit an event for transparency and off-chain tracking
        // This allows external systems to monitor dispute resolutions
        env.events().publish((symbol_short!("resolved"), payment_id), payment_id);

        Ok(())
    }
}