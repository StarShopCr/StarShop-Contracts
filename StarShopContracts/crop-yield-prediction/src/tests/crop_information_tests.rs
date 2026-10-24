use crate::prediction::{CropYieldPredictionContract, CropYieldPredictionContractClient};
use crate::types::CropYieldError;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

#[test]
fn test_get_crop_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let crop_id = BytesN::from_array(&env, &[50u8; 32]);
    let name = String::from_str(&env, "SugarCane");
    let mut yields = Vec::new(&env);
    yields.push_back(2000i128);
    yields.push_back(2500i128);

    client.register_crop(&crop_id, &name, &yields);

    let retrieved = client.get_crop(&crop_id);
    assert_eq!(retrieved.crop_id, crop_id);
    assert_eq!(retrieved.name, name);
    assert_eq!(retrieved.historical_yields.len(), 2);
}

#[test]
fn test_get_non_existent_crop_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let non_existent_crop_id = BytesN::from_array(&env, &[99u8; 32]);
    let res = client.try_get_crop(&non_existent_crop_id);
    match res {
        Err(Ok(err)) => assert_eq!(err, CropYieldError::CropNotFound),
        _ => panic!("Expected CropNotFound error"),
    }
}
