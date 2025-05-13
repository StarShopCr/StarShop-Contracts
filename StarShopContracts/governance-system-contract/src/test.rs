#![cfg(test)]
use super::*;
use governance::{GovernanceContract, GovernanceContractClient};
use proposals::ProposalManager;
use soroban_sdk::{
    contract, contractimpl, log, symbol_short,
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient as TokenAdmin, TokenClient},
    vec, Address, Env, IntoVal, Map, String, Symbol, Vec,
};
use types::*;
use voting::VotingSystem;
use weights::WeightCalculator;

// Test Constants
const COOLDOWN_PERIOD: u64 = 86400;
const REQUIRED_STAKE: i128 = 1000;
const PROPOSAL_LIMIT: u32 = 5;
const MAX_VOTING_POWER: i128 = 10000;
const VOTING_DURATION: u64 = 86400;
const QUORUM: u128 = 1000;
const THRESHOLD: u128 = 5000;
const EXECUTION_DELAY: u64 = 3600;

/// Create a test environment with the necessary contracts and clients.
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

/// Setup arguments for initializing governance contract
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

/// Create Stellar Asset Token Admin and Client contracts
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

/// Setup arguments for creatig a proposal
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

/// Mark user verified status as true and set referral status to highest level
fn verify_user_and_set_status(ref_client: MockReferralClient, users: Vec<Address>) {
    for user in users.iter() {
        ref_client.set_user_verified(&user, &true);
        ref_client.set_user_level(&user, &UserLevel::Platinum);
        ref_client.set_total_users(&10);
    }
}

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();

    // Create governance client and initialize
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

    // Compare governance state variables after initialization
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

    // Create governance client and initialize
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

    // Initializing governance contract a second time should throw an error
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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
fn test_create_proposal_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (_, _, _, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (_, token_admin, _) = create_token_contracts(&env, &admin);

    // Set token balance and referral status
    token_admin.mint(&proposer, &1000);
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

    // Create proposal without initializing governance client
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
        Err(Ok(Error::NotInitialized)),
        "Expected NotInitialized error"
    );
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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
        li.timestamp += (COOLDOWN_PERIOD / 2) + 1; // Past cooldown
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
fn test_proposal_not_found_error() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (_, _, _, governance_client, _) = create_test_contracts(&env);
    let (admin, _, config) = setup_governance_args(&env);

    governance_client.initialize(
        &admin,
        &Address::generate(&env),
        &Address::generate(&env),
        &Address::generate(&env),
        &config,
    );

    // Attempt to activate a non-existent proposal
    let non_existent_proposal_id = 999; // Arbitrary ID that doesn't exist
    let result = governance_client.try_activate_proposal(&admin, &non_existent_proposal_id);

    // Verify the result
    assert_eq!(
        result,
        Err(Ok(Error::ProposalNotFound)),
        "Expected ProposalNotFound error"
    );
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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
            "Status should be Draft"
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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), voter.clone()]);

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
fn test_cast_vote_insufficient_referral_level() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let moderator = Address::generate(&env);
    let voter = Address::generate(&env);

    governance_client.initialize(
        &admin,
        &token_id,
        &referral_id,
        &auction_id,
        &config,
    );

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    token_admin.mint(&voter, &2000);
    referral_client.set_user_verified(&voter, &true);
    referral_client.set_user_level(&voter, &UserLevel::Gold);
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

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
        &config,
    );

    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });

    governance_client.activate_proposal(&moderator, &proposal_id);

    let result = governance_client.try_cast_vote(&voter, &proposal_id, &false);
    assert_eq!(
        result,
        Err(Ok(Error::InsufficientReferralLevel)),
        "Expected InsufficientReferralLevel error"
    );
}

#[test]
fn test_vote_proposal_inactive() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);

    let moderator = Address::generate(&env);
    let voter = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &1000);
    token_admin.mint(&voter, &2000);
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), voter.clone()]);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    // Create proposal
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

    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });

    // Cast vote without activating proposal
    let result = governance_client.try_cast_vote(&voter, &proposal_id, &false);
    assert_eq!(
        result,
        Err(Ok(Error::ProposalNotActive)),
        "Expected ProposalNotActive error"
    );
}

