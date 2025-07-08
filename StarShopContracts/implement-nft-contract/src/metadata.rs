use crate::{NFTContractClient, NFTContractArgs};
use soroban_sdk::contractimpl;
use soroban_sdk::{Address, Env, String, Vec, symbol_short};

#[contractimpl]
impl super::NFTContract {
    pub fn update_metadata(
        env: Env,
        admin: Address,
        token_id: u32,
        name: String,
        description: String,
        attributes: Vec<String>,
    ) {
        Self::check_admin(&env, &admin);
        // SECURITY FIX: Require proper authentication to prevent unauthorized metadata updates
        admin.require_auth();

        // SECURITY FIX: Validate input metadata using correct signature
        Self::validate_metadata(env.clone(), name.clone(), description.clone(), attributes.clone());

        let mut nft: crate::NFTDetail = env
            .storage()
            .persistent()
            .get(&token_id)
            .expect("NFT not exist");

        nft.metadata = crate::NFTMetadata {
            name: name.clone(),
            description: description.clone(),
            attributes: attributes.clone(),
        };

        env.storage().persistent().set(&token_id, &nft);

        // SECURITY FIX: Emit metadata update event with simpler data
        env.events().publish(
            (symbol_short!("UPDATE"), &admin),
            token_id
        );
    }

    pub fn get_metadata(env: Env, token_id: u32) -> crate::NFTMetadata {
        let nft: crate::NFTDetail = env
            .storage()
            .persistent()
            .get(&token_id)
            .expect("NFT not exist");
        nft.metadata
    }

    // Forward declaration of validate_metadata function from minting.rs
    // This will be available since it's implemented in the same contract
}
