use crate::{NFTContractClient, NFTContractArgs};
use soroban_sdk::contractimpl;
use soroban_sdk::{Address, Env, symbol_short};

#[contractimpl]
impl super::NFTContract {
    pub fn transfer_nft(env: Env, from: Address, to: Address, token_id: u32) {
        from.require_auth();

        if from == to {
            panic!("Cannot transfer to self");
        }

        let mut nft: crate::NFTDetail = env
            .storage()
            .persistent()
            .get(&token_id)
            .expect("NFT not exist");

        if nft.owner != from {
            panic!("You are not the owner");
        }

        nft.owner = to.clone();
        env.storage().persistent().set(&token_id, &nft);

        env.events().publish(
            (symbol_short!("TRANSFER"), &from, &to),
            &token_id
        );
    }

    pub fn burn_nft(env: Env, owner: Address, token_id: u32) {
        owner.require_auth();

        let nft: crate::NFTDetail = env
            .storage()
            .persistent()
            .get(&token_id)
            .expect("NFT not exist");

        if nft.owner != owner {
            panic!("You can't burn this NFT");
        }

        env.storage().persistent().remove(&token_id);

        env.events().publish(
            (symbol_short!("BURN"), &owner),
            &token_id
        );
    }

    pub fn get_owner(env: Env, token_id: u32) -> Address {
        let nft: crate::NFTDetail = env
            .storage()
            .persistent()
            .get(&token_id)
            .expect("NFT not exist");
        nft.owner
    }

    pub fn nft_exists(env: Env, token_id: u32) -> bool {
        env.storage().persistent().has(&token_id)
    }
}
