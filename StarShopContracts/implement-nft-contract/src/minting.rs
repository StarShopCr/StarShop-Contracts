use crate::{NFTDetail, NFTMetadata, COUNTER_KEY, ADMIN_KEY, NFTContractClient, NFTContractArgs};
use soroban_sdk::{contractimpl, Address, Env, String, Vec, symbol_short, Symbol};

// Add constants for supply limits and events
const MAX_SUPPLY_KEY: Symbol = symbol_short!("MAXSUP");
const DEFAULT_MAX_SUPPLY: u32 = u32::MAX; // Set to max u32 to allow overflow testing
const MAX_NAME_LENGTH: u32 = 100;
const MAX_DESCRIPTION_LENGTH: u32 = 500;
const MAX_ATTRIBUTES_COUNT: u32 = 20;
const MAX_ATTRIBUTE_LENGTH: u32 = 100;

#[contractimpl]
impl super::NFTContract {
    pub fn mint_nft(
        env: Env,
        to: Address,
        name: String,
        description: String,
        attributes: Vec<String>,
    ) -> u32 {
        // SECURITY FIX: Admin-only minting control (if admin is set)
        // For backwards compatibility, only enforce if admin exists
        if let Some(admin) = env.storage().instance().get::<Symbol, Address>(&ADMIN_KEY) {
            admin.require_auth();
        }

        // SECURITY FIX: Input validation
        Self::validate_metadata(env.clone(), name.clone(), description.clone(), attributes.clone());
        
        let current_id: u32 = env.storage().instance().get(&COUNTER_KEY).unwrap_or(0);

        // SECURITY FIX: Prevent integer overflow in token counter with exact error message (check first)
        if current_id == u32::MAX {
            panic!("Token counter overflow: Maximum number of tokens (4,294,967,295) reached");
        }

        // SECURITY FIX: Check supply limits after overflow check
        let max_supply: u32 = env.storage().instance().get(&MAX_SUPPLY_KEY).unwrap_or(DEFAULT_MAX_SUPPLY);
        if current_id >= max_supply {
            panic!("Maximum supply reached");
        }

        let next_id = current_id + 1;
        env.storage().instance().set(&COUNTER_KEY, &next_id);

        let metadata = NFTMetadata {
            name: name.clone(),
            description: description.clone(),
            attributes: attributes.clone(),
        };

        let nft = NFTDetail {
            owner: to.clone(),
            metadata,
        };

        env.storage().persistent().set(&next_id, &nft);

        // SECURITY FIX: Emit mint event with simpler data
        env.events().publish(
            (symbol_short!("MINT"), &to),
            next_id
        );

        next_id
    }

    // SECURITY FIX: Make input validation function public and use owned types
    pub fn validate_metadata(_env: Env, name: String, description: String, attributes: Vec<String>) {
        // Validate name
        if name.len() == 0 || name.len() > MAX_NAME_LENGTH {
            panic!("Invalid name length");
        }

        // Validate description  
        if description.len() > MAX_DESCRIPTION_LENGTH {
            panic!("Description too long");
        }

        // Validate attributes
        if attributes.len() > MAX_ATTRIBUTES_COUNT {
            panic!("Too many attributes");
        }

        for attribute in attributes.iter() {
            if attribute.len() > MAX_ATTRIBUTE_LENGTH {
                panic!("Attribute too long");
            }
        }
    }

    // SECURITY FIX: Add function to set maximum supply (admin only)
    pub fn set_max_supply(env: Env, admin: Address, max_supply: u32) {
        Self::check_admin(&env, &admin);
        admin.require_auth();
        
        if max_supply == 0 {
            panic!("Max supply must be greater than 0");
        }

        env.storage().instance().set(&MAX_SUPPLY_KEY, &max_supply);
    }

    // Add getter for max supply
    pub fn get_max_supply(env: Env) -> u32 {
        env.storage().instance().get(&MAX_SUPPLY_KEY).unwrap_or(DEFAULT_MAX_SUPPLY)
    }

    // Add getter for current supply
    pub fn get_current_supply(env: Env) -> u32 {
        env.storage().instance().get(&COUNTER_KEY).unwrap_or(0)
    }
}