#[test]
fn test_vote_not_verified() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);

    let moderator = Address::generate(&env);
    let voter = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set token balance and referral status
    token_admin.mint(&proposer, &1000);
    token_admin.mint(&voter, &1000);
    referral_client.set_user_verified(&voter, &false); // Not verified
    referral_client.set_user_level(&voter, &UserLevel::Platinum);
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone()]);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    // Create proposal
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

    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });

    governance_client.activate_proposal(&moderator, &proposal_id);

    let result = governance_client.try_cast_vote(&voter, &proposal_id, &false);

    assert_eq!(
        result,
        Err(Ok(Error::NotVerified)),
        "Expected NotVerified error"
    );
}

#[test]
fn test_cast_vote_no_voting_power() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let moderator = Address::generate(&env);
    let voter = Address::generate(&env);

    governance_client.initialize(
        &admin,
        &token_id,
        &referral_id,
        &auction_id,
        &config,
    );

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), voter.clone()]);

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
        &config,
    );

    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });

    governance_client.activate_proposal(&moderator, &proposal_id);

    // Cast vote without sufficient voting power (no tokens minted)
    let result = governance_client.try_cast_vote(&voter, &proposal_id, &false);
    assert_eq!(
        result,
        Err(Ok(Error::NoVotingPower)),
        "Expected NoVotingPower error"
    );
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
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), voter.clone()]);

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
    assert_eq!(
        result,
        Err(Ok(Error::AlreadyVoted)),
        "Expected AlreadyVoted error"
    );
}

#[test]
fn test_tally_votes_passing() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let moderator = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set voter token balances
    token_admin.mint(&proposer, &2000);
    token_admin.mint(&voter1, &6000);
    token_admin.mint(&voter2, &4000);

    // Set user referral statuses
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), voter1.clone(), voter2.clone()]);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    // Create proposal
    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    // Take snapshot of voting power
    governance_client.take_snapshot(&proposal_id);

    // Activate proposal
    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });
    governance_client.activate_proposal(&moderator, &proposal_id);

    // Cast votes
    governance_client.cast_vote(&voter1, &proposal_id, &true); // For
    governance_client.cast_vote(&voter2, &proposal_id, &false); // Against

    env.as_contract(&governance_id, || {
        let total_voting_power = VotingSystem::get_total_voting_power(&env, proposal_id);
        let for_votes = VotingSystem::get_for_votes(&env, proposal_id);
        let total_votes = VotingSystem::get_total_votes(&env, proposal_id);
        let passed = VotingSystem::tally_votes(&env, proposal_id, &config).unwrap();

        assert_eq!(
            total_voting_power, 10000,
            "Total voting power should be 10000"
        );
        assert_eq!(for_votes, 6000, "For votes should be 6000");
        assert_eq!(total_votes, 10000, "Total votes should be 10000");
        assert!(passed, "Proposal should pass (6000/10000 > 50%)");
    });
}

#[test]
fn test_tally_votes_not_enough_quorum() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);
    let moderator = Address::generate(&env);
    let voter = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set voter token balance
    token_admin.mint(&proposer, &2000);
    token_admin.mint(&voter, &500);

    // Set user referral statuses
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), voter.clone()]);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    // Create proposal
    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );
        
        // Activate proposal
        env.ledger().with_mut(|li| {
            li.timestamp += COOLDOWN_PERIOD / 24;
        });
        governance_client.activate_proposal(&moderator, &proposal_id);
        
        // Cast votes
        governance_client.cast_vote(&voter, &proposal_id, &true);

    // Take snapshot of voting power
    governance_client.take_snapshot(&proposal_id);

    env.as_contract(&governance_id, || {
        let total_voting_power = VotingSystem::get_total_voting_power(&env, proposal_id);
        let for_votes = VotingSystem::get_for_votes(&env, proposal_id);
        let total_votes = VotingSystem::get_total_votes(&env, proposal_id);
        let passed = VotingSystem::tally_votes(&env, proposal_id, &config).unwrap();

        assert_eq!(
            total_voting_power, 10000,
            "Total voting power should be 10000"
        );
        assert_eq!(for_votes, 500, "For votes should be 500");
        assert_eq!(total_votes, 500, "Total votes should be 500");
        assert!(!passed, "Proposal should not pass (500/10000 < 10% quorum)");
    });
}

