use crate::prediction::{CropYieldPredictionContract, CropYieldPredictionContractClient};
use crate::types::CropYieldError;
use soroban_sdk::{testutils::Address as _, Address, Env, BytesN, String, Vec};

#[test]
fn test_successful_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    assert!(client.try_initialize(&admin).is_ok());
}

#[test]
fn test_reinitialization_overwrites_or_updates_admin() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);

    assert!(client.try_initialize(&admin1).is_ok());
    assert!(client.try_initialize(&admin2).is_ok());
}

#[test]
fn test_contract_uninitialized_crop_registration_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);

    let crop_id = BytesN::from_array(&env, &[2u8; 32]);
    let name = String::from_str(&env, "Wheat");
    let mut yields = Vec::new(&env);
    yields.push_back(500i128);

    let res = client.try_register_crop(&crop_id, &name, &yields);
    assert_eq!(res, Err(Ok(CropYieldError::ContractNotInitialized)));
}

#[test]
fn test_contract_uninitialized_update_data_source_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);

    let pred_id = BytesN::from_array(&env, &[3u8; 32]);
    let data_source = crate::types::DataSource {
        weather_data: String::from_str(&env, "Sunny"),
        soil_data: String::from_str(&env, "Loamy"),
        temperature: 28,
        humidity: 55,
        rainfall: 100,
    };

    let res = client.try_update_data_source(&pred_id, &data_source);
    assert_eq!(res, Err(Ok(CropYieldError::ContractNotInitialized)));
}
