use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

use crate::error::ValidationError;

pub mod string;
pub mod number;
pub mod array;
pub mod object;
pub mod boolean;
pub mod transform;

pub use string::StringSchema;
pub use number::NumberSchema;
pub use array::ArraySchema;
pub use object::ObjectSchema;
pub use boolean::BooleanSchema;
pub use transform::{Transform, Transformable, WithTransform};

#[derive(Clone)]
pub enum SchemaType {
    String(string::StringSchemaImpl),
    Number(NumberSchema),
    Boolean(BooleanSchema),
    Array(Box<ArraySchema>),
    Object(Box<ObjectSchema>),
    Union(Box<UnionSchema>),
}

pub trait Schema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError>;
    fn into_schema_type(self) -> SchemaType where Self: Sized;
}

pub trait ValueTransform {
    fn transform(&self, value: Value) -> Value;
}

pub trait Refinement {
    fn refine(&self, value: &Value) -> Result<(), String>;
}

#[derive(Clone)]
pub enum UnionStrategy {
    First,  // Use first schema that validates
    All,    // All schemas must validate (intersection)
    Best {  // Use schema with least errors
        error_score: Arc<dyn Fn(&ValidationError) -> u32 + Send + Sync>,
    },
}

#[derive(Clone)]
pub struct UnionSchema {
    schemas: Vec<SchemaType>,
    strategy: UnionStrategy,
    error_messages: HashMap<String, String>,
}

impl UnionSchema {
    pub fn new(schemas: Vec<SchemaType>) -> Self {
        Self {
            schemas,
            strategy: UnionStrategy::First,
            error_messages: HashMap::new(),
        }
    }

    pub fn strategy(mut self, strategy: UnionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn error_message(mut self, code: impl Into<String>, message: impl Into<String>) -> Self {
        self.error_messages.insert(code.into(), message.into());
        self
    }
}

impl HasErrorMessages for UnionSchema {
    fn error_messages(&self) -> &HashMap<String, String> {
        &self.error_messages
    }
}

impl Schema for UnionSchema {
    fn validate(&self, value: &Value) -> Result<Value, ValidationError> {
        match &self.strategy {
            UnionStrategy::First => {
                let mut last_error = None;
                for schema in &self.schemas {
                    match validate_schema_type(schema, value) {
                        Ok(v) => return Ok(v),
                        Err(e) => last_error = Some(e),
                    }
                }
                Err(last_error.unwrap_or_else(|| ValidationError::new("union.no_match")
                    .message("Value did not match any schema")))
            }
            UnionStrategy::All => {
                for schema in &self.schemas {
                    validate_schema_type(schema, value)?;
                }
                Ok(value.clone())
            }
            UnionStrategy::Best { error_score } => {
                let mut best_result = None;
                let mut best_score = u32::MAX;

                for schema in &self.schemas {
                    match validate_schema_type(schema, value) {
                        Ok(v) => return Ok(v),
                        Err(e) => {
                            let score = error_score(&e);
                            if score < best_score {
                                best_score = score;
                                best_result = Some((value.clone(), e));
                            }
                        }
                    }
                }

                match best_result {
                    Some((_, e)) => Err(e),
                    None => Err(ValidationError::new("union.no_match")
                        .message("Value did not match any schema")),
                }
            }
        }
    }

    fn into_schema_type(self) -> SchemaType {
        SchemaType::Union(Box::new(self))
    }
}

pub trait ErrorMessage {
    fn get_error_message(&self, code: &str) -> Option<String>;
}

impl<T> ErrorMessage for T
where
    T: HasErrorMessages,
{
    fn get_error_message(&self, code: &str) -> Option<String> {
        self.error_messages().get(code).cloned()
    }
}

pub trait HasErrorMessages {
    fn error_messages(&self) -> &HashMap<String, String>;
}

pub fn validate_schema_type(schema: &SchemaType, value: &Value) -> Result<Value, ValidationError> {
    match schema {
        SchemaType::String(s) => s.validate(value),
        SchemaType::Number(n) => n.validate(value),
        SchemaType::Boolean(b) => b.validate(value),
        SchemaType::Array(a) => a.as_ref().validate(value),
        SchemaType::Object(o) => o.as_ref().validate(value),
        SchemaType::Union(u) => u.as_ref().validate(value),
    }
}

pub fn get_type_name(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::{string, number};

    #[test]
    fn test_type_name() {
        assert_eq!(get_type_name(&json!(null)), "null");
        assert_eq!(get_type_name(&json!(true)), "boolean");
        assert_eq!(get_type_name(&json!(42)), "number");
        assert_eq!(get_type_name(&json!("hello")), "string");
        assert_eq!(get_type_name(&json!([])), "array");
        assert_eq!(get_type_name(&json!({})), "object");
    }

    #[test]
    fn test_union_first_match() {
        let schema = UnionSchema::new(vec![
            string().into_schema_type(),
            number().into_schema_type(),
        ]);

        assert!(schema.validate(&json!("hello")).is_ok());
        assert!(schema.validate(&json!(42)).is_ok());
        assert!(schema.validate(&json!(true)).is_err());
    }

    #[test]
    fn test_union_all_match() {
        let schema = UnionSchema::new(vec![
            string().min_length(3).into_schema_type(),
            string().max_length(10).into_schema_type(),
        ]).strategy(UnionStrategy::All);

        assert!(schema.validate(&json!("hello")).is_ok());  // 5 chars
        assert!(schema.validate(&json!("hi")).is_err());    // too short
        assert!(schema.validate(&json!("hello world")).is_err());  // too long
    }

    #[test]
    fn test_union_best_match() {
        let schema = UnionSchema::new(vec![
            string().min_length(5).into_schema_type(),
            string().max_length(3).into_schema_type(),
        ]).strategy(UnionStrategy::Best {
            error_score: Arc::new(|e| match e.context.code.as_str() {
                "string.too_short" => 1,
                "string.too_long" => 2,
                _ => 3,
            }),
        });

        // For "hi", min_length(5) gives "too_short" (score 1)
        // and max_length(3) gives OK, so max_length(3) wins
        assert!(schema.validate(&json!("hi")).is_ok());

        // For "hello", min_length(5) gives OK
        // and max_length(3) gives "too_long" (score 2)
        // so min_length(5) wins
        assert!(schema.validate(&json!("hello")).is_ok());

        // For "1234", both fail but "too_short" has lower score
        let err = schema.validate(&json!("1234")).unwrap_err();
        assert_eq!(err.context.code, "string.too_short");
    }
}