#![cfg(test)]
extern crate std;

// Note: For full upgrade testing with WASM files, we would need to:
// 1. Build the WASM: cargo build --release --target wasm32-unknown-unknown
// 2. Import it like this:
// mod old_contract {
//     soroban_sdk::contractimport!(file = "target/wasm32-unknown-unknown/release/payment_escrow_contract.wasm");
// }
// For now, we'll test the upgrade function without WASM files

use crate::{
    datatypes::{DisputeDecision, PaymentStatus},
    PaymentEscrowContract, PaymentEscrowContractClient,
};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
    Address, Env, IntoVal, String, Symbol,
    BytesN
};
use soroban_sdk::{
    testutils::Ledger,
    token::{StellarAssetClient as TokenAdmin, TokenClient}
};


// FOR CONTRACT UPGRADE TESTING ONLY
mod new_contract {
    soroban_sdk::contractimport!(
        file = "/Users/mac/StarShop-Contracts/target/wasm32-unknown-unknown/release/payment_escrow_contract.wasm"
    );
}

fn install_new_wasm(e: &Env) -> BytesN<32> {
    e.deployer().upload_contract_wasm(new_contract::WASM)
}



#[test]
fn test_process_deposit_with_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);

    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 30;
    let description = String::from_str(&env, "Test payment");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Execute transaction
    client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Verify signed transactions
    assert_eq!(
        env.auths(),
        std::vec![(
            buyer.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(&env, "create_payment"),
                    (
                        buyer.clone(),
                        seller.clone(),
                        100_i128,
                        token_contract_id.clone(),
                        30_u32,
                        description.clone()
                    )
                        .into_val(&env),
                )),
                sub_invocations: std::vec![AuthorizedInvocation {
                    function: AuthorizedFunction::Contract((
                        token_contract_id.clone(),
                        symbol_short!("transfer"),
                        (buyer.clone(), contract_id.clone(), 100_i128).into_val(&env),
                    )),
                    sub_invocations: std::vec![],
                }]
            }
        )]
    );

    // Verify balances
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&buyer), 900);
    assert_eq!(token_client.balance(&contract_id), 100);
}

#[test]
fn test_get_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 30;
    let description = String::from_str(&env, "Test payment for get function");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Get the payment using get_a_payment
    let retrieved_payment = client.get_a_payment(&payment_id);

    // Verify the retrieved payment matches the created payment
    assert_eq!(retrieved_payment.id, payment_id);
    assert_eq!(retrieved_payment.buyer, buyer);
    assert_eq!(retrieved_payment.seller, seller);
    assert_eq!(retrieved_payment.amount, amount);
    assert_eq!(retrieved_payment.token, token_contract_id);
    assert_eq!(retrieved_payment.status, PaymentStatus::Pending);
    assert_eq!(retrieved_payment.description, description);
}

#[test]
fn test_confirm_deliveries() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 30;
    let description = String::from_str(&env, "Test payment for buyer confirm delivery");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Seller confirms delivery first (changes status to Delivered)
    client.seller_confirm_delivery(&payment_id, &seller);

    // Buyer confirms delivery (changes status to Completed)
    client.buyer_confirm_delivery(&payment_id, &buyer);

    // Verify the payment status is now Completed
    let payment = client.get_a_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Completed);

    // Verify seller received the funds
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&seller), amount);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_get_delivery_status() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 30;
    let description = String::from_str(&env, "Test payment for delivery status");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Check initial status (should be Pending)
    let initial_status = client.get_delivery_status(&payment_id);
    assert_eq!(initial_status, PaymentStatus::Pending);

    // Seller confirms delivery
    client.seller_confirm_delivery(&payment_id, &seller);

    // Check status after seller confirmation (should be Delivered)
    let delivered_status = client.get_delivery_status(&payment_id);
    assert_eq!(delivered_status, PaymentStatus::Delivered);

    // Buyer confirms delivery
    client.buyer_confirm_delivery(&payment_id, &buyer);

    // Check status after buyer confirmation (should be Completed)
    let completed_status = client.get_delivery_status(&payment_id);
    assert_eq!(completed_status, PaymentStatus::Completed);
}

#[test]
fn test_get_delivery_details() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 30;
    let description = String::from_str(&env, "Test payment for delivery details");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Get delivery details
    let delivery_details = client.get_delivery_details(&payment_id);

    // Verify all delivery details fields
    assert_eq!(delivery_details.payment_id, payment_id);
    assert_eq!(delivery_details.buyer, buyer);
    assert_eq!(delivery_details.seller, seller);
    assert_eq!(delivery_details.status, PaymentStatus::Pending);
    assert_eq!(delivery_details.description, description);
}

