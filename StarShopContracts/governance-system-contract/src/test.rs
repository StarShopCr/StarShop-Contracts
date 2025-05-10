#![cfg(test)]
use super::*;
use governance::{GovernanceContract, GovernanceContractClient};
use proposals::ProposalManager;
use soroban_sdk::{
    contract, contractimpl, log, symbol_short,
    testutils::{Address as _, Events, Ledger, LedgerInfo},
    token::{StellarAssetClient as TokenAdmin, TokenClient},
    vec, Address, Env, IntoVal, Map, String, Symbol, Val, Vec,
};
use types::*;
use voting::VotingSystem;

// Test Constants
// const PROPOSAL_TITLE: &str = "TestProposal";
// const PROPOSAL_DESC: &str = "Description";
// const METADATA_HASH: &str = "hash123";
const COOLDOWN_PERIOD: u64 = 86400;
const REQUIRED_STAKE: i128 = 1000;
const PROPOSAL_LIMIT: u32 = 5;
const MAX_VOTING_POWER: i128 = 10000;
const VOTING_DURATION: u64 = 86400;
const QUORUM: u128 = 1000;
const THRESHOLD: u128 = 5000;
const EXECUTION_DELAY: u64 = 3600;

fn create_test_contracts(
    env: &Env,
) -> (
    Address,
    Address,
    Address,
    GovernanceContractClient,
    MockReferralClient,
) {
    let governance_id = env.register(GovernanceContract, ());
    let referral_id = env.register(MockReferral, ());
    let auction_id = env.register(MockAuction, ());
    let governance_client = GovernanceContractClient::new(&env, &governance_id);
    let referral_client = MockReferralClient::new(&env, &referral_id);

    (
        governance_id,
        referral_id,
        auction_id,
        governance_client,
        referral_client,
    )
}

fn setup_governance_args(env: &Env) -> (Address, Address, VotingConfig) {
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let default_config = VotingConfig {
        duration: VOTING_DURATION,
        quorum: QUORUM,
        threshold: THRESHOLD,
        execution_delay: EXECUTION_DELAY,
        one_address_one_vote: false,
    };

    (admin, proposer, default_config)
}

fn create_token_contracts<'a>(
    env: &'a Env,
    admin: &'a Address,
) -> (Address, TokenAdmin<'a>, TokenClient<'a>) {
    let stellar_asset = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = stellar_asset.address();
    let token_admin = TokenAdmin::new(&env, &token_id);
    let token_client = TokenClient::new(&env, &token_id);
    (token_id, token_admin, token_client)
}

fn setup_proposal_args(
    env: &Env,
    proposer: &Address,
) -> (Symbol, Symbol, String, ProposalType, Vec<Action>) {
    let title = symbol_short!("TestProp");
    let description = symbol_short!("Descr");
    let metadata_hash = String::from_str(&env, "hash123");
    let proposal_type = ProposalType::EconomicChange; // Matches 5
    let actions = vec![&env, Action::AppointModerator(proposer.clone())];

    (title, description, metadata_hash, proposal_type, actions)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    let governance_id = env.register(GovernanceContract, ());
    let governance_client = GovernanceContractClient::new(&env, &governance_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let referral = Address::generate(&env);
    let auction = Address::generate(&env);
    let config = VotingConfig {
        duration: VOTING_DURATION,
        quorum: QUORUM,
        threshold: THRESHOLD,
        execution_delay: EXECUTION_DELAY,
        one_address_one_vote: false,
    };

    governance_client.initialize(&admin, &token, &referral, &auction, &config);

    env.as_contract(&governance_id, || {
        let stored_admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        let stored_token: Address = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let stored_referral: Address = env.storage().instance().get(&REFERRAL_KEY).unwrap();
        let stored_auction: Address = env.storage().instance().get(&AUCTION_KEY).unwrap();
        let stored_config: VotingConfig =
            env.storage().instance().get(&DEFAULT_CONFIG_KEY).unwrap();
        let stored_requirements: ProposalRequirements =
            env.storage().instance().get(&REQUIREMENTS_KEY).unwrap();

        assert_eq!(stored_admin, admin, "Admin address mismatch");
        assert_eq!(stored_token, token, "Token address mismatch");
        assert_eq!(stored_referral, referral, "Referral address mismatch");
        assert_eq!(stored_auction, auction, "Auction address mismatch");
        assert_eq!(stored_config, config, "Voting config duration mismatch");
        assert_eq!(
            stored_requirements,
            ProposalRequirements {
                cooldown_period: COOLDOWN_PERIOD,
                required_stake: REQUIRED_STAKE,
                proposal_limit: PROPOSAL_LIMIT,
                max_voting_power: MAX_VOTING_POWER,
            },
            "Requirements mismatch"
        );
    });

    // let events = env.events().all();
    // log!(&env, "Captured events: {:?}", events);
    // assert_eq!(events.len(), 1, "Expected one initialization event");
    // assert_eq!(
    //     events,
    //     vec![
    //         &env,
    //         (
    //             governance_id.clone(),
    //             (symbol_short!("govern"), symbol_short!("init")).into_val(&env),
    //             (
    //                 admin.clone(),
    //                 token.clone(),
    //                 referral.clone(),
    //                 auction.clone()
    //             )
    //                 .into_val(&env)
    //         ),
    //     ],
    //     "Initialization event mismatch"
    // );
}

#[test]
fn test_initialize_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let governance_id = env.register(GovernanceContract, ());
    let governance_client = GovernanceContractClient::new(&env, &governance_id);

    let admin = Address::generate(&env);
    let token = Address::generate(&env);
    let referral = Address::generate(&env);
    let auction = Address::generate(&env);
    let config = VotingConfig {
        duration: VOTING_DURATION,
        quorum: QUORUM,
        threshold: THRESHOLD,
        execution_delay: EXECUTION_DELAY,
        one_address_one_vote: false,
    };

    governance_client.initialize(&admin, &token, &referral, &auction, &config);
    let result = governance_client.try_initialize(&admin, &token, &referral, &auction, &config);
    assert_eq!(
        result,
        Err(Ok(Error::AlreadyInitialized)),
        "Expected AlreadyInitialized error"
    );
}

