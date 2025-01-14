use std::collections::{HashMap, HashSet};
use serde::{de::DeserializeOwned};
use serde_json::Value;

use crate::error::{ValidationError, ParseError};
use super::{Schema, SchemaType, HasErrorMessages, get_type_name, validate_schema_type};

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
            error_messages: HashMap::from([
                ("object.unknown_field".to_string(), "Unknown field: {field}".to_string())
            ]),
        }
    }
}

impl ObjectSchema {
    pub fn field(mut self, name: &str, schema: impl Schema) -> Self {
        let schema_type = schema.into_schema_type();
        let name = name.to_string();
        self.fields.insert(name.clone(), Box::new(schema_type));
        self.required.insert(name.clone());
        self.error_messages.insert(format!("field.{}.required", name), format!("Field '{}' is required", name));
        self
    }

    pub fn optional_field(mut self, name: &str, schema: impl Schema) -> Self {
        let schema_type = schema.into_schema_type();
        let name = name.to_string();
        self.fields.insert(name.clone(), Box::new(schema_type));
        self.required.remove(&name);
        self.error_messages.insert(format!("field.{}.optional", name), "This field is optional".to_string());
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

    pub fn parse<T>(&self, value: &Value) -> Result<T, ParseError>
    where
        T: DeserializeOwned,
    {
        // First validate the value
        self.validate(value).map_err(ParseError::from)?;
        
        // Then try to deserialize into the target type
        serde_json::from_value(value.clone())
            .map_err(|e| ParseError::Parse(format!("Failed to parse object: {}", e)))
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
                let mut result = serde_json::Map::new();

                // Check required fields and validate each field
                for (field, schema) in &self.fields {
                    match obj.get(field) {
                        Some(value) => {
                            match validate_schema_type(schema.as_ref(), value) {
                                Ok(validated) => {
                                    result.insert(field.clone(), validated);
                                }
                                Err(e) => {
                                    return Err(e.with_path_prefix(field));
                                }
                            }
                        }
                        None => {
                            if self.required.contains(field) {
                                let mut err = ValidationError::new("object.required")
                                    .at(field)
                                    .with_details(|d| {
                                        d.field_name = Some(field.clone());
                                    });
                                err = err.message(format!("Field '{}' is required", field));
                                return Err(err);
                            }
                        }
                    }
                }

                // Check unknown fields if strict mode is enabled
                if self.error_messages.contains_key("object.unknown_field") {
                    for field in obj.keys() {
                        if !self.fields.contains_key(field) {
                            let mut err = ValidationError::new("object.unknown_field")
                                .at(field)
                                .with_details(|d| {
                                    d.field_name = Some(field.clone());
                                });
                            err = err.message(format!("Unknown field: {}", field));
                            return Err(err);
                        }
                    }
                } else {
                    // Copy over any additional fields in non-strict mode
                    for (field, value) in obj {
                        if !self.fields.contains_key(field) {
                            result.insert(field.clone(), value.clone());
                        }
                    }
                }

                Ok(Value::Object(result))
            }
            Value::Null if self.optional => Ok(value.clone()),
            Value::Null => {
                let err = ValidationError::new("object.required")
                    .message("This field is required");
                Err(err)
            }
            _ => {
                let err = ValidationError::new("object.invalid_type")
                    .with_details(|d| {
                        d.expected_type = Some("object".to_string());
                        d.actual_type = Some(get_type_name(value).to_string());
                    })
                    .message("Must be an object");
                Err(err)
            }
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Object(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use crate::schemas::{string::StringSchemaImpl, NumberSchema};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct User {
        name: String,
        age: u32,
        email: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Address {
        street: String,
        city: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Person {
        name: String,
        address: Address,
    }

    #[test]
    fn test_object_required_fields() {
        let schema = ObjectSchema::default()
            .field("name", StringSchemaImpl::default())
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
        assert!(err.to_string().contains("Field 'age' is required"));
    }

    #[test]
    fn test_object_optional_fields() {
        let schema = ObjectSchema::default()
            .field("name", StringSchemaImpl::default())
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
            .field("name", StringSchemaImpl::default())
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
            .field("street", StringSchemaImpl::default())
            .field("city", StringSchemaImpl::default());

        let schema = ObjectSchema::default()
            .field("name", StringSchemaImpl::default())
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
            .field("name", StringSchemaImpl::default())
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
            .field("name", StringSchemaImpl::default())
            .error_message("object.invalid_type", "Must be an object");

        let err = schema.validate(&json!("not an object")).unwrap_err();
        assert_eq!(err.context.code, "object.invalid_type");
        assert!(err.to_string().contains("Must be an object"));
    }

    #[test]
    fn test_object_parse_simple() {
        let schema = ObjectSchema::default()
            .field("name", StringSchemaImpl::default())
            .field("age", NumberSchema::default())
            .optional_field("email", StringSchemaImpl::default());

        let value = json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com"
        });

        let user: User = schema.parse(&value).unwrap();
        assert_eq!(user.name, "John");
        assert_eq!(user.age, 30);
        assert_eq!(user.email, Some("john@example.com".to_string()));

        // Test without optional field
        let value = json!({
            "name": "John",
            "age": 30
        });

        let user: User = schema.parse(&value).unwrap();
        assert_eq!(user.name, "John");
        assert_eq!(user.age, 30);
        assert_eq!(user.email, None);
    }

    #[test]
    fn test_object_parse_nested() {
        let address_schema = ObjectSchema::default()
            .field("street", StringSchemaImpl::default())
            .field("city", StringSchemaImpl::default());

        let schema = ObjectSchema::default()
            .field("name", StringSchemaImpl::default())
            .field("address", address_schema);

        let value = json!({
            "name": "John",
            "address": {
                "street": "123 Main St",
                "city": "New York"
            }
        });

        let person: Person = schema.parse(&value).unwrap();
        assert_eq!(person.name, "John");
        assert_eq!(person.address.street, "123 Main St");
        assert_eq!(person.address.city, "New York");
    }

    #[test]
    fn test_object_parse_validation_error() {
        let schema = ObjectSchema::default()
            .field("name", StringSchemaImpl::default())
            .field("age", NumberSchema::default());

        let value = json!({
            "name": "John",
            "age": "not a number"
        });

        let result = schema.parse::<User>(&value);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ParseError::Validation(err) => {
                assert!(err.to_string().contains("Expected number"));
            }
            ParseError::Parse(_) => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_object_parse_type_mismatch() {
        let schema = ObjectSchema::default()
            .field("name", StringSchemaImpl::default())
            .field("age", NumberSchema::default());

        let value = json!({
            "name": "John",
            "age": 30.5 // User expects u32, but JSON has f64
        });

        let result = schema.parse::<User>(&value);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ParseError::Parse(msg) => {
                assert!(msg.contains("Failed to parse object"));
            }
            ParseError::Validation(_) => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_object_parse_missing_field() {
        let schema = ObjectSchema::default()
            .field("name", StringSchemaImpl::default())
            .field("age", NumberSchema::default());

        let value = json!({
            "name": "John"
            // missing age field
        });

        let result = schema.parse::<User>(&value);
        assert!(result.is_err());
        
        match result.unwrap_err() {
            ParseError::Validation(err) => {
                assert_eq!(err.context.code, "object.required");
                assert_eq!(err.context.path, "age");
            }
            ParseError::Parse(_) => panic!("Expected ValidationError"),
        }
    }
}