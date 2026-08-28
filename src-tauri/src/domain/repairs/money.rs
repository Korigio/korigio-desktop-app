//! Integer-cent tax math (tax-exclusive). No money crate.

use crate::error::AppError;

/// Compute tax cents from base cents and rate in basis points (1% = 100 bps).
/// Rounds half up for non-negative inputs.
pub fn tax_cents_from_base(base_cents: i64, tax_rate_bps: i64) -> Result<i64, AppError> {
    if base_cents < 0 {
        return Err(AppError::Validation {
            field: Some("estimateBaseCents".into()),
            message: "Estimate must be zero or greater.".into(),
        });
    }
    if tax_rate_bps < 0 {
        return Err(AppError::Internal {
            message: "tax rate bps must be non-negative".into(),
        });
    }
    // round_half_up(base * bps / 10000) for non-negative integers
    let product = base_cents
        .checked_mul(tax_rate_bps)
        .ok_or(AppError::Validation {
            field: Some("estimateBaseCents".into()),
            message: "Estimate amount is too large.".into(),
        })?;
    Ok((product + 5_000) / 10_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_half_up_tax() {
        // 10000 * 19% = 1900 exact
        assert_eq!(tax_cents_from_base(10_000, 1_900).unwrap(), 1_900);
        // 1 cent at 19% → 0.19 → rounds to 0
        assert_eq!(tax_cents_from_base(1, 1_900).unwrap(), 0);
        // 3 cents at 19% → 0.57 → rounds to 1
        assert_eq!(tax_cents_from_base(3, 1_900).unwrap(), 1);
        // half: 5 * 10% = 0.5 → 1
        assert_eq!(tax_cents_from_base(5, 1_000).unwrap(), 1);
        assert_eq!(tax_cents_from_base(0, 1_900).unwrap(), 0);
    }
}
