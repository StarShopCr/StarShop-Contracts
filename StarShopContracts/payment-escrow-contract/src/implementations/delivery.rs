use crate::{
    datatypes::{DataKey, Payment, PaymentEscrowError, PaymentStatus},
    interface::DeliveryInterface,
    PaymentEscrowContract, PaymentEscrowContractClient, PaymentEscrowContractArgs
};
use soroban_sdk::token::Client as TokenClient;
use soroban_sdk::{contractimpl, symbol_short, String, Address, Env, Vec};


#[contractimpl]
impl DeliveryInterface for PaymentEscrowContract {
    fn buyer_confirm_delivery(env: Env, payment_id: u128, buyer: Address) -> Result<(), PaymentEscrowError> {
        todo!()
    }

}