use crate::{Schema, SchemaType, ValidationError, StringSchema, NumberSchema, ArraySchema, ObjectSchema, BooleanSchema, get_type_name};
use serde_json::Value;

impl Schema for StringSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Null if self.optional => Ok(value.clone()),
            Value::String(s) => {
                if let Some(min_len) = self.min_length {
                    if s.len() < min_len {
                        return Err(ValidationError::new("string.too_short", "")
                            .with_message(self.get_error_message("string.too_short")
                                .unwrap_or_else(|| format!("String length {} is less than minimum {}", s.len(), min_len)))
                            .with_min_length(min_len));
                    }
                }

                if let Some(max_len) = self.max_length {
                    if s.len() > max_len {
                        return Err(ValidationError::new("string.too_long", "")
                            .with_message(self.get_error_message("string.too_long")
                                .unwrap_or_else(|| format!("String length {} is greater than maximum {}", s.len(), max_len)))
                            .with_max_length(max_len));
                    }
                }

                if let Some(pattern) = &self.pattern {
                    if !pattern.is_match(s) {
                        return Err(ValidationError::new("string.pattern", "")
                            .with_message(self.get_error_message("string.pattern")
                                .unwrap_or_else(|| "String does not match pattern".to_string()))
                            .with_pattern(pattern.as_str().to_string()));
                    }
                }

                if self.email {
                    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
                    if !email_regex.is_match(s) {
                        return Err(ValidationError::new("string.email", "")
                            .with_message(self.get_error_message("string.email")
                                .unwrap_or_else(|| "Invalid email format".to_string())));
                    }
                }

                for validator in &self.custom_validators {
                    if let Err(msg) = validator(s) {
                        return Err(ValidationError::new("string.custom", "")
                            .with_message(msg));
                    }
                }

                Ok(value.clone())
            }
            Value::Null => Err(ValidationError::new("string.required", "")
                .with_message(self.get_error_message("string.required")
                    .unwrap_or_else(|| "This field is required".to_string()))),
            _ => Err(ValidationError::new("string.invalid_type", "")
                .with_message(self.get_error_message("string.invalid_type")
                    .unwrap_or_else(|| format!("Expected string, got {}", get_type_name(value))))
                .with_type_info("string", get_type_name(value).to_string())),
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::String(self)
    }
}

impl Schema for NumberSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        let num = match value {
            Value::Number(n) => n.as_f64().unwrap(),
            Value::String(s) if self.coerce => {
                s.parse::<f64>().map_err(|_| ValidationError::new("number.invalid_type", "")
                    .with_message("Could not parse string as number".to_string())
                    .with_type_info("number", "string (not a valid number)".to_string()))?
            }
            Value::Null if self.optional => return Ok(value.clone()),
            Value::Null => return Err(ValidationError::new("number.required", "")
                .with_message("This field is required".to_string())),
            _ => return Err(ValidationError::new("number.invalid_type", "")
                .with_message(format!("Expected number, got {}", get_type_name(value)))
                .with_type_info("number", get_type_name(value).to_string())),
        };

        if self.integer && num.fract() != 0.0 {
            return Err(ValidationError::new("number.integer", "")
                .with_message("Expected integer value".to_string()));
        }

        if let Some(min) = self.min {
            if num < min {
                return Err(ValidationError::new("number.min", "")
                    .with_message(format!("Value {} is less than minimum {}", num, min))
                    .with_min(min as i64));
            }
        }

        if let Some(max) = self.max {
            if num > max {
                return Err(ValidationError::new("number.max", "")
                    .with_message(format!("Value {} is greater than maximum {}", num, max))
                    .with_max(max as i64));
            }
        }

        Ok(Value::Number(serde_json::Number::from_f64(num).unwrap()))
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Number(self)
    }
}

impl Schema for BooleanSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Bool(_) => Ok(value.clone()),
            _ => Err(ValidationError::new("boolean.invalid_type", "")
                .with_message(format!("Expected boolean, got {}", get_type_name(value)))
                .with_type_info("boolean", get_type_name(value).to_string())),
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Boolean(self)
    }
}

impl Schema for ArraySchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Array(arr) => {
                if let Some(min_items) = self.min_items {
                    if arr.len() < min_items {
                        return Err(ValidationError::new("array.min_items", "")
                            .with_message(format!("Array length {} is less than minimum {}", arr.len(), min_items)));
                    }
                }

                if let Some(max_items) = self.max_items {
                    if arr.len() > max_items {
                        return Err(ValidationError::new("array.max_items", "")
                            .with_message(format!("Array length {} is greater than maximum {}", arr.len(), max_items)));
                    }
                }

                for (i, item) in arr.iter().enumerate() {
                    if let Err(e) = validate_schema_type(self.item_schema.as_ref(), item) {
                        return Err(e.with_path_prefix(&i.to_string()));
                    }
                }

                Ok(value.clone())
            }
            Value::Null if self.optional => Ok(value.clone()),
            Value::Null => Err(ValidationError::new("array.required", "")
                .with_message("This field is required".to_string())),
            _ => Err(ValidationError::new("array.invalid_type", "")
                .with_message(format!("Expected array, got {}", get_type_name(value)))
                .with_type_info("array", get_type_name(value).to_string())),
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Array(Box::new(self))
    }
}

impl Schema for ObjectSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match value {
            Value::Object(obj) => {
                for field in &self.required {
                    if !obj.contains_key(field.as_str()) {
                        return Err(ValidationError::new("object.required", field)
                            .with_message("Required field is missing".to_string()));
                    }
                }

                for (field, schema) in &self.fields {
                    if let Some(value) = obj.get(field.as_str()) {
                        if let Err(e) = validate_schema_type(schema.as_ref(), value) {
                            return Err(e.with_path_prefix(field));
                        }
                    }
                }

                Ok(value.clone())
            }
            Value::Null if self.optional => Ok(value.clone()),
            Value::Null => Err(ValidationError::new("object.required", "")
                .with_message("This field is required".to_string())),
            _ => Err(ValidationError::new("object.invalid_type", "")
                .with_message(format!("Expected object, got {}", get_type_name(value)))
                .with_type_info("object", get_type_name(value).to_string())),
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Object(Box::new(self))
    }
}

impl Schema for SchemaType {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        validate_schema_type(self, value)
    }

    fn into_schema_type(self) -> SchemaType {
        self
    }
}

fn validate_schema_type(schema: &SchemaType, value: &Value) -> Result<Value, ValidationError> {
    match schema {
        SchemaType::String(s) => s.validate(value),
        SchemaType::Number(n) => n.validate(value),
        SchemaType::Boolean(b) => b.validate(value),
        SchemaType::Array(a) => a.as_ref().validate(value),
        SchemaType::Object(o) => o.as_ref().validate(value),
    }
}