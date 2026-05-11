pub fn is_valid(digits: &str) -> bool {
    if digits.len() < 13 || digits.len() > 19 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    let mut sum = 0u32;
    let mut double_digit = false;

    for character in digits.chars().rev() {
        let mut digit = character.to_digit(10).expect("validated ASCII digit");

        if double_digit {
            digit *= 2;
            if digit > 9 {
                digit -= 9;
            }
        }

        sum += digit;
        double_digit = !double_digit;
    }

    sum % 10 == 0
}