#[test]
fn test_dispute_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 30;
    let description = String::from_str(&env, "Test payment for dispute");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Buyer disputes the payment
    let dispute_reason = String::from_str(&env, "Item not received as described");
    client.dispute_payment(&payment_id, &buyer, &dispute_reason);

    // Verify the payment status is now Disputed
    let payment = client.get_a_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Disputed);

    // Verify funds are still in contract (not transferred)
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&contract_id), amount);
    assert_eq!(token_client.balance(&buyer), 900);
    assert_eq!(token_client.balance(&seller), 0);
}

#[test]
fn test_resolve_dispute() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 30;
    let description = String::from_str(&env, "Test payment for dispute resolution");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Buyer disputes the payment
    let dispute_reason = String::from_str(&env, "Item not received as described");
    client.dispute_payment(&payment_id, &buyer, &dispute_reason);

    // Arbitrator resolves dispute in favor of seller
    let resolution_reason = String::from_str(&env, "Evidence shows item was delivered correctly");
    client.resolve_dispute(
        &payment_id,
        &arbitrator,
        &DisputeDecision::PaySeller,
        &resolution_reason,
    );

    // Verify the payment status is now Completed
    let payment = client.get_a_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Completed);

    // Verify seller received the funds
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&seller), amount);
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(token_client.balance(&buyer), 900);
}
#[test]
fn test_claim_expired_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let description = String::from_str(&env, "Test expired payment for claim");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment with 1 day expiry
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &1,
        &description,
    );

    // Verify the payment was created successfully
    let payment = client.get_a_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Pending);

    // Verify initial balances
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&buyer), 900);
    assert_eq!(token_client.balance(&contract_id), 100);
    assert_eq!(token_client.balance(&seller), 0);

    // Advance time to make the payment expire
    let current_time = env.ledger().timestamp();
    let future_time = current_time + 2 * 24 * 60 * 60; // 2 days in the future
    env.ledger().set_timestamp(future_time);

    // Now claim the expired payment
    client.claim_payment(&payment_id, &buyer);

    // Verify the payment status is now Refunded
    let updated_payment = client.get_a_payment(&payment_id);
    assert_eq!(updated_payment.status, PaymentStatus::Refunded);

    // Verify buyer received the funds back
    assert_eq!(token_client.balance(&buyer), 1000);
    assert_eq!(token_client.balance(&contract_id), 0);
}


#[test]
#[should_panic]
fn test_claim_before_expiry_should_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let description = String::from_str(&env, "Test payment for early claim failure");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment with 30-day expiry
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &30,
        &description,
    );

    // Verify initial status
    let payment = client.get_a_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Pending);

    // Try to claim the payment immediately (before expiry) - this should panic
    client.claim_payment(&payment_id, &buyer);
}

#[test]
fn test_add_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let initial_arbitrator = Address::generate(&env);
    client.init(&initial_arbitrator);

    // Create a new arbitrator
    let new_arbitrator = Address::generate(&env);

    // Add the new arbitrator
    client.add_arbitrator(&initial_arbitrator, &new_arbitrator);

    // Verify the arbitrators list
    let arbitrators = client.get_arbitrators();
    assert_eq!(arbitrators.len(), 2);
    assert!(arbitrators.contains(&initial_arbitrator));
    assert!(arbitrators.contains(&new_arbitrator));
}

#[test]
fn test_get_arbitrators() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    // Get the arbitrators list
    let arbitrators = client.get_arbitrators();

    // Verify the initial arbitrator is in the list
    assert_eq!(arbitrators.len(), 1);
    assert!(arbitrators.contains(&arbitrator));
}

#[test]
fn test_transfer_arbitrator_rights() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let old_arbitrator = Address::generate(&env);
    client.init(&old_arbitrator);

    // Create a new arbitrator
    let new_arbitrator = Address::generate(&env);

    // Transfer rights from old to new arbitrator
    client.transfer_arbitrator_rights(&old_arbitrator, &new_arbitrator);

    // Verify the arbitrators list
    let arbitrators = client.get_arbitrators();
    assert_eq!(arbitrators.len(), 1);
    assert!(!arbitrators.contains(&old_arbitrator));
    assert!(arbitrators.contains(&new_arbitrator));
}

#[test]
#[should_panic]
fn test_add_arbitrator_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    // Try to add arbitrator without authorization
    let unauthorized_address = Address::generate(&env);
    let new_arbitrator = Address::generate(&env);

    client.add_arbitrator(&unauthorized_address, &new_arbitrator);
}

#[test]
#[should_panic]
fn test_add_existing_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    // Try to add the same arbitrator again
    client.add_arbitrator(&arbitrator, &arbitrator);
}

