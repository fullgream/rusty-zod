use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // String errors
    StringTooShort,
    StringTooLong,
    InvalidEmail,
    PatternMismatch,
    
    // Number errors
    NumberTooSmall,
    NumberTooLarge,
    InvalidNumber,
    NotInteger,
    
    // Array errors
    ArrayTooShort,
    ArrayTooLong,
    InvalidArrayItem,
    
    // Object errors
    RequiredField,
    UnknownField,
    InvalidType,
    
    // Custom error
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::StringTooShort.to_string(), "string.too_short");
        assert_eq!(ErrorCode::InvalidEmail.to_string(), "string.email");
        assert_eq!(ErrorCode::NumberTooSmall.to_string(), "number.too_small");
        assert_eq!(ErrorCode::RequiredField.to_string(), "object.required");
    }

    #[test]
    fn test_error_code_default_messages() {
        assert_eq!(
            ErrorCode::StringTooShort.default_message(),
            "String is too short"
        );
        assert_eq!(
            ErrorCode::InvalidEmail.default_message(),
            "Invalid email format"
        );
        assert_eq!(
            ErrorCode::NumberTooSmall.default_message(),
            "Number is too small"
        );
    }

    #[test]
    fn test_custom_error_code() {
        let error = ErrorCode::Custom("custom.error".to_string());
        assert_eq!(error.code(), "custom");
        assert_eq!(error.default_message(), "custom.error");
    }
}

impl ErrorCode {
    pub fn default_message(&self) -> String {
        match self {
            // String errors
            ErrorCode::StringTooShort => "String is too short".into(),
            ErrorCode::StringTooLong => "String is too long".into(),
            ErrorCode::InvalidEmail => "Invalid email format".into(),
            ErrorCode::PatternMismatch => "String does not match pattern".into(),
            
            // Number errors
            ErrorCode::NumberTooSmall => "Number is too small".into(),
            ErrorCode::NumberTooLarge => "Number is too large".into(),
            ErrorCode::InvalidNumber => "Invalid number".into(),
            
            // Object errors
            ErrorCode::RequiredField => "Field is required".into(),
            ErrorCode::UnknownField => "Unknown field".into(),
            ErrorCode::InvalidType => "Invalid type".into(),
            
            // Custom error
            ErrorCode::Custom(msg) => msg.clone(),
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            // String errors
            ErrorCode::StringTooShort => "string.too_short",
            ErrorCode::StringTooLong => "string.too_long",
            ErrorCode::InvalidEmail => "string.email",
            ErrorCode::PatternMismatch => "string.pattern",
            
            // Number errors
            ErrorCode::NumberTooSmall => "number.too_small",
            ErrorCode::NumberTooLarge => "number.too_large",
            ErrorCode::InvalidNumber => "number.invalid",
            
            // Object errors
            ErrorCode::RequiredField => "object.required",
            ErrorCode::UnknownField => "object.unknown_field",
            ErrorCode::InvalidType => "object.invalid_type",
            
            // Custom error
            ErrorCode::Custom(_) => "custom",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}