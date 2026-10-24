use crate::reporting::ReportingService;
use crate::types::{Crop, YieldPrediction};
use soroban_sdk::{BytesN, Env, String, Vec};

fn create_sample_prediction(env: &Env, predicted_yield: i128, region: &str) -> YieldPrediction {
    YieldPrediction {
        prediction_id: BytesN::from_array(env, &[70u8; 32]),
        crop_id: BytesN::from_array(env, &[71u8; 32]),
        region: String::from_str(env, region),
        predicted_yield,
        data_hash: BytesN::from_array(env, &[72u8; 32]),
        timestamp: env.ledger().timestamp(),
    }
}

fn create_sample_crop(env: &Env, name: &str) -> Crop {
    let mut yields = Vec::new(env);
    yields.push_back(1000i128);
    Crop {
        crop_id: BytesN::from_array(env, &[71u8; 32]),
        name: String::from_str(env, name),
        historical_yields: yields,
    }
}

#[test]
fn test_farmer_report_high_yield_recommendations() {
    let env = Env::default();
    let pred = create_sample_prediction(&env, 1200, "South");
    let crop = create_sample_crop(&env, "Corn");

    let report = ReportingService::generate_farmer_report(&env, &pred, &crop);
    assert_eq!(report.predicted_yield, 1200);
    assert_eq!(report.crop_name, crop.name);
    assert_eq!(report.region, pred.region);
    assert_eq!(
        report.recommendations,
        String::from_str(&env, "High yield expected. Consider additional harvesting equipment.")
    );
}

#[test]
fn test_farmer_report_low_yield_recommendations() {
    let env = Env::default();
    let pred = create_sample_prediction(&env, 400, "East");
    let crop = create_sample_crop(&env, "Wheat");

    let report = ReportingService::generate_farmer_report(&env, &pred, &crop);
    assert_eq!(report.predicted_yield, 400);
    assert_eq!(
        report.recommendations,
        String::from_str(&env, "Low yield predicted. Consider crop insurance or alternative crops.")
    );
}

#[test]
fn test_farmer_report_moderate_yield_recommendations() {
    let env = Env::default();
    let pred = create_sample_prediction(&env, 750, "West");
    let crop = create_sample_crop(&env, "Barley");

    let report = ReportingService::generate_farmer_report(&env, &pred, &crop);
    assert_eq!(report.predicted_yield, 750);
    assert_eq!(
        report.recommendations,
        String::from_str(&env, "Moderate yield expected. Monitor weather conditions closely.")
    );
}

#[test]
fn test_buyer_insights_filtering_and_recommendations() {
    let env = Env::default();
    let mut preds = Vec::new(&env);
    preds.push_back(create_sample_prediction(&env, 1500, "TargetRegion"));
    preds.push_back(create_sample_prediction(&env, 300, "OtherRegion"));

    let target = String::from_str(&env, "TargetRegion");
    let insights = ReportingService::generate_buyer_insights(&env, &preds, target.clone());

    assert_eq!(insights.len(), 1);
    let item = insights.get(0).unwrap();
    assert_eq!(item.region, target);
    assert_eq!(item.expected_supply, 1500);
    assert_eq!(item.price_trend, String::from_str(&env, "Decreasing"));
    assert_eq!(
        item.buying_recommendation,
        String::from_str(&env, "Good time to buy - high supply expected")
    );
}

#[test]
fn test_buyer_insights_empty_predictions() {
    let env = Env::default();
    let preds = Vec::new(&env);
    let region = String::from_str(&env, "NoWhere");
    let insights = ReportingService::generate_buyer_insights(&env, &preds, region);
    assert_eq!(insights.len(), 0);
}
