use super::*;
use crate::storage::{DEFAULT_ADMIN_ROLE, MINTER_ROLE, REDEEMER_ROLE};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, Env, String,
};

fn setup_promo_contract<'a>() -> (
    Env,
    StarshopPromoNFTClient<'a>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(StarshopPromoNFT, ());
    let client = StarshopPromoNFTClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    let name = String::from_str(&env, "StarShop Promo NFT");
    let symbol = String::from_str(&env, "SSPROMO");
    let base_uri = String::from_str(&env, "https://api.starshop.io/nft/");

    client.initialize(&admin, &name, &symbol, &base_uri);

    (env, client, admin, user1, user2)
}

#[test]
fn test_initialize_and_default_roles() {
    let (_, client, admin, _, _) = setup_promo_contract();

    assert!(client.has_role(&DEFAULT_ADMIN_ROLE, &admin));
    assert!(client.has_role(&MINTER_ROLE, &admin));
    assert!(client.has_role(&REDEEMER_ROLE, &admin));
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_reinitialize_panics() {
    let (env, client, admin, _, _) = setup_promo_contract();
    let name = String::from_str(&env, "Duplicate");
    let symbol = String::from_str(&env, "DUP");
    let base_uri = String::from_str(&env, "https://dup.io/");

    client.initialize(&admin, &name, &symbol, &base_uri);
}

#[test]
fn test_grant_and_revoke_role_by_admin() {
    let (_, client, admin, user1, _) = setup_promo_contract();

    assert!(!client.has_role(&MINTER_ROLE, &user1));

    client.grant_role(&MINTER_ROLE, &user1, &admin);
    assert!(client.has_role(&MINTER_ROLE, &user1));

    client.revoke_role(&MINTER_ROLE, &user1, &admin);
    assert!(!client.has_role(&MINTER_ROLE, &user1));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_grant_role_by_non_admin_fails() {
    let (_, client, _, user1, user2) = setup_promo_contract();

    // user1 is not admin, attempting to grant MINTER_ROLE to user2 should fail
    client.grant_role(&MINTER_ROLE, &user2, &user1);
}

#[test]
fn test_mint_promo_success_with_custom_metadata() {
    let (env, client, admin, user1, _) = setup_promo_contract();
    let meta_uri = String::from_str(&env, "ipfs://bafybeipromo123");

    let token_id = client.mint_promo(&admin, &user1, &Some(meta_uri.clone()));
    assert_eq!(token_id, 0);

    assert_eq!(client.get_token_metadata(&token_id), Some(meta_uri.clone()));
    assert_eq!(client.token_uri(&token_id), meta_uri);
    assert!(!client.is_redeemed(&token_id));
    assert_eq!(client.redemption_timestamp(&token_id), None);
}

#[test]
fn test_mint_promo_success_without_custom_metadata() {
    let (_, client, admin, user1, _) = setup_promo_contract();

    let token_id = client.mint_promo(&admin, &user1, &None);
    assert_eq!(token_id, 0);

    assert_eq!(client.get_token_metadata(&token_id), None);
    assert!(!client.is_redeemed(&token_id));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_mint_promo_unauthorized_fails() {
    let (_, client, _, user1, user2) = setup_promo_contract();

    // user1 is not a minter
    client.mint_promo(&user1, &user2, &None);
}

#[test]
fn test_sequential_token_minting() {
    let (_, client, admin, user1, user2) = setup_promo_contract();

    let id1 = client.mint_promo(&admin, &user1, &None);
    let id2 = client.mint_promo(&admin, &user2, &None);

    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
}

#[test]
fn test_redeem_by_owner_success() {
    let (env, client, admin, user1, _) = setup_promo_contract();
    let token_id = client.mint_promo(&admin, &user1, &None);

    env.ledger().set_timestamp(1700000000);
    client.redeem(&user1, &token_id);

    assert!(client.is_redeemed(&token_id));
    assert_eq!(client.redemption_timestamp(&token_id), Some(1700000000));
}

#[test]
fn test_redeem_by_authorized_redeemer_role() {
    let (env, client, admin, user1, user2) = setup_promo_contract();
    let token_id = client.mint_promo(&admin, &user1, &None);

    // Grant user2 REDEEMER_ROLE (e.g. shop clerk / cashier)
    client.grant_role(&REDEEMER_ROLE, &user2, &admin);

    env.ledger().set_timestamp(1700000500);
    client.redeem(&user2, &token_id);

    assert!(client.is_redeemed(&token_id));
    assert_eq!(client.redemption_timestamp(&token_id), Some(1700000500));
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_redeem_unauthorized_caller_fails() {
    let (_, client, admin, user1, user2) = setup_promo_contract();
    let token_id = client.mint_promo(&admin, &user1, &None);

    // user2 is neither owner nor has REDEEMER_ROLE
    client.redeem(&user2, &token_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_redeem_already_redeemed_fails() {
    let (_, client, admin, user1, _) = setup_promo_contract();
    let token_id = client.mint_promo(&admin, &user1, &None);

    client.redeem(&user1, &token_id);
    // Second redeem should panic with AlreadyRedeemed (#5)
    client.redeem(&user1, &token_id);
}

#[test]
fn test_unredeemed_nft_transfer_success() {
    let (_, client, admin, user1, user2) = setup_promo_contract();
    let token_id = client.mint_promo(&admin, &user1, &None);

    // Unredeemed token can be transferred
    client.transfer(&user1, &user2, &token_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_redeemed_nft_transfer_restricted_panics() {
    let (_, client, admin, user1, user2) = setup_promo_contract();
    let token_id = client.mint_promo(&admin, &user1, &None);

    client.redeem(&user1, &token_id);
    // Redeemed token transfer must panic with TransferRestricted (#7)
    client.transfer(&user1, &user2, &token_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_redeemed_nft_transfer_from_restricted_panics() {
    let (_, client, admin, user1, user2) = setup_promo_contract();
    let token_id = client.mint_promo(&admin, &user1, &None);

    client.redeem(&user1, &token_id);
    // Redeemed token transfer_from must panic with TransferRestricted (#7)
    client.transfer_from(&admin, &user1, &user2, &token_id);
}
