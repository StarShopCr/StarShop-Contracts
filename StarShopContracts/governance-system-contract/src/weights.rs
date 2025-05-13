use crate::types::{Error, ProposalRequirements, WeightSnapshot, REFERRAL_KEY, REQUIREMENTS_KEY, TOKEN_KEY};
use crate::utils::{get_governance_op_key, get_key_str};
use soroban_sdk::{
    symbol_short, token::TokenClient, vec, Address,
    Bytes, Env, Symbol, Vec,
};

/// WeightCalculator handles voting power calculations and delegations
/// This is a critical component for token-weighted governance voting
pub struct WeightCalculator;

impl WeightCalculator {
    /// Take a snapshot of voting power for a proposal
    /// Captures the voting power distribution at a specific point in time
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn take_snapshot(env: &Env, proposal_id: u32) -> Result<(), Error> {
        // Get the referral contract address
        let referral: Address = env
            .storage()
            .instance()
            .get(&REFERRAL_KEY)
            .ok_or(Error::NotInitialized)?;

        // Create a snapshot record with current timestamp
        let snapshot = WeightSnapshot {
            proposal_id,
            snapshot_at: env.ledger().timestamp(),
        };

        // Store the snapshot
        let key = Self::get_snapshot_key(env, proposal_id);
        env.storage().instance().set(&key, &snapshot);

        // Get total users from referral contract
        let total_users: u32 =
            env.invoke_contract(&referral, &Symbol::new(&env, "get_total_users"), vec![&env]);

        // Calculate total voting power (placeholder implementation)
        let mut total_power = 0i128;
        // Placeholder: Need user registry
        for _ in 0..total_users {
            total_power += 1000; // Simulate balance calculation
        }

        // Store the total voting power
        let key_bytes = Bytes::from_slice(env, b"VOTE_POW_");
        let key = get_key_str(env, key_bytes, proposal_id);
        env.storage().instance().set(&key, &total_power);

        // Emit an event for the snapshot
        env.events().publish(
            (symbol_short!("weight"), symbol_short!("snapshot")),
            (proposal_id, total_power),
        );

        Ok(())
    }

    /// Get the voting weight of an address for a specific proposal
    /// Considers token balance and delegated voting power
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `voter` - The address of the voter
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `Result<i128, Error>` - The voting weight or an error
    pub fn get_weight(env: &Env, voter: &Address, proposal_id: u32) -> Result<i128, Error> {
        // Check if this voter has delegated their voting power
        if let Some(_delegatee) = Self::get_delegation(env, voter) {
            // If they delegated, they have zero voting power themselves
            return Ok(0);
        }

        // Get the requirements to enforce max voting power cap
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();

        // Calculate base weight from token balance
        let base_weight = Self::get_base_weight(env, voter);

        // Add any delegated voting power from other addresses
        let delegated_weight = Self::get_delegated_weight(env, voter, proposal_id);

        // Total weight is base + delegated, capped by max_voting_power
        let total_weight = base_weight + delegated_weight;

        Ok(total_weight.min(requirements.max_voting_power))
    }

    /// Get the base voting weight from token balance
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `voter` - The address of the voter
    ///
    /// # Returns
    /// * `i128` - The base voting weight
    pub fn get_base_weight(env: &Env, voter: &Address) -> i128 {
        // Get the governance token
        let token: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = TokenClient::new(env, &token);

        // Return token balance as voting power
        token_client.balance(voter)
    }

    /// Get delegated voting weight from other addresses
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `delegatee` - The address that received delegations
    /// * `_proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `i128` - The delegated voting weight
    pub fn get_delegated_weight(env: &Env, delegatee: &Address, _proposal_id: u32) -> i128 {
        // Get all delegators for this delegatee
        let delegators = Self::get_delegators(env, &delegatee);

        // Sum up the base weights of all delegators
        delegators
            .iter()
            .map(|delegator| Self::get_base_weight(env, &delegator))
            .sum()
    }

