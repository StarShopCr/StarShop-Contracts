use crate::{
    datatypes::{DeliveryDetails, Payment, PaymentEscrowError, PaymentStatus},
    interface::DeliveryInterface,
    PaymentEscrowContract, PaymentEscrowContractArgs, PaymentEscrowContractClient,
};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{contractimpl, symbol_short, Address, Env};

/// Implementation of the DeliveryInterface trait for PaymentEscrowContract
/// This module handles delivery confirmation and status management for escrow payments,
/// including buyer and seller confirmations, fund releases, and delivery tracking.
#[contractimpl]
impl DeliveryInterface for PaymentEscrowContract {

    /// Allows the buyer to confirm delivery and release funds to the seller
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `payment_id` - The unique identifier of the payment to confirm
    /// * `buyer` - The address of the buyer confirming delivery
    /// 
    /// # Returns
    /// * `Result<(), PaymentEscrowError>` - Success or error
    /// 
    /// # Business Logic
    /// * Only works after seller has confirmed delivery (status = Delivered)
    /// * Transfers funds from escrow to seller
    /// * Marks payment as Completed
    /// * Final step in successful escrow completion
    /// 
    /// # Security
    /// * Requires buyer authentication
    /// * Validates payment status and authorization
    /// * Prevents confirmation of disputed payments
    /// * Ensures only buyer can confirm delivery
    fn buyer_confirm_delivery(
        env: Env,
        payment_id: u128,
        buyer: Address,
    ) -> Result<(), PaymentEscrowError> {
        // Authentication - buyer must authorize this transaction
        // This ensures only the intended buyer can confirm delivery
        buyer.require_auth();

        // Retrieve the payment details from persistent storage
        // This contains all payment information including current status and participants
        let payment: Payment = env
            .storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)?;

        // Authorization check: verify that the caller is actually the buyer
        // This prevents unauthorized parties from confirming delivery
        if payment.buyer != buyer {
            return Err(PaymentEscrowError::UnauthorizedAccess);
        }

        // Status validation: ensure payment is in Delivered status
        // Buyer can only confirm delivery after seller has marked it as delivered
        if payment.status != PaymentStatus::Delivered {
            return Err(PaymentEscrowError::NotDelivered);
        }

        // Dispute check: prevent confirmation of disputed payments
        // Disputed payments must be resolved by an arbitrator before any confirmations
        if payment.status == PaymentStatus::Disputed {
            return Err(PaymentEscrowError::PaymentDisputed);
        }

        // Create token client for fund transfer operations
        // This enables interaction with the token contract to release funds
        let token_client = TokenClient::new(&env, &payment.token);

        // Transfer funds from escrow contract to the seller
        // This releases the escrowed funds to complete the payment
        token_client.transfer(
            &env.current_contract_address(),
            &payment.seller,
            &payment.amount,
        );

        // Update payment status to Completed to reflect successful delivery
        // This marks the payment as successfully completed and prevents further actions
        let updated_payment = Payment {
            status: PaymentStatus::Completed,
            ..payment
        };

        // Persist the updated payment status to storage
        // This ensures the completion is permanent and the payment state is updated
        env.storage()
            .persistent()
            .set(&payment_id, &updated_payment);

        // Emit an event for transparency and off-chain tracking
        // This allows external systems to track successful payment completions
        env.events()
            .publish((symbol_short!("completed"), payment_id), payment_id);

        Ok(())
    }

    /// Allows the seller to confirm that delivery has been made
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `payment_id` - The unique identifier of the payment to mark as delivered
    /// * `seller` - The address of the seller confirming delivery
    /// 
    /// # Returns
    /// * `Result<(), PaymentEscrowError>` - Success or error
    /// 
    /// # Business Logic
    /// * Changes payment status from Pending to Delivered
    /// * First step in the delivery confirmation process
    /// * Enables buyer to then confirm and release funds
    /// 
    /// # Security
    /// * Requires seller authentication
    /// * Validates payment status and authorization
    /// * Prevents confirmation of disputed payments
    /// * Ensures only seller can mark as delivered
    fn seller_confirm_delivery(
        env: Env,
        payment_id: u128,
        seller: Address,
    ) -> Result<(), PaymentEscrowError> {
        // Authentication - seller must authorize this transaction
        // This ensures only the intended seller can confirm delivery
        seller.require_auth();

        // Retrieve the payment details from persistent storage
        // This contains all payment information including current status and participants
        let payment: Payment = env
            .storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)?;

        // Status validation: ensure payment is in Pending status
        // Seller can only confirm delivery for payments that haven't been processed yet
        if payment.status != PaymentStatus::Pending {
            return Err(PaymentEscrowError::NotValid);
        }

        // Dispute check: prevent confirmation of disputed payments
        // Disputed payments must be resolved by an arbitrator before any confirmations
        if payment.status == PaymentStatus::Disputed {
            return Err(PaymentEscrowError::PaymentDisputed);
        }

        // Authorization check: verify that the caller is actually the seller
        // This prevents unauthorized parties from marking deliveries
        if payment.seller != seller {
            return Err(PaymentEscrowError::UnauthorizedAccess);
        }

        // Update payment status to Delivered to indicate delivery confirmation
        // This enables the buyer to then confirm and release funds
        let updated_payment = Payment {
            status: PaymentStatus::Delivered,
            ..payment
        };

        // Persist the updated payment status to storage
        // This ensures the delivery confirmation is permanent
        env.storage()
            .persistent()
            .set(&payment_id, &updated_payment);

        // Emit an event for transparency and off-chain tracking
        // This allows external systems to track delivery confirmations
        env.events()
            .publish((symbol_short!("delivered"), payment_id), payment_id);

        Ok(())
    }

    /// Retrieves the current delivery status of a payment
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `payment_id` - The unique identifier of the payment
    /// 
    /// # Returns
    /// * `Result<PaymentStatus, PaymentEscrowError>` - Current payment status or error
    /// 
    /// # Purpose
    /// * Enables status checking for payment participants
    /// * Supports UI/UX for showing current payment state
    /// * Helps determine next available actions
    fn get_delivery_status(
        env: Env,
        payment_id: u128,
    ) -> Result<PaymentStatus, PaymentEscrowError> {
        // Retrieve the payment details from persistent storage
        // This contains the current status of the payment
        let payment: Payment = env
            .storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)?;

        // Return the current payment status
        // This indicates where the payment is in its lifecycle
        Ok(payment.status)
    }

    /// Retrieves comprehensive delivery details for a payment
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `payment_id` - The unique identifier of the payment
    /// 
    /// # Returns
    /// * `Result<DeliveryDetails, PaymentEscrowError>` - Delivery details or error
    /// 
    /// # Purpose
    /// * Provides complete delivery information for transparency
    /// * Enables detailed payment tracking and reporting
    /// * Supports dispute resolution and claim verification
    fn get_delivery_details(
        env: Env,
        payment_id: u128,
    ) -> Result<DeliveryDetails, PaymentEscrowError> {
        // Retrieve the payment details from persistent storage
        // This contains all payment information needed for delivery details
        let payment: Payment = env
            .storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)?;

        // Create delivery details struct with relevant information
        // This provides a clean interface for delivery-related data
        let delivery_details = DeliveryDetails {
            payment_id,
            buyer: payment.buyer,
            seller: payment.seller,
            status: payment.status,
            created_at: payment.created_at,
            expiry: payment.expiry,
            description: payment.description,
        };

        Ok(delivery_details)
    }
}
