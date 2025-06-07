#![cfg(test)]

use soroban_sdk::{
    contract, contractclient, contracterror, contractimpl, contracttype,
    testutils::{Address as _, Events as _, Ledger, LedgerInfo},
    token::{StellarAssetClient as TokenAdmin, TokenClient},
    vec, Address, Bytes, Env, IntoVal, Map, String, Symbol, Vec,
};

use super::{
    types::{AirdropError, AirdropEvent, DataKey, EventStats},
    AirdropContract, AirdropContractClient,
};

// === Mock Referral Contract ===

// === Error Definitions ===
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProviderError {
    InvalidUser = 1,
    MetricNotSupported = 2,
}

// === Metric Provider Trait ===
#[contractclient(name = "MetricProviderClient")]
pub trait MetricProvider {
    fn get_user_metric(env: Env, user: Address, metric: Symbol) -> Result<u64, ProviderError>;
}

// === Data Structures ===
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum UserLevel {
    Basic = 0,    // New users, basic commission rates
    Silver = 1,   // Intermediate level, improved rates
    Gold = 2,     // Advanced level, premium rates
    Platinum = 3, // Highest level, maximum benefits
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerificationStatus {
    Unverified, // User has not passed verification
    Verified,   // User has passed verification
}

#[contracttype]
#[derive(Clone)]
pub struct UserData {
    pub direct_referrals: Vec<Address>,
    pub team_size: u32,
    pub total_rewards: i128,
    pub level: UserLevel,
    pub join_date: u64,
    pub verification_status: VerificationStatus,
}

// === Storage Keys ===
#[contracttype]
pub enum Key {
    Admin,          // Contract administrator
    RewardToken,    // Token used for rewards
    User(Address),  // User data storage
    Milestone(u32), // Milestone data
}

// === Storage Helper ===
pub struct ReferralModule;

impl ReferralModule {
    /// Stores user data for a given user address.
    pub fn set_user_data(env: &Env, user: &Address, data: UserData) {
        env.storage()
            .instance()
            .set(&Key::User(user.clone()), &data);
    }

    /// Retrieves user data for a given user address.
    pub fn get_user_data(env: &Env, user: &Address) -> Result<UserData, ProviderError> {
        env.storage()
            .instance()
            .get(&Key::User(user.clone()))
            .ok_or(ProviderError::InvalidUser)
    }

    /// Checks if user data exists for a given user address.
    pub fn user_exists(env: &Env, user: &Address) -> bool {
        env.storage().instance().has(&Key::User(user.clone()))
    }

    /// Returns a mock referral conversion rate (75%).
    pub fn get_referral_conversion_rate(_env: &Env, _user: &Address) -> Result<u32, ProviderError> {
        Ok(75)
    }
}

// === Contract ===
#[contract]
pub struct ReferralContract;

// === Metric Provider Impl ===
#[contractimpl]
impl MetricProvider for ReferralContract {
    fn get_user_metric(env: Env, user: Address, metric: Symbol) -> Result<u64, ProviderError> {
        if !ReferralModule::user_exists(&env, &user) {
            return Err(ProviderError::InvalidUser);
        }

        let user_data = ReferralModule::get_user_data(&env, &user)?;

        let referrals = Symbol::new(&env, "referrals");
        let team_size = Symbol::new(&env, "team_size");
        let total_rewards = Symbol::new(&env, "total_rewards");
        let user_level = Symbol::new(&env, "user_level");
        let conversion_rate = Symbol::new(&env, "conversion_rate");
        let active_days = Symbol::new(&env, "active_days");
        let is_verified = Symbol::new(&env, "is_verified");

        let result = match metric.clone() {
            m if m == referrals => user_data.direct_referrals.len() as u64,
            m if m == team_size => user_data.team_size as u64,
            m if m == total_rewards => {
                let scaled = user_data.total_rewards / 10_000;
                if scaled < 0 || scaled > u64::MAX as i128 {
                    u64::MAX
                } else {
                    scaled as u64
                }
            }
            m if m == user_level => match user_data.level {
                UserLevel::Basic => 0,
                UserLevel::Silver => 1,
                UserLevel::Gold => 2,
                UserLevel::Platinum => 3,
            },
            m if m == conversion_rate => {
                ReferralModule::get_referral_conversion_rate(&env, &user)? as u64
            }
            m if m == active_days => {
                let current_time = env.ledger().timestamp();
                (current_time - user_data.join_date) / (24 * 60 * 60)
            }
            m if m == is_verified => {
                matches!(user_data.verification_status, VerificationStatus::Verified) as u64
            }
            _ => return Err(ProviderError::MetricNotSupported),
        };

        env.events()
            .publish((Symbol::new(&env, "MetricQueried"), user, metric), result);

        Ok(result)
    }
}

// === Additional Impl ===
#[contractimpl]
impl ReferralContract {
    /// Sets user data for a given user address.
    pub fn set_user_data(
        env: Env,
        user: Address,
        direct_referrals: Vec<Address>,
        team_size: u32,
        total_rewards: i128,
        level: u32,
        join_date: u64,
        is_verified: bool,
    ) {
        user.require_auth(); // Ensure caller is authorized

        let level_enum = match level {
            1 => UserLevel::Silver,
            2 => UserLevel::Gold,
            3 => UserLevel::Platinum,
            _ => UserLevel::Basic,
        };

        let verification_enum = if is_verified {
            VerificationStatus::Verified
        } else {
            VerificationStatus::Unverified
        };

        let user_data = UserData {
            direct_referrals,
            team_size,
            total_rewards,
            level: level_enum,
            join_date,
            verification_status: verification_enum,
        };

        ReferralModule::set_user_data(&env, &user, user_data);
    }

    /// Initialize the referral contract (stub for tests)
    pub fn initialize(env: Env, admin: Address, token: Address) {
        admin.require_auth();
        env.storage().instance().set(&Key::Admin, &admin);
        env.storage().instance().set(&Key::RewardToken, &token);
    }

    /// Register a user with a referral (stub for tests)
    pub fn register_with_referral(env: Env, user: Address, referrer: Address, _proof: String) {
        user.require_auth();
        let user_data = UserData {
            direct_referrals: Vec::new(&env),
            team_size: 0,
            total_rewards: 0,
            level: UserLevel::Basic,
            join_date: env.ledger().timestamp(),
            verification_status: VerificationStatus::Unverified,
        };
        ReferralModule::set_user_data(&env, &user, user_data);

        if ReferralModule::user_exists(&env, &referrer) {
            let mut referrer_data = ReferralModule::get_user_data(&env, &referrer).unwrap();
            referrer_data.direct_referrals.push_back(user.clone());
            referrer_data.team_size += 1;
            ReferralModule::set_user_data(&env, &referrer, referrer_data);
        }
    }

    /// Approve user verification (stub for tests)
    pub fn approve_verification(env: Env, user: Address) {
        if ReferralModule::user_exists(&env, &user) {
            let mut user_data = ReferralModule::get_user_data(&env, &user).unwrap();
            user_data.verification_status = VerificationStatus::Verified;
            ReferralModule::set_user_data(&env, &user, user_data);
        }
    }
}

// === Helper Functions ===
fn create_test_env() -> (Env, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let airdrop_contract = env.register(AirdropContract, ());
    (env, airdrop_contract)
}

pub fn setup_contract(e: &Env) -> (ReferralContractClient, Address, Address, Address) {
    let admin = Address::generate(e);
    let token = e.register_stellar_asset_contract_v2(admin.clone());
    let contract_id = e.register(ReferralContract, {});
    let client = ReferralContractClient::new(e, &contract_id);

    e.mock_all_auths();

    (client, contract_id, admin, token.address())
}

fn setup_token(env: &Env) -> (Address, TokenAdmin) {
    let token_admin = Address::generate(env);
    let token = env.register_stellar_asset_contract_v2(token_admin.clone());
    (token.address(), TokenAdmin::new(env, &token.address()))
}

fn create_airdrop_event(
    client: &AirdropContractClient,
    admin: &Address,
    conditions: Map<Symbol, u64>,
    amount: i128,
    token_address: &Address,
) -> u64 {
    let name = Symbol::new(&client.env, "Airdrop1");
    let description = Bytes::from_slice(&client.env, b"Test airdrop");
    let start_time = client.env.ledger().timestamp();
    let end_time = start_time + 1000;

    client.create_airdrop(
        admin,
        &name,
        &description,
        &conditions,
        &amount,
        token_address,
        &start_time,
        &end_time,
        &None,
        &None,
    );

    client.env.as_contract(&client.address, || {
        client
            .env
            .storage()
            .persistent()
            .get(&DataKey::EventId)
            .unwrap()
    })
}

// === Tests ===
#[test]
fn test_initialize_success() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), Address::generate(&env))],
    ));

    let client = AirdropContractClient::new(&env, &airdrop_contract);

    client.initialize(&admin, &providers);

    env.as_contract(&airdrop_contract, || {
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        let event_id: u64 = env.storage().persistent().get(&DataKey::EventId).unwrap();
        assert_eq!(stored_admin, admin);
        assert_eq!(event_id, 0);
    });
}

