pub fn evaluate(text: &str) -> String {
    add_only(text).map(format_number).unwrap_or_default()
}

fn add_only(text: &str) -> Option<f64> {
    let mut saw_part = false;
    let mut sum = 0.0;

    for part in text.split('+') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            return None;
        }

        sum += trimmed.parse::<f64>().ok()?;
        saw_part = true;
    }

    saw_part.then_some(sum)
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        (value as i64).to_string()
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::evaluate;

    #[test]
    fn adds_numbers() {
        assert_eq!(evaluate("1 + 2 + 3"), "6");
    }

    #[test]
    fn returns_empty_for_invalid_input() {
        assert_eq!(evaluate("1 + nope"), "");
        assert_eq!(evaluate(""), "");
        assert_eq!(evaluate("1 +"), "");
    }
}