#[test]
fn test_delegate_and_get_weight() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, _) =
        create_test_contracts(&env);
    let (admin, _, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let delegator = Address::generate(&env);
    let delegatee = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set voter token balance
    token_admin.mint(&delegator, &2000);
    token_admin.mint(&delegatee, &3000);

    governance_client.delegate_vote(&delegator, &delegatee);

    env.as_contract(&governance_id, || {
        let delegation = WeightCalculator::get_delegation(&env, &delegator).unwrap();
        let delegators = WeightCalculator::get_delegators(&env, &delegatee);
        assert_eq!(delegation, delegatee, "Delegation should be set");
        assert_eq!(delegators.len(), 1, "Should have one delegator");
        assert_eq!(delegators.get_unchecked(0), delegator, "Delegator mismatch");

        let proposal_id = 1u32;
        WeightCalculator::take_snapshot(&env, proposal_id).unwrap();
        let delegator_weight = WeightCalculator::get_weight(&env, &delegator, proposal_id).unwrap();
        let delegatee_weight = WeightCalculator::get_weight(&env, &delegatee, proposal_id).unwrap();
        assert_eq!(delegator_weight, 0, "Delegator weight should be 0");
        assert_eq!(
            delegatee_weight, 5000,
            "Delegatee weight should be 2000 + 3000"
        );
    });
}

#[test]
fn test_delegate_self_not_allowed() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, _) =
        create_test_contracts(&env);
    let (admin, _, config) = setup_governance_args(&env);
    let (token_id, _, _) = create_token_contracts(&env, &admin);
    let voter = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    let result = governance_client.try_delegate_vote(&voter, &voter);
    assert_eq!(
        result,
        Err(Ok(Error::SelfDelegationNotAllowed)),
        "Expected SelfDelegationNotAllowed error"
    );

    env.as_contract(&governance_id, || {
        let delegation = WeightCalculator::get_delegation(&env, &voter);
        assert!(delegation.is_none(), "No delegation should be set");
    });
}

#[test]
fn test_circular_delegation_not_allowed() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, _) =
        create_test_contracts(&env);
    let (admin, _, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let delegator = Address::generate(&env);
    let delegatee = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set voter token balance
    token_admin.mint(&delegator, &2000);
    token_admin.mint(&delegatee, &3000);

    governance_client.delegate_vote(&delegator, &delegatee);
    let result = governance_client.try_delegate_vote(&delegatee, &delegator);
    assert_eq!(
        result,
        Err(Ok(Error::InvalidDelegation)),
        "Expected InvalidDelegation error"
    );

    env.as_contract(&governance_id, || {
        let delegation = WeightCalculator::get_delegation(&env, &delegatee);
        assert!(delegation.is_none(), "No delegation should be set");
    });
}

#[test]
fn test_cast_vote_afrer_delegation() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let moderator = Address::generate(&env);
    let delegator = Address::generate(&env);
    let delegatee = Address::generate(&env);

    governance_client.initialize(
        &admin,
        &token_id,
        &referral_id,
        &auction_id,
        &config,
    );

    // Set token balance and referral status
    token_admin.mint(&proposer, &2000);
    token_admin.mint(&delegator, &2000);
    token_admin.mint(&delegatee, &3000);
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), delegator.clone(), delegatee.clone()]);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    // Create and activate proposal
    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });

    governance_client.activate_proposal(&moderator, &proposal_id);

    // Delegate vote
    governance_client.delegate_vote(&delegator, &delegatee);

    // Cast vote without sufficient voting power (voting power delegated away)
    let result = governance_client.try_cast_vote(&delegator, &proposal_id, &true);
    assert_eq!(
        result,
        Err(Ok(Error::NoVotingPower)),
        "Expected NoVotingPower error"
    );
}

#[test]
fn test_execution_no_delay() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, _) = create_token_contracts(&env, &admin);
    let (title, description, metadata_hash, proposal_type, actions) =
        setup_proposal_args(&env, &proposer);

    let moderator = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set voter token balances
    token_admin.mint(&proposer, &2000);
    token_admin.mint(&voter1, &6000);
    token_admin.mint(&voter2, &4000);

    // Set user referral statuses
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), voter1.clone(), voter2.clone()]);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    // Create proposal
    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    // Take snapshot of voting power
    governance_client.take_snapshot(&proposal_id);

    // Activate proposal
    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });
    governance_client.activate_proposal(&moderator, &proposal_id);

    // Cast votes
    governance_client.cast_vote(&voter1, &proposal_id, &true); // For
    governance_client.cast_vote(&voter2, &proposal_id, &false); // Against

    // Execute the proposal
    let result = governance_client.try_execute_proposal(&admin, &proposal_id);

    // Verify the result
    assert_eq!(
        result,
        Err(Ok(Error::ExecutionDelayNotMet)),
        "Expected ExecutionDelay error"
    );
}

