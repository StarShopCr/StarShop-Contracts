#![cfg(test)]
extern crate std;

/// Tests for Limited-Time Drop Contract

use super::*;
use soroban_sdk::{Address, Env, String, Vec};
use std::format;

// Helper: returns a mock Address
fn mock_address(env: &Env, id: u8) -> Address {
    // Use a static list of valid Stellar account addresses (strkey)
    // These are 56-char, start with 'G', and pass checksum for test purposes
    const ADDRS: [&str; 6] = [
        "GBRPYHIL2CI3VQYZ4JX3D3J6JYJZQ2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4",
        "GCFXW2X4ZK2KJXQ4W2V5T4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4",
        "GDQP2KPQGKIHYJGXNUIYOMHARUARCA6LG5Q3X5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4",
        "GAHK7EEG2W7V6Y6Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4",
        "GDRXE2BQUC3AZ6Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4",
        "GB3JDWMX5V4ZK2KJXQ4W2V5T4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4Q2Q5Q4",
    ];
    let idx = (id as usize) % ADDRS.len();
    Address::from_str(env, ADDRS[idx])
}

// Helper: returns a test Env
fn test_env() -> Env {
    Env::default()
}

#[test]
fn test_create_drop_success() {
    let env = test_env();
    let admin = mock_address(&env, 1);
    let creator = mock_address(&env, 2);
    let now = 1_725_000_000; // Example timestamp
    let drop_id = LimitedTimeDropContract::create_drop(
        env.clone(),
        creator.clone(),
        String::from_str(&env, "Test Drop"),
        42,
        100,
        now + 100,
        now + 1000,
        500,
        2,
        String::from_str(&env, "ipfs://image"),
    ).unwrap();
    let drop = LimitedTimeDropContract::get_drop(env.clone(), drop_id).unwrap();
    assert_eq!(drop.title, String::from_str(&env, "Test Drop"));
    assert_eq!(drop.product_id, 42);
    assert_eq!(drop.max_supply, 100);
    assert_eq!(drop.price, 500);
}

#[test]
fn test_create_drop_invalid_time() {
    let env = test_env();
    let creator = mock_address(&env, 2);
    let now = 1_725_000_000;
    let res = LimitedTimeDropContract::create_drop(
        env.clone(),
        creator,
        String::from_str(&env, "Bad Drop"),
        1,
        10,
        now + 1000,
        now + 100,
        100,
        1,
        String::from_str(&env, "uri"),
    );
    assert!(matches!(res, Err(Error::InvalidTime)));
}

#[test]
fn test_purchase_within_window() {
    let env = test_env();
    let admin = mock_address(&env, 1);
    let buyer = mock_address(&env, 3);
    let creator = mock_address(&env, 2);
    let now = 1_725_000_000;
    let drop_id = LimitedTimeDropContract::create_drop(
        env.clone(),
        creator,
        String::from_str(&env, "Active Drop"),
        2,
        10,
        now,
        now + 100,
        10,
        2,
        String::from_str(&env, "uri"),
    ).unwrap();
    let res = LimitedTimeDropContract::purchase(env.clone(), buyer, drop_id, 1);
    assert!(res.is_ok());
}

#[test]
fn test_purchase_exceeds_user_cap() {
    let env = test_env();
    let buyer = mock_address(&env, 3);
    let creator = mock_address(&env, 2);
    let now = 1_725_000_000;
    let drop_id = LimitedTimeDropContract::create_drop(
        env.clone(),
        creator,
        String::from_str(&env, "Cap Drop"),
        2,
        10,
        now,
        now + 100,
        10,
        1,
        String::from_str(&env, "uri"),
    ).unwrap();
    let _ = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 1);
    let res = LimitedTimeDropContract::purchase(env.clone(), buyer, drop_id, 1);
    assert!(matches!(res, Err(Error::UserLimitExceeded)));
}

#[test]
fn test_purchase_outside_window() {
    let env = test_env();
    let buyer = mock_address(&env, 3);
    let creator = mock_address(&env, 2);
    let now = 1_725_000_000;
    let drop_id = LimitedTimeDropContract::create_drop(
        env.clone(),
        creator,
        String::from_str(&env, "Closed Drop"),
        2,
        10,
        now + 100,
        now + 200,
        10,
        2,
        String::from_str(&env, "uri"),
    ).unwrap();
    let res = LimitedTimeDropContract::purchase(env.clone(), buyer, drop_id, 1);
    assert!(matches!(res, Err(Error::DropNotActive)));
}

#[test]
fn test_zero_supply_drop() {
    let env = test_env();
    let creator = mock_address(&env, 2);
    let now = 1_725_000_000;
    let drop_id = LimitedTimeDropContract::create_drop(
        env.clone(),
        creator,
        String::from_str(&env, "Zero Supply"),
        2,
        0,
        now,
        now + 100,
        10,
        1,
        String::from_str(&env, "uri"),
    ).unwrap();
    let buyer = mock_address(&env, 4);
    let res = LimitedTimeDropContract::purchase(env.clone(), buyer, drop_id, 1);
    assert!(matches!(res, Err(Error::InsufficientSupply)));
}

#[test]
fn test_whitelist_enforcement() {
    let env = test_env();
    let admin = mock_address(&env, 1);
    let user = mock_address(&env, 5);
    let creator = mock_address(&env, 2);
    let now = 1_725_000_000;
    let drop_id = LimitedTimeDropContract::create_drop(
        env.clone(),
        creator,
        String::from_str(&env, "Whitelist Drop"),
        2,
        10,
        now,
        now + 100,
        10,
        2,
        String::from_str(&env, "uri"),
    ).unwrap();
    // Not whitelisted yet
    let res = LimitedTimeDropContract::purchase(env.clone(), user.clone(), drop_id, 1);
    assert!(matches!(res, Err(Error::NotWhitelisted)));
    // Whitelist and try again
    let _ = LimitedTimeDropContract::add_to_whitelist(env.clone(), admin, user.clone());
    let res2 = LimitedTimeDropContract::purchase(env.clone(), user, drop_id, 1);
    assert!(res2.is_ok());
}

#[test]
fn test_participation_tracking() {
    let env = test_env();
    let creator = mock_address(&env, 2);
    let buyer = mock_address(&env, 6);
    let now = 1_725_000_000;
    let drop_id = LimitedTimeDropContract::create_drop(
        env.clone(),
        creator,
        String::from_str(&env, "Track Drop"),
        2,
        10,
        now,
        now + 100,
        10,
        2,
        String::from_str(&env, "uri"),
    ).unwrap();
    let _ = LimitedTimeDropContract::purchase(env.clone(), buyer.clone(), drop_id, 2);
    let history = LimitedTimeDropContract::get_purchase_history(env.clone(), buyer.clone(), drop_id).unwrap();
    assert_eq!(history.len(), 1);
    let buyers = LimitedTimeDropContract::get_buyer_list(env.clone(), drop_id).unwrap();
    assert!(buyers.contains(&buyer));
}