#[test]
fn test_create_proposal_success() {
    let env = Env::default();
    env.mock_all_auths();

    // Register contracts
    let governance_id = env.register(GovernanceContract, ());
    let referral_id = env.register(MockReferral, ());
    let auction_id = env.register(MockAuction, ());
    let governance_client = GovernanceContractClient::new(&env, &governance_id);
    let referral_client = MockReferralClient::new(&env, &referral_id);

    // Setup
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let config = VotingConfig {
        duration: VOTING_DURATION,
        quorum: QUORUM,
        threshold: THRESHOLD,
        execution_delay: EXECUTION_DELAY,
        one_address_one_vote: false,
    };

    let stellar_asset = env.register_stellar_asset_contract_v2(admin.clone());
    let token_id = stellar_asset.address();
    let token_admin = TokenAdmin::new(&env, &token_id);
    let token_client = TokenClient::new(&env, &token_id);

    // Initialize governance contract
    log!(&env, "Initializing governance contract");
    let init_result =
        governance_client.try_initialize(&admin, &token_id, &referral_id, &auction_id, &config);
    assert!(
        init_result.is_ok(),
        "Initialization failed: {:?}",
        init_result
    );

    // Set token balance and referral status
    log!(&env, "Setting token balance and referral status");
    token_admin.mint(&proposer, &2000);
    // token_client.approve(&proposer, &governance_id, &2000, &1000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Create proposal
    log!(&env, "Creating proposal");
    let title = symbol_short!("TestProp");
    let description = symbol_short!("Descr");
    let metadata_hash = String::from_str(&env, "hash123");
    let proposal_type = ProposalType::EconomicChange; // Matches 5
    let actions = vec![&env, Action::AppointModerator(proposer.clone())];

    let proposal_id_result = governance_client.try_create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );
    assert!(
        proposal_id_result.is_ok(),
        "Create proposal failed: {:?}",
        proposal_id_result
    );
    let proposal_id = proposal_id_result
        .expect("Failed to get proposal ID after successful creation")
        .unwrap();

    // Verify proposal
    log!(&env, "verifying proposal");
    let stored_proposal: Proposal = env.invoke_contract(
        &governance_id,
        &Symbol::new(&env, "get_proposal"),
        vec![&env, proposal_id.into_val(&env)],
    );
    assert_eq!(stored_proposal.title, title);
    assert_eq!(stored_proposal.description, description);
    assert_eq!(stored_proposal.proposer, proposer);
    assert_eq!(stored_proposal.metadata_hash, metadata_hash);
    assert_eq!(stored_proposal.proposal_type, proposal_type);
    assert_eq!(stored_proposal.actions, actions);
    assert_eq!(stored_proposal.status, ProposalStatus::Draft);

    // Verify token balance (stake locked)
    assert_eq!(token_client.balance(&proposer), 1000); // 2000 - 1000 stake
}

#[test]
fn test_create_proposal_insufficient_stake() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (_, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, token_client) = create_token_contracts(&env, &admin);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &500);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Create proposal
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);
    let result = governance_client.try_create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    assert_eq!(
        result,
        Err(Ok(Error::InsufficientStake)),
        "Expected InsufficientStake error"
    );

    // Verify proposal not created
    let balance = token_client.balance(&proposer);
    assert_eq!(balance, 500, "Balance should not change");
}

#[test]
fn test_create_proposal_not_verified() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (_, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &1000);
    referral_client.set_user_verified(&proposer, &false); // Not verified
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Create proposal
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);
    let result = governance_client.try_create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    assert_eq!(
        result,
        Err(Ok(Error::NotVerified)),
        "Expected NotVerified error"
    );
}