#[test]
fn test_execute_actions() {
    let env = Env::default();
    env.mock_all_auths();

    // Setup test environment
    let (governance_id, referral_id, auction_id, governance_client, referral_client) =
        create_test_contracts(&env);
    let (admin, proposer, config) = setup_governance_args(&env);
    let (token_id, token_admin, token_client) = create_token_contracts(&env, &admin);
    let (title, description, metadata_hash, proposal_type, _) =
        setup_proposal_args(&env, &proposer);

    let moderator = Address::generate(&env);
    let voter1 = Address::generate(&env);
    let voter2 = Address::generate(&env);
    let new_moderator1 = Address::generate(&env);
    let new_moderator2 = Address::generate(&env);
    let actions = vec![
        &env,
        Action::AppointModerator(new_moderator1.clone()),
        Action::RemoveModerator(new_moderator1.clone()),
        Action::AppointModerator(new_moderator2.clone()),
    ];

    governance_client.initialize(&admin, &token_id, &referral_id, &auction_id, &config);

    // Set voter token balances
    token_admin.mint(&proposer, &2000);
    token_admin.mint(&voter1, &6000);
    token_admin.mint(&voter2, &4000);

    // Set user referral statuses
    verify_user_and_set_status(referral_client, vec![&env, proposer.clone(), voter1.clone(), voter2.clone()]);

    // Set moderator
    env.as_contract(&governance_id, || {
        let mut moderators: Vec<Address> = vec![&env];
        moderators.push_back(moderator.clone());
        env.storage().instance().set(&MODERATOR_KEY, &moderators);
    });

    // Create a proposal with appoint and remove moderator actions
    let proposal_id = governance_client.create_proposal(
        &proposer,
        &title,
        &description,
        &metadata_hash,
        &proposal_type,
        &actions,
        &config,
    );

    // Take snapshot of voting power
    governance_client.take_snapshot(&proposal_id);

    // Activate proposal
    env.ledger().with_mut(|li| {
        li.timestamp += COOLDOWN_PERIOD / 24;
    });
    governance_client.activate_proposal(&moderator, &proposal_id);

    // Cast votes
    log!(&env, "Votes castxxxxxx");
    governance_client.cast_vote(&voter1, &proposal_id, &true); // For
    governance_client.cast_vote(&voter2, &proposal_id, &false); // Against
    log!(&env, "Votes castxxxxxxwwwwwwww");
    
    // Simulate time to end voting and pass execution delay
    env.ledger().with_mut(|li| {
        li.timestamp += VOTING_DURATION + EXECUTION_DELAY + 1;
    });

    // Check voting ended
    env.as_contract(&governance_id, || {
        let ended = VotingSystem::check_voting_ended(&env, proposal_id, &config).unwrap();
        assert!(ended, "Voting should have ended");
    });

    // Tally votes
    env.as_contract(&governance_id, || {
        let passed = VotingSystem::tally_votes(&env, proposal_id, &config).unwrap();
        assert!(passed, "Proposal should pass");
    });

    // Execute proposal
    governance_client.execute_proposal(&admin, &proposal_id);

    env.as_contract(&governance_id, || {
        let proposal = ProposalManager::get_proposal(&env, proposal_id).unwrap();
        assert_eq!(
            proposal.status,
            ProposalStatus::Executed,
            "Status should be Executed"
        );

        let moderators: Vec<Address> = env.storage().instance().get(&MODERATOR_KEY).unwrap();
        assert!(
            !moderators.contains(&new_moderator1),
            "Moderator should have been removed"
        );
        assert!(
            moderators.contains(&new_moderator2),
            "Moderator should have been added"
        );
    });

    let balance = token_client.balance(&proposer);
    assert_eq!(balance, 2000, "Stake should be returned");
}

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