#[test]
fn test_initialize_already_initialized() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), Address::generate(&env))],
    ));

    let client = AirdropContractClient::new(&env, &airdrop_contract);

    client.initialize(&admin, &providers);

    let result = client.try_initialize(&admin, &providers);

    assert_eq!(result, Err(Ok(AirdropError::AlreadyInitialized)));
}

#[test]
fn test_initialize_invalid_metric_symbol() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);

    // Empty symbol metric
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, ""), Address::generate(&env))],
    ));

    let client = AirdropContractClient::new(&env, &airdrop_contract);
    let result = client.try_initialize(&admin, &providers);

    assert_eq!(result, Err(Ok(AirdropError::InvalidEventConfig)));
}

#[test]
fn test_initialize_sets_provider_registry_correctly() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);

    let metric = Symbol::new(&env, "referrals");
    let provider = Address::generate(&env);
    let providers = Some(Map::from_array(&env, [(metric.clone(), provider.clone())]));

    let client = AirdropContractClient::new(&env, &airdrop_contract);
    client.initialize(&admin, &providers);

    env.as_contract(&airdrop_contract, || {
        let stored: Address = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderRegistry(metric.clone()))
            .unwrap();
        assert_eq!(stored, provider);
    });
}

// Helper to set up the contract
fn referral_contract(env: &Env) -> (ReferralContractClient, Address) {
    let contract_id = env.register(ReferralContract, ());
    let contract = ReferralContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    (contract, admin)
}