#[test]
fn test_create_proposal_proposal_limit_reached() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (_, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &10000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Create proposal
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    // Create 5 proposals (reaching limit)
    for _ in 1..=PROPOSAL_LIMIT {
        governance_client.create_proposal(
            &proposer,
            &title,
            &description,
            &metadata_hash,
            &proposal_type,
            &actions,
            &config,
        );

        env.ledger().with_mut(|li| {
            li.timestamp += COOLDOWN_PERIOD + 1; // Advance past the cooldown period
        });
    }

    // Try to create one more
    let result = governance_client.try_create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    assert_eq!(
        result,
        Err(Ok(Error::ProposalLimitReached)),
        "Expected ProposalLimitReached error"
    );
}

#[test]
fn test_create_proposal_in_cooldown() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (_, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &5000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Create proposal
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    // Create first proposal
    governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    // Try to create another within cooldown
    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 2; // Not enough time passed
    });

    let result = governance_client.try_create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    assert_eq!(
        result,
        Err(Ok(Error::ProposalInCooldown)),
        "Expected ProposalInCooldown error"
    );

    // Try after cooldown
    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD; // Past cooldown
    });

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );
    assert_eq!(proposal_id, 2, "Second proposal should succeed");
}

#[test]
fn test_activate_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let moderator = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    let proposal: Proposal = env.invoke_contract(
        &governance_id,
        &Symbol::new(&env, "get_proposal"),
        vec![&env, proposal_id.into_val(&env)],
    );
    assert_eq!(proposal.status, ProposalStatus::Draft);

    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });
    governance_client.activate_proposal(&moderator, &proposal_id);

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Active,
            "Status should be Active"
        );
        assert_ne!(proposal.activated_at, 0, "Activated_at should be set");
    });
}

#[test]
fn test_activate_proposal_no_moderators() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (_, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let moderator = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    let result = governance_client.try_activate_proposal(&moderator, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::Unauthorized)),
        "Expected Unauthorized error"
    );
}

#[test]
fn test_cancel_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, token_client) = create_token_contracts(&env, &admin);
    let moderator = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    let balance = token_client.balance(&proposer);
    assert_eq!(balance, 1000, "Stake should be deducted");

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Draft,
            "Status should be Canceled"
        );
    });

    governance_client.activate_proposal(&moderator, &proposal_id);

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Active,
            "Status should be Active"
        );
    });

    governance_client.cancel_proposal(&moderator, &proposal_id);

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Canceled,
            "Status should be Canceled"
        );
    });

    let balance = token_client.balance(&proposer);
    assert_eq!(balance, 2000, "Stake should be returned");
}

#[test]
fn test_veto_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, token_client) = create_token_contracts(&env, &admin);
    let moderator = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    governance_client.activate_proposal(&moderator, &proposal_id);
    governance_client.mark_passed(&moderator, &proposal_id);
    governance_client.veto_proposal(&moderator, &proposal_id);

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Vetoed,
            "Status should be Vetoed"
        );
    });

    let balance = token_client.balance(&proposer);
    assert_eq!(balance, 1000, "Stake should be burned (2000 - 1000)");
}

#[test]
fn test_veto_proposal_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let moderator = Address::generate(&env);
    let non_moderator = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    governance_client.activate_proposal(&moderator, &proposal_id);
    governance_client.mark_passed(&moderator, &proposal_id);

    let result = governance_client.try_veto_proposal(&non_moderator, &proposal_id);
    assert_eq!(
        result,
        Err(Ok(Error::Unauthorized)),
        "Expected Unauthorized error"
    );

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Passed,
            "Status should remain Passed"
        );
    });
}

#[test]
fn test_mark_passed_and_executed() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, token_client) = create_token_contracts(&env, &admin);
    let moderator = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    governance_client.activate_proposal(&moderator, &proposal_id);
    governance_client.mark_passed(&moderator, &proposal_id);
    governance_client.mark_executed(&moderator, &proposal_id);

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Executed,
            "Status should be Executed"
        );
    });

    let balance = token_client.balance(&proposer);
    assert_eq!(balance, 2000, "Stake should be returned");
}

#[test]
fn test_mark_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, token_client) = create_token_contracts(&env, &admin);
    let moderator = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    governance_client.activate_proposal(&moderator, &proposal_id);
    governance_client.mark_rejected(&moderator, &proposal_id);

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Rejected,
            "Status should be Rejected"
        );
    });

    let balance = token_client.balance(&proposer);
    assert_eq!(balance, 2000, "Stake should be returned");
}

