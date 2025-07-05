use crate::{
    datatypes::{DataKey, Payment, PaymentEscrowError, PaymentStatus},
    interface::PaymentInterface,
    PaymentEscrowContract, PaymentEscrowContractClient, PaymentEscrowContractArgs
};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{contractimpl, symbol_short, String, Address, Env, Vec};


#[contractimpl]
impl PaymentInterface for PaymentEscrowContract {
    fn get_payment_count(env: &Env) -> u128 {
        env.storage()
            .persistent()
            .get(&DataKey::PaymentCounter)
            .unwrap_or(0)
    }

    fn create_payment(
        env: Env,
        buyer: Address,
        seller: Address,
        amount: i128,
        token: Address,
        expiry_days: u32,
        description: String,
    ) -> Result<u128, PaymentEscrowError> {
        // Authentication
        buyer.require_auth();

        // add buyeraddress to the datakey
        env.storage()
            .persistent()
            .set(&DataKey::Buyer(buyer.clone()), &true);

        // Validate inputs
        if amount <= 0 {
            return Err(PaymentEscrowError::InvalidAmount);
        }

        if buyer == seller {
            return Err(PaymentEscrowError::CannotPaySelf);
        }

        // get payment counter and increment it
        let mut payment_counter = Self::get_payment_count(&env);
        payment_counter += 1;
        let payment_id = payment_counter;

        // Calculate timestamps
        let current_ledger = env.ledger().timestamp();
        let expiry_days = if expiry_days == 0 { 30 } else { expiry_days }; // Default to 30 days if not specified
        let expiry_timestamp = current_ledger + (expiry_days as u64 * 24 * 60 * 60); // Convert days to seconds
        let dispute_deadline = expiry_timestamp - (7 * 24 * 60 * 60); // 7 days before expiry

        // 4. create token client
        let token_client = TokenClient::new(&env, &token);

        // 5. Check buyer's balance token amount
        let buyer_balance = token_client.balance(&buyer);
        if buyer_balance < amount {
            return Err(PaymentEscrowError::InsufficientFunds);
        }
        if amount <= 0 {
            return Err(PaymentEscrowError::InvalidAmount);
        }

        let transfer_result =
            token_client.transfer(&buyer, &env.current_contract_address(), &amount);
        if transfer_result != () {
            return Err(PaymentEscrowError::DepositPaymentFailed);
        }

        // Create payment struct
        let payment = Payment {
            id: payment_id as u64,
            buyer,
            seller,
            amount,
            token,
            status: PaymentStatus::Pending,
            created_at: current_ledger,
            expiry: expiry_timestamp,
            dispute_deadline,
            description,
        };

    

        // Update payment counter
        env.storage()
            .persistent()
            .set(&DataKey::PaymentCounter, &payment_id);

        // Store payment
        env.storage()
            .instance()
            .set(&payment_id, &payment);

         env.events()
         .publish((DataKey::PaymentCounter, symbol_short!("payment")), payment_id);


        Ok(payment_id as u128)
    }


    fn get_a_payment(env: Env, payment_id: u128) -> Result<Payment, PaymentEscrowError> {
        env.storage()
            .persistent()
            .get(&payment_id)
            .ok_or(PaymentEscrowError::NotFound)
    }
}
