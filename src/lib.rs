pub mod error;
pub mod schemas;

pub use error::ValidationError;
pub use schemas::{
    Schema, SchemaType,
    UnionSchema, UnionStrategy,
    string::{StringSchema, StringSchemaImpl},
    NumberSchema, BooleanSchema, ArraySchema, ObjectSchema,
    transform::Transformable,
};

pub mod prelude {
    pub use crate::{
        string, number, boolean, array, object,
        union, union_best,
        Schema, StringSchema,
    };
}

/// Create a new string schema
pub fn string() -> StringSchemaImpl {
    StringSchemaImpl::default()
}

/// Create a new number schema
pub fn number() -> NumberSchema {
    NumberSchema::default()
}

/// Create a new boolean schema
pub fn boolean() -> BooleanSchema {
    BooleanSchema::default()
}

/// Create a new array schema
pub fn array<S: Schema>(schema: S) -> ArraySchema {
    ArraySchema::new(schema)
}

/// Create a new object schema
pub fn object() -> ObjectSchema {
    ObjectSchema::default()
}

/// Create a new union schema
pub fn union<S: Schema>(schemas: Vec<S>) -> UnionSchema {
    UnionSchema::new(schemas.into_iter().map(|s| s.into_schema_type()).collect())
}

#[macro_export]
macro_rules! union {
    ($($schema:expr),+ $(,)?) => {{
        let schemas = vec![$($schema),+];
        UnionSchema::new(schemas)
    }};
}

#[macro_export]
macro_rules! union_best {
    ($error_score:expr, $($schema:expr),+ $(,)?) => {{
        let schemas = vec![$($schema),+];
        UnionSchema::new(schemas).strategy(UnionStrategy::Best { error_score: std::sync::Arc::new($error_score) })
    }};
}

#[macro_export]
macro_rules! object {
    () => {
        $crate::object()
    };

    ({ $($key:tt => $value:expr),* $(,)? }) => {{
        let mut schema = $crate::object();
        $(
            let value = $value;
            schema = schema.field($key, value);
        )*
        schema
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_object_macro_basic() {
        let schema = object!( {
            "name" => string().min_length(2),
            "age" => number().min(0.0),
            "email" => string().email().optional()
        });

        // Test valid data with all fields
        let valid_data = json!({
            "name": "John",
            "age": 25,
            "email": "john@example.com"
        });
        assert!(schema.validate(&valid_data).is_ok());

        // Test valid data without optional field
        let valid_data_no_email = json!({
            "name": "John",
            "age": 25
        });
        assert!(schema.validate(&valid_data_no_email).is_ok());

        // Test invalid data (missing required field)
        let invalid_data = json!({
            "name": "John"
        });
        assert!(schema.validate(&invalid_data).is_err());

        // Test invalid data (wrong type)
        let invalid_data = json!({
            "name": 123,
            "age": "25",
            "email": "not-an-email"
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_with_string_keys() {
        let schema = object! ({
            "name" => string(),
            "age" => number()
        });

        let valid_data = json!({
            "name": "John",
            "age": 25
        });
        assert!(schema.validate(&valid_data).is_ok());

        let invalid_data = json!({
            "name": "John",
            "age": "25"
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_empty() {
        let schema = object!();

        let valid_data = json!({});
        assert!(schema.validate(&valid_data).is_ok());

        let invalid_data = json!({
            "name": "John"
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_with_transformations() {
        let schema = object! ({
            "username" => string().trim().to_lowercase(),
            "age" => number().min(0.0).to_integer()
        });

        let valid_data = json!({
            "username": "  JohnDoe  ",
            "age": "25"
        });
        let result = schema.validate(&valid_data).unwrap();
        assert_eq!(result["username"], "johndoe");
        assert_eq!(result["age"], 25);
    }

    #[test]
    fn test_object_macro_with_custom_validation() {
        let schema = object! ({
            "name" => string().trim()
                .custom(|s| if s.len() >= 3 {
                    Ok(())
                } else {
                    Err("Name must be at least 3 characters long".to_string())
                })
        });

        let valid_data = json!({
            "name": "  John  "
        });
        assert!(schema.validate(&valid_data).is_ok());

        let invalid_data = json!({
            "name": "  J  "
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_with_nested_objects() {
        let schema = object! ({
            "user" => object! ({
                "username" => string()
                    .min_length(3)
                    .max_length(20)
                    .pattern(r"^[a-zA-Z0-9_]+$")
            })
        });

        let valid_data = json!({
            "user": {
                "username": "johndoe_123"
            }
        });
        assert!(schema.validate(&valid_data).is_ok());

        let invalid_data = json!({
            "user": {
                "username": "j@"
            }
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_with_arrays() {
        let schema = object! ({
            "id" => number().integer().min(1.0),
            "tags" => array(string().min_length(1))
                .min_items(1)
                .max_items(5)
        });

        let valid_data = json!({
            "id": 1,
            "tags": ["tag1", "tag2"]
        });
        assert!(schema.validate(&valid_data).is_ok());

        let invalid_data = json!({
            "id": 0,
            "tags": []
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_with_error_messages() {
        let schema = object!({
            "name" => string().trim().min_length(1)
                .error_message("string.too_short", "Name cannot be empty"),
            "age" => number().min(18.0)
                .error_message("number.too_small", "Must be at least 18 years old")
        });

        let err = schema.validate(&json!({
            "name": "",
            "age": 16
        })).unwrap_err();

        assert!(err.to_string().contains("Name cannot be empty"));
    }

    #[test]
    fn test_object_macro_with_email_validation() {
        let schema = object!( {
            "email" => string().trim().to_lowercase().email(),
            "name" => string().min_length(2)
        });

        let valid_data = json!({
            "email": "  USER@EXAMPLE.COM  ",
            "name": "John"
        });
        let result = schema.validate(&valid_data).unwrap();
        assert_eq!(result["email"], "user@example.com");

        let invalid_data = json!({
            "email": "not-an-email",
            "name": "John"
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_with_optional_fields() {
        let schema = object! ({
            "name" => string().min_length(2),
            "email" => string().email().optional()
        });

        let valid_data = json!({
            "name": "John"
        });
        assert!(schema.validate(&valid_data).is_ok());

        let invalid_data = json!({
            "email": "user@example.com"
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_with_boolean_fields() {
        let schema = object! ({
            "name" => string(),
            "is_active" => boolean()
        });

        let valid_data = json!({
            "name": "John",
            "is_active": true
        });
        assert!(schema.validate(&valid_data).is_ok());

        let invalid_data = json!({
            "name": "John",
            "is_active": "true"
        });
        assert!(schema.validate(&invalid_data).is_err());
    }
}