#[test]
fn test_cast_vote_one_address_one_vote() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, _) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let moderator = Address::generate(&env);
    let voter = Address::generate(&env);
    let one_vote_config = VotingConfig {
        duration: VOTING_DURATION,
        quorum: QUORUM,
        threshold: THRESHOLD,
        execution_delay: EXECUTION_DELAY,
        one_address_one_vote: true,
    };

    governance_client.initialize(
        &admin,
        &token_id,
        &referral_id,
        &auction_id,
        &one_vote_config,
    );

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_verified(&voter, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);
    referral_client.set_user_level(&voter, &UserLevel::Platinum);
    referral_client.set_total_users(&100);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &one_vote_config,
    );

    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24; // Advance ledger to simulate proposal activation time
    });

    governance_client.activate_proposal(&moderator, &proposal_id);
    governance_client.cast_vote(&voter, &proposal_id, &true); // Weight ignored in one_address_one_vote election

    env.as_contract(&governance_id, || {
        let for_votes = VotingSystem::get_for_votes(&env, proposal_id);
        log!(&env, "For votes: {:?}", for_votes);
        let total_votes = VotingSystem::get_total_votes(&env, proposal_id);
        log!(&env, "Total votes: {:?}", total_votes);
        let voter_count = VotingSystem::get_voter_count(&env, proposal_id);
        log!(&env, "Voter count: {:?}", voter_count);

        assert_eq!(for_votes, 1, "For votes should be 1");
        assert_eq!(total_votes, 1, "Total votes should be 1");
        assert_eq!(voter_count, 1, "Voter count should be 1");
    });
}

#[test]
fn test_cast_vote_already_voted() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, _) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let moderator = Address::generate(&env);
    let voter = Address::generate(&env);
    let one_vote_config = VotingConfig {
        duration: VOTING_DURATION,
        quorum: QUORUM,
        threshold: THRESHOLD,
        execution_delay: EXECUTION_DELAY,
        one_address_one_vote: true,
    };

    governance_client.initialize(
        &admin,
        &token_id,
        &referral_id,
        &auction_id,
        &one_vote_config,
    );

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    referral_client.set_user_verified(&proposer, &true);
    referral_client.set_user_verified(&voter, &true);
    referral_client.set_user_level(&proposer, &UserLevel::Platinum);
    referral_client.set_user_level(&voter, &UserLevel::Platinum);
    referral_client.set_total_users(&100);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &one_vote_config,
    );

    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });

    governance_client.activate_proposal(&moderator, &proposal_id);
    governance_client.cast_vote(&voter, &proposal_id, &true);

    let result = governance_client.try_cast_vote(&voter, &proposal_id, &true);
    assert_eq!(result, Err(Ok(Error::AlreadyVoted)), "Expected AlreadyVoted error");
}

// #[test]
// fn test_tally_votes_passing() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let governance_id = env.register(GovernanceContract, ());
//     let governance_client = GovernanceContractClient::new(&env, &governance_id);

//     let admin = Address::generate(&env);
//     let proposer = Address::generate(&env);
//     let voter1 = Address::generate(&env);
//     let voter2 = Address::generate(&env);
//     let token_id = env.register(MockToken, ());
//     let referral_id = env.register(MockReferral, ());
//     let auction = Address::generate(&env);
//     let config = VotingConfig {
//         duration: VOTING_DURATION,
//         quorum: QUORUM,
//         threshold: THRESHOLD,
//         execution_delay: EXECUTION_DELAY,
//         one_address_one_vote: false,
//     };

//     governance_client.initialize(&admin, &token_id, &referral_id, &auction, &config);

//     let token_client = MockTokenClient::new(&env, &token_id);
//     token_client.set_balance(&proposer, 2000);
//     token_client.set_balance(&voter1, 6000);
//     token_client.set_balance(&voter2, 4000);
//     let referral_client = MockReferralClient::new(&env, &referral_id);
//     referral_client.set_user_verified(&proposer, true);
//     referral_client.set_user_level(&proposer, UserLevel::Platinum);
//     referral_client.set_user_verified(&voter1, true);
//     referral_client.set_user_level(&voter1, UserLevel::Platinum);
//     referral_client.set_user_verified(&voter2, true);
//     referral_client.set_user_level(&voter2, UserLevel::Platinum);
//     referral_client.set_total_users(100);

//     let title = Symbol::new(&env, PROPOSAL_TITLE);
//     let description = Symbol::new(&env, PROPOSAL_DESC);
//     let metadata_hash = String::from_str(&env, METADATA_HASH);
//     let proposal_type = ProposalType::EconomicChange;
//     let actions = vec![&env, Action::AppointModerator(proposer.clone())];

//     let proposal_id = governance_client.create_proposal(
//         &proposer,
//         &title,
//         &description,
//         &metadata_hash,
//         &proposal_type,
//         &actions,
//         &config,
//     );

//     governance_client.take_snapshot(&proposal_id);

