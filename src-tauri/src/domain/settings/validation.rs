//! Shop settings validation (tax rate percent, currency).

use crate::error::AppError;

/// Parse a tax percent string (0–100, ≤2 fractional digits) into normalized display + bps.
/// Example: `"19"` → (`"19"`, 1900); `"19.5"` → (`"19.5"`, 1950).
pub fn parse_tax_rate_percent(raw: &str) -> Result<(String, i64), AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation {
            field: Some("taxRatePercent".into()),
            message: "Tax rate is required.".into(),
        });
    }

    let (int_part, frac_part) = match trimmed.split_once('.') {
        None => (trimmed, ""),
        Some((i, f)) => (i, f),
    };

    if int_part.is_empty()
        || !int_part.chars().all(|c| c.is_ascii_digit())
        || (!frac_part.is_empty() && !frac_part.chars().all(|c| c.is_ascii_digit()))
    {
        return Err(AppError::Validation {
            field: Some("taxRatePercent".into()),
            message: "Tax rate must be a number between 0 and 100 with at most 2 decimals.".into(),
        });
    }
    if frac_part.chars().count() > 2 {
        return Err(AppError::Validation {
            field: Some("taxRatePercent".into()),
            message: "Tax rate must have at most 2 decimal places.".into(),
        });
    }

    let whole: i64 = int_part.parse().map_err(|_| AppError::Validation {
        field: Some("taxRatePercent".into()),
        message: "Tax rate must be a number between 0 and 100 with at most 2 decimals.".into(),
    })?;

    let frac_digits = frac_part.chars().count();
    let frac_value: i64 = if frac_part.is_empty() {
        0
    } else {
        frac_part.parse().map_err(|_| AppError::Validation {
            field: Some("taxRatePercent".into()),
            message: "Tax rate must be a number between 0 and 100 with at most 2 decimals.".into(),
        })?
    };
    let frac_bps = match frac_digits {
        0 => 0,
        1 => frac_value * 10,
        2 => frac_value,
        _ => unreachable!(),
    };

    let bps = whole
        .checked_mul(100)
        .and_then(|v| v.checked_add(frac_bps))
        .ok_or(AppError::Validation {
            field: Some("taxRatePercent".into()),
            message: "Tax rate must be a number between 0 and 100 with at most 2 decimals.".into(),
        })?;

    if !(0..=10_000).contains(&bps) {
        return Err(AppError::Validation {
            field: Some("taxRatePercent".into()),
            message: "Tax rate must be between 0 and 100.".into(),
        });
    }

    let normalized = normalize_percent_display(whole, frac_part);
    Ok((normalized, bps))
}

fn normalize_percent_display(whole: i64, frac_part: &str) -> String {
    if frac_part.is_empty() || frac_part.chars().all(|c| c == '0') {
        return whole.to_string();
    }
    let trimmed_frac: String = frac_part.trim_end_matches('0').chars().collect();
    if trimmed_frac.is_empty() {
        whole.to_string()
    } else {
        format!("{whole}.{trimmed_frac}")
    }
}

pub fn normalize_currency(raw: &str) -> Result<String, AppError> {
    let trimmed = raw.trim();
    if trimmed.len() != 3 || !trimmed.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(AppError::Validation {
            field: Some("currency".into()),
            message: "Currency must be a 3-letter code.".into(),
        });
    }
    Ok(trimmed.to_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_percent_defaults_and_decimals() {
        assert_eq!(parse_tax_rate_percent("19").unwrap(), ("19".into(), 1_900));
        assert_eq!(parse_tax_rate_percent("19.5").unwrap(), ("19.5".into(), 1_950));
        assert_eq!(parse_tax_rate_percent("19.55").unwrap(), ("19.55".into(), 1_955));
        assert_eq!(parse_tax_rate_percent("0").unwrap(), ("0".into(), 0));
        assert_eq!(parse_tax_rate_percent("100").unwrap(), ("100".into(), 10_000));
        assert_eq!(parse_tax_rate_percent(" 19.50 ").unwrap(), ("19.5".into(), 1_950));
    }

    #[test]
    fn reject_invalid_percent() {
        assert!(parse_tax_rate_percent("").is_err());
        assert!(parse_tax_rate_percent("19.555").is_err());
        assert!(parse_tax_rate_percent("101").is_err());
        assert!(parse_tax_rate_percent("-1").is_err());
        assert!(parse_tax_rate_percent("abc").is_err());
    }

    #[test]
    fn currency_normalization() {
        assert_eq!(normalize_currency("eur").unwrap(), "EUR");
        assert!(normalize_currency("EURO").is_err());
        assert!(normalize_currency("EU").is_err());
    }
}
