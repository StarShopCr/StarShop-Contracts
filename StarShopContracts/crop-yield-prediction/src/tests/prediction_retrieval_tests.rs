use crate::prediction::{CropYieldPredictionContract, CropYieldPredictionContractClient};
use crate::types::{CropYieldError, DataSource};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

fn setup_and_seed<'a>() -> (
    Env,
    CropYieldPredictionContractClient<'a>,
    BytesN<32>,
    BytesN<32>,
    BytesN<32>,
    BytesN<32>,
) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CropYieldPredictionContract, ());
    let client = CropYieldPredictionContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);

    let crop1 = BytesN::from_array(&env, &[31u8; 32]);
    let crop2 = BytesN::from_array(&env, &[32u8; 32]);

    let mut yields1 = Vec::new(&env);
    yields1.push_back(500i128);
    client.register_crop(&crop1, &String::from_str(&env, "CropOne"), &yields1);

    let mut yields2 = Vec::new(&env);
    yields2.push_back(700i128);
    client.register_crop(&crop2, &String::from_str(&env, "CropTwo"), &yields2);

    let ds = DataSource {
        weather_data: String::from_str(&env, "Normal"),
        soil_data: String::from_str(&env, "Good"),
        temperature: 28,
        humidity: 50,
        rainfall: 100,
    };

    let p1 = client.generate_prediction(&crop1, &String::from_str(&env, "RegionNorth"), &ds);
    env.ledger().set_timestamp(1000);
    let p2 = client.generate_prediction(&crop1, &String::from_str(&env, "RegionSouth"), &ds);

    (env, client, crop1, crop2, p1, p2)
}

#[test]
fn test_get_existing_prediction() {
    let (_, client, crop1, _, p1, _) = setup_and_seed();
    let pred = client.get_prediction(&p1);
    assert_eq!(pred.prediction_id, p1);
    assert_eq!(pred.crop_id, crop1);
}

#[test]
fn test_get_non_existent_prediction_fails() {
    let (env, client, _, _, _, _) = setup_and_seed();
    let fake_id = BytesN::from_array(&env, &[99u8; 32]);
    let res = client.try_get_prediction(&fake_id);
    match res {
        Err(Ok(err)) => assert_eq!(err, CropYieldError::PredictionNotFound),
        _ => panic!("Expected PredictionNotFound error"),
    }
}

#[test]
fn test_list_predictions_by_crop() {
    let (_, client, crop1, crop2, _, _) = setup_and_seed();
    let list1 = client.list_predictions_by_crop(&crop1);
    assert_eq!(list1.len(), 2);

    let list2 = client.list_predictions_by_crop(&crop2);
    assert_eq!(list2.len(), 0);
}

#[test]
fn test_list_predictions_by_region() {
    let (env, client, _, _, _, _) = setup_and_seed();
    let region_north = String::from_str(&env, "RegionNorth");
    let north_preds = client.list_predictions_by_region(&region_north);
    assert_eq!(north_preds.len(), 1);

    let region_west = String::from_str(&env, "RegionWest");
    let west_preds = client.list_predictions_by_region(&region_west);
    assert_eq!(west_preds.len(), 0);
}

#[test]
fn test_list_predictions_by_empty_region_fails() {
    let (env, client, _, _, _, _) = setup_and_seed();
    let empty_region = String::from_str(&env, "");
    let res = client.try_list_predictions_by_region(&empty_region);
    assert!(res.is_err());
}