//     governance_client.activate_proposal(&proposal_id);
//     governance_client.cast_vote(&proposal_id, &voter1, true, 6000); // For
//     governance_client.cast_vote(&proposal_id, &voter2, false, 4000); // Against

//     env.as_contract(&governance_id, || {
//         let total_voting_power = VotingSystem::get_total_voting_power(&env, proposal_id);
//         assert_eq!(total_voting_power, 10000, "Total voting power should be 10000");
//         let for_votes = VotingSystem::get_for_votes(&env, proposal_id);
//         let total_votes = VotingSystem::get_total_votes(&env, proposal_id);
//         assert_eq!(for_votes, 6000, "For votes should be 6000");
//         assert_eq!(total_votes, 10000, "Total votes should be 10000");
//         let passed = VotingSystem::tally_votes(&env, proposal_id, &config).unwrap();
//         assert!(passed, "Proposal should pass (6000/10000 > 50%)");
//     });
// }

// #[test]
// fn test_tally_votes_not_enough_quorum() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let governance_id = env.register(GovernanceContract, ());
//     let governance_client = GovernanceContractClient::new(&env, &governance_id);

//     let admin = Address::generate(&env);
//     let proposer = Address::generate(&env);
//     let voter = Address::generate(&env);
//     let token_id = env.register(MockToken, ());
//     let referral_id = env.register(MockReferral, ());
//     let auction = Address::generate(&env);
//     let config = VotingConfig {
//         duration: VOTING_DURATION,
//         quorum: QUORUM,
//         threshold: THRESHOLD,
//         execution_delay: EXECUTION_DELAY,
//         one_address_one_vote: false,
//     };

//     governance_client.initialize(&admin, &token_id, &referral_id, &auction, &config);

//     let token_client = MockTokenClient::new(&env, &token_id);
//     token_client.set_balance(&proposer, 2000);
//     token_client.set_balance(&voter, 500);
//     let referral_client = MockReferralClient::new(&env, &referral_id);
//     referral_client.set_user_verified(&proposer, true);
//     referral_client.set_user_level(&proposer, UserLevel::Platinum);
//     referral_client.set_user_verified(&voter, true);
//     referral_client.set_user_level(&voter, UserLevel::Platinum);
//     referral_client.set_total_users(100);

//     let title = Symbol::new(&env, PROPOSAL_TITLE);
//     let description = Symbol::new(&env, PROPOSAL_DESC);
//     let metadata_hash = String::from_str(&env, METADATA_HASH);
//     let proposal_type = ProposalType::EconomicChange;
//     let actions = vec![&env, Action::AppointModerator(proposer.clone())];

//     let proposal_id = governance_client.create_proposal(
//         &proposer,
//         &title,
//         &description,
//         &metadata_hash,
//         &proposal_type,
//         &actions,
//         &config,
//     );

//     governance_client.take_snapshot(&proposal_id);

//     governance_client.activate_proposal(&proposal_id);
//     governance_client.cast_vote(&proposal_id, &voter, true, 500);

//     env.as_contract(&governance_id, || {
//         let total_voting_power = VotingSystem::get_total_voting_power(&env, proposal_id);
//         assert_eq!(total_voting_power, 10000, "Total voting power should be 10000");
//         let for_votes = VotingSystem::get_for_votes(&env, proposal_id);
//         let total_votes = VotingSystem::get_total_votes(&env, proposal_id);
//         assert_eq!(for_votes, 500, "For votes should be 500");
//         assert_eq!(total_votes, 500, "Total votes should be 500");
//         let passed = VotingSystem::tally_votes(&env, proposal_id, &config).unwrap();
//         assert!(!passed, "Proposal should not pass (500/10000 < 10% quorum)");
//     });
// }

// #[test]
// fn test_delegate_and_get_weight() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let governance_id = env.register(GovernanceContract, ());
//     let governance_client = GovernanceContractClient::new(&env, &governance_id);

//     let admin = Address::generate(&env);
//     let delegator = Address::generate(&env);
//     let delegatee = Address::generate(&env);
//     let token_id = env.register(MockToken, ());
//     let referral_id = env.register(MockReferral, ());
//     let auction = Address::generate(&env);
//     let config = VotingConfig {
//         duration: VOTING_DURATION,
//         quorum: QUORUM,
//         threshold: THRESHOLD,
//         execution_delay: EXECUTION_DELAY,
//         one_address_one_vote: false,
//     };

//     governance_client.initialize(&admin, &token_id, &referral_id, &auction, &config);

//     let token_client = MockTokenClient::new(&env, &token_id);
//     token_client.set_balance(&delegator, 2000);
//     token_client.set_balance(&delegatee, 3000);

//     governance_client.delegate(&delegator, &delegatee);

