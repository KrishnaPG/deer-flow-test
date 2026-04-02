use deer_foundation_contracts::CanonicalLevel;

use crate::error::NormalizationError;

pub fn validate_promotion(
    from: CanonicalLevel,
    to: CanonicalLevel,
) -> Result<(), NormalizationError> {
    let valid = matches!(
        (from, to),
        (CanonicalLevel::L0, CanonicalLevel::L1)
            | (CanonicalLevel::L1, CanonicalLevel::L2)
            | (CanonicalLevel::L2, CanonicalLevel::L3)
            | (CanonicalLevel::L3, CanonicalLevel::L4)
            | (CanonicalLevel::L4, CanonicalLevel::L5)
            | (CanonicalLevel::L5, CanonicalLevel::L6)
    );

    if valid {
        Ok(())
    } else {
        Err(NormalizationError::InvalidPromotion {
            from: format!("{from:?}"),
            to: format!("{to:?}"),
        })
    }
}