#[test]
fn test_get_user_metric_through_referral_contract() {
    let env = Env::default();
    let (contract, _) = referral_contract(&env);

    env.mock_all_auths();

    // Set up user1 (main user)
    let user1 = Address::generate(&env);
    let timestamp = env.ledger().timestamp();
    contract.set_user_data(
        &user1,          // user
        &Vec::new(&env), // direct_referrals (updated later)
        &0,              // team_size (updated later)
        &0,              // total_rewards (updated later)
        &0,              // level (Basic)
        &timestamp,      // join_date
        &true,           // is_verified
    );

    // Add 4 referrals: 3 verified, 1 pending
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let user4 = Address::generate(&env);
    let user5 = Address::generate(&env);

    // Register user2 (verified)
    contract.set_user_data(&user2, &Vec::new(&env), &0, &0, &0, &timestamp, &true);

    // Register user3 (verified)
    contract.set_user_data(&user3, &Vec::new(&env), &0, &0, &0, &timestamp, &true);

    // Register user4 (verified)
    contract.set_user_data(&user4, &Vec::new(&env), &0, &0, &0, &timestamp, &true);

    // Register user5 (unverified)
    contract.set_user_data(&user5, &Vec::new(&env), &0, &0, &0, &timestamp, &false);

    // Update user1 with referrals and rewards
    let referrals = vec![
        &env,
        user2.clone(),
        user3.clone(),
        user4.clone(),
        user5.clone(),
    ];
    contract.set_user_data(
        &user1, &referrals, // direct_referrals
        &4,         // team_size
        &50_000,    // total_rewards (5% of 1,000,000 stroops)
        &0,         // level (Basic)
        &timestamp, // join_date
        &true,      // is_verified
    );

    // Mock ledger timestamp to simulate 30 days
    env.ledger().with_mut(|li: &mut LedgerInfo| {
        li.timestamp = timestamp + 30 * 24 * 60 * 60; // 30 days later
    });

    // Create MetricProviderClient for get_user_metric
    let metric_client = MetricProviderClient::new(&env, &contract.address);

    // Test all metrics
    let referrals = metric_client.get_user_metric(&user1, &Symbol::new(&env, "referrals"));
    assert_eq!(referrals, 4); // 4 direct referrals

    let team_size = metric_client.get_user_metric(&user1, &Symbol::new(&env, "team_size"));
    assert_eq!(team_size, 4); // 4 total team size

    let total_rewards = metric_client.get_user_metric(&user1, &Symbol::new(&env, "total_rewards"));
    assert_eq!(total_rewards, 5); // 50,000 stroops / 10,000 = 5

    let user_level = metric_client.get_user_metric(&user1, &Symbol::new(&env, "user_level"));
    assert_eq!(user_level, 0); // Still Basic

    let conversion_rate =
        metric_client.get_user_metric(&user1, &Symbol::new(&env, "conversion_rate"));
    assert_eq!(conversion_rate, 75); // Hardcoded 75%

    let active_days = metric_client.get_user_metric(&user1, &Symbol::new(&env, "active_days"));
    assert_eq!(active_days, 30); // 30 days

    let is_verified = metric_client.get_user_metric(&user1, &Symbol::new(&env, "is_verified"));
    assert_eq!(is_verified, 1); // User1 is verified
}

#[test]
fn airdrop_event_created() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let (token_address, _) = setup_token(&env);

    let client = AirdropContractClient::new(&env, &airdrop_contract);
    client.initialize(&admin, &None);

    let conditions = Map::from_array(
        &env,
        [
            (Symbol::new(&env, "referrals"), 3u64),
            (Symbol::new(&env, "is_verified"), 1u64),
        ],
    );
    let amount = 1000;
    let name = Symbol::new(&env, "Airdrop1");
    let description = Bytes::from_slice(&env, b"Test airdrop");
    let start_time = env.ledger().timestamp();
    let end_time = start_time + 1000;

    let event_id = client.create_airdrop(
        &admin,
        &name,
        &description,
        &conditions,
        &amount,
        &token_address,
        &start_time,
        &end_time,
        &None,
        &None,
    );

    env.as_contract(&airdrop_contract, || {
        let event: AirdropEvent = env
            .storage()
            .persistent()
            .get(&DataKey::AirdropEvent(event_id))
            .unwrap();
        assert_eq!(event.name, name);
        assert_eq!(event.description, description);
        assert_eq!(event.conditions, conditions);
        assert_eq!(event.amount, amount);
        assert_eq!(event.token_address, token_address);
        assert_eq!(event.start_time, start_time);
        assert_eq!(event.end_time, end_time);
        assert_eq!(event.is_active, true);

        let stats: EventStats = env
            .storage()
            .persistent()
            .get(&DataKey::EventStats(event_id))
            .unwrap();
        assert_eq!(stats.recipient_count, 0);
        assert_eq!(stats.total_amount_distributed, 0);
    });
}

#[test]
fn create_airdrop_fails_invalid_config() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let (token_address, _) = setup_token(&env);

    let client = AirdropContractClient::new(&env, &airdrop_contract);
    client.initialize(&admin, &None);

    let invalid_conditions = Map::new(&env); // empty
    let invalid_name = Symbol::new(&env, ""); // empty name
    let description = Bytes::from_slice(&env, b"Test airdrop");
    let now = env.ledger().timestamp();
    let end_time = now + 1000;

    let result = client.try_create_airdrop(
        &admin,
        &invalid_name,
        &description,
        &invalid_conditions,
        &0, // invalid amount
        &token_address,
        &now, // start_time now is OK, but we'll try other cases
        &end_time,
        &None,
        &None,
    );

    assert_eq!(result, Err(Ok(AirdropError::InvalidEventConfig)));
}

