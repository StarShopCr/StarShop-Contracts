#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, Vec, BytesN};
use crate::datatypes::{DataKey, PaymentEscrowError};

/// Payment Escrow Contract
/// 
/// This is the main contract struct that implements a decentralized payment escrow system
/// with arbitrator dispute resolution. The contract provides secure peer-to-peer transactions
/// with automated fund management and dispute handling capabilities.
/// 
/// Key Features:
/// - Payment creation and management with expiry periods
/// - Two-phase delivery confirmation (seller → buyer)
/// - Dispute resolution system with authorized arbitrators
/// - Expired payment claim functionality
/// - Contract upgrade capability with state preservation
/// - Multiple arbitrator support with dynamic management
#[contract]
pub struct PaymentEscrowContract;

#[contractimpl]
impl PaymentEscrowContract {
    /// Initialize the payment escrow contract with an initial arbitrator
    /// 
    /// This function sets up the contract with the first arbitrator who will have
    /// the authority to add additional arbitrators and resolve disputes. The contract
    /// can only be initialized once to prevent re-initialization attacks.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `arbitrator` - The address of the initial arbitrator
    /// 
    /// # Returns
    /// * `Ok(())` - Contract successfully initialized
    /// * `Err(PaymentEscrowError::AlreadyInitialized)` - Contract already initialized
    /// 
    /// # Security Considerations
    /// - Prevents re-initialization to maintain contract state integrity
    /// - Initial arbitrator has full authority to manage the system
    /// - Arbitrator address should be carefully chosen for trustworthiness
    pub fn init(env: Env, arbitrator: Address) -> Result<(), PaymentEscrowError> {
        // Check if contract is already initialized to prevent re-initialization
        if env.storage().persistent().has(&DataKey::Arbitrator) {
            return Err(PaymentEscrowError::AlreadyInitialized);
        }

        // Create a vector to store arbitrators and add the initial arbitrator
        let mut arbitrators = Vec::new(&env);
        arbitrators.push_back(arbitrator);
        
        // Store the arbitrators vector in persistent storage
        env.storage().persistent().set(&DataKey::Arbitrator, &arbitrators);

        Ok(())
    }

    /// Get the current version of the contract
    /// 
    /// This function returns the contract version number, which is useful for
    /// tracking contract upgrades and ensuring compatibility with frontend applications.
    /// The version number should be incremented when the contract is upgraded.
    /// 
    /// # Returns
    /// * `u32` - The current contract version
    pub fn version() -> u32 {
        1
    }

    /// Upgrade the contract with new WASM code
    /// 
    /// This function allows the contract to be upgraded with new WASM code while
    /// preserving all existing state. The upgrade requires authorization from all
    /// current arbitrators to ensure security and prevent unauthorized upgrades.
    /// 
    /// # Arguments
    /// * `e` - The Soroban environment
    /// * `new_wasm_hash` - The hash of the new WASM code to deploy
    /// 
    /// # Security Features
    /// - Requires authorization from all arbitrators (consensus mechanism)
    /// - Preserves all existing contract state during upgrade
    /// - Prevents unauthorized upgrades that could compromise the system
    /// 
    /// # Upgrade Process
    /// 1. All arbitrators must authorize the upgrade
    /// 2. New WASM code is deployed to the contract
    /// 3. All existing state (payments, arbitrators, etc.) is preserved
    /// 4. Contract continues to function with new code
    /// 
    /// # Important Notes
    /// - The new WASM must be compatible with existing state structure
    /// - All arbitrators must be available to authorize the upgrade
    /// - Upgrade should be thoroughly tested before deployment
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), PaymentEscrowError> {
        // Retrieve the current list of arbitrators from storage
        let arbitrators: Vec<Address> = e.storage().persistent().get(&DataKey::Arbitrator)
                   .ok_or(PaymentEscrowError::NotInitialized)?;
                
        // Ensure there are arbitrators to authorize the upgrade
        if arbitrators.is_empty() {
            return Err(PaymentEscrowError::NotInitialized);
        }
        
        // Require authorization from all arbitrators (consensus mechanism)
        // This ensures that no single arbitrator can upgrade the contract alone
        arbitrators.iter().for_each(|a| a.require_auth());

        // Deploy the new WASM code to the contract
        // This updates the contract's code while preserving all state
        e.deployer().update_current_contract_wasm(new_wasm_hash);
        
        Ok(())
    }
}

/// Re-export all implementation modules for external access
/// 
/// This makes all the contract functions available through the main contract interface.
/// The implementations are organized into logical modules:
/// - arbitrator.rs: Arbitrator management functions
/// - claim.rs: Payment claim functionality
/// - create.rs: Payment creation logic
/// - delivery.rs: Delivery confirmation
/// - dispute.rs: Dispute resolution
pub use implementations::*;

// Declare modules
mod datatypes;        // Data structures, enums, and storage keys
mod interface;        // Contract interface definitions and traits
mod implementations;  // Main contract function implementations
// mod token;        // Token integration (commented out for future use)
mod test;            // Comprehensive test suite