use crate::proposals::ProposalManager;
use crate::types::{
    Action, Error, Proposal, ProposalStatus, AUCTION_KEY, MODERATOR_KEY, REFERRAL_KEY,
    REQUIREMENTS_KEY,
};
use soroban_sdk::{symbol_short, vec, Address, Env, IntoVal, Symbol, Vec};

pub struct ExecutionEngine;

impl ExecutionEngine {
    pub fn check_execution_delay(
        env: &Env,
        _proposal_id: u32,
        proposal: &Proposal,
    ) -> Result<bool, Error> {
        let current_time = env.ledger().timestamp();
        if proposal.status != ProposalStatus::Passed {
            return Ok(false);
        }
        let voting_end_time = proposal.activated_at + proposal.voting_config.duration;
        let execution_time = voting_end_time + proposal.voting_config.execution_delay;
        Ok(current_time >= execution_time)
    }

    pub fn execute(
        env: &Env,
        executor: &Address,
        proposal_id: u32,
        proposal: &Proposal,
    ) -> Result<(), Error> {
        executor.require_auth();
        if !ProposalManager::is_admin(env, executor)
            && !ProposalManager::is_moderator(env, executor)
        {
            return Err(Error::Unauthorized);
        }
        Self::log_execution_start(env, proposal_id, executor);
        for (i, action) in proposal.actions.iter().enumerate() {
            Self::log_action_execution(env, proposal_id, i as u32);
            match Self::execute_action(env, &action) {
                Ok(_) => Self::log_action_result(env, proposal_id, i as u32, true),
                Err(e) => {
                    Self::log_action_result(env, proposal_id, i as u32, false);
                    return Err(e);
                }
            }
        }
        Self::log_execution_complete(env, proposal_id, true);
        Ok(())
    }

    fn execute_action(env: &Env, action: &Action) -> Result<(), Error> {
        match action {
            Action::UpdateProposalRequirements(requirements) => {
                env.storage()
                    .instance()
                    .set(&REQUIREMENTS_KEY, requirements);
                Ok(())
            }

            Action::AppointModerator(addr) => {
                let mut moderators: Vec<Address> = env
                    .storage()
                    .instance()
                    .get(&MODERATOR_KEY)
                    .unwrap_or(vec![env]);
                if moderators.contains(addr) {
                    return Err(Error::AlreadyModerator);
                }
                moderators.push_back(addr.clone());
                env.storage().instance().set(&MODERATOR_KEY, &moderators);
                Ok(())
            }

            Action::RemoveModerator(addr) => {
                let moderators: Vec<Address> = env
                    .storage()
                    .instance()
                    .get(&MODERATOR_KEY)
                    .unwrap_or(vec![env]);
                if !moderators.contains(addr) {
                    return Err(Error::ModeratorNotFound);
                }
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

            Action::UpdateRewardRates(rates) => {
                let referral: Address = env
                    .storage()
                    .instance()
                    .get(&REFERRAL_KEY)
                    .ok_or(Error::NotInitialized)?;

                env.invoke_contract::<()>(
                    &referral,
                    &Symbol::new(&env, "set_reward_rates"),
                    Vec::from_array(&env, [rates.into_val(env)]),
                );

                Ok(())
            }

            Action::UpdateLevelRequirements(requirements) => {
                let referral: Address = env
                    .storage()
                    .instance()
                    .get(&REFERRAL_KEY)
                    .ok_or(Error::NotInitialized)?;

                env.invoke_contract::<()>(
                    &referral,
                    &Symbol::new(&env, "set_level_requirements"),
                    Vec::from_array(&env, [requirements.into_val(env)]),
                );

                Ok(())
            }

            Action::UpdateAuctionConditions(auction_id, conditions) => {
                let auction: Address = env
                    .storage()
                    .instance()
                    .get(&AUCTION_KEY)
                    .ok_or(Error::NotInitialized)?;

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
    fn log_execution_start(env: &Env, proposal_id: u32, executor: &Address) {
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("start")),
            (proposal_id, executor),
        );
    }

    /// Internal helper to log the completion of execution with success or failure
    fn log_execution_complete(env: &Env, proposal_id: u32, success: bool) {
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("complete")),
            (proposal_id, success),
        );
    }

    /// Internal helper to log the execution of each action
    fn log_action_execution(env: &Env, proposal_id: u32, action_index: u32) {
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("execute")),
            (proposal_id, action_index),
        );
    }

    /// Internal helper to log the result of each action
    fn log_action_result(env: &Env, proposal_id: u32, action_index: u32, success: bool) {
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("result")),
            (proposal_id, action_index, success),
        );
    }
}
