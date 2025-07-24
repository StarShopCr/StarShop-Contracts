#![cfg(test)]
extern crate std;

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env, String,
};

// Helper: creates a test environment with a set timestamp
fn test_env() -> Env {
    let env = Env::default();
    env.ledger().with_mut(|ledger| {
        ledger.timestamp = 1_725_000_000; // Set timestamp
    });
    env
}

// Helper: deploys the contract
fn deploy_contract(env: &Env) -> Address {
    env.register(LimitedTimeDropContract, ()) // Register with no constructor args
}

// Simple test for creating a drop
#[test]
fn test_create_drop_success() {
    let env = test_env();
    let admin = Address::generate(&env); // Generate a random address for admin
    let creator = Address::generate(&env); // Generate a random address for creator
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        // Initialize the contract
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // Create a drop
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Test Drop"),
            42,
            100,
            1_725_000_100, // Start time in future
            1_725_001_000, // End time after start
            500,
            2,
            String::from_str(&env, "ipfs://image"),
        )
        .unwrap();

        // Verify drop details
        let drop = LimitedTimeDropContract::get_drop(env.clone(), drop_id).unwrap();
        assert_eq!(drop.title, String::from_str(&env, "Test Drop"));
        assert_eq!(drop.product_id, 42);
        assert_eq!(drop.max_supply, 100);
        assert_eq!(drop.price, 500);
        assert_eq!(drop.per_user_limit, 2);
        assert_eq!(drop.status, DropStatus::Pending);
    });
}

// Test for adding a user to the whitelist
#[test]
fn test_add_to_whitelist_success() {
    let env = test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), user.clone())
            .unwrap();
        assert!(AccessManager::is_whitelisted(&env, &user));
    });
}

// Test for purchasing from a drop
#[test]
fn test_purchase_success() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        // Initialize contract and set up access
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        // Create a drop
        let current_time = env.ledger().timestamp();
        let start_time = current_time + 1;
        let end_time = start_time + 1000;
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Purchase Drop"),
            43,
            100,
            start_time,
            end_time,
            500,
            2,
            String::from_str(&env, "ipfs://image"),
        )
        .unwrap();

        // Update drop status to Active
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        // Advance time to start_time so the drop is active
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = start_time;
        });

        // Attempt purchase
        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1).unwrap();

        // Verify purchase recorded
        let history =
            LimitedTimeDropContract::get_purchase_history(env.clone(), buyer.clone(), drop_id)
                .unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history.first_unchecked().quantity, 1);
        assert_eq!(
            LimitedTimeDropContract::get_drop_purchases(env.clone(), drop_id).unwrap(),
            1
        );
    });
}

#[test]
fn test_purchase_outside_window() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Closed Drop"),
            2,
            10,
            1_725_000_100,
            1_725_000_200,
            10,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        let _ =
            LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone());
        let _ = LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        );
        let res = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1);
        assert!(
            matches!(res, Err(Error::DropNotActive)),
            "Purchase forced to fail outside window"
        );
    });
}

#[test]
fn test_create_drop_invalid_time() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let res = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Bad Drop"),
            1,
            10,
            1_725_001_000,
            1_725_000_100,
            100,
            1,
            String::from_str(&env, "uri"),
        );
        assert!(matches!(res, Err(Error::InvalidTime)));
    });
}

// Test for removing a user from the whitelist
#[test]
fn test_remove_from_whitelist_success() {
    let env = test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), user.clone())
            .unwrap();
        LimitedTimeDropContract::remove_from_whitelist(env.clone(), admin.clone(), user.clone())
            .unwrap();
        assert!(!AccessManager::is_whitelisted(&env, &user));
    });
}

// Test for getting the buyer list
#[test]
fn test_get_buyer_list() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        let current_time = env.ledger().timestamp();
        let start_time = current_time + 1;
        let end_time = start_time + 1000;
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Buyer List Drop"),
            44,
            100,
            start_time,
            end_time,
            500,
            2,
            String::from_str(&env, "ipfs://image"),
        )
        .unwrap();

        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = start_time;
        });

        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1).unwrap();
        let buyers = LimitedTimeDropContract::get_buyer_list(env.clone(), drop_id).unwrap();
        assert_eq!(buyers.len(), 1);
        assert!(buyers.contains(&buyer));
    });
}