//     env.as_contract(&governance_id, || {
//         let delegation = WeightCalculator::get_delegation(&env, &delegator).unwrap();
//         assert_eq!(delegation, delegatee, "Delegation should be set");
//         let delegators = WeightCalculator::get_delegators(&env, &delegatee);
//         assert_eq!(delegators.len(), 1, "Should have one delegator");
//         assert_eq!(delegators.get_unchecked(0), delegator, "Delegator mismatch");

//         let proposal_id = 1u32;
//         WeightCalculator::take_snapshot(&env, proposal_id).unwrap();
//         let delegator_weight = WeightCalculator::get_weight(&env, &delegator, proposal_id).unwrap();
//         let delegatee_weight = WeightCalculator::get_weight(&env, &delegatee, proposal_id).unwrap();
//         assert_eq!(delegator_weight, 0, "Delegator weight should be 0");
//         assert_eq!(
//             delegatee_weight, 5000,
//             "Delegatee weight should be 2000 + 3000"
//         );
//     });

//     let events = env.events().all();
//     assert_eq!(events.len(), 2, "Expected init and delegated events");
//     assert_eq!(
//         events.get_unchecked(events.len() - 1),
//         (
//             governance_id.clone(),
//             (symbol_short!("vote"), symbol_short!("delegated")).into_val(&env),
//             (delegator.clone(), delegatee.clone()).into_val(&env)
//         )
//     );
// }

// #[test]
// fn test_delegate_self_not_allowed() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let governance_id = env.register(GovernanceContract, ());
//     let governance_client = GovernanceContractClient::new(&env, &governance_id);

//     let admin = Address::generate(&env);
//     let user = Address::generate(&env);
//     let token_id = env.register(MockToken, ());
//     let referral_id = env.register(MockReferral, ());
//     let auction = Address::generate(&env);
//     let config = VotingConfig {
//         duration: VOTING_DURATION,
//         quorum: QUORUM,
//         threshold: THRESHOLD,
//         execution_delay: EXECUTION_DELAY,
//         one_address_one_vote: false,
//     };

//     governance_client.initialize(&admin, &token_id, &referral_id, &auction, &config);

//     let result = governance_client.try_delegate(&user, &user);
//     assert_eq!(
//         result,
//         Err(Ok(Error::SelfDelegationNotAllowed)),
//         "Expected SelfDelegationNotAllowed error"
//     );

//     env.as_contract(&governance_id, || {
//         let delegation = WeightCalculator::get_delegation(&env, &user);
//         assert!(delegation.is_none(), "No delegation should be set");
//     });
// }

// #[test]
// fn test_read_only_functions() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let governance_id = env.register(GovernanceContract, ());
//     let governance_client = GovernanceContractClient::new(&env, &governance_id);

//     let admin = Address::generate(&env);
//     let proposer = Address::generate(&env);
//     let token_id = env.register(MockToken, ());
//     let referral_id = env.register(MockReferral, ());
//     let auction = Address::generate(&env);
//     let config = VotingConfig {
//         duration: VOTING_DURATION,
//         quorum: QUORUM,
//         threshold: THRESHOLD,
//         execution_delay: EXECUTION_DELAY,
//         one_address_one_vote: false,
//     };

//     governance_client.initialize(&admin, &token_id, &referral_id, &auction, &config);

//     let token_client = MockTokenClient::new(&env, &token_id);
//     token_client.set_balance(&proposer, 2000);
//     let referral_client = MockReferralClient::new(&env, &referral_id);
//     referral_client.set_user_verified(&proposer, true);
//     referral_client.set_user_level(&proposer, UserLevel::Platinum);

//     let title = Symbol::new(&env, PROPOSAL_TITLE);
//     let description = Symbol::new(&env, PROPOSAL_DESC);
//     let metadata_hash = String::from_str(&env, METADATA_HASH);
//     let proposal_type = ProposalType::EconomicChange;
//     let actions = vec![&env, Action::AppointModerator(proposer.clone())];

//     let proposal_id = governance_client.create_proposal(
//         &proposer,
//         &title,
//         &description,
//         &metadata_hash,
//         &proposal_type,
//         &actions,
//         &config,
//     );

//     let storage_snapshot = snapshot_instance_storage(&env, &governance_id);

//     governance_client.get_proposal(&proposal_id);
//     governance_client.get_proposals_by_status(&ProposalStatus::Draft);
//     governance_client.get_for_votes(&proposal_id);
//     governance_client.get_against_votes(&proposal_id);
//     governance_client.get_total_votes(&proposal_id);
//     governance_client.get_voter_count(&proposal_id);
//     governance_client.get_total_voters();
//     governance_client.get_weight(&proposer, &proposal_id);

//     let storage_snapshot_after = snapshot_instance_storage(&env, &governance_id);
//     assert_eq!(
//         storage_snapshot, storage_snapshot_after,
//         "Read-only functions should not modify storage"
//     );
// }

// #[test]
// fn test_full_proposal_lifecycle_simulation() {
//     let env = Env::default();
//     env.mock_all_auths();

