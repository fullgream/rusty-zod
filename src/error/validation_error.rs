use std::fmt;
#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationError {
    pub context: ValidationErrorContext,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ValidationErrorContext {
    pub code: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "ValidationDetails::is_empty")]
    pub details: ValidationDetails,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct ValidationDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub fn new(code: impl Into<String>) -> Self {
        let code = code.into();
        let message = match code.as_str() {
            "string.too_short" => "String must be at least {min_length} characters long",
            "string.too_long" => "String must be at most {max_length} characters long",
            "string.email" => "Invalid email address",
            "string.pattern" => "String must match pattern: {pattern}",
            "number.too_small" => "Number must be greater than or equal to {min_value}",
            "number.too_large" => "Number must be less than or equal to {max_value}",
            "object.required" => "Field '{field_name}' is required",
            "object.unknown_field" => "Unknown field: {field_name}",
            "object.invalid_type" => "Expected {expected_type}, got {actual_type}",
            "array.min_items" => "Must have at least {min_items} items",
            "array.max_items" => "Must have at most {max_items} items",
            "array.type" => "Must be an array",
            "boolean.type" => "Must be a boolean value",
            "number.type" => "Must be a number",
            "number.integer" => "Must be an integer",
            "object.type" => "Must be an object",
            _ => "Validation error"
        }.to_string();

        Self {
            context: ValidationErrorContext {
                code,
                path: String::new(),
                message: Some(message),
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

    pub fn with_message(self, message: impl Into<String>) -> Self {
        self.message(message)
    }

    pub fn with_path_prefix(mut self, prefix: impl Into<String>) -> Self {
        let prefix = prefix.into();
        if self.context.path.is_empty() {
            self.context.path = prefix;
        } else {
            self.context.path = format!("{}.{}", prefix, self.context.path);
        }
        self
    }

    pub fn with_details(mut self, f: impl FnOnce(&mut ValidationDetails)) -> Self {
        f(&mut self.context.details);
        self
    }

    pub fn with_min(mut self, min: i64) -> Self {
        self.context.details.min_value = Some(min as f64);
        self
    }

    pub fn with_max(mut self, max: i64) -> Self {
        self.context.details.max_value = Some(max as f64);
        self
    }

    pub fn with_type_info(mut self, expected: impl Into<String>, actual: impl Into<String>) -> Self {
        self.context.details.expected_type = Some(expected.into());
        self.context.details.actual_type = Some(actual.into());
        self
    }

    pub fn format_message(&mut self) -> String {
        let msg = if let Some(ref message) = self.context.message {
            message.clone()
        } else {
            match self.context.code.as_str() {
                "string.too_short" => "String must be at least {min_length} characters long",
                "string.too_long" => "String must be at most {max_length} characters long",
                "string.email" => "Invalid email address",
                "string.pattern" => "String must match pattern: {pattern}",
                "number.too_small" => "Number must be greater than or equal to {min_value}",
                "number.too_large" => "Number must be less than or equal to {max_value}",
                "object.required" => "Field '{field_name}' is required",
                "object.unknown_field" => "Unknown field: {field_name}",
                "object.invalid_type" => "Expected {expected_type}, got {actual_type}",
                "array.min_items" => "Must have at least {min_items} items",
                "array.max_items" => "Must have at most {max_items} items",
                "array.type" => "Must be an array",
                "boolean.type" => "Must be a boolean value",
                "number.type" => "Must be a number",
                "number.integer" => "Must be an integer",
                "object.type" => "Must be an object",
                _ => "Validation error"
            }.to_string()
        };

        // Update the message
        self.context.message = Some(msg.clone());

        // Replace placeholders with actual values
        let mut formatted_msg = msg.clone();
        if let Some(min) = self.context.details.min_length {
            formatted_msg = formatted_msg.replace("{min_length}", &min.to_string());
            formatted_msg = formatted_msg.replace("{min_items}", &min.to_string());
        }
        if let Some(max) = self.context.details.max_length {
            formatted_msg = formatted_msg.replace("{max_length}", &max.to_string());
            formatted_msg = formatted_msg.replace("{max_items}", &max.to_string());
        }
        if let Some(min) = self.context.details.min_value {
            formatted_msg = formatted_msg.replace("{min_value}", &min.to_string());
            formatted_msg = formatted_msg.replace("{min}", &min.to_string());
        }
        if let Some(max) = self.context.details.max_value {
            formatted_msg = formatted_msg.replace("{max_value}", &max.to_string());
            formatted_msg = formatted_msg.replace("{max}", &max.to_string());
        }
        if let Some(ref pattern) = self.context.details.pattern {
            formatted_msg = formatted_msg.replace("{pattern}", pattern);
        }
        if let Some(ref field) = self.context.details.field_name {
            formatted_msg = formatted_msg.replace("{field_name}", field);
            formatted_msg = formatted_msg.replace("{field}", field);
        }
        if let (Some(ref expected), Some(ref actual)) = (
            self.context.details.expected_type.as_ref(),
            self.context.details.actual_type.as_ref()
        ) {
            formatted_msg = formatted_msg.replace("{expected_type}", expected);
            formatted_msg = formatted_msg.replace("{actual_type}", actual);
        }

        // Return the formatted message
        formatted_msg
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut error = self.clone();
        write!(f, "{}", error.format_message())
    }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
    use crate::error::error_code;

    use super::*;
    use error_code::ErrorCode;
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
                "message": "String must be at least {min_length} characters long",
                "details": {
                    "min_length": 3
                }
            }
        }));
        assert_eq!(error.to_string(), "String must be at least 3 characters long");
    }

    #[test]
    fn test_string_max_length_error() {
        let error = ValidationError::new(ErrorCode::StringTooLong)
            .at("description")
            .with_details(|d| {
                d.max_length = Some(100);
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "string.too_long",
                "path": "description",
                "message": "String must be at most {max_length} characters long",
                "details": {
                    "max_length": 100
                }
            }
        }));
        assert_eq!(error.to_string(), "String must be at most 100 characters long");
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
                "message": "Invalid email address"
            }
        }));
        assert_eq!(error.to_string(), "Invalid email address");
    }

    #[test]
    fn test_pattern_mismatch_error() {
        let error = ValidationError::new(ErrorCode::PatternMismatch)
            .at("phone")
            .with_details(|d| {
                d.pattern = Some(r"^\+\d{1,3}-\d{3}-\d{3}-\d{4}$".to_string());
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "string.pattern",
                "path": "phone",
                "message": "String must match pattern: {pattern}",
                "details": {
                    "pattern": r"^\+\d{1,3}-\d{3}-\d{3}-\d{4}$"
                }
            }
        }));
        assert_eq!(error.to_string(), "String must match pattern: ^\\+\\d{1,3}-\\d{3}-\\d{3}-\\d{4}$");
    }

    #[test]
    fn test_number_range_error() {
        let error = ValidationError::new(ErrorCode::NumberTooSmall)
            .at("age")
            .with_details(|d| {
                d.min_value = Some(0.0);
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "number.too_small",
                "path": "age",
                "message": "Number must be greater than or equal to {min_value}",
                "details": {
                    "min_value": 0.0
                }
            }
        }));
        assert_eq!(error.to_string(), "Number must be greater than or equal to 0");

        let error = ValidationError::new(ErrorCode::NumberTooLarge)
            .at("age")
            .with_details(|d| {
                d.max_value = Some(150.0);
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "number.too_large",
                "path": "age",
                "message": "Number must be less than or equal to {max_value}",
                "details": {
                    "max_value": 150.0
                }
            }
        }));
        assert_eq!(error.to_string(), "Number must be less than or equal to 150");
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
                "message": "Expected {expected_type}, got {actual_type}",
                "details": {
                    "expected_type": "number",
                    "actual_type": "string"
                }
            }
        }));
        assert_eq!(error.to_string(), "Expected number, got string");
    }

    #[test]
    fn test_unknown_field_error() {
        let error = ValidationError::new(ErrorCode::UnknownField)
            .at("unknown_field")
            .with_details(|d| {
                d.field_name = Some("unknown_field".to_string());
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "object.unknown_field",
                "path": "unknown_field",
                "message": "Unknown field: {field_name}",
                "details": {
                    "field_name": "unknown_field"
                }
            }
        }));
        assert_eq!(error.to_string(), "Unknown field: unknown_field");
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
                "message": "Field '{field_name}' is required"
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

    #[test]
    fn test_custom_error() {
        let error = ValidationError::new(ErrorCode::Custom("custom.validation.error".to_string()))
            .at("field")
            .message("Custom validation failed");

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "custom",
                "path": "field",
                "message": "Custom validation failed"
            }
        }));
    }

    #[test]
    fn test_multiple_details() {
        let error = ValidationError::new(ErrorCode::Custom("validation.error".to_string()))
            .at("field")
            .with_details(|d| {
                d.min_length = Some(3);
                d.max_length = Some(10);
                d.pattern = Some(r"\d+".to_string());
                d.min_value = Some(0.0);
                d.max_value = Some(100.0);
                d.expected_type = Some("string".to_string());
                d.actual_type = Some("number".to_string());
                d.field_name = Some("test_field".to_string());
            });

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "custom",
                "path": "field",
                "message": "Validation error",
                "details": {
                    "min_length": 3,
                    "max_length": 10,
                    "pattern": r"\d+",
                    "min_value": 0.0,
                    "max_value": 100.0,
                    "expected_type": "string",
                    "actual_type": "number",
                    "field_name": "test_field"
                }
            }
        }));
    }

    #[test]
    fn test_empty_details() {
        let error = ValidationError::new(ErrorCode::RequiredField)
            .at("field");

        let json = error.to_json();
        assert_eq!(json, json!({
            "context": {
                "code": "object.required",
                "path": "field",
                "message": "Field '{field_name}' is required"
            }
        }));
    }

    #[test]
    fn test_error_display() {
        let error = ValidationError::new(ErrorCode::StringTooShort)
            .at("name")
            .with_details(|d| {
                d.min_length = Some(3);
            });

        assert_eq!(
            error.to_string(),
            "String must be at least 3 characters long"
        );

        let error = ValidationError::new(ErrorCode::StringTooShort)
            .at("name")
            .message("Custom error message");

        assert_eq!(
            error.to_string(),
            "Custom error message"
        );
    }
}