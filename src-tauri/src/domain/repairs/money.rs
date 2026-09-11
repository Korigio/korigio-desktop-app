//! Integer-cent tax and discount math (tax-exclusive). No money crate.

use crate::error::AppError;

const BPS_DENOM: i64 = 10_000;
const HALF_UP_BIAS: i64 = 5_000;

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
    Ok((product + HALF_UP_BIAS) / BPS_DENOM)
}

/// Net (pre-tax) cents after a discount in basis points.
/// `round_half_up(list_cents * (10000 - discount_bps) / 10000)`.
pub fn net_cents_after_discount(list_cents: i64, discount_bps: i64) -> Result<i64, AppError> {
    validate_list_and_discount(list_cents, discount_bps)?;
    let keep_bps = BPS_DENOM - discount_bps;
    let product = list_cents
        .checked_mul(keep_bps)
        .ok_or(AppError::Validation {
            field: Some("estimateBaseCents".into()),
            message: "Estimate amount is too large.".into(),
        })?;
    Ok((product + HALF_UP_BIAS) / BPS_DENOM)
}

/// Discount amount in cents: `list_cents - net_cents_after_discount(...)`.
pub fn discount_cents_from_list(list_cents: i64, discount_bps: i64) -> Result<i64, AppError> {
    let net = net_cents_after_discount(list_cents, discount_bps)?;
    Ok(list_cents - net)
}

fn validate_list_and_discount(list_cents: i64, discount_bps: i64) -> Result<(), AppError> {
    if list_cents < 0 {
        return Err(AppError::Validation {
            field: Some("estimateBaseCents".into()),
            message: "Estimate must be zero or greater.".into(),
        });
    }
    if !(0..=BPS_DENOM).contains(&discount_bps) {
        return Err(AppError::Validation {
            field: Some("estimateDiscountBps".into()),
            message: "Discount must be between 0% and 100%.".into(),
        });
    }
    Ok(())
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

    #[test]
    fn net_after_discount_half_up() {
        // 10% off 10000 → 9000
        assert_eq!(net_cents_after_discount(10_000, 1_000).unwrap(), 9_000);
        // 0% → full list
        assert_eq!(net_cents_after_discount(10_000, 0).unwrap(), 10_000);
        // 100% → 0
        assert_eq!(net_cents_after_discount(10_000, 10_000).unwrap(), 0);
        // half-up: 3 cents * 50% keep = 1.5 → 2
        assert_eq!(net_cents_after_discount(3, 5_000).unwrap(), 2);
        // 1 cent * 10% off → keep 0.9 → 1
        assert_eq!(net_cents_after_discount(1, 1_000).unwrap(), 1);
    }

    #[test]
    fn discount_cents_from_list_matches_list_minus_net() {
        assert_eq!(discount_cents_from_list(10_000, 1_000).unwrap(), 1_000);
        assert_eq!(discount_cents_from_list(10_000, 0).unwrap(), 0);
        assert_eq!(discount_cents_from_list(10_000, 10_000).unwrap(), 10_000);
        // 3 @ 50%: net 2 → discount 1
        assert_eq!(discount_cents_from_list(3, 5_000).unwrap(), 1);
    }

    #[test]
    fn rejects_invalid_discount_bps() {
        assert!(discount_cents_from_list(100, -1).is_err());
        assert!(discount_cents_from_list(100, 10_001).is_err());
        assert!(net_cents_after_discount(-1, 0).is_err());
    }
}
