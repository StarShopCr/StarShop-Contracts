use crate::datatypes::{DisputeDecision, Payment, PaymentEscrowError, PaymentStatus, DeliveryDetails};
use soroban_sdk::{Address, Env, String, Vec};

/// PaymentInterface trait defines core payment management functionality
/// This trait handles payment creation, retrieval, and basic payment operations
/// for the escrow system.
pub trait PaymentInterface {
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
    fn get_payment_count(env: &Env) -> u128 ;

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
    fn create_payment(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: i128,
        token: Address,
        expiry_days: u32,
        description: String,
    ) -> Result<u128, PaymentEscrowError>;

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
    fn get_a_payment(env: Env, payment_id: u128) -> Result<Payment, PaymentEscrowError>;
}




/// DeliveryInterface trait defines delivery confirmation and status management
/// This trait handles the delivery confirmation process, fund releases,
/// and delivery status tracking for escrow payments.
pub trait DeliveryInterface {
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
    fn buyer_confirm_delivery(env: Env, payment_id: u128, buyer: Address) -> Result<(), PaymentEscrowError>;

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
    fn seller_confirm_delivery(env: Env, payment_id: u128, seller: Address) -> Result<(), PaymentEscrowError>;

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
    fn get_delivery_status(env: Env, payment_id: u128) -> Result<PaymentStatus, PaymentEscrowError>;

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
    fn get_delivery_details(env: Env, payment_id: u128) -> Result<DeliveryDetails, PaymentEscrowError>;
}



/// DisputeInterface trait defines dispute creation and resolution functionality
/// This trait handles dispute initiation by participants and resolution by arbitrators,
/// including fund distribution based on arbitrator decisions.
pub trait DisputeInterface {
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
    fn dispute_payment(env: Env, payment_id: u128, disputer: Address, reason: String) -> Result<(), PaymentEscrowError>;
    
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
    fn resolve_dispute(env: Env, payment_id: u128, arbitrator: Address, decision: DisputeDecision, reason: String) -> Result<(), PaymentEscrowError>;
}




/// ClaimInterface trait defines payment claiming functionality
/// This trait handles refund claims for expired payments that haven't been
/// completed or disputed, allowing buyers to recover their funds.
pub trait ClaimInterface {
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
    fn claim_payment(env: Env, payment_id: u128, claimer: Address) -> Result<(), PaymentEscrowError>;
}





/// ArbitratorInterface trait defines arbitrator management functionality
/// This trait handles adding, transferring, and retrieving arbitrators who
/// are authorized to resolve disputes in the escrow system.
pub trait ArbitratorInterface {
    /// Adds a new arbitrator to the system
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `arbitrator` - The existing arbitrator who is authorizing this transaction
    /// * `new_arbitrator` - The new arbitrator address to be added
    /// 
    /// # Returns
    /// * `Result<(), PaymentEscrowError>` - Success or error
    /// 
    /// # Security
    /// * Only existing arbitrators can add new arbitrators
    /// * Prevents duplicate arbitrators
    /// * Requires authentication from the existing arbitrator
    fn add_arbitrator(env: Env, arbitrator: Address, new_arbitrator: Address) -> Result<(), PaymentEscrowError>;
    
    /// Retrieves the current list of all authorized arbitrators
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// * `Result<Vec<Address>, PaymentEscrowError>` - List of arbitrator addresses or error
    /// 
    /// # Purpose
    /// * Provides transparency about who can resolve disputes
    /// * Allows external systems to verify arbitrator authorization
    /// * Useful for UI/UX to show available arbitrators
    fn get_arbitrators(env: Env) -> Result<Vec<Address>, PaymentEscrowError>;
    
    /// Transfers arbitrator rights from one address to another
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `old_arbitrator` - The current arbitrator transferring their rights
    /// * `new_arbitrator` - The new address to receive arbitrator rights
    /// 
    /// # Returns
    /// * `Result<(), PaymentEscrowError>` - Success or error
    /// 
    /// # Security
    /// * Only existing arbitrators can transfer their rights
    /// * Prevents transferring to existing arbitrators
    /// * Requires authentication from the old arbitrator
    /// 
    /// # Use Cases
    /// * Key rotation for security
    /// * Changing arbitrator addresses
    /// * Replacing compromised arbitrator keys
    fn transfer_arbitrator_rights(env: Env, old_arbitrator: Address, new_arbitrator: Address) -> Result<(), PaymentEscrowError>;
}