use crate::types::{
    Action, Error, Proposal, ProposalStatus, AUCTION_KEY, MODERATOR_KEY, REFERRAL_KEY,
    REQUIREMENTS_KEY,
};
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Symbol, Vec};

/// ExecutionEngine is responsible for executing approved proposals
/// Handles the execution logic for different types of governance actions
pub struct ExecutionEngine;

impl ExecutionEngine {
    /// Check if the execution delay period has passed for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal` - The proposal to check
    ///
    /// # Returns
    /// * `Result<bool, Error>` - True if execution delay has passed, false otherwise
    pub fn check_execution_delay(env: &Env, proposal: &Proposal) -> Result<bool, Error> {
        let current_time = env.ledger().timestamp();
        let voting_end_time = proposal.activated_at + proposal.voting_config.duration;
        let execution_time = voting_end_time + proposal.voting_config.execution_delay;

        Ok(current_time >= execution_time)
    }

    /// Execute an approved proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `executor` - The address of the account executing the proposal
    /// * `proposal` - The proposal to execute
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn execute(env: &Env, executor: &Address, proposal: &Proposal) -> Result<(), Error> {
        // Verify authorization from the executor
        executor.require_auth();

        // Ensure the proposal is in the 'Passed' status
        if proposal.status != ProposalStatus::Passed {
            return Err(Error::InvalidProposalStatus);
        }

        // Check if the execution delay has passed
        if !Self::check_execution_delay(env, proposal)? {
            return Err(Error::ExecutionDelayNotMet);
        }

        let proposal_id = proposal.id;
        Self::log_execution_start(env, proposal_id, executor);

        // Execute each action in the proposal and log results
        for (i, action) in proposal.actions.iter().enumerate() {
            Self::log_action_execution(env, proposal_id, i as u32);
            match Self::execute_action(env, &action) {
                Ok(_) => {
                    Self::log_action_result(env, proposal_id, i as u32, true);
                }
                Err(e) => {
                    Self::log_action_result(env, proposal_id, i as u32, false);
                    return Err(e);
                }
            }
        }

        Self::log_execution_complete(env, proposal_id, true);
        Ok(())
    }

    /// Execute a single action from a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `action` - The action to execute
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    fn execute_action(env: &Env, action: &Action) -> Result<(), Error> {
        match action {
            // Update proposal requirements
            Action::UpdateProposalRequirements(requirements) => {
                env.storage()
                    .instance()
                    .set(&REQUIREMENTS_KEY, requirements);
                Ok(())
            }

            // Add a new moderator to the system
            Action::AppointModerator(addr) => {
                let mut moderators: Vec<Address> = env
                    .storage()
                    .instance()
                    .get(&MODERATOR_KEY)
                    .unwrap_or(vec![env]);

                // Check if address is already a moderator
                if moderators.contains(addr) {
                    return Err(Error::AlreadyModerator);
                }

                // Add to moderators list
                moderators.push_back(addr.clone());
                env.storage().instance().set(&MODERATOR_KEY, &moderators);
                Ok(())
            }

            // Remove a moderator from the system
            Action::RemoveModerator(addr) => {
                let moderators: Vec<Address> = env
                    .storage()
                    .instance()
                    .get(&MODERATOR_KEY)
                    .unwrap_or(vec![env]);

                // Check if address is a moderator
                if !moderators.contains(addr) {
                    return Err(Error::ModeratorNotFound);
                }

                // Create new list excluding the specified moderator
                let mut new_moderators = vec![env];
                for m in moderators.iter() {
                    if m != *addr {
                        new_moderators.push_back(m);
                    }
                }

                env.storage()
                    .instance()
                    .set(&MODERATOR_KEY, &new_moderators);
                Ok(())
            }

            // Update reward rates in the referral contract
            Action::UpdateRewardRates(rates) => {
                // Get the referral contract address
                let referral: Address = env
                    .storage()
                    .instance()
                    .get(&REFERRAL_KEY)
                    .ok_or(Error::NotInitialized)?;

                // Call the set_reward_rates function on the referral contract
                env.invoke_contract::<()>(
                    &referral,
                    &Symbol::new(&env, "set_reward_rates"),
                    Vec::from_array(&env, [rates.into_val(env)]),
                );

                Ok(())
            }

            // Update level requirements in the referral contract
            Action::UpdateLevelRequirements(requirements) => {
                // Get the referral contract address
                let referral: Address = env
                    .storage()
                    .instance()
                    .get(&REFERRAL_KEY)
                    .ok_or(Error::NotInitialized)?;

                // Call the set_level_requirements function on the referral contract
                env.invoke_contract::<()>(
                    &referral,
                    &Symbol::new(&env, "set_level_requirements"),
                    Vec::from_array(&env, [requirements.into_val(env)]),
                );

                Ok(())
            }

            // Update auction conditions in the auction contract
            Action::UpdateAuctionConditions(auction_id, conditions) => {
                // Get the auction contract address
                let auction: Address = env
                    .storage()
                    .instance()
                    .get(&AUCTION_KEY)
                    .ok_or(Error::NotInitialized)?;

                // Call the update_conditions function on the auction contract
                env.invoke_contract::<()>(
                    &auction,
                    &Symbol::new(&env, "update_conditions"),
                    Vec::from_array(env, [auction_id.into_val(env), conditions.into_val(env)]),
                );

                Ok(())
            }
        }
    }

    /// Internal helper to log the start of execution
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `executor` - The address of the executor
    fn log_execution_start(env: &Env, proposal_id: u32, executor: &Address) {
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("start")),
            (proposal_id, executor),
        );
    }

    /// Internal helper to log the completion of execution with success or failure
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `success` - Whether execution was successful
    fn log_execution_complete(env: &Env, proposal_id: u32, success: bool) {
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("complete")),
            (proposal_id, success),
        );
    }

    /// Internal helper to log the execution of each action
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `action_index` - The index of the action being executed
    fn log_action_execution(env: &Env, proposal_id: u32, action_index: u32) {
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("execute")),
            (proposal_id, action_index),
        );
    }

    /// Internal helper to log the result of each action
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `action_index` - The index of the action
    /// * `success` - Whether the action execution was successful
    fn log_action_result(env: &Env, proposal_id: u32, action_index: u32, success: bool) {
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("result")),
            (proposal_id, action_index, success),
        );
    }
}
