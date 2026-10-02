//! Money and label formatting. Amounts are stored as integer cents.

fn group_thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn dollars(abs_cents: u64) -> String {
    format!("{}.{:02}", group_thousands(abs_cents / 100), abs_cents % 100)
}

/// Signed amount, e.g. `+$5,250.00` / `-$10.00`.
pub fn format_money(cents: i64) -> String {
    let sign = if cents >= 0 { '+' } else { '-' };
    format!("{sign}${}", dollars(cents.unsigned_abs()))
}

/// Unsigned compact amount, e.g. `$123.45` / `$1.5k`.
pub fn format_money_short(cents: i64) -> String {
    let abs = cents.unsigned_abs();
    if abs >= 100_000 {
        format!("${:.1}k", abs as f64 / 100_000.0)
    } else {
        format!("${}", dollars(abs))
    }
}

/// Balance without a plus sign; negatives keep the old `$-1,200.00` shape.
pub fn format_balance(cents: i64) -> String {
    let minus = if cents < 0 { "-" } else { "" };
    format!("${minus}{}", dollars(cents.unsigned_abs()))
}

/// Parse a user-typed dollar amount ("12.5", "-3", "1,250.00") into cents.
/// Returns `None` for empty or malformed input, or more than two decimals.
pub fn parse_cents(input: &str) -> Option<i64> {
    let cleaned: String = input.trim().chars().filter(|c| *c != ',' && *c != '$').collect();
    if cleaned.is_empty() {
        return None;
    }
    let (negative, digits) = match cleaned.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, cleaned.strip_prefix('+').unwrap_or(&cleaned)),
    };
    let (whole, frac) = match digits.split_once('.') {
        Some((w, f)) => (w, f),
        None => (digits, ""),
    };
    if (whole.is_empty() && frac.is_empty())
        || frac.len() > 2
        || !whole.chars().all(|c| c.is_ascii_digit())
        || !frac.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let whole: i64 = if whole.is_empty() { 0 } else { whole.parse().ok()? };
    let frac: i64 = match frac.len() {
        0 => 0,
        1 => frac.parse::<i64>().ok()? * 10,
        _ => frac.parse().ok()?,
    };
    let cents = whole.checked_mul(100)?.checked_add(frac)?;
    Some(if negative { -cents } else { cents })
}

/// Lenient dollars-to-cents for free-form numeric fields: any finite number,
/// rounded to the nearest cent ("1e3", "12.345" and "-4" all parse).
pub fn parse_dollars_lenient(input: &str) -> Option<i64> {
    let value: f64 = input.trim().parse().ok()?;
    value.is_finite().then(|| (value * 100.0).round() as i64)
}

pub fn account_type_label(kind: &str) -> String {
    match kind {
        "cash" => "Cash",
        "bank" => "Bank",
        "savings" => "Savings",
        "credit_card" => "Credit Card",
        "mobile_money" => "Mobile Money",
        "investment" => "Investment",
        "other" => "Other",
        other => other,
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_is_signed() {
        assert_eq!(format_money(1000), "+$10.00");
        assert_eq!(format_money(525000), "+$5,250.00");
        assert_eq!(format_money(-120000), "-$1,200.00");
        assert_eq!(format_money(0), "+$0.00");
        assert_eq!(format_money(50), "+$0.50");
        assert_eq!(format_money(-25), "-$0.25");
    }

    #[test]
    fn money_short_uses_k_suffix() {
        assert_eq!(format_money_short(500), "$5.00");
        assert_eq!(format_money_short(12345), "$123.45");
        assert_eq!(format_money_short(100000), "$1.0k");
        assert_eq!(format_money_short(5250000), "$52.5k");
    }

    #[test]
    fn balance_has_no_plus() {
        assert_eq!(format_balance(525000), "$5,250.00");
        assert_eq!(format_balance(-120000), "$-1,200.00");
        assert_eq!(format_balance(0), "$0.00");
    }

    #[test]
    fn parses_typed_amounts() {
        assert_eq!(parse_cents("12.50"), Some(1250));
        assert_eq!(parse_cents("12.5"), Some(1250));
        assert_eq!(parse_cents("12"), Some(1200));
        assert_eq!(parse_cents(".75"), Some(75));
        assert_eq!(parse_cents("-3"), Some(-300));
        assert_eq!(parse_cents("$1,250.00"), Some(125000));
        assert_eq!(parse_cents(" 0 "), Some(0));
        assert_eq!(parse_cents(""), None);
        assert_eq!(parse_cents("abc"), None);
        assert_eq!(parse_cents("1.234"), None);
        assert_eq!(parse_cents("1.2.3"), None);
        assert_eq!(parse_cents("-"), None);
    }

    #[test]
    fn lenient_parse_rounds_and_rejects_junk() {
        assert_eq!(parse_dollars_lenient("12.345"), Some(1235));
        assert_eq!(parse_dollars_lenient(" 1200 "), Some(120000));
        assert_eq!(parse_dollars_lenient("-4"), Some(-400));
        assert_eq!(parse_dollars_lenient("1e3"), Some(100000));
        assert_eq!(parse_dollars_lenient(""), None);
        assert_eq!(parse_dollars_lenient("abc"), None);
        assert_eq!(parse_dollars_lenient("inf"), None);
        assert_eq!(parse_dollars_lenient("NaN"), None);
    }

    #[test]
    fn account_labels() {
        assert_eq!(account_type_label("credit_card"), "Credit Card");
        assert_eq!(account_type_label("weird"), "weird");
    }
}