#[test]
fn create_airdrop_emits_created_event() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let (token_address, _) = setup_token(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    client.initialize(&admin, &None);

    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);
    let amount = 1000;
    let name = Symbol::new(&env, "Drop");
    let description = Bytes::from_slice(&env, b"Test event");
    let start_time = env.ledger().timestamp();
    let end_time = start_time + 1000;

    let event_id = client.create_airdrop(
        &admin,
        &name,
        &description,
        &conditions,
        &amount,
        &token_address,
        &start_time,
        &end_time,
        &None,
        &None,
    );

    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                airdrop_contract.clone(),
                (
                    Symbol::new(&env, "CreatedAirdropEvent"),
                    event_id,
                    admin.clone()
                )
                    .into_val(&env),
                (start_time, amount).into_val(&env)
            )
        ]
    );
}

#[test]
fn test_claim_airdrop_not_found() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let client = AirdropContractClient::new(&env, &airdrop_contract);

    client.initialize(&admin, &None);

    // Try to claim a non-existent event
    let non_existent_event_id = 999;

    env.mock_all_auths();

    let result = client.try_claim_airdrop(&user, &non_existent_event_id);

    assert_eq!(result, Err(Ok(AirdropError::AirdropNotFound)));
}

#[test]
fn test_claim_airdrop_inactive_events() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = setup_token(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    client.initialize(&admin, &None);

    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    let amount = 1000;
    let name = Symbol::new(&env, "Drop");
    let description = Bytes::from_slice(&env, b"Test inactive event");
    let start_time = env.ledger().timestamp();

    let end_time = start_time + 100;

    let event_id = client.create_airdrop(
        &admin,
        &name,
        &description,
        &conditions,
        &amount,
        &token_address,
        &start_time,
        &end_time,
        &None,
        &None,
    );

    env.as_contract(&airdrop_contract, || {
        let event: AirdropEvent = env
            .storage()
            .persistent()
            .get(&DataKey::AirdropEvent(event_id))
            .unwrap();
        assert_eq!(event.is_active, true);
    });

    env.ledger().with_mut(|li| li.timestamp = start_time + 2000); // Move time beyond the end time

    let result = client.try_claim_airdrop(&user, &event_id);

    assert_eq!(result, Err(Ok(AirdropError::EventInactive)));

    client.finalize_event(&admin, &event_id);

    let result = client.try_claim_airdrop(&user, &event_id);

    assert_eq!(result, Err(Ok(AirdropError::EventInactive)));
}

#[test]
fn test_claim_airdrop_success() {
    let (env, airdrop_contract) = create_test_env();

    env.mock_all_auths();

    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions that the user will meet
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Verify minting
    let token_client = TokenClient::new(&env, &token_address);
    assert_eq!(token_client.balance(&airdrop_contract), 10000);

    // Create the airdrop event
    let event_id = create_airdrop_event(&client, &admin, conditions, 10000, &token_address);

    // Setup user with sufficient referrals using set_user_data
    let referrals = vec![
        &env,
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];
    referral_client.set_user_data(
        &user,
        &referrals,
        &3,
        &0,
        &0,
        &env.ledger().timestamp(),
        &true,
    );

    assert_eq!(
        referral_client.get_user_metric(&user, &Symbol::new(&env, "referrals")),
        3
    );

    // Call claim_airdrop
    client.claim_airdrop(&user, &event_id);

    // // Verify user received tokens
    let user_balance = token_client.balance(&user);
    assert_eq!(user_balance, 10000);

    // Verify event stats were updated
    env.as_contract(&airdrop_contract, || {
        let stats: EventStats = env
            .storage()
            .persistent()
            .get(&DataKey::EventStats(event_id))
            .unwrap();
        assert_eq!(stats.recipient_count, 1);
        assert_eq!(stats.total_amount_distributed, 10000);
    });

    // Verify user is marked as claimed
    env.as_contract(&airdrop_contract, || {
        let claimed = env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::Claimed(event_id, user.clone()))
            .unwrap_or(false);
        assert_eq!(claimed, true);
    });
}

#[test]
fn test_claim_airdrop_already_claimed() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Create the airdrop event
    let event_id = create_airdrop_event(&client, &admin, conditions, 1000, &token_address);

    // Setup user with sufficient referrals
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);
    for _ in 0..3 {
        let referred = Address::generate(&env);
        referral_client.register_with_referral(&referred, &user, &String::from_str(&env, "proof"));
        referral_client.approve_verification(&referred);
    }

    env.mock_all_auths();

    // First claim
    client.claim_airdrop(&user, &event_id);

    // Second claim attempt
    let result = client.try_claim_airdrop(&user, &event_id);

    assert_eq!(result, Err(Ok(AirdropError::AlreadyClaimed)));

    // Verify only one distribution occurred
    let user_balance = TokenClient::new(&env, &token_address).balance(&user);
    assert_eq!(user_balance, 1000);
}

#[test]
fn test_claim_airdrop_conditions_not_met() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions that require 3 referrals
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Create the airdrop event
    let event_id = create_airdrop_event(&client, &admin, conditions, 1000, &token_address);

    // User has 0 referrals (condition not met)
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);

    env.mock_all_auths();

    let result = client.try_claim_airdrop(&user, &event_id);

    assert_eq!(result, Err(Ok(AirdropError::UserNotEligible)));
}