#[test]
#[should_panic]
fn test_transfer_to_existing_arbitrator() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    // Try to transfer rights to the same arbitrator
    client.transfer_arbitrator_rights(&arbitrator, &arbitrator);
}

#[test]
fn test_multiple_arbitrators() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator1 = Address::generate(&env);
    client.init(&arbitrator1);

    // Add second arbitrator
    let arbitrator2 = Address::generate(&env);
    client.add_arbitrator(&arbitrator1, &arbitrator2);

    // Add third arbitrator
    let arbitrator3 = Address::generate(&env);
    client.add_arbitrator(&arbitrator1, &arbitrator3);

    // Verify all arbitrators are in the list
    let arbitrators = client.get_arbitrators();
    assert_eq!(arbitrators.len(), 3);
    assert!(arbitrators.contains(&arbitrator1));
    assert!(arbitrators.contains(&arbitrator2));
    assert!(arbitrators.contains(&arbitrator3));
}

#[test]
fn test_dispute_after_one_day_for_short_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 3; // 3-day payment
    let description = String::from_str(&env, "Test short payment dispute deadline");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment with 3-day expiry
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Verify initial status
    let payment = client.get_a_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Pending);

    // Advance time by 1 day (86400 seconds)
    let current_time = env.ledger().timestamp();
    let one_day_later = current_time + 24 * 60 * 60; // 1 day in seconds
    env.ledger().set_timestamp(one_day_later);

    // Try to dispute the payment 1 day after creation
    let dispute_reason = String::from_str(&env, "Item not received as described");
    client.dispute_payment(&payment_id, &buyer, &dispute_reason);

    // Verify the payment status is now Disputed (should succeed for 3-day payment)
    let updated_payment = client.get_a_payment(&payment_id);
    assert_eq!(updated_payment.status, PaymentStatus::Disputed);

    // Verify funds are still in contract (not transferred)
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&contract_id), amount);
    assert_eq!(token_client.balance(&buyer), 900);
    assert_eq!(token_client.balance(&seller), 0);
}

#[test]
#[should_panic]
fn test_dispute_after_expiry_for_short_payment() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    client.init(&arbitrator);

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    // Setup test accounts
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 3; // 3-day payment
    let description = String::from_str(&env, "Test dispute after expiry for short payment");

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment with 3-day expiry
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Verify initial status
    let payment = client.get_a_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Pending);

    // Advance time by 4 days (more than the 3-day expiry)
    let current_time = env.ledger().timestamp();
    let four_days_later = current_time + 4 * 24 * 60 * 60; // 4 days in seconds
    env.ledger().set_timestamp(four_days_later);

    // Try to dispute the payment after expiry - this should fail
    let dispute_reason = String::from_str(&env, "Item not received as described");
    client.dispute_payment(&payment_id, &buyer, &dispute_reason);
}


#[test]
fn test_contract_upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(PaymentEscrowContract, ());
    let client = PaymentEscrowContractClient::new(&env, &contract_id);

    // Initialize the contract with an arbitrator
    let arbitrator = Address::generate(&env);
    let buyer = Address::generate(&env);
    let seller = Address::generate(&env);
    let amount = 100;
    let expiry_days = 3; // 3-day payment
    let description = String::from_str(&env, "Test dispute after expiry for short payment");
 
   

    let token_admin = Address::generate(&env);
    let stellar_asset = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_contract_id = stellar_asset.address();
    let token = TokenAdmin::new(&env, &token_contract_id);

    client.init(&arbitrator);

    // Mint tokens to buyer
    token.mint(&buyer, &1000);

    // Create a payment with 3-day expiry
    let payment_id = client.create_payment(
        &buyer,
        &seller,
        &amount,
        &token_contract_id,
        &expiry_days,
        &description,
    );

    // Verify initial status
    let payment = client.get_a_payment(&payment_id);
    assert_eq!(payment.status, PaymentStatus::Pending);

    // Install new WASM and perform upgrade
    let new_wasm_hash = install_new_wasm(&env);
    client.upgrade(&new_wasm_hash);

    // Verify the upgrade was authorized correctly
    assert_eq!(
        env.auths(),
        std::vec![(
            arbitrator,
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    symbol_short!("upgrade"),
                    (new_wasm_hash,).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Verify the payment status is still Pending (upgrade doesn't change payment status)
    let updated_payment = client.get_a_payment(&payment_id);
    assert_eq!(updated_payment.status, PaymentStatus::Pending);

    // Verify funds are still in contract (not transferred)
    let token_client = TokenClient::new(&env, &token_contract_id);
    assert_eq!(token_client.balance(&contract_id), amount);
    assert_eq!(token_client.balance(&buyer), 900);
    assert_eq!(token_client.balance(&seller), 0); 

}