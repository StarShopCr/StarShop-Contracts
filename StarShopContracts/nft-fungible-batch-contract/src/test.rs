use crate::storage::{DEFAULT_ADMIN_ROLE, MAX_BATCH_SIZE, MINTER_ROLE};
use crate::{BatchNFTContract, BatchNFTContractClient};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup_test<'a>() -> (Env, BatchNFTContractClient<'a>, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BatchNFTContract, ());
    let client = BatchNFTContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    (env, client, admin, user1, user2)
}

fn initialize_contract<'a>(
    client: &BatchNFTContractClient<'a>,
    admin: &Address,
    env: &Env,
) {
    client.initialize(
        admin,
        &String::from_str(env, "StarShop NFT Collection"),
        &String::from_str(env, "SSNFT"),
        &String::from_str(env, "https://api.starshop.io/nft/"),
    );
}

#[test]
fn test_initialization() {
    let (env, client, admin, _user1, _user2) = setup_test();

    initialize_contract(&client, &admin, &env);

    assert_eq!(
        client.name(),
        String::from_str(&env, "StarShop NFT Collection")
    );
    assert_eq!(client.symbol(), String::from_str(&env, "SSNFT"));
    assert_eq!(
        client.base_uri(),
        String::from_str(&env, "https://api.starshop.io/nft/")
    );
    assert_eq!(client.total_supply(), 0);

    assert!(client.has_role(&DEFAULT_ADMIN_ROLE, &admin));
    assert!(client.has_role(&MINTER_ROLE, &admin));
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // AlreadyInitialized
fn test_double_initialization_fails() {
    let (env, client, admin, _user1, _user2) = setup_test();

    initialize_contract(&client, &admin, &env);
    // Second initialization should fail
    initialize_contract(&client, &admin, &env);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")] // NotInitialized
fn test_uninitialized_call_fails() {
    let (_env, client, _admin, user1, _user2) = setup_test();
    client.balance_of(&user1);
}

#[test]
fn test_roles_management() {
    let (env, client, admin, user1, _user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    assert!(!client.has_role(&MINTER_ROLE, &user1));

    // Admin grants MINTER_ROLE to user1
    client.grant_role(&admin, &MINTER_ROLE, &user1);
    assert!(client.has_role(&MINTER_ROLE, &user1));

    // user1 can now mint
    let token_ids = client.mint_batch(&user1, &user1, &3);
    assert_eq!(token_ids.len(), 3);
    assert_eq!(client.balance_of(&user1), 3);

    // Admin revokes MINTER_ROLE from user1
    client.revoke_role(&admin, &MINTER_ROLE, &user1);
    assert!(!client.has_role(&MINTER_ROLE, &user1));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")] // MissingRole
fn test_unauthorized_grant_role_fails() {
    let (env, client, admin, user1, user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    // user1 (not admin) attempts to grant MINTER role
    client.grant_role(&user1, &MINTER_ROLE, &user2);
}

#[test]
fn test_batch_minting() {
    let (env, client, admin, user1, user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    // Admin mints 5 tokens to user1
    let ids1 = client.mint_batch(&admin, &user1, &5);
    assert_eq!(ids1.len(), 5);
    assert_eq!(ids1.get(0).unwrap(), 1);
    assert_eq!(ids1.get(4).unwrap(), 5);
    assert_eq!(client.balance_of(&user1), 5);
    assert_eq!(client.total_supply(), 5);

    // Verify token owners
    for i in 1..=5 {
        assert_eq!(client.owner_of(&i), user1);
        assert_eq!(
            client.token_uri(&i),
            String::from_str(&env, "https://api.starshop.io/nft/")
        );
    }

    // Admin mints 3 tokens to user2
    let ids2 = client.mint_batch(&admin, &user2, &3);
    assert_eq!(ids2.len(), 3);
    assert_eq!(ids2.get(0).unwrap(), 6);
    assert_eq!(ids2.get(2).unwrap(), 8);
    assert_eq!(client.balance_of(&user2), 3);
    assert_eq!(client.total_supply(), 8);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")] // ZeroQuantity
fn test_mint_batch_zero_quantity_fails() {
    let (env, client, admin, user1, _user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    client.mint_batch(&admin, &user1, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")] // BatchLimitExceeded
fn test_mint_batch_exceeds_limit_fails() {
    let (env, client, admin, user1, _user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    client.mint_batch(&admin, &user1, &(MAX_BATCH_SIZE + 1));
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")] // MissingRole
fn test_mint_batch_without_minter_role_fails() {
    let (env, client, admin, user1, user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    // user1 does not have MINTER_ROLE
    client.mint_batch(&user1, &user2, &2);
}

#[test]
fn test_token_transfer() {
    let (env, client, admin, user1, user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    // Mint token #1 to user1
    let ids = client.mint_batch(&admin, &user1, &1);
    let token_id = ids.get(0).unwrap();
    assert_eq!(client.owner_of(&token_id), user1);
    assert_eq!(client.balance_of(&user1), 1);
    assert_eq!(client.balance_of(&user2), 0);

    // user1 transfers to user2
    client.transfer(&user1, &user2, &token_id);
    assert_eq!(client.owner_of(&token_id), user2);
    assert_eq!(client.balance_of(&user1), 0);
    assert_eq!(client.balance_of(&user2), 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")] // Unauthorized
fn test_transfer_not_owner_fails() {
    let (env, client, admin, user1, user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    // Mint token #1 to user1
    let ids = client.mint_batch(&admin, &user1, &1);
    let token_id = ids.get(0).unwrap();

    // user2 attempts to transfer token #1 owned by user1
    client.transfer(&user2, &admin, &token_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")] // TokenNotFound
fn test_transfer_nonexistent_token_fails() {
    let (env, client, admin, user1, user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    client.transfer(&user1, &user2, &999);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")] // SelfTransfer
fn test_transfer_to_self_fails() {
    let (env, client, admin, user1, _user2) = setup_test();
    initialize_contract(&client, &admin, &env);

    let ids = client.mint_batch(&admin, &user1, &1);
    let token_id = ids.get(0).unwrap();

    client.transfer(&user1, &user1, &token_id);
}
