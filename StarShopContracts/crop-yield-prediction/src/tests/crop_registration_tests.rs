use crate::prediction::{CropYieldPredictionContract, CropYieldPredictionContractClient};
use crate::types::CropYieldError;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

fn setup_env<'a>() -> (Env, CropYieldPredictionContractClient<'a>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn test_successful_crop_registration() {
    let (env, client, _) = setup_env();
    let crop_id = BytesN::from_array(&env, &[10u8; 32]);
    let name = String::from_str(&env, "Rice");
    let mut yields = Vec::new(&env);
    yields.push_back(800i128);
    yields.push_back(950i128);

    let res = client.register_crop(&crop_id, &name, &yields);
    assert_eq!(res, crop_id);

    let crop = client.get_crop(&crop_id);
    assert_eq!(crop.name, name);
    assert_eq!(crop.historical_yields, yields);
}

#[test]
fn test_crop_registration_empty_name_fails() {
    let (env, client, _) = setup_env();
    let crop_id = BytesN::from_array(&env, &[11u8; 32]);
    let name = String::from_str(&env, "");
    let mut yields = Vec::new(&env);
    yields.push_back(800i128);

    let res = client.try_register_crop(&crop_id, &name, &yields);
    assert_eq!(res, Err(Ok(CropYieldError::InvalidInput)));
}

#[test]
fn test_crop_registration_empty_historical_yields_fails() {
    let (env, client, _) = setup_env();
    let crop_id = BytesN::from_array(&env, &[12u8; 32]);
    let name = String::from_str(&env, "Barley");
    let yields = Vec::new(&env);

    let res = client.try_register_crop(&crop_id, &name, &yields);
    assert_eq!(res, Err(Ok(CropYieldError::InvalidInput)));
}

#[test]
fn test_duplicate_crop_id_overwrites_or_updates_data() {
    let (env, client, _) = setup_env();
    let crop_id = BytesN::from_array(&env, &[13u8; 32]);
    let name1 = String::from_str(&env, "Soybean A");
    let mut yields1 = Vec::new(&env);
    yields1.push_back(500i128);

    client.register_crop(&crop_id, &name1, &yields1);

    let name2 = String::from_str(&env, "Soybean B");
    let mut yields2 = Vec::new(&env);
    yields2.push_back(600i128);
    yields2.push_back(700i128);

    client.register_crop(&crop_id, &name2, &yields2);

    let updated = client.get_crop(&crop_id);
    assert_eq!(updated.name, name2);
    assert_eq!(updated.historical_yields.len(), 2);
}

#[test]
fn test_multiple_unique_crops_storage() {
    let (env, client, _) = setup_env();
    for i in 1..=5u8 {
        let crop_id = BytesN::from_array(&env, &[i; 32]);
        let name = String::from_str(&env, "CropItem");
        let mut yields = Vec::new(&env);
        yields.push_back(100i128 * (i as i128));
        client.register_crop(&crop_id, &name, &yields);

        let retrieved = client.get_crop(&crop_id);
        assert_eq!(retrieved.historical_yields.get(0), Some(100i128 * (i as i128)));
    }
}
