use crate::storage::METADATA_URI_KEY;
use soroban_sdk::{Env, String};

pub fn set_token_metadata(env: &Env, token_id: u32, uri: &String) {
    let metadata_key = (METADATA_URI_KEY, token_id);
    env.storage().persistent().set(&metadata_key, uri);
}

pub fn get_token_metadata(env: &Env, token_id: u32) -> Option<String> {
    let metadata_key = (METADATA_URI_KEY, token_id);
    env.storage().persistent().get(&metadata_key)
}
