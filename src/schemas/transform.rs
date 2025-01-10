use std::sync::Arc;
use serde_json::Value;

/// A transformation that can be applied to a value during validation
#[derive(Clone)]
pub enum Transform {
    /// Apply a custom transformation function
    Custom(Arc<dyn Fn(Value) -> Value + Send + Sync>),
    /// Convert string to lowercase
    ToLowerCase,
    /// Convert string to uppercase
    ToUpperCase,
    /// Trim whitespace from string
    Trim,
    /// Parse string as number
    ParseNumber,
    /// Convert number to integer
    ToInteger,
    /// Convert to string
    ToString,
}

impl Transform {
    pub fn apply(&self, value: Value) -> Value {
        match self {
            Transform::Custom(f) => f(value),
            Transform::ToLowerCase => {
                if let Value::String(s) = value {
                    Value::String(s.to_lowercase())
                } else {
                    value
                }
            }
            Transform::ToUpperCase => {
                if let Value::String(s) = value {
                    Value::String(s.to_uppercase())
                } else {
                    value
                }
            }
            Transform::Trim => {
                if let Value::String(s) = value {
                    Value::String(s.trim().to_string())
                } else {
                    value
                }
            }
            Transform::ParseNumber => {
                if let Value::String(s) = &value {
                    if let Ok(n) = s.parse::<f64>() {
                        Value::Number(serde_json::Number::from_f64(n).unwrap())
                    } else {
                        value
                    }
                } else {
                    value
                }
            }
            Transform::ToInteger => {
                if let Value::Number(n) = &value {
                    if let Some(i) = n.as_i64() {
                        Value::Number(i.into())
                    } else {
                        Value::Number(serde_json::Number::from_f64(n.as_f64().unwrap().floor()).unwrap())
                    }
                } else {
                    value
                }
            }
            Transform::ToString => {
                match value {
                    Value::String(s) => Value::String(s),
                    Value::Number(n) => Value::String(n.to_string()),
                    Value::Bool(b) => Value::String(b.to_string()),
                    Value::Null => Value::String("null".to_string()),
                    _ => value,
                }
            }
        }
    }
}

/// A trait for schemas that support transformations
pub trait Transformable: Sized {
    /// Apply a custom transformation function
    fn transform<F>(self, f: F) -> WithTransform<Self>
    where
        F: Fn(Value) -> Value + Send + Sync + 'static,
    {
        self.with_transform(Transform::Custom(Arc::new(f)))
    }

    /// Convert string to lowercase
    fn to_lowercase(self) -> WithTransform<Self> {
        self.with_transform(Transform::ToLowerCase)
    }

    /// Convert string to uppercase
    fn to_uppercase(self) -> WithTransform<Self> {
        self.with_transform(Transform::ToUpperCase)
    }

    /// Trim whitespace from string
    fn trim(self) -> WithTransform<Self> {
        self.with_transform(Transform::Trim)
    }

    /// Parse string as number
    fn parse_number(self) -> WithTransform<Self> {
        self.with_transform(Transform::ParseNumber)
    }

    /// Convert number to integer
    fn to_integer(self) -> WithTransform<Self> {
        self.with_transform(Transform::ToInteger)
    }

    /// Convert to string
    fn to_string(self) -> WithTransform<Self> {
        self.with_transform(Transform::ToString)
    }

    /// Add a transformation
    fn with_transform(self, transform: Transform) -> WithTransform<Self>;
}

/// A wrapper that adds transformation to a schema
#[derive(Clone)]
pub struct WithTransform<S> {
    schema: S,
    transforms: Vec<Transform>,
}

impl<S> WithTransform<S> {
    pub fn new(schema: S) -> Self {
        Self {
            schema,
            transforms: Vec::new(),
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transforms.push(transform);
        self
    }

    pub fn into_inner(self) -> S {
        self.schema
    }
}

impl<S: super::StringSchema> WithTransform<S> {
    pub fn min_length(self, length: usize) -> Self {
        WithTransform::new(self.into_inner().min_length(length))
    }

    pub fn max_length(self, length: usize) -> Self {
        WithTransform::new(self.into_inner().max_length(length))
    }

    pub fn pattern(self, pattern: &str) -> Self {
        WithTransform::new(self.into_inner().pattern(pattern))
    }

