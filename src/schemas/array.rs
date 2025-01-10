use std::collections::HashMap;
use serde_json::Value;

use crate::error::ValidationError;
use super::{Schema, SchemaType, HasErrorMessages, get_type_name, validate_schema_type};

#[derive(Clone)]
pub struct ArraySchema {
    item_schema: Box<SchemaType>,
    min_items: Option<usize>,
    max_items: Option<usize>,
    optional: bool,
    error_messages: HashMap<String, String>,
}

impl ArraySchema {
    pub fn new(schema: impl Schema) -> Self {
        Self {
            item_schema: Box::new(schema.into_schema_type()),
            min_items: None,
            max_items: None,
            optional: false,
            error_messages: HashMap::new(),
        }
    }

    pub fn min_items(mut self, count: usize) -> Self {
        self.min_items = Some(count);
        self.error_messages.insert("array.min_items".to_string(), format!("Must have at least {} items", count));
        self
    }

    pub fn max_items(mut self, count: usize) -> Self {
        self.max_items = Some(count);
        self.error_messages.insert("array.max_items".to_string(), format!("Must have at most {} items", count));
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

impl HasErrorMessages for ArraySchema {
    fn error_messages(&self) -> &HashMap<String, String> {
        &self.error_messages
    }
}

impl Schema for ArraySchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Array(arr) => {
                if let Some(min_items) = self.min_items {
                    if arr.len() < min_items {
                        let mut err = ValidationError::new("array.min_items")
                            .with_details(|d| {
                                d.min_length = Some(min_items);
                            });
                        if let Some(msg) = self.error_messages.get("array.min_items") {
                            err = err.message(msg.clone());
                        } else {
                            err = err.message("less than minimum".to_string());
                        }
                        return Err(err);
                    }
                }

                if let Some(max_items) = self.max_items {
                    if arr.len() > max_items {
                        let mut err = ValidationError::new("array.max_items")
                            .with_details(|d| {
                                d.max_length = Some(max_items);
                            });
                        if let Some(msg) = self.error_messages.get("array.max_items") {
                            err = err.message(msg.clone());
                        } else {
                            err = err.message(format!("Must have at most {} items", max_items));
                        }
                        return Err(err);
                    }
                }

                let mut result = Vec::new();
                for (i, item) in arr.iter().enumerate() {
                    match validate_schema_type(self.item_schema.as_ref(), item) {
                        Ok(validated) => result.push(validated),
                        Err(e) => {
                            let mut err = e.with_path_prefix(&i.to_string());
                            if let Some(msg) = self.error_messages.get("array.item") {
                                err = err.message(msg.clone());
                            } else {
                                err = err.message(format!("Item {} is invalid", i));
                            }
                            return Err(err);
                        }
                    }
                }

                Ok(Value::Array(result))
            }
            Value::Null if self.optional => Ok(value.clone()),
            Value::Null => {
                let mut err = ValidationError::new("array.required");
                if let Some(msg) = self.error_messages.get("array.required") {
                    err = err.message(msg.clone());
                } else {
                    err = err.message("This field is required");
                }
                Err(err)
            }
            _ => {
                let mut err = ValidationError::new("array.invalid_type")
                    .with_details(|d| {
                        d.expected_type = Some("array".to_string());
                        d.actual_type = Some(get_type_name(value).to_string());
                    });
                if let Some(msg) = self.error_messages.get("array.invalid_type") {
                    err = err.message(msg.clone());
                } else {
                    err = err.message("Must be an array");
                }
                Err(err)
            }
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Array(Box::new(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::schemas::{string::StringSchemaImpl, NumberSchema};

    #[test]
    fn test_array_length_validation() {
        let schema = ArraySchema::new(StringSchemaImpl::default())
            .min_items(2)
            .max_items(4)
            .error_message("array.min_items", "Must have at least 2 items")
            .error_message("array.max_items", "Must have at most 4 items");

        assert!(schema.validate(&json!(["a", "b", "c"])).is_ok());
        
        let err = schema.validate(&json!(["a"])).unwrap_err();
        assert_eq!(err.context.code, "array.min_items");
        assert_eq!(err.context.details.min_length, Some(2));
        assert!(err.to_string().contains("Must have at least 2 items"));

        let err = schema.validate(&json!(["a", "b", "c", "d", "e"])).unwrap_err();
        assert_eq!(err.context.code, "array.max_items");
        assert_eq!(err.context.details.max_length, Some(4));
        assert!(err.to_string().contains("Must have at most 4 items"));
    }

    #[test]
    fn test_array_item_validation() {
        let schema = ArraySchema::new(NumberSchema::default().min(0.0).max(100.0));

        assert!(schema.validate(&json!([1, 50, 100])).is_ok());
        
        let err = schema.validate(&json!([1, -1, 50])).unwrap_err();
        assert!(err.context.path.contains("1"));
        assert_eq!(err.to_string(), "Item 1 is invalid");
    }

    #[test]
    fn test_array_optional() {
        let schema = ArraySchema::new(StringSchemaImpl::default()).optional();

        assert!(schema.validate(&json!(["a", "b"])).is_ok());
        assert!(schema.validate(&json!(null)).is_ok());
        assert!(schema.validate(&json!(42)).is_err());
    }

    #[test]
    fn test_array_type_validation() {
        let schema = ArraySchema::new(StringSchemaImpl::default())
            .error_message("array.invalid_type", "Must be an array");

        assert!(schema.validate(&json!(["a", "b"])).is_ok());
        
        let err = schema.validate(&json!("not an array")).unwrap_err();
        assert_eq!(err.context.code, "array.invalid_type");
        assert!(err.to_string().contains("Must be an array"));
    }

    #[test]
    fn test_nested_array_validation() {
        let inner_schema = ArraySchema::new(NumberSchema::default().integer());
        let schema = ArraySchema::new(inner_schema);

        assert!(schema.validate(&json!([[1, 2], [3, 4]])).is_ok());
        assert!(schema.validate(&json!([[1, 2.5]])).is_err());
    }
}