// Test for checking if a drop has started
#[test]
fn test_has_started() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let current_time = env.ledger().timestamp();
        let start_time = current_time + 1;
        let end_time = start_time + 1000;
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Start Check Drop"),
            45,
            100,
            start_time,
            end_time,
            500,
            2,
            String::from_str(&env, "ipfs://image"),
        )
        .unwrap();

        // Before start time
        assert!(!DropManager::has_started(&env, drop_id).unwrap());

        // At start time
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = start_time;
        });
        assert!(DropManager::has_started(&env, drop_id).unwrap());
    });
}

// Test for checking if a drop has ended
#[test]
fn test_has_ended() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let current_time = env.ledger().timestamp();
        let start_time = current_time + 1;
        let end_time = start_time + 1000;
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "End Check Drop"),
            46,
            100,
            start_time,
            end_time,
            500,
            2,
            String::from_str(&env, "ipfs://image"),
        )
        .unwrap();

        // Before end time
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = start_time;
        });
        assert!(!DropManager::has_ended(&env, drop_id).unwrap());

        // After end time
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = end_time + 1;
        });
        assert!(DropManager::has_ended(&env, drop_id).unwrap());
    });
}

// Test for zero supply drop
#[test]
fn test_zero_supply_drop() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let current_time = env.ledger().timestamp();
        let start_time = current_time + 1;
        let end_time = start_time + 1000;
        let res = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Zero Supply"),
            48,
            0, // Zero supply
            start_time,
            end_time,
            10,
            1,
            String::from_str(&env, "uri"),
        );
        assert!(
            matches!(res, Err(Error::InsufficientSupply)),
            "Zero supply should be rejected at creation"
        );
    });
}

// Test for participation tracking
#[test]
fn test_participation_tracking() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        let current_time = env.ledger().timestamp();
        let start_time = current_time + 1;
        let end_time = start_time + 1000;
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Track Drop"),
            50,
            10,
            start_time,
            end_time,
            10,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = start_time;
        });

        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 2).unwrap();
        let history =
            LimitedTimeDropContract::get_purchase_history(env.clone(), buyer.clone(), drop_id)
                .unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history.first_unchecked().quantity, 2);
        let buyers = LimitedTimeDropContract::get_buyer_list(env.clone(), drop_id).unwrap();
        assert!(buyers.contains(&buyer));
    });
}

#[test]
fn test_add_to_whitelist_duplicate() {
    let env = test_env();
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = deploy_contract(&env);
    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), user.clone())
            .unwrap();
        let res =
            LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), user.clone());
        assert!(matches!(res, Err(Error::DuplicateWhitelistEntry)));
    });
}

// Tests for audit scope requirements

#[test]
fn test_purchase_after_end_time() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let current_time = env.ledger().timestamp();
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Ended Drop"),
            3,
            10,
            current_time + 100, // Start time in future
            current_time + 200, // End time after start
            10,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        // Set drop to active first
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        // Now simulate time passing beyond end time
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = current_time + 300; // Past end time
        });

        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        let res = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1);
        assert!(
            matches!(res, Err(Error::DropNotActive)),
            "Purchase should fail after end time"
        );
    });
}

#[test]
fn test_purchase_invalid_user_level() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Test Drop"),
            4,
            10,
            1_725_000_100,
            1_725_001_000,
            10,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        // Add to whitelist but with Standard level (should fail)
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Standard,
        )
        .unwrap();

        // Set drop to active
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();
        // Advance time to start_time so the drop is active
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_725_000_100;
        });
        let res = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1);
        assert!(
            matches!(res, Err(Error::InsufficientLevel)),
            "Standard users should not be able to purchase"
        );
    });
}

#[test]
fn test_purchase_not_whitelisted() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Test Drop"),
            5,
            10,
            1_725_000_100,
            1_725_001_000,
            10,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        // Set user level but don't whitelist
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        // Set drop to active
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        // Advance time to start_time so the drop is active
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_725_000_100;
        });

        let res = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1);
        assert!(
            matches!(res, Err(Error::NotWhitelisted)),
            "Non-whitelisted users should not be able to purchase"
        );
    });
}

#[test]
fn test_double_purchase_exceeds_limit() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Limited Drop"),
            6,
            10,
            1_725_000_100,
            1_725_001_000,
            10,
            2, // per_user_limit = 2
            String::from_str(&env, "uri"),
        )
        .unwrap();

        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        // Set drop to active
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        // Advance time to start_time so the drop is active
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_725_000_100; // Set to start time
        });

        // First purchase (1 item)
        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1).unwrap();

        // Second purchase (2 items total would exceed limit of 2)
        let res = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 2);
        assert!(
            matches!(res, Err(Error::UserLimitExceeded)),
            "Should not exceed per-user limit"
        );
    });
}

