use std::collections::{HashMap, HashSet};
use serde_json::Value;

use crate::error::ValidationError;
use super::{Schema, SchemaType, HasErrorMessages, ErrorMessage, get_type_name, validate_schema_type};

#[derive(Clone)]
pub struct ObjectSchema {
    fields: HashMap<String, Box<SchemaType>>,
    required: HashSet<String>,
    optional: bool,
    error_messages: HashMap<String, String>,
}

impl Default for ObjectSchema {
    fn default() -> Self {
        Self {
            fields: HashMap::new(),
            required: HashSet::new(),
            optional: false,
            error_messages: HashMap::new(),
        }
    }
}

impl ObjectSchema {
    pub fn field(mut self, name: &str, schema: impl Schema) -> Self {
        self.fields.insert(name.to_string(), Box::new(schema.into_schema_type()));
        self.required.insert(name.to_string());
        self
    }

    pub fn optional_field(mut self, name: &str, schema: impl Schema) -> Self {
        self.fields.insert(name.to_string(), Box::new(schema.into_schema_type()));
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

    pub fn strict(self) -> Self {
        self.error_message("object.unknown_field", "Unknown field: {field}")
    }
}

impl HasErrorMessages for ObjectSchema {
    fn error_messages(&self) -> &HashMap<String, String> {
        &self.error_messages
    }
}

impl Schema for ObjectSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Object(obj) => {
                // Check required fields
                for field in &self.required {
                    if !obj.contains_key(field) {
                        return Err(ValidationError::new("object.required", field)
                            .with_message(self.get_error_message("object.required")
                                .unwrap_or_else(|| format!("Field {} is required", field))));
                    }
                }

                // Check unknown fields if strict mode is enabled
                if self.error_messages.contains_key("object.unknown_field") {
                    for field in obj.keys() {
                        if !self.fields.contains_key(field) {
                            return Err(ValidationError::new("object.unknown_field", field)
                                .with_message(self.get_error_message("object.unknown_field")
                                    .unwrap_or_else(|| format!("Unknown field: {}", field))
                                    .replace("{field}", field)));
                        }
                    }
                }

                // Validate each field
                for (field, schema) in &self.fields {
                    if let Some(value) = obj.get(field) {
                        if let Err(e) = validate_schema_type(schema.as_ref(), value) {
                            return Err(e.with_path_prefix(field));
                        }
                    }
                }

                Ok(value.clone())
            }
            Value::Null if self.optional => Ok(value.clone()),
            Value::Null => Err(ValidationError::new("object.required", "")
                .with_message(self.get_error_message("object.required")
                    .unwrap_or_else(|| "This field is required".to_string()))),
            _ => Err(ValidationError::new("object.invalid_type", "")
                .with_message(self.get_error_message("object.invalid_type")
                    .unwrap_or_else(|| format!("Expected object, got {}", get_type_name(value))))
                .with_type_info("object", get_type_name(value).to_string())),
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Object(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::schemas::{StringSchema, NumberSchema};

    #[test]
    fn test_object_required_fields() {
        let schema = ObjectSchema::default()
            .field("name", StringSchema::default())
            .field("age", NumberSchema::default());

        assert!(schema.validate(&json!({
            "name": "John",
            "age": 30
        })).is_ok());
        
        let err = schema.validate(&json!({
            "name": "John"
        })).unwrap_err();
        assert_eq!(err.context.code, "object.required");
        assert_eq!(err.context.path, "age");
        assert!(err.to_string().contains("Field age is required"));
    }

    #[test]
    fn test_object_optional_fields() {
        let schema = ObjectSchema::default()
            .field("name", StringSchema::default())
            .optional_field("age", NumberSchema::default());

        assert!(schema.validate(&json!({
            "name": "John",
            "age": 30
        })).is_ok());

        assert!(schema.validate(&json!({
            "name": "John"
        })).is_ok());

        assert!(schema.validate(&json!({
            "age": 30
        })).is_err());
    }

    #[test]
    fn test_object_strict_mode() {
        let schema = ObjectSchema::default()
            .field("name", StringSchema::default())
            .strict();

        assert!(schema.validate(&json!({
            "name": "John"
        })).is_ok());

        let err = schema.validate(&json!({
            "name": "John",
            "unknown": "field"
        })).unwrap_err();
        assert_eq!(err.context.code, "object.unknown_field");
        assert!(err.to_string().contains("Unknown field: unknown"));
    }

    #[test]
    fn test_object_nested_validation() {
        let address_schema = ObjectSchema::default()
            .field("street", StringSchema::default())
            .field("city", StringSchema::default());

        let schema = ObjectSchema::default()
            .field("name", StringSchema::default())
            .field("address", address_schema);

        assert!(schema.validate(&json!({
            "name": "John",
            "address": {
                "street": "123 Main St",
                "city": "New York"
            }
        })).is_ok());

        let err = schema.validate(&json!({
            "name": "John",
            "address": {
                "street": "123 Main St"
            }
        })).unwrap_err();
        assert_eq!(err.context.path, "address.city");
    }

    #[test]
    fn test_object_optional() {
        let schema = ObjectSchema::default()
            .field("name", StringSchema::default())
            .optional();

        assert!(schema.validate(&json!({
            "name": "John"
        })).is_ok());
        assert!(schema.validate(&json!(null)).is_ok());
        assert!(schema.validate(&json!("not an object")).is_err());
    }

    #[test]
    fn test_object_type_validation() {
        let schema = ObjectSchema::default()
            .field("name", StringSchema::default())
            .error_message("object.invalid_type", "Must be an object");

        let err = schema.validate(&json!("not an object")).unwrap_err();
        assert_eq!(err.context.code, "object.invalid_type");
        assert!(err.to_string().contains("Must be an object"));
    }
}