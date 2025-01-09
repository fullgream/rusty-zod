use std::fmt;
use super::error_code::ErrorCode;

#[derive(Debug, serde::Serialize)]
pub struct ValidationError {
    pub context: ValidationErrorContext,
}

#[derive(Debug, serde::Serialize)]
pub struct ValidationErrorContext {
    pub code: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "ValidationDetails::is_empty")]
    pub details: ValidationDetails,
}

#[derive(Debug, Default, serde::Serialize)]
pub struct ValidationDetails {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub pattern: Option<String>,
    pub expected_type: Option<String>,
    pub actual_type: Option<String>,
    pub field_name: Option<String>,
}

impl ValidationDetails {
    pub fn is_empty(&self) -> bool {
        self.min_length.is_none() &&
        self.max_length.is_none() &&
        self.min_value.is_none() &&
        self.max_value.is_none() &&
        self.pattern.is_none() &&
        self.expected_type.is_none() &&
        self.actual_type.is_none() &&
        self.field_name.is_none()
    }
}

impl ValidationError {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            context: ValidationErrorContext {
                code: code.code().to_string(),
                path: String::new(),
                message: None,
                details: ValidationDetails::default(),
            },
        }
    }

    pub fn at(mut self, path: impl Into<String>) -> Self {
        self.context.path = path.into();
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.context.message = Some(message.into());
        self
    }

    pub fn with_details(mut self, f: impl FnOnce(&mut ValidationDetails)) -> Self {
        f(&mut self.context.details);
        self
    }

    pub fn format_message(&self) -> String {
        if let Some(ref message) = self.context.message {
            return message.clone();
        }

        let mut msg = match self.context.code.as_str() {
            "string.too_short" => "String must be at least {min_length} characters long",
            "string.too_long" => "String must be at most {max_length} characters long",
            "string.email" => "Must be a valid email address",
            "string.pattern" => "String must match pattern: {pattern}",
            "number.too_small" => "Number must be greater than or equal to {min}",
            "number.too_large" => "Number must be less than or equal to {max}",
            "object.required" => "This field is required",
            "object.unknown_field" => "Unknown field: {field}",
            "object.invalid_type" => "Expected {expected_type}, got {actual_type}",
            _ => "Validation error"
        }.to_string();
        
        // Replace placeholders with actual values
        if let Some(min) = self.context.details.min_length {
            msg = msg.replace("{min_length}", &min.to_string());
        }
        if let Some(max) = self.context.details.max_length {
            msg = msg.replace("{max_length}", &max.to_string());
        }
        if let Some(min) = self.context.details.min_value {
            msg = msg.replace("{min}", &min.to_string());
        }
        if let Some(max) = self.context.details.max_value {
            msg = msg.replace("{max}", &max.to_string());
        }
        if let Some(ref pattern) = self.context.details.pattern {
            msg = msg.replace("{pattern}", pattern);
        }
        if let Some(ref field) = self.context.details.field_name {
            msg = msg.replace("{field}", field);
        }
        if let (Some(ref expected), Some(ref actual)) = (
            self.context.details.expected_type.as_ref(),
            self.context.details.actual_type.as_ref()
        ) {
            msg = msg
                .replace("{expected_type}", expected)
                .replace("{actual_type}", actual);
        }

        msg
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_message())
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_length_error() {
        let error = ValidationError::new(ErrorCode::StringTooShort)
            .at("name")
            .with_details(|d| {
                d.min_length = Some(3);
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "string.too_short",
                "path": "name",
                "message": "String must be at least 3 characters long",
                "details": {
                    "min_length": 3
                }
            }
        }));
    }

    #[test]
    fn test_invalid_email_error() {
        let error = ValidationError::new(ErrorCode::InvalidEmail)
            .at("email");

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "string.email",
                "path": "email",
                "message": "Must be a valid email address"
            }
        }));
    }

    #[test]
    fn test_type_mismatch_error() {
        let error = ValidationError::new(ErrorCode::InvalidType)
            .at("age")
            .with_details(|d| {
                d.expected_type = Some("number".to_string());
                d.actual_type = Some("string".to_string());
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "object.invalid_type",
                "path": "age",
                "message": "Expected number, got string",
                "details": {
                    "expected_type": "number",
                    "actual_type": "string"
                }
            }
        }));
    }

    #[test]
    fn test_nested_path_error() {
        let error = ValidationError::new(ErrorCode::RequiredField)
            .at("address.street");

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "object.required",
                "path": "address.street",
                "message": "This field is required"
            }
        }));
    }

    #[test]
    fn test_custom_message() {
        let error = ValidationError::new(ErrorCode::StringTooShort)
            .at("name")
            .message("Name is too short")
            .with_details(|d| {
                d.min_length = Some(3);
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "string.too_short",
                "path": "name",
                "message": "Name is too short",
                "details": {
                    "min_length": 3
                }
            }
        }));
    }
}