#[test]
fn test_claim_airdrop_max_users_exceeded() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Create name and description for the airdrop
    let name = Symbol::new(&env, "LimitedAirdrop");
    let description = Bytes::from_slice(&env, b"Max1user");
    let start_time = env.ledger().timestamp();
    let end_time = start_time + 1000;

    // Create event with max_users = 1
    let event_id = client.create_airdrop(
        &admin,
        &name,
        &description,
        &conditions,
        &1000,
        &token_address,
        &start_time,
        &end_time,
        &Some(1), // max_users = 1
        &None,
    );

    // Setup users with sufficient referrals
    for user in [&user1, &user2] {
        referral_client.register_with_referral(user, &admin, &String::from_str(&env, "proof"));
        referral_client.approve_verification(user);
        for _ in 0..3 {
            let referred = Address::generate(&env);
            referral_client.register_with_referral(
                &referred,
                user,
                &String::from_str(&env, "proof"),
            );
            referral_client.approve_verification(&referred);
        }
    }

    env.mock_all_auths();

    // First user claims successfully
    client.claim_airdrop(&user1, &event_id);

    // Second user should fail
    let result = client.try_claim_airdrop(&user2, &event_id);

    assert_eq!(result, Err(Ok(AirdropError::CapExceeded)));

    // Verify balances
    let token_client = TokenClient::new(&env, &token_address);
    assert_eq!(token_client.balance(&user1), 1000);
    assert_eq!(token_client.balance(&user2), 0);
}

#[test]
fn test_claim_airdrop_max_total_exceeded() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Create name and description for the airdrop
    let name = Symbol::new(&env, "LimitedTotalAirdrop");
    let description = Bytes::from_slice(&env, b"Max1000");
    let start_time = env.ledger().timestamp();
    let end_time = start_time + 1000;

    // Create event with max_total = 1000
    let event_id = client.create_airdrop(
        &admin,
        &name,
        &description,
        &conditions,
        &1000,
        &token_address,
        &start_time,
        &end_time,
        &None,
        &Some(1000), // max_total = 1000
    );

    // Setup users with sufficient referrals
    for user in [&user1, &user2] {
        referral_client.register_with_referral(user, &admin, &String::from_str(&env, "proof"));
        referral_client.approve_verification(user);
        for _ in 0..3 {
            let referred = Address::generate(&env);
            referral_client.register_with_referral(
                &referred,
                user,
                &String::from_str(&env, "proof"),
            );
            referral_client.approve_verification(&referred);
        }
    }

    env.mock_all_auths();

    // First user claims successfully (1000 tokens)
    client.claim_airdrop(&user1, &event_id);

    // Second user should fail (would exceed max total)
    let result = client.try_claim_airdrop(&user2, &event_id);

    assert_eq!(result, Err(Ok(AirdropError::CapExceeded)));

    // Verify balances
    let token_client = TokenClient::new(&env, &token_address);
    assert_eq!(token_client.balance(&user1), 1000);
    assert_eq!(token_client.balance(&user2), 0);
}

#[test]
fn test_claim_tokens_emits_event() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Create airdrop with specific name for event verification
    let name = Symbol::new(&env, "EventTest");
    let description = Bytes::from_slice(&env, b"Testeventemission");
    let start_time = env.ledger().timestamp();
    let end_time = start_time + 1000;

    let event_id = client.create_airdrop(
        &admin,
        &name,
        &description,
        &conditions,
        &1000,
        &token_address,
        &start_time,
        &end_time,
        &None,
        &None,
    );

    // Setup user with sufficient referrals
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);
    for _ in 0..3 {
        let referred = Address::generate(&env);
        referral_client.register_with_referral(&referred, &user, &String::from_str(&env, "proof"));
        referral_client.approve_verification(&referred);
    }

    env.mock_all_auths();

    // Call claim_airdrop
    client.claim_airdrop(&user, &event_id);

    // Verify the Claimed event
    let events = env.events().all();
    assert_eq!(events.len(), 5);
}

#[test]
fn test_claim_airdrop_updates_stats_correctly() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Create the airdrop event
    let event_id = create_airdrop_event(&client, &admin, conditions, 1000, &token_address);

    // Setup users with sufficient referrals
    for user in [&user1, &user2] {
        referral_client.register_with_referral(user, &admin, &String::from_str(&env, "proof"));
        referral_client.approve_verification(user);
        for _ in 0..3 {
            let referred = Address::generate(&env);
            referral_client.register_with_referral(
                &referred,
                user,
                &String::from_str(&env, "proof"),
            );
            referral_client.approve_verification(&referred);
        }
    }

    env.mock_all_auths();

    // First user claims
    client.claim_airdrop(&user1, &event_id);

    // Check stats after first claim
    env.as_contract(&airdrop_contract, || {
        let stats: EventStats = env
            .storage()
            .persistent()
            .get(&DataKey::EventStats(event_id))
            .unwrap();
        assert_eq!(stats.recipient_count, 1);
        assert_eq!(stats.total_amount_distributed, 1000);
    });

    // Second user claims
    client.claim_airdrop(&user2, &event_id);

    // Check stats after second claim
    env.as_contract(&airdrop_contract, || {
        let stats: EventStats = env
            .storage()
            .persistent()
            .get(&DataKey::EventStats(event_id))
            .unwrap();
        assert_eq!(stats.recipient_count, 2);
        assert_eq!(stats.total_amount_distributed, 2000);
    });
}