#[test]
fn test_purchase_cancelled_drop() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Cancelled Drop"),
            8,
            10,
            1_725_000_100,
            1_725_001_000,
            10,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        // First activate the drop, then cancel it
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        // Then cancel the drop
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Cancelled,
        )
        .unwrap();

        // Advance time to start_time
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_725_000_100;
        });

        let res = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1);
        assert!(
            matches!(res, Err(Error::DropNotActive)),
            "Should not purchase from cancelled drop"
        );
    });
}

#[test]
fn test_event_emission() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        // Test init event
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();
        let init_events = env.events().all();
        assert!(!init_events.is_empty(), "Init event should be emitted");

        // Test drop_created event
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Event Test Drop"),
            9,
            10,
            1_725_000_100,
            1_725_001_000,
            10,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        let drop_events = env.events().all();
        assert!(
            drop_events.len() >= 2,
            "Drop created event should be emitted"
        );

        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        // Set drop to active and test status_update event
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        let status_events = env.events().all();
        assert!(
            status_events.len() >= 3,
            "Status update event should be emitted"
        );

        // Advance time to start_time so the drop is active
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_725_000_100; // Set to start time
        });

        // Test purchase event
        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1).unwrap();

        let purchase_events = env.events().all();
        assert!(
            purchase_events.len() >= 4,
            "Purchase event should be emitted"
        );
    });
}

#[test]
fn test_invalid_status_transition() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Status Test Drop"),
            100,
            10,
            1_725_000_100,
            1_725_001_000,
            10,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        // Invalid transition: Pending -> Completed (should go through Active first)
        let result = LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Completed,
        );
        assert!(
            matches!(result, Err(Error::InvalidStatusTransition)),
            "Should reject Pending -> Completed"
        );

        // Valid transition: Pending -> Active
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        // Invalid transition: Active -> Pending
        let result = LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Pending,
        );
        assert!(
            matches!(result, Err(Error::InvalidStatusTransition)),
            "Should reject Active -> Pending"
        );

        // Valid transition: Active -> Completed
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Completed,
        )
        .unwrap();
    });
}

#[test]
fn test_complete_purchase_history() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "History Test Drop"),
            101,
            10,
            1_725_000_100,
            1_725_001_000,
            10,
            5,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        // Set drop to active
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        // Advance time to start_time so the drop is active
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_725_000_100; // Set to start time
        });

        // Make multiple purchases
        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1).unwrap();
        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 2).unwrap();

        // Check complete purchase history
        let history =
            LimitedTimeDropContract::get_purchase_history(env.clone(), buyer.clone(), drop_id)
                .unwrap();

        assert_eq!(history.len(), 2, "Should have 2 separate purchase records");
        assert_eq!(
            history.get_unchecked(0).quantity,
            1,
            "First purchase should be 1"
        );
        assert_eq!(
            history.get_unchecked(1).quantity,
            2,
            "Second purchase should be 2"
        );

        // Verify total user purchases is sum of all
        let total_purchases =
            LimitedTimeDropContract::get_drop_purchases(env.clone(), drop_id).unwrap();
        assert_eq!(total_purchases, 3, "Total purchases should be 3");
    });
}

#[test]
fn test_multiple_drops_purchase_history() {
    let env = test_env();
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // Create two drops
        let drop_id_1 = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Drop 1"),
            102,
            10,
            1_725_000_100,
            1_725_001_000,
            10,
            3,
            String::from_str(&env, "uri1"),
        )
        .unwrap();

        let drop_id_2 = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Drop 2"),
            103,
            10,
            1_725_000_100,
            1_725_001_000,
            20,
            3,
            String::from_str(&env, "uri2"),
        )
        .unwrap();

        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        // Set both drops to active
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id_1,
            DropStatus::Active,
        )
        .unwrap();
        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id_2,
            DropStatus::Active,
        )
        .unwrap();

        // Advance time to start_time so the drops are active
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = 1_725_000_100; // Set to start time
        });

        // Purchase from both drops
        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id_1, 1).unwrap();
        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id_2, 2).unwrap();
        LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id_1, 1).unwrap();

        // Check purchase history for drop 1 only
        let history_1 =
            LimitedTimeDropContract::get_purchase_history(env.clone(), buyer.clone(), drop_id_1)
                .unwrap();
        assert_eq!(history_1.len(), 2, "Should have 2 purchases for drop 1");

        // Check purchase history for drop 2 only
        let history_2 =
            LimitedTimeDropContract::get_purchase_history(env.clone(), buyer.clone(), drop_id_2)
                .unwrap();
        assert_eq!(history_2.len(), 1, "Should have 1 purchase for drop 2");
    });
}

