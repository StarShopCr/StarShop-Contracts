use crate::{
    types::{
        Action, Error, Proposal, ProposalRequirements, ProposalStatus, ProposalType, UserLevel,
        VotingConfig, ADMIN_KEY, DEFAULT_CONFIG_KEY, MODERATOR_KEY, PROPOSAL_COUNTER_KEY,
        REFERRAL_KEY, REQUIREMENTS_KEY, TOKEN_KEY,
    },
    utils::get_key_str,
};
use soroban_sdk::{
    symbol_short, token::TokenClient, vec, Address, Bytes, Env, String, Symbol, Vec,
};

/// ProposalManager handles all proposal-related operations in the governance system
/// Including creation, activation, cancellation, and status management of proposals
pub struct ProposalManager;

impl ProposalManager {
    /// Initialize the proposal management system
    /// Sets up initial states for proposal counting and requirements
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `config` - The default voting configuration
    pub fn init(env: &Env, config: &VotingConfig) {
        // Initialize the proposal counter
        env.storage().instance().set(&PROPOSAL_COUNTER_KEY, &0u32);

        // Set default proposal requirements
        let requirements = ProposalRequirements {
            cooldown_period: 86400,  // 24 hours in seconds
            required_stake: 1000,    // Amount of tokens required to stake
            proposal_limit: 5,       // Maximum active proposals per user
            max_voting_power: 10000, // Cap on voting power per user
        };

        // Store default requirements and configuration
        env.storage()
            .instance()
            .set(&REQUIREMENTS_KEY, &requirements);
        env.storage().instance().set(&DEFAULT_CONFIG_KEY, config);

        // Initialize empty lists for each proposal status
        for status in 0..7 {
            let status_key = Self::get_status_key(env, ProposalStatus::from_u32(status));
            env.storage()
                .instance()
                .set::<Symbol, Vec<u32>>(&status_key, &vec![env]);
        }
    }