//     let governance_id = env.register(GovernanceContract, ());
//     let governance_client = GovernanceContractClient::new(&env, &governance_id);

//     let admin = Address::generate(&env);
//     let proposer = Address::generate(&env);
//     let moderator = Address::generate(&env);
//     let voter1 = Address::generate(&env);
//     let voter2 = Address::generate(&env);
//     let token_id = env.register(MockToken, ());
//     let referral_id = env.register(MockReferral, ());
//     let auction = Address::generate(&env);
//     let config = VotingConfig {
//         duration: VOTING_DURATION,
//         quorum: QUORUM,
//         threshold: THRESHOLD,
//         execution_delay: EXECUTION_DELAY,
//         one_address_one_vote: false,
//     };

//     governance_client.initialize(&admin, &token_id, &referral_id, &auction, &config);

//     let token_client = MockTokenClient::new(&env, &token_id);
//     token_client.set_balance(&proposer, 2000);
//     token_client.set_balance(&voter1, 6000);
//     token_client.set_balance(&voter2, 4000);
//     let referral_client = MockReferralClient::new(&env, &referral_id);
//     referral_client.set_user_verified(&proposer, true);
//     referral_client.set_user_level(&proposer, UserLevel::Platinum);
//     referral_client.set_user_verified(&voter1, true);
//     referral_client.set_user_level(&voter1, UserLevel::Platinum);
//     referral_client.set_user_verified(&voter2, true);
//     referral_client.set_user_level(&voter2, UserLevel::Platinum);
//     referral_client.set_total_users(100);

//     env.as_contract(&governance_id, || {
//         let mut moderators: Vec<Address> = vec![&env];
//         moderators.push_back(moderator.clone());
//         env.storage().instance().set(&MODERATOR_KEY, &moderators);
//     });

//     let title = Symbol::new(&env, PROPOSAL_TITLE);
//     let description = Symbol::new(&env, PROPOSAL_DESC);
//     let metadata_hash = String::from_str(&env, METADATA_HASH);
//     let proposal_type = ProposalType::EconomicChange;
//     let actions = vec![&env, Action::AppointModerator(proposer.clone())];

//     // Create proposal
//     let proposal_id = governance_client.create_proposal(
//         &proposer,
//         &title,
//         &description,
//         &metadata_hash,
//         &proposal_type,
//         &actions,
//         &config,
//     );
//     assert_eq!(proposal_id, 1, "Proposal ID should be 1");

//     // Take snapshot
//     governance_client.take_snapshot(&proposal_id);

//     // Activate proposal
//     governance_client.activate_proposal(&proposal_id);

//     // Cast votes
//     governance_client.cast_vote(&proposal_id, &voter1, true, 6000);
//     governance_client.cast_vote(&proposal_id, &voter2, false, 4000);

//     // Advance time to end voting
//     env.ledger().with_mut(|li| {
//         li.timestamp += VOTING_DURATION + 1;
//     });

//     // Check voting ended
//     env.as_contract(&governance_id, || {
//         let ended = VotingSystem::check_voting_ended(&env, proposal_id, &config).unwrap();
//         assert!(ended, "Voting should have ended");
//     });

//     // Tally votes
//     env.as_contract(&governance_id, || {
//         let passed = VotingSystem::tally_votes(&env, proposal_id, &config).unwrap();
//         assert!(passed, "Proposal should pass");
//     });

//     // Mark passed
//     governance_client.mark_passed(&proposal_id);

//     // Execute proposal
//     governance_client.mark_executed(&proposal_id);

//     env.as_contract(&governance_id, || {
//         let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
//         assert_eq!(proposal.status, ProposalStatus::Executed, "Status should be Executed");
//     });

//     let balance = token_client.balance(&proposer);
//     assert_eq!(balance, 2000, "Stake should be returned");

//     let events = env.events().all();
//     assert_eq!(
//         events.len(),
//         7,
//         "Expected init, created, activated, two votes, passed, and executed events"
//     );
// }

// // Helper function to capture instance storage state
// fn snapshot_instance_storage(env: &Env, governance_id: &Address) -> Map<Bytes, Bytes> {
//     env.as_contract(governance_id, || {
//         let storage = env.storage().instance();
//         let keys = map![
//             env,
//             (
//                 Bytes::from_slice(env, ADMIN_KEY.as_bytes()),
//                 Bytes::from_slice(env, ADMIN_KEY.as_bytes())
//             ),
//             (
//                 Bytes::from_slice(env, TOKEN_KEY.as_bytes()),
//                 Bytes::from_slice(env, TOKEN_KEY.as_bytes())
//             ),
//             (
//                 Bytes::from_slice(env, REFERRAL_KEY.as_bytes()),
//                 Bytes::from_slice(env, REFERRAL_KEY.as_bytes())
//             ),
//             (
//                 Bytes::from_slice(env, AUCTION_KEY.as_bytes()),
//                 Bytes::from_slice(env, AUCTION_KEY.as_bytes())
//             ),
//             (
//                 Bytes::from_slice(env, DEFAULT_CONFIG_KEY.as_bytes()),
//                 Bytes::from_slice(env, DEFAULT_CONFIG_KEY.as_bytes())
//             ),
//             (
//                 Bytes::from_slice(env, REQUIREMENTS_KEY.as_bytes()),
//                 Bytes::from_slice(env, REQUIREMENTS_KEY.as_bytes())
//             ),
//             (
//                 Bytes::from_slice(env, PROPOSAL_COUNTER_KEY.as_bytes()),
//                 Bytes::from_slice(env, PROPOSAL_COUNTER_KEY.as_bytes())
//             )
//         ];

