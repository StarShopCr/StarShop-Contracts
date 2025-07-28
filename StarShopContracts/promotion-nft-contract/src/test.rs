#![cfg(test)]

use soroban_sdk::{Env};
use crate::{StarshopPromoNFT, StarshopPromoNFTClient};

fn setup_test_environment<'a>() -> (Env, StarshopPromoNFTClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(StarshopPromoNFT, ());
    let client = StarshopPromoNFTClient::new(&env, &contract_id);
    (env, client)
}
