#![cfg(test)]
extern crate std;

use super::*;
use crate::{
    refund::{RefundContract, RefundContractClient},
    transaction::TransactionContractClient,
};
use soroban_sdk::token::{StellarAssetClient as TokenAdmin, TokenClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, Symbol,
};

// Create a simulated WASM for tests
fn install_new_wasm(e: &Env) -> BytesN<32> {
    // Generate random bytes as a simulated WASM hash
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = i as u8;
    }
    BytesN::from_array(e, &bytes)
}

#[test]
fn test_process_deposit_with_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(TransactionContract, ());
    let client = TransactionContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);

    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let sender = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Mint tokens to sender
    token.mint(&sender, &1000);

    // Execute transaction
    client.process_deposit(&token_contract_id.clone(), &sender, &recipient, &100);

    // Verify signed transactions
    assert_eq!(
        env.auths(),
        std::vec![(
            sender.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "process_deposit"),
                    (
                        token_contract_id.clone(),
                        sender.clone(),
                        recipient.clone(),
                        100_i128
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token_contract_id.clone(),
                        symbol_short!("transfer"),
                        (sender.clone(), recipient.clone(), 100_i128).into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }]
            }
        )]
    );

    // Verify balances
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&sender), 900);
    assert_eq!(token_client.balance(&recipient), 100);
}

#[test]
fn test_initialize() {
    // Create a mock environment
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentContract, ());
    let client = PaymentContractClient::new(&env, &contract_id);

    // Create an admin address
    let admin = Address::generate(&env);

    // Initialize the contract
    client.initialize(&admin);

    // Verify the admin is set correctly
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin, "Admin address should match");
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_get_admin_before_initialize() {
    let env = Env::default();

    let contract_id = env.register(PaymentContract, ());
    let client = PaymentContractClient::new(&env, &contract_id);

    client.get_admin();
}

#[test]
#[should_panic(expected = "Error(Storage, MissingValue)")]
fn test_successful_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentContract, ());
    let client = PaymentContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    let new_wasm_hash = install_new_wasm(&env);

    // Initialize first
    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);

    // This should panic with a MissingValue error since the WASM hash doesn't exist in storage
    client.upgrade(&new_wasm_hash);
}

#[test]
#[should_panic(expected = "Error(Storage, MissingValue)")]
fn test_upgrade_with_empty_hash() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentContract, ());
    let client = PaymentContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);

    // Initialize first
    client.initialize(&admin);
    assert_eq!(client.get_admin(), admin);

    // Attempt to upgrade with an empty hash
    assert_eq!(client.upgrade(&BytesN::from_array(&env, &[0; 32])), ());
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_upgrade_not_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentContract, ());
    let client = PaymentContractClient::new(&env, &contract_id);

    let new_wasm_hash = install_new_wasm(&env);

    // Attempt to upgrade without initializing
    assert_eq!(client.upgrade(&new_wasm_hash), ());
}

#[test]
fn test_succesful_transfer_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentContract, ());
    let client = PaymentContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    // Initialize contract
    client.initialize(&admin);

    // Test admin transfer
    client.transfer_admin(&new_admin);

    // Verify authorization
    let auths = env.auths();
    assert_eq!(auths.len(), 2);

    // Verify new admin is set
    assert_eq!(client.get_admin(), new_admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_reinitialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentContract, ());
    let client = PaymentContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let new_admin = Address::generate(&env);

    // Create an admin address
    client.initialize(&admin);

    // Attempt to re-initialize (should fail)
    client.initialize(&new_admin);
}

