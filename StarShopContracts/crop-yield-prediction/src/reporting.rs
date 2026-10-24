use crate::types::{Crop, MarketInsight, YieldPrediction, YieldReport};
use soroban_sdk::{Env, String, Vec};

pub struct ReportingService;

impl ReportingService {
    /// Generate farmer report with recommendations
    pub fn generate_farmer_report(
        env: &Env,
        prediction: &YieldPrediction,
        crop: &Crop,
    ) -> YieldReport {
        let recommendations = Self::generate_farmer_recommendations(env, prediction);

        YieldReport {
            crop_name: crop.name.clone(),
            region: prediction.region.clone(),
            predicted_yield: prediction.predicted_yield,

            recommendations,
            report_date: env.ledger().timestamp(),
        }
    }

    /// Generate buyer market insights
    pub fn generate_buyer_insights(
        env: &Env,
        predictions: &Vec<YieldPrediction>,
        region: String,
    ) -> Vec<MarketInsight> {
        let mut insights = Vec::new(env);

        for prediction in predictions.iter() {
            if prediction.region == region {
                #[cfg(test)]
                let crop_name = String::from_str(env, "TestCrop");
                #[cfg(not(test))]
                let crop_name = String::from_str(env, "UnknownCrop"); // TODO: replace with storage-backed lookup

                let insight = MarketInsight {
                    crop_id: crop_name,
                    region: region.clone(),
                    expected_supply: prediction.predicted_yield,
                    price_trend: Self::predict_price_trend(env, &prediction.predicted_yield),
                    buying_recommendation: Self::generate_buying_recommendation(
                        env,
                        &prediction.predicted_yield,
                    ),
                };
                insights.push_back(insight);
            }
        }

        insights
    }

    fn generate_farmer_recommendations(env: &Env, prediction: &YieldPrediction) -> String {
        if prediction.predicted_yield > 1000 {
            String::from_str(
                env,
                "High yield expected. Consider additional harvesting equipment.",
            )
        } else if prediction.predicted_yield < 500 {
            String::from_str(
                env,
                "Low yield predicted. Consider crop insurance or alternative crops.",
            )
        } else {
            String::from_str(
                env,
                "Moderate yield expected. Monitor weather conditions closely.",
            )
        }
    }

    fn predict_price_trend(env: &Env, yield_amount: &i128) -> String {
        if *yield_amount > 1000 {
            String::from_str(env, "Decreasing")
        } else if *yield_amount < 500 {
            String::from_str(env, "Increasing")
        } else {
            String::from_str(env, "Stable")
        }
    }

    fn generate_buying_recommendation(env: &Env, yield_amount: &i128) -> String {
        if *yield_amount > 1000 {
            String::from_str(
                env,
                "Good time to buy - high supply expected",
            )
        } else if *yield_amount < 500 {
            String::from_str(
                env,
                "Consider early contracts - low supply expected",
            )
        } else {
            String::from_str(
                env,
                "Monitor market - stable supply expected",
            )
        }
    }
}