#[test]
fn test_claim_tokens_transfer_failure() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, _) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Create the airdrop event
    let event_id = create_airdrop_event(&client, &admin, conditions, 1000, &token_address);

    // Setup user with sufficient referrals
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);
    for _ in 0..3 {
        let referred = Address::generate(&env);
        referral_client.register_with_referral(&referred, &user, &String::from_str(&env, "proof"));
        referral_client.approve_verification(&referred);
    }

    env.mock_all_auths();

    // Try to claim tokens - should fail due to insufficient balance
    let result = client.try_claim_airdrop(&user, &event_id);

    assert_eq!(result, Err(Ok(AirdropError::InsufficientContractBalance)));

    // Verify no tokens transferred
    let token_client = TokenClient::new(&env, &token_address);
    assert_eq!(token_client.balance(&user), 0);
}

#[test]
fn test_pause_resume_event() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Create the airdrop event
    let event_id = create_airdrop_event(&client, &admin, conditions, 1000, &token_address);

    // Setup user with sufficient referrals
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);
    for _ in 0..3 {
        let referred = Address::generate(&env);
        referral_client.register_with_referral(&referred, &user, &String::from_str(&env, "proof"));
        referral_client.approve_verification(&referred);
    }

    env.mock_all_auths();

    // Pause event
    client.pause_event(&admin, &event_id);

    // Try to claim from paused event
    let result = client.try_claim_airdrop(&user, &event_id);
    assert_eq!(result, Err(Ok(AirdropError::EventInactive)));

    // Resume event
    client.resume_event(&admin, &event_id);

    // Claim should now succeed
    client.claim_airdrop(&user, &event_id);

    // Verify user received tokens
    let token_client = TokenClient::new(&env, &token_address);
    assert_eq!(token_client.balance(&user), 1000);

    // Verify event stats
    env.as_contract(&airdrop_contract, || {
        let stats: EventStats = env
            .storage()
            .persistent()
            .get(&DataKey::EventStats(event_id))
            .unwrap();
        assert_eq!(stats.recipient_count, 1);
        assert_eq!(stats.total_amount_distributed, 1000);
    });

    // Try to resume already active event
    let result = client.try_resume_event(&admin, &event_id);
    assert_eq!(result, Err(Ok(AirdropError::InvalidEventConfig)));
}

#[test]
fn test_event_status_override() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Create the airdrop event
    let event_id = create_airdrop_event(&client, &admin, conditions, 1000, &token_address);

    // Setup user with sufficient referrals
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);
    for _ in 0..3 {
        let referred = Address::generate(&env);
        referral_client.register_with_referral(&referred, &user, &String::from_str(&env, "proof"));
        referral_client.approve_verification(&referred);
    }

    // Pause event
    client.pause_event(&admin, &event_id);

    env.mock_all_auths();

    // Try to claim from inactive event
    let result = client.try_claim_airdrop(&user, &event_id);

    assert_eq!(result, Err(Ok(AirdropError::EventInactive)));
}

#[test]
fn test_claim_airdrop_success_xlm() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let (xlm_address, _) = setup_token(&env);

    let client = AirdropContractClient::new(&env, &airdrop_contract);
    client.initialize(&admin, &None);

    let conditions = Map::from_array(
        &env,
        [
            (Symbol::new(&env, "referrals"), 3u64),
            (Symbol::new(&env, "is_verified"), 1u64),
        ],
    );
    let amount = 1000;
    let name = Symbol::new(&env, "Airdrop1");
    let description = Bytes::from_slice(&env, b"Testairdrop");
    let start_time = env.ledger().timestamp();
    let end_time = start_time + 1000;

    // Create airdrop event
    let event_id = client.create_airdrop(
        &admin,
        &name,
        &description,
        &conditions,
        &amount,
        &xlm_address,
        &start_time,
        &end_time,
        &None,
        &None,
    );

    // Verify event creation
    env.as_contract(&airdrop_contract, || {
        let event: AirdropEvent = env
            .storage()
            .persistent()
            .get(&DataKey::AirdropEvent(event_id))
            .unwrap();
        assert_eq!(event.conditions, conditions);
        assert_eq!(event.amount, amount);
        assert_eq!(event.token_address, xlm_address);
    });
}

#[test]
fn test_claim_airdrop_not_eligible() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let airdrop_client = AirdropContractClient::new(&env, &airdrop_contract);

    // Setup contracts
    referral_client.initialize(&admin, &token_address);
    airdrop_client.initialize(&admin, &None);
    airdrop_client.register_provider(
        &admin,
        &Symbol::new(&env, "referrals"),
        &referral_contract_id,
    );

    // Create airdrop event with high referral requirement
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);
    let amount = 1000;
    let event_id =
        create_airdrop_event(&airdrop_client, &admin, conditions, amount, &token_address);

    // Fund the contract
    token_admin.mint(&airdrop_contract, &10000);

    // Register user but don't add any referrals
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);

    env.mock_all_auths();

    // Try to claim - should fail due to insufficient referrals
    let result = airdrop_client.try_claim_airdrop(&user, &event_id);
    assert_eq!(result, Err(Ok(AirdropError::UserNotEligible)));
}