    /// Check if a proposer is eligible to create a proposal
    /// Verifies user verification, stake, limits, and cooldown periods
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposer` - The address of the proposer
    /// * `proposal_type` - The type of proposal being created
    ///
    /// # Returns
    /// * `Result<bool, Error>` - True if eligible, or an error if not
    pub fn check_proposer_eligibility(
        env: &Env,
        proposer: &Address,
        proposal_type: &ProposalType,
    ) -> Result<bool, Error> {
        // Retrieve system requirements and configuration
        let requirements: ProposalRequirements = env
            .storage()
            .instance()
            .get(&REQUIREMENTS_KEY)
            .ok_or(Error::NotInitialized)?;
        let referral: Address = env
            .storage()
            .instance()
            .get(&REFERRAL_KEY)
            .ok_or(Error::NotInitialized)?;
        let token_address = env
            .storage()
            .instance()
            .get(&TOKEN_KEY)
            .ok_or(Error::NotInitialized)?;
        let token_client = TokenClient::new(env, &token_address);

        // Check KYC/verification status of the proposer
        let args = vec![&env, proposer.to_val()];
        let is_verified: bool =
            env.invoke_contract(&referral, &Symbol::new(&env, "is_user_verified"), args);
        if !is_verified {
            return Err(Error::NotVerified);
        }

        // Check referral level for economic changes - require platinum level
        if matches!(proposal_type, ProposalType::EconomicChange) {
            let args = vec![&env, proposer.to_val()];
            let user_level: UserLevel =
                env.invoke_contract(&referral, &Symbol::new(&env, "get_user_level"), args);
            if !matches!(user_level, UserLevel::Platinum) {
                return Err(Error::InsufficientReferralLevel);
            }
        }

        // Check if proposer has sufficient token stake
        let balance = token_client.balance(proposer);
        if balance < requirements.required_stake {
            return Err(Error::InsufficientStake);
        }

        // Check if proposer has reached their proposal limit
        let draft_proposals = Self::get_proposals_by_status(env, ProposalStatus::Draft);
        let active_proposals = Self::get_proposals_by_status(env, ProposalStatus::Active);
        let proposer_count = draft_proposals
            .iter()
            .chain(active_proposals.iter())
            .filter(|id| {
                let prop = Self::get_proposal(env, *id).unwrap();
                prop.proposer == *proposer
            })
            .count();
        if proposer_count as u32 >= requirements.proposal_limit {
            return Err(Error::ProposalLimitReached);
        }

        // Check if proposer is in the cooldown period
        let latest_proposal_time = Self::get_latest_proposal_time(env, proposer);
        if let Some(time) = latest_proposal_time {
            let current_time = env.ledger().timestamp();
            if current_time < time + requirements.cooldown_period {
                return Err(Error::ProposalInCooldown);
            }
        }

        Ok(true)
    }

    /// Create a new proposal in the system
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposer` - The address creating the proposal
    /// * `title` - The title of the proposal
    /// * `description` - The description of the proposal
    /// * `metadata_hash` - A hash pointing to additional metadata
    /// * `proposal_type` - The type of proposal
    /// * `actions` - The actions to be executed if proposal passes
    /// * `voting_config` - Configuration for the voting process
    ///
    /// # Returns
    /// * `Result<u32, Error>` - The proposal ID or an error
    pub fn create_proposal(
        env: &Env,
        proposer: &Address,
        title: Symbol,
        description: Symbol,
        metadata_hash: String,
        proposal_type: ProposalType,
        actions: Vec<Action>,
        voting_config: VotingConfig,
    ) -> Result<u32, Error> {
        // Require authentication from the proposer
        proposer.require_auth();

        if actions.is_empty() || actions.len() > 5 {
            return Err(Error::InvalidAction);
        }

        // Check if proposer is eligible to create a proposal
        Self::check_proposer_eligibility(env, proposer, &proposal_type)?;

        // Lock stake from the proposer
        let requirements: ProposalRequirements = env
            .storage()
            .instance()
            .get(&REQUIREMENTS_KEY)
            .ok_or(Error::NotInitialized)?;
        let token_address = env
            .storage()
            .instance()
            .get(&TOKEN_KEY)
            .ok_or(Error::NotInitialized)?;
        let token_client = TokenClient::new(env, &token_address);
        let contract_addr = env.current_contract_address();

        // Transfer tokens from proposer to contract as stake
        token_client.transfer(proposer, &contract_addr, &requirements.required_stake);

        // Validate the voting configuration against defaults
        let default_config: VotingConfig = env
            .storage()
            .instance()
            .get(&DEFAULT_CONFIG_KEY)
            .ok_or(Error::NotInitialized)?;

        // Validate voting duration
        if voting_config.duration < default_config.duration / 2
            || voting_config.duration > default_config.duration * 2
        {
            return Err(Error::InvalidVotingPeriod);
        }

        // Additional validations for weighted voting
        if !voting_config.one_address_one_vote {
            // Validate quorum
            if voting_config.quorum < default_config.quorum / 2
                || voting_config.quorum > default_config.quorum * 2
            {
                return Err(Error::InvalidVotingPeriod);
            }

            // Validate threshold
            if voting_config.threshold < default_config.threshold / 2
                || voting_config.threshold > default_config.threshold * 2
            {
                return Err(Error::InvalidVotingPeriod);
            }
        }

        // Get and increment the proposal counter
        let mut proposal_counter: u32 = env
            .storage()
            .instance()
            .get(&PROPOSAL_COUNTER_KEY)
            .unwrap_or(0);
        proposal_counter += 1;

        // Create the new proposal
        let proposal = Proposal {
            id: proposal_counter,
            proposer: proposer.clone(),
            title: title.clone(),
            description,
            metadata_hash,
            proposal_type,
            status: ProposalStatus::Draft,
            created_at: env.ledger().timestamp(),
            activated_at: 0,
            voting_config,
            actions,
        };

        // Store the proposal and update the counter
        let key = Self::get_proposal_key(env, proposal_counter);
        env.storage().instance().set(&key, &proposal);
        env.storage()
            .instance()
            .set(&PROPOSAL_COUNTER_KEY, &proposal_counter);
        Self::add_to_status_list(env, proposal_counter, ProposalStatus::Draft);

        // Emit an event for the proposal creation
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("created")),
            (proposal_counter, proposer, title),
        );

        Ok(proposal_counter)
    }

    /// Activate a proposal to begin the voting period
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `caller` - The address activating the proposal
    /// * `proposal_id` - The ID of the proposal to activate
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn activate_proposal(env: &Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        // Require authentication from the caller
        caller.require_auth();

        // Retrieve the proposal
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;

        // Check that the proposal is in Draft status
        if proposal.status != ProposalStatus::Draft {
            return Err(Error::InvalidProposalStatus);
        }

        // Verify there are moderators in the system
        let moderators: Vec<Address> = env
            .storage()
            .instance()
            .get(&MODERATOR_KEY)
            .unwrap_or(vec![env]);
        if moderators.is_empty() {
            return Err(Error::ModeratorNotFound);
        }

        // Update proposal status to Active
        proposal.status = ProposalStatus::Active;
        proposal.activated_at = env.ledger().timestamp();

        // Save the updated proposal
        env.storage().instance().set(&key, &proposal);

        // Update status lists
        Self::remove_from_status_list(env, proposal_id, ProposalStatus::Draft);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Active);

        // Emit an event for the proposal activation
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("activated")),
            proposal_id,
        );

        Ok(())
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
    pub fn cancel_proposal(env: &Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        // Require authentication from the caller
        caller.require_auth();

        // Retrieve the proposal
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;

        // Save the old status for list management
        let old_status = proposal.status.clone();

        // Check that the proposal is in a cancellable state
        if old_status != ProposalStatus::Draft && old_status != ProposalStatus::Active {
            return Err(Error::InvalidProposalStatus);
        }

        // Retrieve requirements and token information for refunding
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();
        let token_address = env
            .storage()
            .instance()
            .get(&TOKEN_KEY)
            .ok_or(Error::NotInitialized)?;
        let token_client = TokenClient::new(env, &token_address);
        let contract_addr = env.current_contract_address();

        // Refund the stake to the proposer
        token_client.transfer(
            &contract_addr,
            &proposal.proposer,
            &requirements.required_stake,
        );

        // Update proposal status to Canceled
        proposal.status = ProposalStatus::Canceled;
        env.storage().instance().set(&key, &proposal);

        // Update status lists
        Self::remove_from_status_list(env, proposal_id, old_status);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Canceled);

        // Emit an event for the proposal cancellation
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("canceled")),
            proposal_id,
        );

        Ok(())
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
    pub fn veto_proposal(env: &Env, moderator: &Address, proposal_id: u32) -> Result<(), Error> {
        // Require authentication from the moderator
        moderator.require_auth();

        // Check if the caller is a moderator
        if !Self::is_moderator(env, moderator) {
            return Err(Error::Unauthorized);
        }

        // Retrieve the proposal
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;

        // Save the old status for list management
        let old_status = proposal.status.clone();

        // Only passed proposals can be vetoed
        if old_status != ProposalStatus::Passed {
            return Err(Error::InvalidProposalStatus);
        }

        // Retrieve requirements and token information
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();
        let token_address: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = TokenClient::new(env, &token_address);
        let contract_addr: Address = env.current_contract_address();

        // Burn the stake as penalty for vetoed proposal
        token_client.burn(&contract_addr, &requirements.required_stake);

        // Update proposal status to Vetoed
        proposal.status = ProposalStatus::Vetoed;
        env.storage().instance().set(&key, &proposal);

        // Update status lists
        Self::remove_from_status_list(env, proposal_id, old_status);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Vetoed);

        // Emit an event for the vetoed proposal
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("vetoed")),
            (proposal_id, moderator),
        );

        Ok(())
    }

    /// Mark a proposal as passed after voting
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal to mark as passed
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn mark_passed(env: &Env, proposal_id: u32) -> Result<(), Error> {
        // Retrieve the proposal
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;

        // Check that the proposal is in Active status
        if proposal.status != ProposalStatus::Active {
            return Err(Error::InvalidProposalStatus);
        }

        // Update proposal status to Passed
        proposal.status = ProposalStatus::Passed;
        env.storage().instance().set(&key, &proposal);

        // Update status lists
        Self::remove_from_status_list(env, proposal_id, ProposalStatus::Active);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Passed);

        // Emit an event for the passed proposal
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("passed")),
            proposal_id,
        );

        Ok(())
    }

    /// Mark a proposal as rejected after voting
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal to mark as rejected
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn mark_rejected(env: &Env, proposal_id: u32) -> Result<(), Error> {
        // Retrieve the proposal
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;

        // Check that the proposal is in Active status
        if proposal.status != ProposalStatus::Active {
            return Err(Error::InvalidProposalStatus);
        }

        // Retrieve requirements and token information for refunding
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();
        let token_address: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = TokenClient::new(env, &token_address);

        // Refund the stake to the proposer
        let contract_addr: Address = env.current_contract_address();
        token_client.transfer(
            &contract_addr,
            &proposal.proposer,
            &requirements.required_stake,
        );

        // Update proposal status to Rejected
        proposal.status = ProposalStatus::Rejected;
        env.storage().instance().set(&key, &proposal);

        // Update status lists
        Self::remove_from_status_list(env, proposal_id, ProposalStatus::Active);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Rejected);

        // Emit an event for the rejected proposal
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("rejected")),
            proposal_id,
        );

        Ok(())
    }

    /// Mark a proposal as executed after its actions have been completed
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal to mark as executed
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn mark_executed(env: &Env, proposal_id: u32) -> Result<(), Error> {
        // Retrieve the proposal
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;

        // Check that the proposal is in Passed status
        if proposal.status != ProposalStatus::Passed {
            return Err(Error::InvalidProposalStatus);
        }

        // Retrieve requirements and token information for refunding
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();
        let token_address: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = TokenClient::new(env, &token_address);
        let contract_addr: Address = env.current_contract_address();

        // Refund the stake to the proposer for successful execution
        token_client.transfer(
            &contract_addr,
            &proposal.proposer,
            &requirements.required_stake,
        );

        // Update proposal status to Executed
        proposal.status = ProposalStatus::Executed;
        env.storage().instance().set(&key, &proposal);

        // Update status lists
        Self::remove_from_status_list(env, proposal_id, ProposalStatus::Passed);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Executed);

        // Emit an event for the executed proposal
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("executed")),
            proposal_id,
        );

        Ok(())
    }

    /// Add a proposal ID to a status-based list
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `status` - The status list to add to
    fn add_to_status_list(env: &Env, proposal_id: u32, status: ProposalStatus) {
        let key = Self::get_status_key(env, status);
        let mut list: Vec<u32> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env]);

        // Add proposal to the list if not already present
        if !list.contains(&proposal_id) {
            list.push_back(proposal_id);
            env.storage().instance().set(&key, &list);
        }
    }

    /// Remove a proposal ID from a status-based list
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `status` - The status list to remove from
    fn remove_from_status_list(env: &Env, proposal_id: u32, status: ProposalStatus) {
        let key = Self::get_status_key(env, status);
        let list: Vec<u32> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env]);

        // Create a new list without the specified proposal ID
        let mut new_list: Vec<u32> = vec![env];
        for id in list.iter() {
            if id != proposal_id {
                new_list.push_back(id);
            }
        }

        env.storage().instance().set(&key, &new_list);
    }

    /// Get the timestamp of the most recent proposal by a proposer
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposer` - The address of the proposer
    ///
    /// # Returns
    /// * `Option<u64>` - The timestamp of the latest proposal, if any
    pub fn get_latest_proposal_time(env: &Env, proposer: &Address) -> Option<u64> {
        let all_proposals = Self::get_all_proposals(env);
        let mut latest_time: Option<u64> = None;

        // Check all proposals to find the most recent from this proposer
        for id in all_proposals.iter() {
            if let Ok(proposal) = Self::get_proposal(env, id) {
                if proposal.proposer == *proposer {
                    if let Some(time) = latest_time {
                        if proposal.created_at > time {
                            latest_time = Some(proposal.created_at);
                        }
                    } else {
                        latest_time = Some(proposal.created_at);
                    }
                }
            }
        }

        latest_time
    }

    /// Get a proposal by ID
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal to retrieve
    ///
    /// # Returns
    /// * `Result<Proposal, Error>` - The proposal or an error if not found
    pub fn get_proposal(env: &Env, proposal_id: u32) -> Result<Proposal, Error> {
        let key = Self::get_proposal_key(env, proposal_id);
        env.storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)
    }

    /// Get all proposal IDs with a given status
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `status` - The status to filter by
    ///
    /// # Returns
    /// * `Vec<u32>` - A list of proposal IDs with the specified status
    pub fn get_proposals_by_status(env: &Env, status: ProposalStatus) -> Vec<u32> {
        let key = Self::get_status_key(env, status);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env])
    }

    /// Get all proposal IDs in the system
    ///
    /// # Arguments
    /// * `env` - The environment object
    ///
    /// # Returns
    /// * `Vec<u32>` - A list of all proposal IDs
    fn get_all_proposals(env: &Env) -> Vec<u32> {
        // Define all possible statuses
        let statuses = [
            ProposalStatus::Draft,
            ProposalStatus::Active,
            ProposalStatus::Passed,
            ProposalStatus::Rejected,
            ProposalStatus::Executed,
            ProposalStatus::Canceled,
            ProposalStatus::Vetoed,
        ];

        // Collect proposals from all status lists
        let mut all = vec![env];
        for status in statuses.iter() {
            let proposals = Self::get_proposals_by_status(env, status.clone());
            for id in proposals.iter() {
                all.push_back(id);
            }
        }

        all
    }

    /// Generate a storage key for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `Symbol` - A unique storage key for this proposal
    fn get_proposal_key(env: &Env, proposal_id: u32) -> Symbol {
        let key_bytes = Bytes::from_slice(env, b"PROP_");
        get_key_str(env, key_bytes.clone(), proposal_id)
    }

    /// Generate a storage key for a status list
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `status` - The status for the list
    ///
    /// # Returns
    /// * `Symbol` - A unique storage key for this status list
    fn get_status_key(env: &Env, status: ProposalStatus) -> Symbol {
        let key_bytes = Bytes::from_slice(env, b"STAT_");
        let status_num = status as u32;
        get_key_str(env, key_bytes.clone(), status_num)
    }

    /// Check if an address is the admin
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `caller` - The address to check
    ///
    /// # Returns
    /// * `bool` - True if the address is the admin, false otherwise
    pub fn is_admin(env: &Env, caller: &Address) -> bool {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin == *caller
    }

    /// Check if an address is a moderator
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `caller` - The address to check
    ///
    /// # Returns
    /// * `bool` - True if the address is a moderator, false otherwise
    pub fn is_moderator(env: &Env, caller: &Address) -> bool {
        let moderators: Vec<Address> = env
            .storage()
            .instance()
            .get(&MODERATOR_KEY)
            .unwrap_or(vec![env]);
        moderators.contains(caller)
    }
}

impl ProposalStatus {
    /// Convert a u32 to a ProposalStatus enum value
    ///
    /// # Arguments
    /// * `status` - The numeric status value
    ///
    /// # Returns
    /// * `Self` - The corresponding ProposalStatus enum value
    pub fn from_u32(status: u32) -> Self {
        match status {
            0 => ProposalStatus::Draft,
            1 => ProposalStatus::Active,
            2 => ProposalStatus::Passed,
            3 => ProposalStatus::Rejected,
            4 => ProposalStatus::Executed,
            5 => ProposalStatus::Canceled,
            6 => ProposalStatus::Vetoed,
            _ => panic!("Invalid proposal status"),
        }
    }
}
