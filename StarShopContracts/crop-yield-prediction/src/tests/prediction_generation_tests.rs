use crate::prediction::{CropYieldPredictionContract, CropYieldPredictionContractClient};
use crate::types::{CropYieldError, DataSource};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

fn setup_with_crop<'a>() -> (Env, CropYieldPredictionContractClient<'a>, BytesN<32>) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let crop_id = BytesN::from_array(&env, &[20u8; 32]);
    let name = String::from_str(&env, "Wheat");
    let mut yields = Vec::new(&env);
    yields.push_back(1000i128);
    yields.push_back(1200i128);
    client.register_crop(&crop_id, &name, &yields);

    (env, client, crop_id)
}

#[test]
fn test_successful_prediction_generation() {
    let (env, client, crop_id) = setup_with_crop();
    let region = String::from_str(&env, "Midwest");
    let data_source = DataSource {
        weather_data: String::from_str(&env, "Warm/Humid"),
        soil_data: String::from_str(&env, "Clay Loam"),
        temperature: 28, // Optimal: 25-35 -> factor 110
        humidity: 50,    // Optimal: 40-70 -> factor 110
        rainfall: 120,   // Optimal: 50-200 -> factor 115
    };

    let pred_id = client.generate_prediction(&crop_id, &region, &data_source);
    let pred = client.get_prediction(&pred_id);

    assert_eq!(pred.crop_id, crop_id);
    assert_eq!(pred.region, region);
    // avg yield = 1100. adjustment = (110 + 110 + 115) / 3 = 111. predicted = 1100 * 111 / 100 = 1221
    assert_eq!(pred.predicted_yield, 1221i128);
    assert_eq!(pred.timestamp, env.ledger().timestamp());
}

#[test]
fn test_prediction_generation_non_existent_crop_fails() {
    let (env, client, _) = setup_with_crop();
    let non_existent_crop_id = BytesN::from_array(&env, &[99u8; 32]);
    let region = String::from_str(&env, "North");
    let data_source = DataSource {
        weather_data: String::from_str(&env, "Clear"),
        soil_data: String::from_str(&env, "Sandy"),
        temperature: 20,
        humidity: 30,
        rainfall: 40,
    };

    let res = client.try_generate_prediction(&non_existent_crop_id, &region, &data_source);
    assert_eq!(res, Err(Ok(CropYieldError::CropNotFound)));
}

#[test]
fn test_prediction_generation_empty_region_fails() {
    let (env, client, crop_id) = setup_with_crop();
    let empty_region = String::from_str(&env, "");
    let data_source = DataSource {
        weather_data: String::from_str(&env, "Clear"),
        soil_data: String::from_str(&env, "Sandy"),
        temperature: 20,
        humidity: 30,
        rainfall: 40,
    };

    let res = client.try_generate_prediction(&crop_id, &empty_region, &data_source);
    assert_eq!(res, Err(Ok(CropYieldError::InvalidInput)));
}

#[test]
fn test_prediction_id_uniqueness_across_crops() {
    let (env, client, crop_id1) = setup_with_crop();
    let crop_id2 = BytesN::from_array(&env, &[21u8; 32]);
    let name = String::from_str(&env, "Oats");
    let mut yields = Vec::new(&env);
    yields.push_back(800i128);
    client.register_crop(&crop_id2, &name, &yields);

    let region = String::from_str(&env, "East");
    let data_source = DataSource {
        weather_data: String::from_str(&env, "Sunny"),
        soil_data: String::from_str(&env, "Silt"),
        temperature: 25,
        humidity: 60,
        rainfall: 80,
    };

    let pred1 = client.generate_prediction(&crop_id1, &region, &data_source);
    let pred2 = client.generate_prediction(&crop_id2, &region, &data_source);

    assert_ne!(pred1, pred2);
}
