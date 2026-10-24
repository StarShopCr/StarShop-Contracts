use crate::types::CropYieldError;

#[test]
fn test_crop_yield_error_codes() {
    assert_eq!(CropYieldError::Unauthorized as u32, 1);
    assert_eq!(CropYieldError::InvalidInput as u32, 2);
    assert_eq!(CropYieldError::CropNotFound as u32, 3);
    assert_eq!(CropYieldError::PredictionNotFound as u32, 4);
    assert_eq!(CropYieldError::ContractNotInitialized as u32, 5);
    assert_eq!(CropYieldError::InvalidYieldData as u32, 6);
    assert_eq!(CropYieldError::DataProcessingError as u32, 7);
}

#[test]
fn test_crop_yield_error_equality() {
    let err1 = CropYieldError::Unauthorized;
    let err2 = CropYieldError::Unauthorized;
    let err3 = CropYieldError::CropNotFound;

    assert_eq!(err1, err2);
    assert_ne!(err1, err3);
}
