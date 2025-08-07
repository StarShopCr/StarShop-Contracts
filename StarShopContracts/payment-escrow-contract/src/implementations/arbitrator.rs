use crate::{
    datatypes::{DataKey, PaymentEscrowError},
    interface::ArbitratorInterface,
    PaymentEscrowContract, PaymentEscrowContractArgs, PaymentEscrowContractClient,
};
use soroban_sdk::{contractimpl, symbol_short, Address, Env, Vec};

/// Implementation of the ArbitratorInterface trait for PaymentEscrowContract
/// This module handles all arbitrator-related operations including adding, 
/// transferring, and retrieving arbitrators for dispute resolution.
#[contractimpl]
impl ArbitratorInterface for PaymentEscrowContract {

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
    fn add_arbitrator(
        env: Env,
        arbitrator: Address,
        new_arbitrator: Address,
    ) -> Result<(), PaymentEscrowError> {
        // Authentication - existing arbitrator must authorize this transaction
        // This ensures only authorized arbitrators can add new ones
        arbitrator.require_auth();

        // Retrieve the current list of arbitrators from persistent storage
        // This list contains all authorized arbitrators who can resolve disputes
        let mut arbitrators: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Arbitrator)
            .ok_or(PaymentEscrowError::NotFound)?;

        // Authorization check: verify that the caller is an existing arbitrator
        // Only current arbitrators can add new ones, preventing unauthorized additions
        if !arbitrators.contains(&arbitrator) {
            return Err(PaymentEscrowError::NotArbitrator);
        }

        // Duplicate prevention: check if the new arbitrator already exists
        // This prevents adding the same arbitrator multiple times
        if arbitrators.contains(&new_arbitrator) {
            return Err(PaymentEscrowError::ArbitratorAlreadyExists);
        }

        // Add the new arbitrator to the list
        // This expands the pool of available arbitrators for dispute resolution
        arbitrators.push_back(new_arbitrator.clone());
        
        // Persist the updated arbitrators list to storage
        // This ensures the new arbitrator is available for future dispute resolutions
        env.storage()
            .persistent()
            .set(&DataKey::Arbitrator, &arbitrators);

        // Emit an event for transparency and off-chain tracking
        // This allows external systems to track arbitrator additions
        env.events().publish(
            (symbol_short!("new_arb"), new_arbitrator.clone()),
            arbitrator.clone(),
        );

        Ok(())
    }

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
    fn get_arbitrators(env: Env) -> Result<Vec<Address>, PaymentEscrowError> {
        // Retrieve the arbitrators list from persistent storage
        // This list is maintained by the add_arbitrator and transfer_arbitrator_rights functions
        let arbitrators: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Arbitrator)
            .ok_or(PaymentEscrowError::NotFound)?;

        Ok(arbitrators)
    }

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
    fn transfer_arbitrator_rights(
        env: Env,
        old_arbitrator: Address,
        new_arbitrator: Address,
    ) -> Result<(), PaymentEscrowError> {
        // Authentication - old arbitrator must authorize this transaction
        // This ensures only the current arbitrator can transfer their rights
        old_arbitrator.require_auth();

        // Retrieve the current list of arbitrators from persistent storage
        // This list will be modified to replace the old arbitrator with the new one
        let arbitrators: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Arbitrator)
            .ok_or(PaymentEscrowError::UnauthorizedAccess)?;

        // Authorization check: verify that the caller is an existing arbitrator
        // Only current arbitrators can transfer their rights
        if !arbitrators.contains(&old_arbitrator) {
            return Err(PaymentEscrowError::NotArbitrator);
        }

        // Duplicate prevention: check if the new arbitrator is already in the list
        // This prevents creating duplicate entries in the arbitrators list
        if arbitrators.contains(&new_arbitrator) {
            return Err(PaymentEscrowError::ArbitratorAlreadyExists);
        }

        // Create a new arbitrators list by filtering out the old arbitrator
        // This removes the old arbitrator from the list
        let mut new_arbitrators = Vec::new(&env);
        for a in arbitrators.iter() {
            // Keep all arbitrators except the one being replaced
            if a != old_arbitrator {
                new_arbitrators.push_back(a.clone());
            }
        }
        
        // Add the new arbitrator to the list
        // This completes the transfer of rights from old to new arbitrator
        new_arbitrators.push_back(new_arbitrator.clone());

        // Persist the updated arbitrators list to storage
        // This ensures the transfer is permanent and the new arbitrator is authorized
        env.storage()
            .persistent()
            .set(&DataKey::Arbitrator, &new_arbitrators);

        // Emit an event for transparency and off-chain tracking
        // This allows external systems to track arbitrator transfers
        env.events().publish(
            (
                symbol_short!("xfer_arb"),
                old_arbitrator.clone(),
                new_arbitrator.clone(),
            ),
            new_arbitrator,
        );

        Ok(())
    }
}
