#![cfg(test)]

use soroban_sdk::{Env, Address, String};
use crate::{StarshopPromoNFT, StarshopPromoNFTClient};
use crate::storage;

fn setup_test_environment<'a>() -> (Env, StarshopPromoNFTClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(StarshopPromoNFT, ());
    let client = StarshopPromoNFTClient::new(&env, &contract_id);
    (env, client)
}

#[test]
fn test_initialize_contract() {
    let (env, client) = setup_test_environment();
    let admin = Address::from_str(&env, "admin");
    let name = String::from_str(&env, "Test NFT");
    let symbol = String::from_str(&env, "TEST");
    let base_uri = String::from_str(&env, "https://test.com");
    client.initialize(&admin, &name, &symbol, &base_uri);

    let initialized = env.storage().persistent().get(&storage::INITIALIZED_KEY).unwrap_or(false);
    assert_eq!(initialized, true);
}