    pub fn email(self) -> Self {
        WithTransform::new(self.into_inner().email())
    }

    pub fn optional(self) -> Self {
        WithTransform::new(self.into_inner().optional())
    }

    pub fn error_message(self, code: impl Into<String>, message: impl Into<String>) -> Self {
        WithTransform::new(self.into_inner().error_message(code, message))
    }

    pub fn custom<F>(self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        WithTransform::new(self.into_inner().custom(validator))
    }
}

impl<S: super::Schema> Transformable for WithTransform<S> {
    fn with_transform(self, transform: Transform) -> WithTransform<Self> {
        WithTransform::new(self).with_transform(transform)
    }
}

impl<S: super::Schema> super::Schema for WithTransform<S> {
    fn validate(&self, value: &Value) -> Result<Value, crate::error::ValidationError> {
        let mut value = value.clone();
        for transform in &self.transforms {
            value = transform.apply(value);
        }
        let validated = self.schema.validate(&value)?;
        Ok(validated)
    }

    fn into_schema_type(self) -> super::SchemaType {
        self.schema.into_schema_type()
    }
}

impl<S: super::string::StringSchema> super::string::StringSchema for WithTransform<S> {
    fn min_length(self, length: usize) -> Self {
        WithTransform::new(self.into_inner().min_length(length))
    }

    fn max_length(self, length: usize) -> Self {
        WithTransform::new(self.into_inner().max_length(length))
    }

    fn pattern(self, pattern: &str) -> Self {
        WithTransform::new(self.into_inner().pattern(pattern))
    }

    fn email(self) -> Self {
        WithTransform::new(self.into_inner().email())
    }

    fn optional(self) -> Self {
        WithTransform::new(self.into_inner().optional())
    }

    fn error_message(self, code: impl Into<String>, message: impl Into<String>) -> Self {
        WithTransform::new(self.into_inner().error_message(code, message))
    }

    fn custom<F>(self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + Send + Sync + 'static,
    {
        WithTransform::new(self.into_inner().custom(validator))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{string, number, schemas::Schema};
    use serde_json::json;

    #[test]
    fn test_string_transforms() {
        let schema = string()
            .to_lowercase()
            .trim();

        assert_eq!(
            schema.validate(&json!("  HELLO  ")).unwrap(),
            json!("hello")
        );
    }

    #[test]
    fn test_number_transforms() {
        let schema = number()
            .transform(|v| {
                if let Value::Number(n) = &v {
                    if let Some(f) = n.as_f64() {
                        Value::Number(serde_json::Number::from_f64(f.floor()).unwrap())
                    } else {
                        v
                    }
                } else {
                    v
                }
            });

        assert_eq!(
            schema.validate(&json!(42.9)).unwrap(),
            json!(42.0)
        );
    }

    #[test]
    fn test_custom_transform() {
        let schema = string()
            .transform(|v| {
                if let Value::String(s) = v {
                    Value::String(s.chars().rev().collect())
                } else {
                    v
                }
            });

        assert_eq!(
            schema.validate(&json!("hello")).unwrap(),
            json!("olleh")
        );
    }

    #[test]
    fn test_multiple_transforms() {
        let schema = string()
            .transform(|v| {
                if let Value::String(s) = v {
                    Value::String(format!("#{}", s.trim().to_uppercase()))
                } else {
                    v
                }
            });

        assert_eq!(
            schema.validate(&json!("  hello  ")).unwrap(),
            json!("#HELLO")
        );
    }

    #[test]
    fn test_type_conversion() {
        let schema = number()
            .coerce()
            .transform(|v| {
                if let Value::String(s) = &v {
                    if let Ok(n) = s.parse::<f64>() {
                        Value::Number(serde_json::Number::from_f64(n.floor()).unwrap())
                    } else {
                        v
                    }
                } else if let Value::Number(n) = &v {
                    if let Some(f) = n.as_f64() {
                        Value::Number(serde_json::Number::from_f64(f.floor()).unwrap())
                    } else {
                        v
                    }
                } else {
                    v
                }
            });

        assert_eq!(
            schema.validate(&json!("42.9")).unwrap(),
            json!(42.0)
        );
    }
}