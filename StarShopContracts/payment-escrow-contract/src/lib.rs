#![no_std]
use soroban_sdk::{contract, contractimpl, Env, Address, Vec};
use crate::datatypes::DataKey;


#[contract]
pub struct PaymentEscrowContract;

#[contractimpl]
impl PaymentEscrowContract {
    pub fn __constructor(env: Env, admin: Address) {
        let mut admins = Vec::new(&env);
        admins.push_back(admin);
        env.storage().persistent().set(&DataKey::Admin, &admins);
    }
}

pub use implementations::*;

// Declare modules
mod datatypes;
mod interface;
mod implementations;
// mod token;
mod test;