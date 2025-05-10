use crate::types::{
    Action, Error, Proposal, ProposalRequirements, ProposalStatus, ProposalType, UserLevel,
    VotingConfig, ADMIN_KEY, DEFAULT_CONFIG_KEY, MODERATOR_KEY, PROPOSAL_COUNTER_KEY, REFERRAL_KEY,
    REQUIREMENTS_KEY, TOKEN_KEY,
};
use soroban_sdk::{log, symbol_short, token::TokenClient, vec, Address, Env, String, Symbol, Vec};

pub struct ProposalManager;

impl ProposalManager {
    pub fn init(env: &Env, config: &VotingConfig) {
        env.storage().instance().set(&PROPOSAL_COUNTER_KEY, &0u32);

        let requirements = ProposalRequirements {
            cooldown_period: 86400,
            required_stake: 1000,
            proposal_limit: 5,
            max_voting_power: 10000,
        };
        env.storage()
            .instance()
            .set(&REQUIREMENTS_KEY, &requirements);
        env.storage().instance().set(&DEFAULT_CONFIG_KEY, config);
        for status in 0..7 {
            let status_key = Self::get_status_key(env, ProposalStatus::from_u32(status));
            env.storage()
                .instance()
                .set::<Symbol, Vec<u32>>(&status_key, &vec![env]);
        }
    }

    pub fn check_proposer_eligibility(
        env: &Env,
        proposer: &Address,
        proposal_type: &ProposalType,
    ) -> Result<bool, Error> {
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

        // Check KYC/verification
        let args = vec![&env, proposer.to_val()];
        let is_verified: bool =
            env.invoke_contract(&referral, &Symbol::new(&env, "is_user_verified"), args);
        if !is_verified {
            return Err(Error::NotVerified);
        }

        // Check referral level for economic changes
        if matches!(proposal_type, ProposalType::EconomicChange) {
            let args = vec![&env, proposer.to_val()];
            let user_level: UserLevel =
                env.invoke_contract(&referral, &Symbol::new(&env, "get_user_level"), args);
            if !matches!(user_level, UserLevel::Platinum) {
                return Err(Error::InsufficientReferralLevel);
            }
        }

        // Check stake
        let balance = token_client.balance(proposer);
        if balance < requirements.required_stake {
            return Err(Error::InsufficientStake);
        }

        // Check proposal limit
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

        // Check cooldown
        let latest_proposal_time = Self::get_latest_proposal_time(env, proposer);
        if let Some(time) = latest_proposal_time {
            let current_time = env.ledger().timestamp();
            if current_time < time + requirements.cooldown_period {
                return Err(Error::ProposalInCooldown);
            }
        }

        Ok(true)
    }

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
        proposer.require_auth();

        if title.to_string().len() > 100
            || description.to_string().len() > 1000
            || metadata_hash.len() > 64
        {
            return Err(Error::InvalidProposalStatus);
        }
        if actions.is_empty() || actions.len() > 5 {
            return Err(Error::InvalidAction);
        }
        Self::check_proposer_eligibility(env, proposer, &proposal_type)?;

        // Lock stake
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
        token_client.transfer(proposer, &contract_addr, &requirements.required_stake);

        // Validate voting config
        let default_config: VotingConfig = env
            .storage()
            .instance()
            .get(&DEFAULT_CONFIG_KEY)
            .ok_or(Error::NotInitialized)?;
        if voting_config.duration < default_config.duration / 2
            || voting_config.duration > default_config.duration * 2
        {
            return Err(Error::InvalidVotingPeriod);
        }

        if !voting_config.one_address_one_vote {
            if voting_config.quorum < default_config.quorum / 2
                || voting_config.quorum > default_config.quorum * 2
            {
                return Err(Error::InvalidVotingPeriod);
            }

            if voting_config.threshold < default_config.threshold / 2
                || voting_config.threshold > default_config.threshold * 2
            {
                return Err(Error::InvalidVotingPeriod);
            }
        }

        let mut proposal_counter: u32 = env
            .storage()
            .instance()
            .get(&PROPOSAL_COUNTER_KEY)
            .unwrap_or(0);
        proposal_counter += 1;

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