#[test]
fn test_insufficient_contract_balance() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let airdrop_client = AirdropContractClient::new(&env, &airdrop_contract);

    // Setup contracts
    referral_client.initialize(&admin, &token_address);
    airdrop_client.initialize(&admin, &None);
    airdrop_client.register_provider(
        &admin,
        &Symbol::new(&env, "referrals"),
        &referral_contract_id,
    );

    // Create airdrop event
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 1u64)]);
    let amount = 1000;
    let event_id =
        create_airdrop_event(&airdrop_client, &admin, conditions, amount, &token_address);

    // Setup user with sufficient referrals
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);

    let referred = Address::generate(&env);
    referral_client.register_with_referral(&referred, &user, &String::from_str(&env, "proof2"));
    referral_client.approve_verification(&referred);

    // Fund contract with insufficient balance (less than airdrop amount)
    token_admin.mint(&airdrop_contract, &500); // Only 500 when need 1000

    env.mock_all_auths();

    // Try to claim - should fail due to insufficient contract balance
    let result = airdrop_client.try_claim_airdrop(&user, &event_id);
    assert_eq!(result, Err(Ok(AirdropError::InsufficientContractBalance)));

    // Verify no tokens were transferred
    let token_client = TokenClient::new(&env, &token_address);
    assert_eq!(token_client.balance(&user), 0);
    assert_eq!(token_client.balance(&airdrop_contract), 500);
}

#[test]
fn test_claim_airdrop_already_claimed_2() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let airdrop_client = AirdropContractClient::new(&env, &airdrop_contract);

    // Setup contracts
    referral_client.initialize(&admin, &token_address);
    airdrop_client.initialize(&admin, &None);
    airdrop_client.register_provider(
        &admin,
        &Symbol::new(&env, "referrals"),
        &referral_contract_id,
    );

    // Create airdrop event with minimal requirements
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 1u64)]);
    let amount = 1000;
    let event_id =
        create_airdrop_event(&airdrop_client, &admin, conditions, amount, &token_address);

    // Setup eligible user
    referral_client.register_with_referral(&user, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user);

    let referred = Address::generate(&env);
    referral_client.register_with_referral(&referred, &user, &String::from_str(&env, "proof2"));
    referral_client.approve_verification(&referred);

    // Fund contract
    token_admin.mint(&airdrop_contract, &10000);

    env.mock_all_auths();

    // First claim should succeed
    airdrop_client.claim_airdrop(&user, &event_id);

    // Second claim should fail
    let result = airdrop_client.try_claim_airdrop(&user, &event_id);
    assert_eq!(result, Err(Ok(AirdropError::AlreadyClaimed)));

    // Verify only one distribution occurred
    let token_client = TokenClient::new(&env, &token_address);
    assert_eq!(token_client.balance(&user), amount);

    env.as_contract(&airdrop_contract, || {
        let stats: EventStats = env
            .storage()
            .persistent()
            .get(&DataKey::EventStats(event_id))
            .unwrap();
        assert_eq!(stats.recipient_count, 1);
        assert_eq!(stats.total_amount_distributed, amount);
    });
}

#[test]
fn test_batch_distribution() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    // Create 5 test users
    let user0 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let user4 = Address::generate(&env);
    let users = Vec::from_array(
        &env,
        [
            user0.clone(),
            user1.clone(),
            user2.clone(),
            user3.clone(),
            user4.clone(),
        ],
    );

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let airdrop_client = AirdropContractClient::new(&env, &airdrop_contract);

    // Setup contracts
    referral_client.initialize(&admin, &token_address);
    airdrop_client.initialize(&admin, &None);
    airdrop_client.register_provider(
        &admin,
        &Symbol::new(&env, "referrals"),
        &referral_contract_id,
    );

    // Create airdrop event requiring 1 referral
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 1u64)]);
    let amount = 1000;
    let event_id =
        create_airdrop_event(&airdrop_client, &admin, conditions, amount, &token_address);

    // Setup referrals for eligible users (0, 1, 3, 4)
    for user in [&user0, &user1, &user3, &user4] {
        referral_client.register_with_referral(user, &admin, &String::from_str(&env, "proof"));
        referral_client.approve_verification(user);

        let referred = Address::generate(&env);
        referral_client.register_with_referral(&referred, user, &String::from_str(&env, "proof2"));
        referral_client.approve_verification(&referred);
    }

    // Setup user2 with no referrals
    referral_client.register_with_referral(&user2, &admin, &String::from_str(&env, "proof"));
    referral_client.approve_verification(&user2);

    // Pre-claim for user3
    token_admin.mint(&airdrop_contract, &amount);
    airdrop_client.claim_airdrop(&user3, &event_id);

    // Fund contract for batch distribution
    token_admin.mint(&airdrop_contract, &(amount * 4));

    env.mock_all_auths();

    // Perform batch distribution
    airdrop_client.distribute_batch(&admin, &event_id, &users);

    // Verify results
    let token_client = TokenClient::new(&env, &token_address);

    // Check balances
    assert_eq!(token_client.balance(&user0), amount); // Eligible, received
    assert_eq!(token_client.balance(&user1), amount); // Eligible, received
    assert_eq!(token_client.balance(&user2), 0); // Not eligible, nothing received
    assert_eq!(token_client.balance(&user3), amount); // Already claimed, no double payment
    assert_eq!(token_client.balance(&user4), amount); // Eligible, received

    // Verify event stats
    env.as_contract(&airdrop_contract, || {
        let stats: EventStats = env
            .storage()
            .persistent()
            .get(&DataKey::EventStats(event_id))
            .unwrap();
        assert_eq!(stats.recipient_count, 4); // 3 from batch + 1 pre-claimed
        assert_eq!(stats.total_amount_distributed, amount * 4);

        // Verify claimed status
        let users_array = [user0, user1, user2, user3, user4];
        for (i, user) in users_array.iter().enumerate() {
            let claimed: bool = env
                .storage()
                .persistent()
                .get(&DataKey::Claimed(event_id, user.clone()))
                .unwrap_or(false);
            if i == 2 {
                assert!(!claimed); // User 2 was not eligible
            } else {
                assert!(claimed); // All others should be marked as claimed
            }
        }
    });
}

