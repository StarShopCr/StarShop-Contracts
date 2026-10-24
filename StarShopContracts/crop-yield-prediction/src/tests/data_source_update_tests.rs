use crate::prediction::{CropYieldPredictionContract, CropYieldPredictionContractClient};
use crate::types::{CropYieldError, DataSource};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

fn setup_prediction<'a>() -> (
    Env,
    CropYieldPredictionContractClient<'a>,
    Address,
    BytesN<32>,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let crop_id = BytesN::from_array(&env, &[40u8; 32]);
    let mut yields = Vec::new(&env);
    yields.push_back(1000i128);
    client.register_crop(&crop_id, &String::from_str(&env, "Corn"), &yields);

    let ds = DataSource {
        weather_data: String::from_str(&env, "Moderate"),
        soil_data: String::from_str(&env, "Normal"),
        temperature: 20, // Suboptimal: factor 90
        humidity: 30,    // Suboptimal: factor 85
        rainfall: 40,    // Suboptimal (<50 and not <30): factor 80 -> avg (90+85+80)/3 = 85 -> 850
    };

    let pred_id = client.generate_prediction(&crop_id, &String::from_str(&env, "Central"), &ds);

    (env, client, admin, pred_id)
}

#[test]
fn test_successful_data_source_update() {
    let (env, client, _, pred_id) = setup_prediction();
    let initial_pred = client.get_prediction(&pred_id);
    assert_eq!(initial_pred.predicted_yield, 850i128);

    let optimal_ds = DataSource {
        weather_data: String::from_str(&env, "Optimal"),
        soil_data: String::from_str(&env, "Fertile"),
        temperature: 30, // Optimal: 110
        humidity: 50,    // Optimal: 110
        rainfall: 100,   // Optimal: 115 -> avg (110+110+115)/3 = 111 -> 1110
    };

    let updated_id = client.update_data_source(&pred_id, &optimal_ds);
    assert_eq!(updated_id, pred_id);

    let updated_pred = client.get_prediction(&pred_id);
    assert_eq!(updated_pred.predicted_yield, 1110i128);
    assert_ne!(updated_pred.data_hash, initial_pred.data_hash);
}

#[test]
fn test_update_non_existent_prediction_fails() {
    let (env, client, _, _) = setup_prediction();
    let fake_id = BytesN::from_array(&env, &[99u8; 32]);
    let ds = DataSource {
        weather_data: String::from_str(&env, "Sunny"),
        soil_data: String::from_str(&env, "Rich"),
        temperature: 28,
        humidity: 55,
        rainfall: 120,
    };

    let res = client.try_update_data_source(&fake_id, &ds);
    assert_eq!(res, Err(Ok(CropYieldError::PredictionNotFound)));
}