//         let mut snapshot = Map::new(env);
//         for (key, _) in keys.iter() {
//             if let Some(value) = storage.get::<Bytes, Bytes>(&key) {
//                 snapshot.set(key.clone(), value.clone());
//                 log!(env, "Snapshot key: {}, value: {:?}", key, value);
//             }
//         }
//         snapshot
//     })
// }

// Mock Contracts

#[contract]
struct MockReferral;

#[contractimpl]
impl MockReferral {
    pub fn set_user_verified(env: Env, user: Address, verified: bool) {
        let mut data: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "verified"))
            .unwrap_or_else(|| Map::new(&env));
        data.set(user.clone(), verified);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "verified"), &data);
        log!(&env, "Set verified: user={:?}, verified={}", user, verified);
        // Self::set_total_users(env.clone(), Self::get_total_users(env) + 1);
    }

    pub fn is_user_verified(env: Env, user: Address) -> bool {
        let data: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "verified"))
            .unwrap_or_else(|| Map::new(&env));
        let result = data.get(user.clone()).unwrap_or(false);
        log!(&env, "Is verified: user={:?}, result={}", user, result);
        result
    }

    pub fn set_user_level(env: Env, user: Address, level: UserLevel) {
        let mut data: Map<Address, UserLevel> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "levels"))
            .unwrap_or_else(|| Map::new(&env));
        data.set(user.clone(), level.clone());
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "levels"), &data);
        log!(&env, "Set level: user={:?}, level={:?}", user, level);
    }

    pub fn get_user_level(env: Env, user: Address) -> Result<UserLevel, Error> {
        let data: Map<Address, UserLevel> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "levels"))
            .unwrap_or_else(|| Map::new(&env));
        let result = data.get(user.clone()).ok_or(Error::UserLevelNotSet)?;
        log!(&env, "Get level: user={:?}, level={:?}", user, result);
        Ok(result)
    }

    pub fn set_total_users(env: Env, count: u32) {
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "total_users"), &count);
        log!(&env, "Set total users: count={}", count);
    }

    pub fn get_total_users(env: Env) -> u32 {
        let result: u32 = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "total_users"))
            .unwrap_or(0);
        log!(&env, "Get total users: count={}", result);
        result
    }
}

#[contract]
struct MockAuction;

#[contractimpl]
impl MockAuction {
    pub fn set_auction(env: Env, auction: Address) {
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "auction"), &auction);
        log!(&env, "Set auction: auction={:?}", auction);
    }

    pub fn get_auction(env: Env) -> Address {
        let result: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "auction"))
            .unwrap_or(Address::generate(&env));
        log!(&env, "Get auction: auction={:?}", result);
        result
    }
}

#[contract]
struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn set_balance(env: Env, addr: Address, amount: i128) {
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "balances"))
            .unwrap_or_else(|| Map::new(&env));
        balances.set(addr.clone(), amount);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "balances"), &balances);
        log!(&env, "Set balance: addr={:?}, amount={}", addr, amount);
    }

    pub fn balance(env: Env, addr: Address) -> i128 {
        let balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "balances"))
            .unwrap_or_else(|| Map::new(&env));
        balances.get(addr).unwrap_or(0)
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "balances"))
            .unwrap_or_else(|| Map::new(&env));
        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }
        balances.set(from.clone(), from_balance - amount);
        let to_balance = balances.get(to.clone()).unwrap_or(0);
        balances.set(to.clone(), to_balance + amount);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "balances"), &balances);
        log!(
            &env,
            "Transfer: from={:?}, to={:?}, amount={}",
            from,
            to,
            amount
        );
    }

    pub fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();
        let mut balances: Map<Address, i128> = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "balances"))
            .unwrap_or_else(|| Map::new(&env));
        let from_balance = balances.get(from.clone()).unwrap_or(0);
        if from_balance < amount {
            panic!("Insufficient balance");
        }
        balances.set(from.clone(), from_balance - amount);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "balances"), &balances);
        log!(&env, "Burn: from={:?}, amount={}", from, amount);
    }
}