// === New Tests ===
#[test]
fn test_provider_management() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let new_provider = Address::generate(&env);
    let metric = Symbol::new(&env, "new_metric");

    let client = AirdropContractClient::new(&env, &airdrop_contract);
    client.initialize(&admin, &None);

    env.mock_all_auths();

    // Register provider
    client.register_provider(&admin, &metric, &new_provider);

    env.as_contract(&airdrop_contract, || {
        let stored: Address = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderRegistry(metric.clone()))
            .unwrap();
        assert_eq!(stored, new_provider);
    });

    // Update provider
    let updated_provider = Address::generate(&env);
    client.update_provider(&admin, &metric, &updated_provider);

    env.as_contract(&airdrop_contract, || {
        let stored: Address = env
            .storage()
            .persistent()
            .get(&DataKey::ProviderRegistry(metric.clone()))
            .unwrap();
        assert_eq!(stored, updated_provider);
    });

    // Remove provider
    client.remove_provider(&admin, &metric);

    env.as_contract(&airdrop_contract, || {
        let exists = env
            .storage()
            .persistent()
            .has(&DataKey::ProviderRegistry(metric.clone()));
        assert!(!exists);
    });

    // Verify get_provider
    let result = client.try_get_provider(&metric);
    assert_eq!(result, Err(Ok(AirdropError::ProviderNotConfigured)));
}

#[test]
fn test_set_admin() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    let client = AirdropContractClient::new(&env, &airdrop_contract);
    client.initialize(&admin, &None);

    env.mock_all_auths();

    // Set new admin
    client.set_admin(&admin, &new_admin);

    env.as_contract(&airdrop_contract, || {
        let stored_admin: Address = env.storage().persistent().get(&DataKey::Admin).unwrap();
        assert_eq!(stored_admin, new_admin);
    });

    // Verify is_admin
    assert!(client.is_admin(&new_admin));
    assert!(!client.is_admin(&admin));
}

#[test]
fn test_list_claimed_users() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let (token_address, token_admin) = setup_token(&env);

    let (referral_client, referral_contract_id, _, _) = setup_contract(&env);
    let client = AirdropContractClient::new(&env, &airdrop_contract);

    // Initialize referral contract
    referral_client.initialize(&admin, &token_address);

    // Set up providers
    let providers = Some(Map::from_array(
        &env,
        [(Symbol::new(&env, "referrals"), referral_contract_id.clone())],
    ));
    client.initialize(&admin, &providers);

    // Create conditions
    let conditions = Map::from_array(&env, [(Symbol::new(&env, "referrals"), 3u64)]);

    // Fund the airdrop contract with tokens
    token_admin.mint(&airdrop_contract, &10000);

    // Create the airdrop event
    let event_id = create_airdrop_event(&client, &admin, conditions, 1000, &token_address);

    // Setup users with sufficient referrals
    for user in [&user1, &user2] {
        referral_client.register_with_referral(user, &admin, &String::from_str(&env, "proof"));
        referral_client.approve_verification(user);
        for _ in 0..3 {
            let referred = Address::generate(&env);
            referral_client.register_with_referral(
                &referred,
                user,
                &String::from_str(&env, "proof"),
            );
            referral_client.approve_verification(&referred);
        }
    }

    env.mock_all_auths();

    // Users claim
    client.claim_airdrop(&user1, &event_id);
    client.claim_airdrop(&user2, &event_id);

    // Test list_claimed_users
    let claimed_users = client.list_claimed_users(&event_id, &3);
    assert_eq!(claimed_users.len(), 2);
    assert!(claimed_users.contains(&user1));
    assert!(claimed_users.contains(&user2));

    // Test with max_results = 1
    let limited_users = client.list_claimed_users(&event_id, &1);
    assert_eq!(limited_users.len(), 1);

    // Test non-existent event
    let result = client.try_list_claimed_users(&999, &3);
    assert_eq!(result, Err(Ok(AirdropError::AirdropNotFound)));
}

#[test]
fn test_get_provider() {
    let (env, airdrop_contract) = create_test_env();
    let admin = Address::generate(&env);
    let provider = Address::generate(&env);
    let metric = Symbol::new(&env, "referrals");

    let client = AirdropContractClient::new(&env, &airdrop_contract);
    client.initialize(&admin, &None);

    env.mock_all_auths();

    // Register provider
    client.register_provider(&admin, &metric, &provider);

    // Verify get_provider
    let retrieved_provider = client.get_provider(&metric);
    assert_eq!(retrieved_provider, provider);

    // Test non-existent provider
    let result = client.try_get_provider(&Symbol::new(&env, "non_existent"));
    assert_eq!(result, Err(Ok(AirdropError::ProviderNotConfigured)));
}
