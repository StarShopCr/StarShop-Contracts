use crate::proposals::ProposalManager;
use crate::types::{Error, ProposalType, UserLevel, Vote, VotingConfig, REFERRAL_KEY};
use crate::utils::{get_key_str, get_governance_op_key};
use soroban_sdk::{symbol_short, vec, Address, Bytes, Env, IntoVal, Symbol, Vec};

/// VotingSystem handles all vote-related operations for the governance system
/// Including casting votes, tallying results, and checking voting status
pub struct VotingSystem;

impl VotingSystem {
    /// Cast a vote for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal being voted on
    /// * `voter` - The address of the voter
    /// * `support` - Whether the vote is in support (true) or against (false)
    /// * `weight` - The voting weight of the voter
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or an error
    pub fn cast_vote(
        env: &Env,
        proposal_id: u32,
        voter: &Address,
        support: bool,
        weight: i128,
    ) -> Result<(), Error> {
        // Require authentication from the voter
        voter.require_auth();

        // Retrieve the proposal
        let proposal = ProposalManager::get_proposal(env, proposal_id)?;

        // Retrieve the referral contract address
        let referral: Address = env
            .storage()
            .instance()
            .get(&REFERRAL_KEY)
            .ok_or(Error::NotInitialized)?;

        // Check if the voter is verified
        let is_verified: bool = env.invoke_contract(
            &referral,
            &Symbol::new(&env, "is_user_verified"),
            Vec::from_array(env, [voter.into_val(env)]),
        );
        if !is_verified {
            return Err(Error::NotVerified);
        }

        // Check referral level against action type - economic changes require platinum level
        if matches!(proposal.proposal_type, ProposalType::EconomicChange) {
            let args = vec![&env, voter.to_val()];
            let user_level: UserLevel =
                env.invoke_contract(&referral, &Symbol::new(&env, "get_user_level"), args);
            if !matches!(user_level, UserLevel::Platinum) {
                return Err(Error::InsufficientReferralLevel);
            }
        }

        // Check if the voter has already voted
        let key = Self::get_vote_key(env, proposal_id, voter);
        if env.storage().instance().has(&key) {
            return Err(Error::AlreadyVoted);
        }
        
        // Create a new vote and store it
        let vote = Vote {
            voter: voter.clone(),
            support,
            weight,
            timestamp: env.ledger().timestamp(),
        };
        env.storage().instance().set(&key, &vote);

        // Update vote totals
        Self::update_vote_totals(env, proposal_id, support, vote.weight);

        // Publish an event for the vote
        env.events().publish(
            (symbol_short!("vote"), symbol_short!("cast")),
            (proposal_id, voter, support, vote.weight),
        );

        Ok(())
    }

    /// Update vote totals for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `support` - Whether the vote is in support or against
    /// * `weight` - The weight of the vote
    fn update_vote_totals(env: &Env, proposal_id: u32, support: bool, weight: i128) {
        // Retrieve keys for vote totals
        let key_for = Self::get_vote_totals_key(env, proposal_id, true);
        let key_against = Self::get_vote_totals_key(env, proposal_id, false);
        let key_total = Self::get_vote_total_key(env, proposal_id);
        let key_voter_count = Self::get_voter_count_key(env, proposal_id);

        // Initialize vote totals if not present
        if !env.storage().instance().has(&key_for) {
            env.storage().instance().set(&key_for, &0u128);
        }
        if !env.storage().instance().has(&key_against) {
            env.storage().instance().set(&key_against, &0u128);
        }
        if !env.storage().instance().has(&key_total) {
            env.storage().instance().set(&key_total, &0u128);
        }
        if !env.storage().instance().has(&key_voter_count) {
            env.storage().instance().set(&key_voter_count, &0u128);
        }

        // Retrieve current vote totals
        let mut for_votes: u128 = env.storage().instance().get(&key_for).unwrap_or(0);
        let mut against_votes: u128 = env.storage().instance().get(&key_against).unwrap_or(0);
        let mut total_votes: u128 = env.storage().instance().get(&key_total).unwrap_or(0);
        let mut voter_count: u128 = env.storage().instance().get(&key_voter_count).unwrap_or(0);

        // Update vote totals based on support
        if support {
            for_votes += weight as u128;
        } else {
            against_votes += weight as u128;
        }
        total_votes += weight as u128;
        voter_count += 1;

        // Store updated vote totals
        env.storage().instance().set(&key_for, &for_votes);
        env.storage().instance().set(&key_against, &against_votes);
        env.storage().instance().set(&key_total, &total_votes);
        env.storage().instance().set(&key_voter_count, &voter_count);
    }

