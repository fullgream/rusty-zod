use std::{collections::HashMap, sync::Arc};
use regex::Regex;
use serde_json::Value;

use crate::error::{ValidationError, ErrorCode};
use super::{Schema, SchemaType, HasErrorMessages, ErrorMessage, get_type_name, transform::{Transformable, Transform, WithTransform}};

#[derive(Clone)]
pub struct StringSchema {
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<Regex>,
    email: bool,
    optional: bool,
    error_messages: HashMap<String, String>,
    custom_validators: Vec<Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>>,
}

impl Default for StringSchema {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            pattern: None,
            email: false,
            optional: false,
            error_messages: HashMap::new(),
            custom_validators: Vec::new(),
        }
    }
}

impl StringSchema {
    pub fn min_length(mut self, length: usize) -> Self {
        self.min_length = Some(length);
        self
    }

    pub fn max_length(mut self, length: usize) -> Self {
        self.max_length = Some(length);
        self
    }

    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = Some(Regex::new(pattern).unwrap());
        self
    }

    pub fn email(mut self) -> Self {
        self.email = true;
        self
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn error_message(mut self, code: impl Into<String>, message: impl Into<String>) -> Self {
        self.error_messages.insert(code.into(), message.into());
        self
    }

    pub fn custom<F>(mut self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        self.custom_validators.push(Arc::new(validator));
        self
    }

    pub fn url(self) -> Self {
        self.pattern(r"^https?://[\w\-]+(\.[\w\-]+)+[/#?]?.*$")
            .error_message("string.url", "Invalid URL format")
    }

    pub fn uuid(self) -> Self {
        self.pattern(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
            .error_message("string.uuid", "Invalid UUID format")
    }

    pub fn ip(self) -> Self {
        self.pattern(r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$")
            .error_message("string.ip", "Invalid IP address format")
    }

    pub fn trim(self) -> WithTransform<Self> {
        self.with_transform(Transform::Trim)
    }

    pub fn to_lower_case(self) -> WithTransform<Self> {
        self.with_transform(Transform::ToLowerCase)
    }

    pub fn to_upper_case(self) -> WithTransform<Self> {
        self.with_transform(Transform::ToUpperCase)
    }
}

impl HasErrorMessages for StringSchema {
    fn error_messages(&self) -> &HashMap<String, String> {
        &self.error_messages
    }
}

impl Transformable for StringSchema {
    fn with_transform(self, transform: Transform) -> WithTransform<Self> {
        WithTransform::new(self).with_transform(transform)
    }
}

impl Schema for StringSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Null if self.optional => Ok(value.clone()),
            Value::String(s) => {
                if let Some(min_len) = self.min_length {
                    if s.len() < min_len {
                        return Err(ValidationError::new(ErrorCode::StringTooShort)
                            .with_details(|d| {
                                d.min_length = Some(min_len);
                            })
                            .message(self.get_error_message("string.too_short")
                                .unwrap_or_else(|| format!("String must be at least {} characters long", min_len))));
                    }
                }

                if let Some(max_len) = self.max_length {
                    if s.len() > max_len {
                        return Err(ValidationError::new(ErrorCode::StringTooLong)
                            .with_details(|d| {
                                d.max_length = Some(max_len);
                            })
                            .message(self.get_error_message("string.too_long")
                                .unwrap_or_else(|| format!("String must be at most {} characters long", max_len))));
                    }
                }

                if let Some(pattern) = &self.pattern {
                    if !pattern.is_match(s) {
                        return Err(ValidationError::new(ErrorCode::PatternMismatch)
                            .with_details(|d| {
                                d.pattern = Some(pattern.as_str().to_string());
                            })
                            .message(self.get_error_message("string.pattern")
                                .unwrap_or_else(|| format!("String must match pattern: {}", pattern.as_str()))));
                    }
                }

                if self.email {
                    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
                    if !email_regex.is_match(s) {
                        return Err(ValidationError::new(ErrorCode::InvalidEmail)
                            .message(self.get_error_message("string.email")
                                .unwrap_or_else(|| "Must be a valid email address".to_string())));
                    }
                }

                for validator in &self.custom_validators {
                    if let Err(msg) = validator(s) {
                        return Err(ValidationError::new(ErrorCode::Custom(msg.clone())));
                    }
                }

                Ok(value.clone())
            }
            Value::Null => Err(ValidationError::new(ErrorCode::RequiredField)
                .message(self.get_error_message("string.required")
                    .unwrap_or_else(|| "This field is required".to_string()))),
            _ => Err(ValidationError::new(ErrorCode::InvalidType)
                .with_details(|d| {
                    d.expected_type = Some("string".to_string());
                    d.actual_type = Some(get_type_name(value).to_string());
                })
                .message(self.get_error_message("string.invalid_type")
                    .unwrap_or_else(|| format!("Expected string, got {}", get_type_name(value))))),
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::String(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_string_length_validation() {
        let schema = StringSchema::default()
            .min_length(3)
            .max_length(5)
            .error_message("string.too_short", "Minimum length is {min_length}")
            .error_message("string.too_long", "Maximum length is {max_length}");

        assert!(schema.validate(&json!("1234")).is_ok());
        
        let err = schema.validate(&json!("12")).unwrap_err();
        assert_eq!(err.context.code, "string.too_short");
        assert_eq!(err.context.min_length, Some(3));
        assert!(err.to_string().contains("Minimum length is 3"));

        let err = schema.validate(&json!("123456")).unwrap_err();
        assert_eq!(err.context.code, "string.too_long");
        assert_eq!(err.context.max_length, Some(5));
        assert!(err.to_string().contains("Maximum length is 5"));
    }

    #[test]
    fn test_string_pattern_validation() {
        let schema = StringSchema::default()
            .pattern(r"^[A-Z]+$")
            .error_message("string.pattern", "Must be uppercase letters only");

        assert!(schema.validate(&json!("ABC")).is_ok());
        
        let err = schema.validate(&json!("abc")).unwrap_err();
        assert_eq!(err.context.code, "string.pattern");
        assert!(err.to_string().contains("Must be uppercase letters only"));
    }

    #[test]
    fn test_string_email_validation() {
        let schema = StringSchema::default()
            .email()
            .error_message("string.email", "Invalid email address");

        assert!(schema.validate(&json!("test@example.com")).is_ok());
        
        let err = schema.validate(&json!("not-an-email")).unwrap_err();
        assert_eq!(err.context.code, "string.email");
        assert!(err.to_string().contains("Invalid email address"));
    }

    #[test]
    fn test_string_optional() {
        let schema = StringSchema::default()
            .min_length(3)
            .optional();

        assert!(schema.validate(&json!("test")).is_ok());
        assert!(schema.validate(&json!(null)).is_ok());
        assert!(schema.validate(&json!("ab")).is_err());
    }

    #[test]
    fn test_string_custom_validation() {
        let schema = StringSchema::default()
            .custom(|s| {
                if s.chars().all(|c| c.is_ascii_digit()) {
                    Ok(())
                } else {
                    Err("Must contain only digits".to_string())
                }
            });

        assert!(schema.validate(&json!("123")).is_ok());
        
        let err = schema.validate(&json!("abc123")).unwrap_err();
        assert_eq!(err.context.code, "string.custom");
        assert!(err.to_string().contains("Must contain only digits"));
    }

    #[test]
    fn test_string_url_validation() {
        let schema = StringSchema::default().url();

        assert!(schema.validate(&json!("https://example.com")).is_ok());
        assert!(schema.validate(&json!("http://sub.domain.com/path?q=1")).is_ok());
        assert!(schema.validate(&json!("not-a-url")).is_err());
    }

    #[test]
    fn test_string_uuid_validation() {
        let schema = StringSchema::default().uuid();

        assert!(schema.validate(&json!("550e8400-e29b-41d4-a716-446655440000")).is_ok());
        assert!(schema.validate(&json!("not-a-uuid")).is_err());
    }

    #[test]
    fn test_string_ip_validation() {
        let schema = StringSchema::default().ip();

        assert!(schema.validate(&json!("192.168.1.1")).is_ok());
        assert!(schema.validate(&json!("256.1.2.3")).is_err());
        assert!(schema.validate(&json!("not-an-ip")).is_err());
    }

    #[test]
    fn test_string_transformations() {
        let schema = StringSchema::default()
            .trim()
            .to_lower_case()
            .email();

        assert_eq!(
            schema.validate(&json!("  TEST@EXAMPLE.COM  ")).unwrap(),
            json!("test@example.com")
        );

        let err = schema.validate(&json!("  NOT-AN-EMAIL  ")).unwrap_err();
        assert_eq!(err.context.code, "string.email");
    }

    #[test]
    fn test_string_transform_chain() {
        let schema = StringSchema::default()
            .trim()
            .to_upper_case()
            .min_length(5);

        assert_eq!(
            schema.validate(&json!("  hello  ")).unwrap(),
            json!("HELLO")
        );

        let err = schema.validate(&json!("  hi  ")).unwrap_err();
        assert_eq!(err.context.code, "string.too_short");
    }
}