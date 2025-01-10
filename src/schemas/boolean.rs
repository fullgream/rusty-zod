use std::collections::HashMap;
use serde_json::Value;

use crate::error::ValidationError;
use super::{Schema, SchemaType, HasErrorMessages, get_type_name};

#[derive(Clone, Default)]
pub struct BooleanSchema {
    optional: bool,
    error_messages: HashMap<String, String>,
}

impl BooleanSchema {
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn error_message(mut self, code: impl Into<String>, message: impl Into<String>) -> Self {
        self.error_messages.insert(code.into(), message.into());
        self
    }
}

impl HasErrorMessages for BooleanSchema {
    fn error_messages(&self) -> &HashMap<String, String> {
        &self.error_messages
    }
}

impl Schema for BooleanSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Bool(_) => Ok(value.clone()),
            Value::Null if self.optional => Ok(value.clone()),
            Value::Null => Err(ValidationError::new("boolean.required")
                .message("This field is required")),
            _ => {
                let mut err = ValidationError::new("boolean.invalid_type")
                    .with_details(|d| {
                        d.expected_type = Some("boolean".to_string());
                        d.actual_type = Some(get_type_name(value).to_string());
                    });
                if let Some(msg) = self.error_messages.get("boolean.invalid_type") {
                    err = err.message(msg.clone());
                } else {
                    err = err.message("Must be a boolean value");
                }
                Err(err)
            }
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Boolean(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_boolean_validation() {
        let schema = BooleanSchema::default()
            .error_message("boolean.invalid_type", "Must be a boolean value");

        assert!(schema.validate(&json!(true)).is_ok());
        assert!(schema.validate(&json!(false)).is_ok());
        
        let err = schema.validate(&json!("true")).unwrap_err();
        assert_eq!(err.context.code, "boolean.invalid_type");
        assert!(err.to_string().contains("Must be a boolean value"));
    }

    #[test]
    fn test_boolean_optional() {
        let schema = BooleanSchema::default().optional();

        assert!(schema.validate(&json!(true)).is_ok());
        assert!(schema.validate(&json!(null)).is_ok());
        assert!(schema.validate(&json!("true")).is_err());
    }

    #[test]
    fn test_boolean_required() {
        let schema = BooleanSchema::default()
            .error_message("boolean.required", "This field is required");

        let err = schema.validate(&json!(null)).unwrap_err();
        assert_eq!(err.context.code, "boolean.required");
        assert!(err.to_string().contains("This field is required"));
    }
}