    /// Delegate voting power to another address
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `delegator` - The address delegating their voting power
    /// * `delegatee` - The address receiving the delegated voting power
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn delegate(env: &Env, delegator: &Address, delegatee: &Address) -> Result<(), Error> {
        // Require authorization from the delegator
        delegator.require_auth();

        // Prevent self-delegation
        if delegator == delegatee {
            return Err(Error::SelfDelegationNotAllowed);
        }

        // Prevent circular delegations
        if let Some(current_delegatee) = Self::get_delegation(env, delegatee) {
            if current_delegatee == *delegator {
                return Err(Error::InvalidDelegation);
            }
        }

        // Store the delegation relationship
        let key = Self::get_delegator_key(env, delegator);
        env.storage().instance().set(&key, delegatee);

        // Update the delegatee's list of delegators
        Self::add_to_delegators_list(env, delegatee, delegator);

        // Emit an event for the delegation
        env.events().publish(
            (symbol_short!("vote"), symbol_short!("delegated")),
            (delegator, delegatee),
        );

        Ok(())
    }

    /// Get the address a voter has delegated to
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `delegator` - The address that might have delegated
    ///
    /// # Returns
    /// * `Option<Address>` - The delegatee if delegation exists
    pub fn get_delegation(env: &Env, delegator: &Address) -> Option<Address> {
        let key = Self::get_delegator_key(env, delegator);
        env.storage().instance().get(&key)
    }

    /// Get all addresses that delegated to a specific delegatee
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `delegatee` - The address that received delegations
    ///
    /// # Returns
    /// * `Vec<Address>` - List of delegators
    pub fn get_delegators(env: &Env, delegatee: &Address) -> Vec<Address> {
        let key = Self::get_delegatees_key(env, delegatee);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env])
    }

    /// Generate a storage key for a proposal snapshot
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `Symbol` - The storage key
    fn get_snapshot_key(env: &Env, proposal_id: u32) -> Symbol {
        let key_bytes = Bytes::from_slice(env, b"SNAP_");
        get_key_str(env, key_bytes, proposal_id)
    }

    /// Generate a storage key for a delegator's delegation
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `delegator` - The address of the delegator
    ///
    /// # Returns
    /// * `Symbol` - The storage key
    fn get_delegator_key(env: &Env, delegator: &Address) -> Symbol {
        // Generate a unique key for the delegator's delegation
        let key_bytes = Bytes::from_slice(env, b"DELGOR_");
        get_governance_op_key(env, key_bytes.clone(), 0, delegator)
    }

    /// Generate a storage key for a delegatee's list of delegators
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `delegatee` - The address of the delegatee
    ///
    /// # Returns
    /// * `Symbol` - The storage key
    fn get_delegatees_key(env: &Env, delegatee: &Address) -> Symbol {
        // Generate a unique key for the delegatee's list of delegators
        let key_bytes = Bytes::from_slice(env, b"DELGEE_");
        get_governance_op_key(env, key_bytes.clone(), 0, delegatee)
    }

    /// Add a delegator to a delegatee's list of delegators
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `delegatee` - The address receiving delegation
    /// * `delegator` - The address delegating voting power
    fn add_to_delegators_list(env: &Env, delegatee: &Address, delegator: &Address) {
        let key = Self::get_delegatees_key(env, delegatee);
        let mut list: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env]);

        // Add the delegator to the list if not already present
        if !list.iter().any(|a| a == *delegator) {
            list.push_back(delegator.clone());
            env.storage().instance().set(&key, &list);
        }
    }

    /// Remove a delegator from a delegatee's list of delegators
    /// Useful when revoking delegation (currently not used)
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `delegatee` - The address that received delegation
    /// * `delegator` - The address that was delegating
    #[allow(dead_code)]
    fn remove_from_delegators_list(env: &Env, delegatee: &Address, delegator: &Address) {
        let key = Self::get_delegatees_key(env, delegatee);
        let list: Vec<Address> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env]);

        // Create a new list excluding the specified delegator
        let mut new_list = Vec::new(&env);
        list.iter().for_each(|a| {
            if a != *delegator {
                new_list.push_back(a.clone());
            }
        });

        // Update the storage with the new list
        env.storage().instance().set(&key, &new_list);
    }
}