#[test]
fn test_process_refund_success() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RefundContract, ());
    let client = RefundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();

    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Mint tokens to the seller
    token.mint(&seller, &1000);

    // Process refund
    let refund_amount = 100;
    client.process_refund(&token_contract_id, &seller, &buyer, &refund_amount);

    // Verify balances
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&seller), 900);
    assert_eq!(token_client.balance(&buyer), 100);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_process_refund_to_self() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RefundContract, ());
    let client = RefundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();

    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup account
    let seller = Address::generate(&env);

    // Mint tokens to the seller
    token.mint(&seller, &1000);

    client.process_refund(&token_contract_id, &seller, &seller, &100);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_process_refund_insufficient_funds() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RefundContract, ());
    let client = RefundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();

    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup accounts
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Mint tokens to the seller (insufficient)
    token.mint(&seller, &50);

    client.process_refund(&token_contract_id, &seller, &buyer, &100);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_process_refund_invalid_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RefundContract, ());
    let client = RefundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();

    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let seller = Address::generate(&env);
    let buyer = Address::generate(&env);

    // Mint tokens to the seller
    token.mint(&seller, &50);

    client.process_refund(&token_contract_id, &seller, &buyer, &0);
}

#[test]
#[should_panic(expected = "Unauthorized function call for address")]
fn test_panic_resolve_dispute_not_authenticated() {
    // Initialize the environment and contract
    let env = Env::default();
    let contract_id = env.register(DisputeContract, ());
    let client = DisputeContractClient::new(&env, &contract_id);

    // Generate test addresses for different roles
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let refund_amount = 1000i128;

    // Set up the token contract and mint tokens to arbitrator
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_contract.address();

    let _token_client = TokenClient::new(&env, &token_address);
    let token_asset_client = TokenAdmin::new(&env, &token_address);
    token_asset_client.mint(&arbitrator, &refund_amount);

    // This should fail because we haven't mocked the authentication
    client.resolve_dispute(
        &token_address,
        &arbitrator,
        &buyer,
        &seller,
        &refund_amount,
        &DisputeDecision::RefundBuyer,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_resolve_dispute_insufficient_funds() {
    // Initialize environment with mocked authentication
    let env = Env::default();
    let contract_id = env.register(DisputeContract, ());
    let client = DisputeContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    // Set up test addresses and amounts
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let refund_amount = 1000i128; // Trying to refund 1000 tokens

    // Set up token contract
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_contract.address();
    let _token_client = TokenClient::new(&env, &token_address);
    let token_asset_client = TokenAdmin::new(&env, &token_address);

    // Mint only 100 tokens - less than the refund amount
    token_asset_client.mint(&arbitrator, &100i128);

    // Simulate resolving a dispute in favor of the buyer
    client.resolve_dispute(
        &token_address,
        &arbitrator,
        &buyer,
        &seller,
        &refund_amount,
        &DisputeDecision::RefundBuyer,
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_resolve_dispute_invalid_amount() {
    // Initialize environment with mocked authentication
    let env = Env::default();
    let contract_id = env.register(DisputeContract, ());
    let client = DisputeContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    // Set up test addresses
    let admin = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let arbitrator = Address::generate(&env);
    let invalid_refund_amount = 0i128; // Invalid amount (zero)

    // Set up token contract
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_contract.address();

    let _token_client = TokenClient::new(&env, &token_address);
    let token_asset_client = TokenAdmin::new(&env, &token_address);
    token_asset_client.mint(&arbitrator, &invalid_refund_amount);

    // This should fail due to invalid amount (zero)
    client.resolve_dispute(
        &token_address,
        &arbitrator,
        &buyer,
        &seller,
        &invalid_refund_amount,
        &DisputeDecision::RefundBuyer,
    );
}

#[test]
fn test_resolve_dispute_refund_buyer() {
    // Initialize environment with mocked authentication
    let env = Env::default();
    let contract_id = env.register(DisputeContract, ());
    let client = DisputeContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    // Set up test addresses and amount
    let admin = Address::generate(&env);
    let buyer = <Address>::generate(&env);
    let seller = <Address>::generate(&env);
    let arbitrator = <Address>::generate(&env);
    let refund_amount = 1000i128;

    // Mock the token client for transferring tokens
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_contract.address();

    let token_client = TokenClient::new(&env, &token_address);
    let token_asset_client = TokenAdmin::new(&env, &token_address);
    token_asset_client.mint(&arbitrator, &refund_amount);

    // Verify initial balances
    let arbitrator_balance_before = token_client.balance(&arbitrator);
    assert_eq!(arbitrator_balance_before, refund_amount);

    let buyer_balance_before = token_client.balance(&buyer);
    assert_eq!(buyer_balance_before, 0);

    let seller_balance_before = token_client.balance(&seller);
    assert_eq!(seller_balance_before, 0);

    // Simulate resolving a dispute in favor of the buyer
    client.resolve_dispute(
        &token_address.clone(),
        &arbitrator,
        &buyer,
        &seller,
        &refund_amount,
        &DisputeDecision::RefundBuyer,
    );

    // Verify signed transactions
    assert_eq!(
        env.auths(),
        std::vec![(
            arbitrator.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "resolve_dispute"),
                    (
                        token_address.clone(),
                        arbitrator.clone(),
                        buyer.clone(),
                        seller.clone(),
                        refund_amount,
                        DisputeDecision::RefundBuyer as u32
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token_address.clone(),
                        symbol_short!("transfer"),
                        (arbitrator.clone(), buyer.clone(), refund_amount).into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }]
            }
        )]
    );

    // Check the balance after the transfer
    let arbitrator_balance_after = token_client.balance(&arbitrator);
    assert_eq!(arbitrator_balance_after, 0);

    // Check the buyer's balance
    let buyer_balance_after = token_client.balance(&buyer);
    assert_eq!(buyer_balance_after, refund_amount);

    // Check the seller's balance
    let seller_balance_after = token_client.balance(&seller);
    assert_eq!(seller_balance_after, 0);
}

#[test]
fn test_resolve_dispute_pay_seller() {
    // Initialize environment with mocked authentication
    let env = Env::default();
    let contract_id = env.register(DisputeContract, ());
    let client = DisputeContractClient::new(&env, &contract_id);
    env.mock_all_auths();

    // Set up test addresses and amount
    let admin = Address::generate(&env);
    let buyer = <Address>::generate(&env);
    let seller = <Address>::generate(&env);
    let arbitrator = <Address>::generate(&env);
    let refund_amount = 1000i128;

    // Mock the token client for transferring tokens
    let token_contract = env.register_stellar_asset_contract_v2(admin.clone());
    let token_address = token_contract.address();

    let token_client = TokenClient::new(&env, &token_address);
    let token_asset_client = TokenAdmin::new(&env, &token_address);
    token_asset_client.mint(&arbitrator, &refund_amount);

    // Verify initial balances
    let arbitrator_balance_before = token_client.balance(&arbitrator);
    assert_eq!(arbitrator_balance_before, refund_amount);

    let buyer_balance_before = token_client.balance(&buyer);
    assert_eq!(buyer_balance_before, 0);

    let seller_balance_before = token_client.balance(&seller);
    assert_eq!(seller_balance_before, 0);

    // Simulate resolving a dispute in favor of the seller
    client.resolve_dispute(
        &token_address.clone(),
        &arbitrator,
        &buyer,
        &seller,
        &refund_amount,
        &DisputeDecision::PaySeller,
    );

    // Verify signed transactions
    assert_eq!(
        env.auths(),
        std::vec![(
            arbitrator.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "resolve_dispute"),
                    (
                        token_address.clone(),
                        arbitrator.clone(),
                        buyer.clone(),
                        seller.clone(),
                        refund_amount,
                        DisputeDecision::PaySeller as u32
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token_address.clone(),
                        symbol_short!("transfer"),
                        (arbitrator.clone(), seller.clone(), refund_amount).into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }]
            }
        )]
    );

    // Check the balance after the transfer
    let arbitrator_balance_after = token_client.balance(&arbitrator);
    assert_eq!(arbitrator_balance_after, 0);

    // Check the buyer's balance
    let buyer_balance_after = token_client.balance(&buyer);
    assert_eq!(buyer_balance_after, 0);

    // Check the seller's balance
    let seller_balance_after = token_client.balance(&seller);
    assert_eq!(seller_balance_after, refund_amount);
}
