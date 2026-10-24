use crate::types::*;
use soroban_sdk::{BytesN, Env, String, Vec};

#[test]
fn test_data_types_cloning_and_properties() {
    let env = Env::default();
    let crop_id = BytesN::from_array(&env, &[80u8; 32]);
    let name = String::from_str(&env, "Soybean");
    let mut yields = Vec::new(&env);
    yields.push_back(1200i128);

    let crop = Crop {
        crop_id: crop_id.clone(),
        name: name.clone(),
        historical_yields: yields.clone(),
    };

    let crop_clone = crop.clone();
    assert_eq!(crop_clone.crop_id, crop_id);
    assert_eq!(crop_clone.name, name);
    assert_eq!(crop_clone.historical_yields, yields);
}

#[test]
fn test_data_keys_enum_variants() {
    let key1 = DataKey::CROPS;
    let key2 = DataKey::ADMIN;
    let key3 = DataKey::PREDICTIONS;

    // Ensure distinct variants
    match key1 {
        DataKey::CROPS => (),
        _ => panic!("Expected DataKey::CROPS"),
    }
    match key2 {
        DataKey::ADMIN => (),
        _ => panic!("Expected DataKey::ADMIN"),
    }
    match key3 {
        DataKey::PREDICTIONS => (),
        _ => panic!("Expected DataKey::PREDICTIONS"),
    }
}