    /// Check if voting has ended for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `config` - The voting configuration
    ///
    /// # Returns
    /// * `Result<bool, Error>` - True if voting has ended, false otherwise, or an error
    pub fn check_voting_ended(
        env: &Env,
        proposal_id: u32,
        config: &VotingConfig,
    ) -> Result<bool, Error> {
        // Retrieve the proposal
        let proposal = ProposalManager::get_proposal(env, proposal_id)?;

        // Check if the proposal has been activated
        if proposal.activated_at == 0 {
            return Ok(false);
        }

        // Check if the voting period has ended based on time
        let current_time = env.ledger().timestamp();
        if current_time >= proposal.activated_at + config.duration {
            return Ok(true);
        }

        // Handle one-address-one-vote mode
        if config.one_address_one_vote {
            let voter_count = Self::get_voter_count(env, proposal_id);
            let total_voters = Self::get_total_voters(env)?;

            // Check if quorum has been reached (voter count as percentage * 10000)
            if voter_count * 10000 / total_voters >= config.quorum {
                let for_votes = Self::get_for_votes(env, proposal_id);
                let total_votes = Self::get_total_votes(env, proposal_id);

                // Check if support threshold has been reached
                if for_votes * 10000 / total_votes >= config.threshold {
                    return Ok(true);
                }

                // Check if opposition threshold makes passing impossible
                if (total_votes - for_votes) * 10000 / total_votes > (10000 - config.threshold) {
                    return Ok(true);
                }
            }
        } else {
            // Handle weighted voting mode
            let total_votes = Self::get_total_votes(env, proposal_id);
            let total_voting_power = Self::get_total_voting_power(env, proposal_id);

            // Check if quorum has been reached (vote weight as percentage * 10000)
            if total_votes * 10000 / total_voting_power as u128 >= config.quorum {
                let for_votes = Self::get_for_votes(env, proposal_id);
                let against_votes = Self::get_against_votes(env, proposal_id);

                // Check if support threshold has been reached
                if for_votes * 10000 / total_votes >= config.threshold {
                    return Ok(true);
                }

                // Check if opposition threshold makes passing impossible
                if against_votes * 10000 / total_votes > (10000 - config.threshold) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Tally votes for a proposal to determine if it passes
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `config` - The voting configuration
    ///
    /// # Returns
    /// * `Result<bool, Error>` - True if the proposal passes, false otherwise, or an error
    pub fn tally_votes(env: &Env, proposal_id: u32, config: &VotingConfig) -> Result<bool, Error> {
        if config.one_address_one_vote {
            // Check quorum in one-address-one-vote mode
            let voter_count = Self::get_voter_count(env, proposal_id);
            let total_voters = Self::get_total_voters(env)?;
            if voter_count * 10000 / total_voters < config.quorum {
                return Ok(false);
            }

            // Check threshold
            let for_votes = Self::get_for_votes(env, proposal_id);
            let total_votes = Self::get_total_votes(env, proposal_id);
            if for_votes * 10000 / total_votes >= config.threshold {
                return Ok(true);
            }
        } else {
            // Check quorum and threshold in weighted voting mode
            let for_votes = Self::get_for_votes(env, proposal_id);
            let total_votes = Self::get_total_votes(env, proposal_id);
            let total_voting_power = Self::get_total_voting_power(env, proposal_id);

            if (total_votes * 10000 / total_voting_power as u128) < config.quorum {
                return Ok(false);
            }
            if for_votes * 10000 / total_votes >= config.threshold {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Retrieve the total votes in favor of a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `u128` - The total votes in favor
    pub fn get_for_votes(env: &Env, proposal_id: u32) -> u128 {
        let key = Self::get_vote_totals_key(env, proposal_id, true);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    /// Retrieve the total votes against a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `u128` - The total votes against
    pub fn get_against_votes(env: &Env, proposal_id: u32) -> u128 {
        let key = Self::get_vote_totals_key(env, proposal_id, false);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    /// Retrieve the total number of votes for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `u128` - The total number of votes
    pub fn get_total_votes(env: &Env, proposal_id: u32) -> u128 {
        let key = Self::get_vote_total_key(env, proposal_id);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    /// Retrieve the total number of voters for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `u128` - The total number of voters
    pub fn get_voter_count(env: &Env, proposal_id: u32) -> u128 {
        let key = Self::get_voter_count_key(env, proposal_id);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    /// Get the total number of eligible voters in the system
    ///
    /// # Arguments
    /// * `env` - The environment object
    ///
    /// # Returns
    /// * `Result<u128, Error>` - The total number of voters or an error
    pub fn get_total_voters(env: &Env) -> Result<u128, Error> {
        let referral: Address = env
            .storage()
            .instance()
            .get(&REFERRAL_KEY)
            .ok_or(Error::NotInitialized)?;
        let total_users: u32 =
            env.invoke_contract(&referral, &Symbol::new(&env, "get_total_users"), vec![&env]);
        Ok(total_users as u128)
    }

    /// Get the total voting power available for a proposal
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `i128` - The total voting power (defaults to 10000 if not set)
    pub fn get_total_voting_power(env: &Env, proposal_id: u32) -> i128 {
        let key_bytes = Bytes::from_slice(env, b"VOTE_POW_");
        let key = get_key_str(env, key_bytes.clone(), proposal_id);
        env.storage().instance().get(&key).unwrap_or(10000)
    }

    /// Generate a unique key for a voter's vote
    /// Uses a hash of the voter's address to create a unique identifier
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `voter` - The address of the voter
    ///
    /// # Returns
    /// * `Symbol` - A unique storage key for this vote
    fn get_vote_key(env: &Env, proposal_id: u32, voter: &Address) -> Symbol {
        let key_bytes = Bytes::from_slice(env, b"VOTER_");
        get_governance_op_key(env, key_bytes, proposal_id, voter)
    }

    /// Generate a key for vote totals (for or against)
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    /// * `support` - Whether the count is for support (true) or against (false)
    ///
    /// # Returns
    /// * `Symbol` - A storage key for the vote totals
    fn get_vote_totals_key(env: &Env, proposal_id: u32, support: bool) -> Symbol {
        // Generate a key based on the proposal ID and whether it's for or against
        let suffix = if support { b"FOR" } else { b"AGT" };
        let mut key_bytes = Bytes::from_slice(env, b"VOTE_");
        key_bytes.extend_from_array(suffix);
        key_bytes.extend_from_array(b"_");

        get_key_str(env, key_bytes, proposal_id)
    }

    /// Generate a key for the total number of votes
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `Symbol` - A storage key for the total votes
    fn get_vote_total_key(env: &Env, proposal_id: u32) -> Symbol {
        let key_bytes = Bytes::from_slice(env, b"VOTE_TOTAL_");
        get_key_str(env, key_bytes, proposal_id)
    }

    /// Generate a key for the total number of voters
    ///
    /// # Arguments
    /// * `env` - The environment object
    /// * `proposal_id` - The ID of the proposal
    ///
    /// # Returns
    /// * `Symbol` - A storage key for the voter count
    fn get_voter_count_key(env: &Env, proposal_id: u32) -> Symbol {
        let key_bytes = Bytes::from_slice(env, b"VOTE_COUNT_");
        get_key_str(env, key_bytes, proposal_id)
    }
}
