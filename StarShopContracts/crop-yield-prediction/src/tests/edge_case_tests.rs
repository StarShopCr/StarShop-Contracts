use crate::prediction::{CropYieldPredictionContract, CropYieldPredictionContractClient};
use crate::types::DataSource;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

#[test]
fn test_large_historical_yield_dataset() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let crop_id = BytesN::from_array(&env, &[90u8; 32]);
    let mut yields = Vec::new(&env);
    for i in 1..=100i128 {
        yields.push_back(i * 10);
    }
    client.register_crop(&crop_id, &String::from_str(&env, "LargeCrop"), &yields);

    let ds = DataSource {
        weather_data: String::from_str(&env, "W"),
        soil_data: String::from_str(&env, "S"),
        temperature: 30,
        humidity: 50,
        rainfall: 100,
    };

    let pred_id = client.generate_prediction(&crop_id, &String::from_str(&env, "Delta"), &ds);
    let pred = client.get_prediction(&pred_id);

    // Sum of 10..1000 is 100 * (10 + 1000) / 2 = 50500. Avg = 505. Optimal factor = 111. Pred = 505 * 111 / 100 = 560
    assert_eq!(pred.predicted_yield, 560);
}

#[test]
fn test_extreme_temperature_and_rainfall() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let crop_id = BytesN::from_array(&env, &[91u8; 32]);
    let mut yields = Vec::new(&env);
    yields.push_back(1000i128);
    client.register_crop(&crop_id, &String::from_str(&env, "ExtremeCrop"), &yields);

    // -50C temperature, 0% humidity, 500mm rainfall
    let ds_extreme = DataSource {
        weather_data: String::from_str(&env, "Blizzard"),
        soil_data: String::from_str(&env, "Frozen"),
        temperature: -50,
        humidity: 0,
        rainfall: 500,
    };

    let pred_id = client.generate_prediction(&crop_id, &String::from_str(&env, "Arctic"), &ds_extreme);
    let pred = client.get_prediction(&pred_id);
    // temp: 90, hum: 85, rain: 80 -> avg (90+85+80)/3 = 85 -> 1000 * 85 / 100 = 850
    assert_eq!(pred.predicted_yield, 850);
}