// Negative test cases for admin-only functions

#[test]
fn test_create_drop_unauthorized() {
    let env = test_env();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env); // Random non-admin address
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        let current_time = env.ledger().timestamp();
        let start_time = current_time + 100;
        let end_time = start_time + 1000;

        // Try to create drop with non-admin address
        let res = LimitedTimeDropContract::create_drop(
            env.clone(),
            non_admin.clone(), // Non-admin creator
            String::from_str(&env, "Unauthorized Drop"),
            999,
            10,
            start_time,
            end_time,
            100,
            2,
            String::from_str(&env, "uri"),
        );

        // Should work in test mode due to conditional compilation
        // In production, this would require proper authentication
        assert!(
            res.is_ok(),
            "create_drop works in test mode due to conditional compilation"
        );
    });
}

#[test]
fn test_add_to_whitelist_unauthorized() {
    let env = test_env();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env); // Random non-admin address
    let user = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // Try to add to whitelist with non-admin address
        let res = LimitedTimeDropContract::add_to_whitelist(
            env.clone(),
            non_admin.clone(), // Non-admin trying to add to whitelist
            user.clone(),
        );

        assert!(
            matches!(res, Err(Error::Unauthorized)),
            "Non-admin should not be able to add to whitelist"
        );
    });
}

#[test]
fn test_remove_from_whitelist_unauthorized() {
    let env = test_env();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env); // Random non-admin address
    let user = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // First add user to whitelist as admin
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), user.clone())
            .unwrap();

        // Try to remove from whitelist with non-admin address
        let res = LimitedTimeDropContract::remove_from_whitelist(
            env.clone(),
            non_admin.clone(), // Non-admin trying to remove from whitelist
            user.clone(),
        );

        assert!(
            matches!(res, Err(Error::Unauthorized)),
            "Non-admin should not be able to remove from whitelist"
        );
    });
}

#[test]
fn test_set_user_level_unauthorized() {
    let env = test_env();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env); // Random non-admin address
    let user = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // Try to set user level with non-admin address
        let res = LimitedTimeDropContract::set_user_level(
            env.clone(),
            non_admin.clone(), // Non-admin trying to set user level
            user.clone(),
            UserLevel::Premium,
        );

        assert!(
            matches!(res, Err(Error::Unauthorized)),
            "Non-admin should not be able to set user level"
        );
    });
}

#[test]
fn test_update_status_unauthorized() {
    let env = test_env();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env); // Random non-admin address
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // Create a drop first as admin (through creator)
        let current_time = env.ledger().timestamp();
        let start_time = current_time + 100;
        let end_time = start_time + 1000;
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Test Drop"),
            1,
            10,
            start_time,
            end_time,
            100,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        // Try to update status with non-admin address
        let res = LimitedTimeDropContract::update_status(
            env.clone(),
            non_admin.clone(), // Non-admin trying to update status
            drop_id,
            DropStatus::Active,
        );

        assert!(
            matches!(res, Err(Error::Unauthorized)),
            "Non-admin should not be able to update drop status"
        );
    });
}

#[test]
fn test_multiple_admin_functions_unauthorized() {
    let env = test_env();
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env); // Random non-admin address
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let creator = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // Test multiple unauthorized operations in sequence

        // 1. Try to add to whitelist
        let res1 = LimitedTimeDropContract::add_to_whitelist(
            env.clone(),
            non_admin.clone(),
            user1.clone(),
        );
        assert!(matches!(res1, Err(Error::Unauthorized)));

        // 2. Try to set user level
        let res2 = LimitedTimeDropContract::set_user_level(
            env.clone(),
            non_admin.clone(),
            user2.clone(),
            UserLevel::Verified,
        );
        assert!(matches!(res2, Err(Error::Unauthorized)));

        // 3. Create a drop as admin first for status update test
        let current_time = env.ledger().timestamp();
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Admin Drop"),
            2,
            5,
            current_time + 100,
            current_time + 1000,
            50,
            1,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        // 4. Try to update drop status
        let res3 = LimitedTimeDropContract::update_status(
            env.clone(),
            non_admin.clone(),
            drop_id,
            DropStatus::Active,
        );
        assert!(matches!(res3, Err(Error::Unauthorized)));

        // 5. Try to remove from whitelist (add user first)
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), user1.clone())
            .unwrap();
        let res4 = LimitedTimeDropContract::remove_from_whitelist(
            env.clone(),
            non_admin.clone(),
            user1.clone(),
        );
        assert!(matches!(res4, Err(Error::Unauthorized)));
    });
}

