use crate::execution::ExecutionEngine;
use crate::proposals::ProposalManager;
use crate::types::{
    Action, Error, Proposal, ProposalStatus, ProposalType, VotingConfig, ADMIN_KEY, AUCTION_KEY,
    REFERRAL_KEY, TOKEN_KEY,
};
use crate::voting::VotingSystem;
use crate::weights::WeightCalculator;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Symbol, Vec};

/// Main contract for the governance system
/// Handles all governance-related operations including proposal management,
/// voting, and execution of approved actions
#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    /// Initialize the governance contract with required configurations
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `admin` - The administrator address with special privileges
    /// * `token` - The governance token address used for voting weight
    /// * `referral_contract` - The address of the referral contract for user verification
    /// * `auction_contract` - The address of the auction contract for economic actions
    /// * `config` - The initial voting configuration
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error if initialization fails
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

        // Check if the admin is a valid address and require authorization
        admin.require_auth();

        // Store critical contract addresses
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage().instance().set(&TOKEN_KEY, &token);
        env.storage()
            .instance()
            .set(&REFERRAL_KEY, &referral_contract);
        env.storage()
            .instance()
            .set(&AUCTION_KEY, &auction_contract);

        // Initialize the proposal management system
        ProposalManager::init(&env, &config);

        // Emit an initialization event
        env.events().publish(
            (symbol_short!("govern"), symbol_short!("init")),
            (admin, token, referral_contract, auction_contract),
        );

        Ok(())
    }

    /// Create a new governance proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposer` - The address creating the proposal
    /// * `title` - The title of the proposal
    /// * `description` - The description of the proposal
    /// * `metadata_hash` - A hash pointing to additional metadata
    /// * `proposal_type` - The type of proposal (e.g., governance, technical, economic)
    /// * `actions` - The actions to be executed if proposal passes
    /// * `voting_config` - Configuration for the voting process
    ///
    /// # Returns
    /// * `Result<u32, Error>` - The proposal ID or an error
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

    /// Activate a proposal to begin the voting period
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `caller` - The address activating the proposal (must be a moderator)
    /// * `proposal_id` - The ID of the proposal to activate
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn activate_proposal(env: Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        // Get proposal and check caller permissions
        let _proposal = ProposalManager::get_proposal(&env, proposal_id)?;
        if !ProposalManager::is_moderator(&env, &caller) {
            return Err(Error::Unauthorized);
        }

        // Take a voting power snapshot for the proposal
        WeightCalculator::take_snapshot(&env, proposal_id)?;

        // Activate the proposal
        ProposalManager::activate_proposal(&env, caller, proposal_id)
    }

    /// Cancel a proposal before execution
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `caller` - The address canceling the proposal
    /// * `proposal_id` - The ID of the proposal to cancel
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn cancel_proposal(env: Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        // Get proposal and check caller permissions
        let proposal = ProposalManager::get_proposal(&env, proposal_id)?;

        // Only the proposer, an admin, or a moderator can cancel a proposal
        if proposal.proposer != caller
            && !ProposalManager::is_admin(&env, &caller)
            && !ProposalManager::is_moderator(&env, &caller)
        {
            return Err(Error::Unauthorized);
        }

        // Ensure proposal is in a cancellable state (Draft or Active)
        if proposal.status != ProposalStatus::Draft && proposal.status != ProposalStatus::Active {
            return Err(Error::InvalidProposalStatus);
        }

        // Cancel the proposal
        ProposalManager::cancel_proposal(&env, caller, proposal_id)
    }

    /// Veto a passed proposal by a moderator
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `moderator` - The address of the moderator vetoing the proposal
    /// * `proposal_id` - The ID of the proposal to veto
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn veto_proposal(env: Env, moderator: Address, proposal_id: u32) -> Result<(), Error> {
        ProposalManager::veto_proposal(&env, &moderator, proposal_id)
    }

    /// Mark a proposal as passed (for moderator use)
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `caller` - The address marking the proposal (must be a moderator)
    /// * `proposal_id` - The ID of the proposal to mark as passed
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn mark_passed(env: Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        // Check if caller is a moderator
        if !ProposalManager::is_moderator(&env, &caller) {
            return Err(Error::Unauthorized);
        }

        ProposalManager::mark_passed(&env, proposal_id)
    }

    /// Mark a proposal as rejected (for moderator use)
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `caller` - The address marking the proposal (must be a moderator)
    /// * `proposal_id` - The ID of the proposal to mark as rejected
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn mark_rejected(env: Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        // Check if caller is a moderator
        if !ProposalManager::is_moderator(&env, &caller) {
            return Err(Error::Unauthorized);
        }

        ProposalManager::mark_rejected(&env, proposal_id)
    }

    /// Mark a proposal as executed (for moderator use)
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `caller` - The address marking the proposal (must be a moderator)
    /// * `proposal_id` - The ID of the proposal to mark as executed
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn mark_executed(env: Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        // Check if caller is a moderator
        if !ProposalManager::is_moderator(&env, &caller) {
            return Err(Error::Unauthorized);
        }

        ProposalManager::mark_executed(&env, proposal_id)
    }

    /// Cast a vote on a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `voter` - The address of the voter
    /// * `proposal_id` - The ID of the proposal being voted on
    /// * `support` - Whether the vote is in support (true) or against (false)
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u32,
        support: bool,
    ) -> Result<(), Error> {
        // Get proposal and verify it's active
        let proposal = ProposalManager::get_proposal(&env, proposal_id)?;
        if proposal.status != ProposalStatus::Active {
            return Err(Error::ProposalNotActive);
        }

        // Calculate voting weight based on configuration
        let weight = if proposal.voting_config.one_address_one_vote {
            // One address, one vote mode
            1
        } else {
            // Weighted voting based on token holdings and delegations
            WeightCalculator::get_weight(&env, &voter, proposal_id)?
        };

        // Ensure voter has voting power
        if weight <= 0 {
            return Err(Error::NoVotingPower);
        }

        // Check if voting has ended and finalize if needed
        if VotingSystem::check_voting_ended(&env, proposal_id, &proposal.voting_config)? {
            let passed = VotingSystem::tally_votes(&env, proposal_id, &proposal.voting_config)?;
            if passed {
                ProposalManager::mark_passed(&env, proposal_id)?;
            } else {
                ProposalManager::mark_rejected(&env, proposal_id)?;
            }
        }

        // Cast the vote
        VotingSystem::cast_vote(&env, proposal_id, &voter, support, weight)?;
        Ok(())
    }

    /// Take a voting power snapshot for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn take_snapshot(env: Env, proposal_id: u32) -> Result<(), Error> {
        WeightCalculator::take_snapshot(&env, proposal_id)
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
    pub fn delegate_vote(env: Env, delegator: Address, delegatee: Address) -> Result<(), Error> {
        WeightCalculator::delegate(&env, &delegator, &delegatee)
    }

    /// Get the voting weight of an address for a specific proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `voter` - The address to check
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `Result<i128, Error>` - The voting weight or an error
    pub fn get_vote_weight(env: Env, voter: Address, proposal_id: u32) -> Result<i128, Error> {
        WeightCalculator::get_weight(&env, &voter, proposal_id)
    }

    /// Execute a passed proposal's actions
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `executor` - The address executing the proposal
    /// * `proposal_id` - The ID of the proposal to execute
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn execute_proposal(env: Env, executor: Address, proposal_id: u32) -> Result<(), Error> {
        // Check if executor is authorized (admin or moderator)
        if !ProposalManager::is_admin(&env, &executor)
            && !ProposalManager::is_moderator(&env, &executor)
        {
            return Err(Error::Unauthorized);
        }

        // Get the proposal
        let proposal = ProposalManager::get_proposal(&env, proposal_id)?;

        // Execute all actions in the proposal
        ExecutionEngine::execute(&env, &executor, &proposal)?;

        // Mark the proposal as executed
        ProposalManager::mark_executed(&env, proposal_id)
    }

    /// Get a proposal by ID
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal to retrieve
    ///
    /// # Returns
    /// * `Result<Proposal, Error>` - The proposal or an error if not found
    pub fn get_proposal(env: Env, proposal_id: u32) -> Result<Proposal, Error> {
        ProposalManager::get_proposal(&env, proposal_id)
    }

    /// Get all active proposals
    ///
    /// # Arguments
    /// * `env` - The environment object
    ///
    /// # Returns
    /// * `Vec<u32>` - A list of active proposal IDs
    pub fn get_active_proposals(env: Env) -> Vec<u32> {
        ProposalManager::get_proposals_by_status(&env, ProposalStatus::Active)
    }

    /// Get all proposals that are passed and ready for execution
    ///
    /// # Arguments
    /// * `env` - The environment object
    ///
    /// # Returns
    /// * `Vec<u32>` - A list of executable proposal IDs
    pub fn get_executable_proposals(env: Env) -> Vec<u32> {
        ProposalManager::get_proposals_by_status(&env, ProposalStatus::Passed)
    }

    /// Get total number of voters
    pub fn get_proposal_voters_count(env: Env, proposal_id: u32) -> u128 {
        VotingSystem::get_voter_count(&env, proposal_id)
    }
}
