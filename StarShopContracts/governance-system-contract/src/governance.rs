use crate::execution::ExecutionEngine;
use crate::proposals::ProposalManager;
use crate::types::{
    Action, Error, Proposal, ProposalStatus, ProposalType, VotingConfig, ADMIN_KEY, AUCTION_KEY,
    REFERRAL_KEY, TOKEN_KEY,
};
use crate::voting::VotingSystem;
use crate::weights::WeightCalculator;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Symbol, Vec};

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn initialize(
        env: Env,
        admin: Address,
        token: Address,
        referral_contract: Address,
        auction_contract: Address,
        config: VotingConfig,
    ) -> Result<(), Error> {
        // Prevent re-initialization
        if env.storage().instance().has(&ADMIN_KEY) {
            return Err(Error::AlreadyInitialized);
        }

        // Check if the admin is a valid address
        admin.require_auth();

        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&TOKEN_KEY, &token);
        env.storage()
            .instance()
            .set(&REFERRAL_KEY, &referral_contract);
        env.storage()
            .instance()
            .set(&AUCTION_KEY, &auction_contract);

        ProposalManager::init(&env, &config);

        env.events().publish(
            (symbol_short!("govern"), symbol_short!("init")),
            (admin, token, referral_contract, auction_contract),
        );

        Ok(())
    }

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: Symbol,
        description: Symbol,
        metadata_hash: String,
        proposal_type: ProposalType,
        actions: Vec<Action>,
        voting_config: VotingConfig,
    ) -> Result<u32, Error> {
        ProposalManager::create_proposal(
            &env,
            &proposer,
            title,
            description,
            metadata_hash,
            proposal_type,
            actions,
            voting_config,
        )
    }

    pub fn activate_proposal(env: Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        let _proposal = ProposalManager::get_proposal(&env, proposal_id)?;
        if !ProposalManager::is_moderator(&env, &caller) {
            return Err(Error::Unauthorized);
        }
        WeightCalculator::take_snapshot(&env, proposal_id)?;
        ProposalManager::activate_proposal(&env, caller, proposal_id)
    }

    pub fn cancel_proposal(env: Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        let proposal = ProposalManager::get_proposal(&env, proposal_id)?;
        if proposal.proposer != caller
            && !ProposalManager::is_admin(&env, &caller)
            && !ProposalManager::is_moderator(&env, &caller)
        {
            return Err(Error::Unauthorized);
        }
        if proposal.status != ProposalStatus::Draft && proposal.status != ProposalStatus::Active {
            return Err(Error::InvalidProposalStatus);
        }
        ProposalManager::cancel_proposal(&env, caller, proposal_id)
    }

    pub fn veto_proposal(env: Env, moderator: Address, proposal_id: u32) -> Result<(), Error> {
        ProposalManager::veto_proposal(&env, &moderator, proposal_id)
    }

    pub fn mark_passed(
        env: Env,
        caller: Address,
        proposal_id: u32,
    ) -> Result<(), Error> {
        if !ProposalManager::is_moderator(&env, &caller) {
            return Err(Error::Unauthorized);
        }
        ProposalManager::mark_passed(&env, proposal_id)
    }

    pub fn mark_rejected(
        env: Env,
        caller: Address,
        proposal_id: u32,
    ) -> Result<(), Error> {
        if !ProposalManager::is_moderator(&env, &caller) {
            return Err(Error::Unauthorized);
        }
        ProposalManager::mark_rejected(&env, proposal_id)
    }

    pub fn mark_executed(
        env: Env,
        caller: Address,
        proposal_id: u32,
    ) -> Result<(), Error> {
        if !ProposalManager::is_moderator(&env, &caller) {
            return Err(Error::Unauthorized);
        }
        ProposalManager::mark_executed(&env, proposal_id)
    }

    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        support: bool,
    ) -> Result<(), Error> {
        let proposal = ProposalManager::get_proposal(&env, proposal_id)?;
        if proposal.status != ProposalStatus::Active {
            return Err(Error::ProposalNotActive);
        }
        let weight = if proposal.voting_config.one_address_one_vote {
            1
        } else {
            WeightCalculator::get_weight(&env, &voter, proposal_id)?
        };
        if weight <= 0 {
            return Err(Error::NoVotingPower);
        }
        VotingSystem::cast_vote(&env, proposal_id, &voter, support, weight)?;
        if VotingSystem::check_voting_ended(&env, proposal_id, &proposal.voting_config)? {
            let passed = VotingSystem::tally_votes(&env, proposal_id, &proposal.voting_config)?;
            if passed {
                ProposalManager::mark_passed(&env, proposal_id)?;
            } else {
                ProposalManager::mark_rejected(&env, proposal_id)?;
            }
        }
        Ok(())
    }

    pub fn delegate_vote(env: Env, delegator: Address, delegatee: Address) -> Result<(), Error> {
        WeightCalculator::delegate(&env, &delegator, &delegatee)
    }

    pub fn get_vote_weight(env: Env, voter: Address, proposal_id: u32) -> Result<i128, Error> {
        WeightCalculator::get_weight(&env, &voter, proposal_id)
    }

    pub fn execute_proposal(env: Env, executor: Address, proposal_id: u32) -> Result<(), Error> {
        let proposal = ProposalManager::get_proposal(&env, proposal_id)?;
        if proposal.status != ProposalStatus::Passed {
            return Err(Error::ProposalNotExecutable);
        }
        if !ExecutionEngine::check_execution_delay(&env, proposal_id, &proposal)? {
            return Err(Error::ExecutionDelayNotMet);
        }
        ExecutionEngine::execute(&env, &executor, proposal_id, &proposal)?;
        ProposalManager::mark_executed(&env, proposal_id)
    }

    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, Error> {
        ProposalManager::get_proposal(&env, proposal_id)
    }

    pub fn get_active_proposals(env: Env) -> Vec<u32> {
        ProposalManager::get_proposals_by_status(&env, ProposalStatus::Active)
    }

    pub fn get_executable_proposals(env: Env) -> Vec<u32> {
        ProposalManager::get_proposals_by_status(&env, ProposalStatus::Passed)
    }
}