        let key = Self::get_proposal_key(env, proposal_counter);
        env.storage().instance().set(&key, &proposal);
        env.storage()
            .instance()
            .set(&PROPOSAL_COUNTER_KEY, &proposal_counter);
        Self::add_to_status_list(env, proposal_counter, ProposalStatus::Draft);

        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("created")),
            (proposal_counter, proposer, title),
        );

        Ok(proposal_counter)
    }

    pub fn activate_proposal(env: &Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        caller.require_auth();
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;
        if proposal.status != ProposalStatus::Draft {
            return Err(Error::InvalidProposalStatus);
        }
        let moderators: Vec<Address> = env
            .storage()
            .instance()
            .get(&MODERATOR_KEY)
            .unwrap_or(vec![env]);
        if moderators.is_empty() {
            return Err(Error::ModeratorNotFound);
        }

        proposal.status = ProposalStatus::Active;
        proposal.activated_at = env.ledger().timestamp();

        env.storage().instance().set(&key, &proposal);
        Self::remove_from_status_list(env, proposal_id, ProposalStatus::Draft);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Active);

        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("activated")),
            proposal_id,
        );
        Ok(())
    }

    pub fn cancel_proposal(env: &Env, caller: Address, proposal_id: u32) -> Result<(), Error> {
        caller.require_auth();
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;
        let old_status = proposal.status.clone();
        if old_status != ProposalStatus::Draft && old_status != ProposalStatus::Active {
            return Err(Error::InvalidProposalStatus);
        }
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();
        let token_address = env
            .storage()
            .instance()
            .get(&TOKEN_KEY)
            .ok_or(Error::NotInitialized)?;
        let token_client = TokenClient::new(env, &token_address);
        let contract_addr = env.current_contract_address();
        token_client.transfer(
            &contract_addr,
            &proposal.proposer,
            &requirements.required_stake,
        );
        proposal.status = ProposalStatus::Canceled;
        env.storage().instance().set(&key, &proposal);
        Self::remove_from_status_list(env, proposal_id, old_status);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Canceled);

        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("canceled")),
            proposal_id,
        );
        Ok(())
    }

    pub fn veto_proposal(env: &Env, moderator: &Address, proposal_id: u32) -> Result<(), Error> {
        moderator.require_auth();
        if !Self::is_moderator(env, moderator) {
            return Err(Error::Unauthorized);
        }
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;
        let old_status = proposal.status.clone();
        if old_status != ProposalStatus::Passed {
            return Err(Error::InvalidProposalStatus);
        }
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();
        let token_address: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = TokenClient::new(env, &token_address);
        let contract_addr: Address = env.current_contract_address();
        token_client.burn(&contract_addr, &requirements.required_stake);
        proposal.status = ProposalStatus::Vetoed;
        env.storage().instance().set(&key, &proposal);
        Self::remove_from_status_list(env, proposal_id, old_status);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Vetoed);
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("vetoed")),
            (proposal_id, moderator),
        );
        Ok(())
    }

    pub fn mark_passed(env: &Env, proposal_id: u32) -> Result<(), Error> {
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;
        if proposal.status != ProposalStatus::Active {
            return Err(Error::InvalidProposalStatus);
        }
        proposal.status = ProposalStatus::Passed;
        env.storage().instance().set(&key, &proposal);
        Self::remove_from_status_list(env, proposal_id, ProposalStatus::Active);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Passed);
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("passed")),
            proposal_id,
        );
        Ok(())
    }

    pub fn mark_rejected(env: &Env, proposal_id: u32) -> Result<(), Error> {
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;
        if proposal.status != ProposalStatus::Active {
            return Err(Error::InvalidProposalStatus);
        }
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
        proposal.status = ProposalStatus::Rejected;
        env.storage().instance().set(&key, &proposal);
        Self::remove_from_status_list(env, proposal_id, ProposalStatus::Active);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Rejected);
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("rejected")),
            proposal_id,
        );
        Ok(())
    }

    pub fn mark_executed(env: &Env, proposal_id: u32) -> Result<(), Error> {
        let key = Self::get_proposal_key(env, proposal_id);
        let mut proposal: Proposal = env
            .storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)?;
        if proposal.status != ProposalStatus::Passed {
            return Err(Error::InvalidProposalStatus);
        }
        let requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();
        let token_address: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let token_client = TokenClient::new(env, &token_address);
        let contract_addr: Address = env.current_contract_address();
        token_client.transfer(
            &contract_addr,
            &proposal.proposer,
            &requirements.required_stake,
        );
        proposal.status = ProposalStatus::Executed;
        env.storage().instance().set(&key, &proposal);
        Self::remove_from_status_list(env, proposal_id, ProposalStatus::Passed);
        Self::add_to_status_list(env, proposal_id, ProposalStatus::Executed);
        env.events().publish(
            (symbol_short!("proposal"), symbol_short!("executed")),
            proposal_id,
        );
        Ok(())
    }

    fn add_to_status_list(env: &Env, proposal_id: u32, status: ProposalStatus) {
        let key = Self::get_status_key(env, status);
        let mut list: Vec<u32> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env]);
        if !list.contains(&proposal_id) {
            list.push_back(proposal_id);
            env.storage().instance().set(&key, &list);
        }
    }

    fn remove_from_status_list(env: &Env, proposal_id: u32, status: ProposalStatus) {
        let key = Self::get_status_key(env, status);
        let list: Vec<u32> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env]);
        let mut new_list: Vec<u32> = vec![env];
        for id in list.iter() {
            if id != proposal_id {
                new_list.push_back(id);
            }
        }
        env.storage().instance().set(&key, &new_list);
    }

    pub fn get_latest_proposal_time(env: &Env, proposer: &Address) -> Option<u64> {
        let all_proposals = Self::get_all_proposals(env);
        let mut latest_time: Option<u64> = None;

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

    pub fn get_proposal(env: &Env, proposal_id: u32) -> Result<Proposal, Error> {
        let key = Self::get_proposal_key(env, proposal_id);
        env.storage()
            .instance()
            .get(&key)
            .ok_or(Error::ProposalNotFound)
    }

    pub fn get_proposals_by_status(env: &Env, status: ProposalStatus) -> Vec<u32> {
        let key = Self::get_status_key(env, status);
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| vec![env])
    }

    fn get_all_proposals(env: &Env) -> Vec<u32> {
        let statuses = [
            ProposalStatus::Draft,
            ProposalStatus::Active,
            ProposalStatus::Passed,
            ProposalStatus::Rejected,
            ProposalStatus::Executed,
            ProposalStatus::Canceled,
            ProposalStatus::Vetoed,
        ];
        let mut all = vec![env];
        for status in statuses.iter() {
            let proposals = Self::get_proposals_by_status(env, status.clone());
            for id in proposals.iter() {
                all.push_back(id);
            }
        }
        all
    }

    fn get_proposal_key(env: &Env, proposal_id: u32) -> Symbol {
        Symbol::new(&env, &format!("PROP_{}", proposal_id))
    }

    fn get_status_key(env: &Env, status: ProposalStatus) -> Symbol {
        Symbol::new(&env, &format!("STAT_{}", status as u32))
    }

    pub fn is_admin(env: &Env, caller: &Address) -> bool {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin == *caller
    }

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
