use std::collections::HashMap;
use serde_json::Value;

use crate::error::ValidationError;
use super::{Schema, SchemaType, HasErrorMessages, ErrorMessage, get_type_name, transform::{Transformable, Transform, WithTransform}};

#[derive(Clone)]
pub struct NumberSchema {
    min: Option<f64>,
    max: Option<f64>,
    integer: bool,
    coerce: bool,
    optional: bool,
    error_messages: HashMap<String, String>,
}

impl Default for NumberSchema {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            integer: false,
            coerce: false,
            optional: false,
            error_messages: HashMap::new(),
        }
    }
}

impl NumberSchema {
    pub fn min(mut self, value: f64) -> Self {
        self.min = Some(value);
        self
    }

    pub fn max(mut self, value: f64) -> Self {
        self.max = Some(value);
        self
    }

    pub fn integer(mut self) -> Self {
        self.integer = true;
        self
    }

    pub fn coerce(mut self) -> Self {
        self.coerce = true;
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
}

impl HasErrorMessages for NumberSchema {
    fn error_messages(&self) -> &HashMap<String, String> {
        &self.error_messages
    }
}

impl Transformable for NumberSchema {
    fn with_transform(self, transform: Transform) -> WithTransform<Self> {
        WithTransform::new(self).with_transform(transform)
    }
}

impl Schema for NumberSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Null if self.optional => Ok(value.clone()),
            Value::Number(n) => {
                let num = n.as_f64().unwrap();
                self.validate_number(num)
            }
            Value::String(s) if self.coerce => {
                match s.parse::<f64>() {
                    Ok(num) => self.validate_number(num),
                    Err(_) => Err(ValidationError::new("number.invalid_type", "")
                        .with_message(self.get_error_message("number.invalid_type")
                            .unwrap_or_else(|| "Could not parse string as number".to_string()))
                        .with_type_info("number", "string (not a valid number)".to_string()))
                }
            }
            Value::Null => Err(ValidationError::new("number.required", "")
                .with_message(self.get_error_message("number.required")
                    .unwrap_or_else(|| "This field is required".to_string()))),
            _ => Err(ValidationError::new("number.invalid_type", "")
                .with_message(self.get_error_message("number.invalid_type")
                    .unwrap_or_else(|| format!("Expected number, got {}", get_type_name(value))))
                .with_type_info("number", get_type_name(value).to_string())),
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Number(self)
    }
}

impl NumberSchema {
    fn validate_number(&self, num: f64) -> Result<Value, ValidationError> {
        if self.integer && num.fract() != 0.0 {
            return Err(ValidationError::new("number.integer", "")
                .with_message(self.get_error_message("number.integer")
                    .unwrap_or_else(|| "Expected integer value".to_string())));
        }

        if let Some(min) = self.min {
            if num < min {
                return Err(ValidationError::new("number.min", "")
                    .with_message(self.get_error_message("number.min")
                        .unwrap_or_else(|| format!("Value {} is less than minimum {}", num, min)))
                    .with_min(min as i64));
            }
        }

        if let Some(max) = self.max {
            if num > max {
                return Err(ValidationError::new("number.max", "")
                    .with_message(self.get_error_message("number.max")
                        .unwrap_or_else(|| format!("Value {} is greater than maximum {}", num, max)))
                    .with_max(max as i64));
            }
        }

        Ok(Value::Number(serde_json::Number::from_f64(num).unwrap()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_number_range_validation() {
        let schema = NumberSchema::default()
            .min(0.0)
            .max(100.0)
            .error_message("number.min", "Must be at least {min}")
            .error_message("number.max", "Must be at most {max}");

        assert!(schema.validate(&json!(50)).is_ok());
        
        let err = schema.validate(&json!(-1)).unwrap_err();
        assert_eq!(err.context.code, "number.min");
        assert_eq!(err.context.min, Some(0));
        assert!(err.to_string().contains("Must be at least 0"));

        let err = schema.validate(&json!(101)).unwrap_err();
        assert_eq!(err.context.code, "number.max");
        assert_eq!(err.context.max, Some(100));
        assert!(err.to_string().contains("Must be at most 100"));
    }

    #[test]
    fn test_number_integer_validation() {
        let schema = NumberSchema::default()
            .integer()
            .error_message("number.integer", "Must be an integer");

        assert!(schema.validate(&json!(42)).is_ok());
        
        let err = schema.validate(&json!(42.5)).unwrap_err();
        assert_eq!(err.context.code, "number.integer");
        assert!(err.to_string().contains("Must be an integer"));
    }

    #[test]
    fn test_number_coercion() {
        let schema = NumberSchema::default()
            .min(0.0)
            .max(100.0)
            .coerce();

        assert!(schema.validate(&json!("42")).is_ok());
        assert!(schema.validate(&json!("42.5")).is_ok());
        assert!(schema.validate(&json!("-1")).is_err());
        assert!(schema.validate(&json!("not a number")).is_err());
    }

    #[test]
    fn test_number_optional() {
        let schema = NumberSchema::default()
            .min(0.0)
            .optional();

        assert!(schema.validate(&json!(42)).is_ok());
        assert!(schema.validate(&json!(null)).is_ok());
        assert!(schema.validate(&json!(-1)).is_err());
    }

    #[test]
    fn test_number_type_validation() {
        let schema = NumberSchema::default()
            .error_message("number.invalid_type", "Must be a number");

        assert!(schema.validate(&json!(42)).is_ok());
        
        let err = schema.validate(&json!("not a number")).unwrap_err();
        assert_eq!(err.context.code, "number.invalid_type");
        assert!(err.to_string().contains("Must be a number"));
    }
}