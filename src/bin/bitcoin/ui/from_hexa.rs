use super::error_ui::ErrorUI;

pub fn from(value: String) -> Result<Vec<u8>, ErrorUI> {
    let mut bytes: Vec<u8> = Vec::new();

    for (i, char) in value.chars().enumerate().step_by(2) {
        let mut byte = String::new();
        byte.push(char);

        match value.chars().nth(i + 1) {
            Some(next_char) => byte.push(next_char),
            None => byte.push('0'),
        }

        match u8::from_str_radix(&byte, 16) {
            Ok(byte) => bytes.push(byte),
            Err(error) => {
                return Err(ErrorUI::ErrorReading(format!(
                    "Error while converting a string ({byte}) into hexa: {:?}",
                    error
                )));
            }
        }
    }

    Ok(bytes)
}