use crate::types::{Crop, DataSource};
use crate::utils::{calculate_yield_prediction, generate_prediction_id, hash_data_source};
use soroban_sdk::{BytesN, Env, String, Vec};

#[test]
fn test_generate_prediction_id_deterministic_for_same_ledger() {
    let env = Env::default();
    let crop_id = BytesN::from_array(&env, &[60u8; 32]);
    let id1 = generate_prediction_id(&env, &crop_id);
    let id2 = generate_prediction_id(&env, &crop_id);
    assert_eq!(id1, id2);
}

#[test]
fn test_generate_prediction_id_differs_for_different_crops() {
    let env = Env::default();
    let crop1 = BytesN::from_array(&env, &[61u8; 32]);
    let crop2 = BytesN::from_array(&env, &[62u8; 32]);
    let id1 = generate_prediction_id(&env, &crop1);
    let id2 = generate_prediction_id(&env, &crop2);
    assert_ne!(id1, id2);
}

#[test]
fn test_hash_data_source_consistency() {
    let env = Env::default();
    let ds1 = DataSource {
        weather_data: String::from_str(&env, "W1"),
        soil_data: String::from_str(&env, "S1"),
        temperature: 25,
        humidity: 60,
        rainfall: 120,
    };
    let ds2 = DataSource {
        weather_data: String::from_str(&env, "W2"),
        soil_data: String::from_str(&env, "S2"),
        temperature: 25,
        humidity: 60,
        rainfall: 120,
    };
    let hash1 = hash_data_source(&env, &ds1);
    let hash2 = hash_data_source(&env, &ds2);
    // Hashing combines temp, humidity, and rainfall
    assert_eq!(hash1, hash2);
}

#[test]
fn test_yield_calculation_empty_historical_data_returns_zero() {
    let env = Env::default();
    let crop = Crop {
        crop_id: BytesN::from_array(&env, &[63u8; 32]),
        name: String::from_str(&env, "EmptyCrop"),
        historical_yields: Vec::new(&env),
    };
    let ds = DataSource {
        weather_data: String::from_str(&env, "W"),
        soil_data: String::from_str(&env, "S"),
        temperature: 30,
        humidity: 50,
        rainfall: 100,
    };
    assert_eq!(calculate_yield_prediction(&crop, &ds), 0);
}

#[test]
fn test_yield_calculation_weather_factor_scenarios() {
    let env = Env::default();
    let mut yields = Vec::new(&env);
    yields.push_back(1000i128);
    let crop = Crop {
        crop_id: BytesN::from_array(&env, &[64u8; 32]),
        name: String::from_str(&env, "TestWeather"),
        historical_yields: yields,
    };

    // Scenario 1: Optimal weather (temp 26-34, hum 41-69, rain 51-199)
    // Factors: temp 110, hum 110, rain 115 -> avg = (110+110+115)/3 = 111 -> yield 1110
    let ds_optimal = DataSource {
        weather_data: String::from_str(&env, "Optimal"),
        soil_data: String::from_str(&env, "Good"),
        temperature: 30,
        humidity: 50,
        rainfall: 100,
    };
    assert_eq!(calculate_yield_prediction(&crop, &ds_optimal), 1110i128);

    // Scenario 2: Severe drought (temp 40, hum 20, rain 15)
    // Factors: temp 90, hum 85, rain (<30) 70 -> avg = (90+85+70)/3 = 81 -> yield 810
    let ds_drought = DataSource {
        weather_data: String::from_str(&env, "Drought"),
        soil_data: String::from_str(&env, "Dry"),
        temperature: 40,
        humidity: 20,
        rainfall: 15,
    };
    assert_eq!(calculate_yield_prediction(&crop, &ds_drought), 810i128);

    // Scenario 3: Flood / Excessive rain (temp 20, hum 80, rain 250)
    // Factors: temp 90, hum 85, rain (>200) 80 -> avg = (90+85+80)/3 = 85 -> yield 850
    let ds_flood = DataSource {
        weather_data: String::from_str(&env, "Flood"),
        soil_data: String::from_str(&env, "Wet"),
        temperature: 20,
        humidity: 80,
        rainfall: 250,
    };
    assert_eq!(calculate_yield_prediction(&crop, &ds_flood), 850i128);
}