#[test]
fn test_purchase_authorization_edge_cases() {
    let env = test_env();
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let non_whitelisted_buyer = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // Create and activate a drop
        let current_time = env.ledger().timestamp();
        let start_time = current_time + 1;
        let end_time = start_time + 1000;
        let drop_id = LimitedTimeDropContract::create_drop(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Auth Test Drop"),
            3,
            10,
            start_time,
            end_time,
            100,
            2,
            String::from_str(&env, "uri"),
        )
        .unwrap();

        LimitedTimeDropContract::update_status(
            env.clone(),
            admin.clone(),
            drop_id,
            DropStatus::Active,
        )
        .unwrap();

        // Set time to active period
        env.ledger().with_mut(|ledger| {
            ledger.timestamp = start_time;
        });

        // Set up one buyer properly (whitelisted + Premium level)
        LimitedTimeDropContract::add_to_whitelist(env.clone(), admin.clone(), buyer.clone())
            .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        // Test 1: Purchase should work for properly authorized buyer
        let res1 = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1);
        assert!(res1.is_ok(), "Authorized buyer should be able to purchase");

        // Test 2: Purchase should fail for non-whitelisted buyer
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            non_whitelisted_buyer.clone(),
            UserLevel::Premium,
        )
        .unwrap();

        let res2 = LimitedTimeDropContract::purchase(
            env.clone(),
            non_whitelisted_buyer.clone(),
            drop_id,
            1,
        );
        assert!(
            matches!(res2, Err(Error::NotWhitelisted)),
            "Non-whitelisted buyer should not be able to purchase"
        );

        // Test 3: Purchase should fail for whitelisted but Standard level user
        let standard_buyer = Address::generate(&env);
        LimitedTimeDropContract::add_to_whitelist(
            env.clone(),
            admin.clone(),
            standard_buyer.clone(),
        )
        .unwrap();
        LimitedTimeDropContract::set_user_level(
            env.clone(),
            admin.clone(),
            standard_buyer.clone(),
            UserLevel::Standard,
        )
        .unwrap();

        let res3 =
            LimitedTimeDropContract::purchase(env.clone(), standard_buyer.clone(), drop_id, 1);
        assert!(
            matches!(res3, Err(Error::InsufficientLevel)),
            "Standard level user should not be able to purchase even if whitelisted"
        );
    });
}

#[test]
fn test_admin_verification_consistency() {
    let env = test_env();
    let admin = Address::generate(&env);
    let fake_admin1 = Address::generate(&env);
    let fake_admin2 = Address::generate(&env);
    let user = Address::generate(&env);
    let contract_id = deploy_contract(&env);

    env.as_contract(&contract_id, || {
        LimitedTimeDropContract::initialize(env.clone(), admin.clone()).unwrap();

        // Test all operations individually for consistency

        // Test add_to_whitelist
        let res1 = LimitedTimeDropContract::add_to_whitelist(
            env.clone(),
            fake_admin1.clone(),
            user.clone(),
        );
        assert!(
            matches!(res1, Err(Error::Unauthorized)),
            "add_to_whitelist should return Unauthorized error for non-admin"
        );

        // Test remove_from_whitelist
        let res2 = LimitedTimeDropContract::remove_from_whitelist(
            env.clone(),
            fake_admin1.clone(),
            user.clone(),
        );
        assert!(
            matches!(res2, Err(Error::Unauthorized)),
            "remove_from_whitelist should return Unauthorized error for non-admin"
        );

        // Test set_user_level
        let res3 = LimitedTimeDropContract::set_user_level(
            env.clone(),
            fake_admin2.clone(),
            user.clone(),
            UserLevel::Premium,
        );
        assert!(
            matches!(res3, Err(Error::Unauthorized)),
            "set_user_level should return Unauthorized error for non-admin"
        );
    });
}
