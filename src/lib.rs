use std::sync::Arc;

pub mod error;
pub mod schemas;

pub use error::ValidationError;
pub use schemas::{
    Schema, SchemaType,
    UnionSchema, UnionStrategy,
    StringSchema, NumberSchema, BooleanSchema, ArraySchema, ObjectSchema,
};

pub mod prelude {
    pub use crate::{
        string, number, boolean, array, object,
        union, all_of, best_of,
        Schema, UnionSchema, UnionStrategy,
    };
    pub use crate::error::ValidationError;
}

/// Create a new string schema
pub fn string() -> StringSchema {
    StringSchema::default()
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
pub fn array(schema: impl Schema) -> ArraySchema {
    ArraySchema::new(schema)
}

/// Create a new object schema
pub fn object() -> ObjectSchema {
    ObjectSchema::default()
}

#[macro_export]
macro_rules! union {
    ($($schema:expr),* $(,)?) => {{
        let schemas = vec![
            $($schema.into_schema_type()),*
        ];
        UnionSchema::new(schemas)
    }};
}

#[macro_export]
macro_rules! all_of {
    ($($schema:expr),* $(,)?) => {{
        let schemas = vec![
            $($schema.into_schema_type()),*
        ];
        UnionSchema::new(schemas).strategy(UnionStrategy::All)
    }};
}

#[macro_export]
macro_rules! best_of {
    ($($schema:expr),* ; $error_score:expr) => {{
        let schemas = vec![
            $($schema.into_schema_type()),*
        ];
        UnionSchema::new(schemas).strategy(UnionStrategy::Best { error_score: Arc::new($error_score) })
    }};
}

#[macro_export]
macro_rules! object {
    ({ $($key:expr => $value:expr),* $(,)? }) => {{
        let mut schema = $crate::object();
        $(
            schema = schema.field($key, $value);
        )*
        schema
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_complex_schema() {
        let schema = object!({
            "id" => number().min(1.0),
            "name" => string()
                .min_length(2)
                .max_length(50)
                .error_message("string.too_short", "Name must be at least {min_length} characters")
                .error_message("string.too_long", "Name cannot exceed {max_length} characters"),
            "email" => string()
                .email()
                .error_message("string.email", "Invalid email address format"),
            "age" => number()
                .min(0.0)
                .max(150.0)
                .optional()
                .error_message("number.min", "Age must be positive")
                .error_message("number.max", "Invalid age value"),
            "tags" => array(string().min_length(1))
                .min_items(1)
                .error_message("array.min_items", "At least one tag is required"),
            "settings" => object()
                .field("theme", string())
                .optional_field("notifications", boolean())
                .strict()
        });

        let valid_data = json!({
            "id": 1,
            "name": "John Doe",
            "email": "john@example.com",
            "age": 30,
            "tags": ["user"],
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        });

        let invalid_data = json!({
            "id": 0,
            "name": "J",
            "email": "invalid-email",
            "age": -1,
            "tags": [],
            "settings": {
                "theme": "dark",
                "unknown": true
            }
        });

        assert!(schema.validate(&valid_data).is_ok());
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_nested_arrays() {
        let schema = array(array(number().integer()))
            .error_message("array.invalid_type", "Must be a matrix of integers");

        assert!(schema.validate(&json!([[1, 2], [3, 4]])).is_ok());
        assert!(schema.validate(&json!([[1, 2.5]])).is_err());
        assert!(schema.validate(&json!([1, 2])).is_err());
    }

    #[test]
    fn test_deep_object() {
        let address_schema = object()
            .field("street", string().min_length(1))
            .field("city", string().min_length(1))
            .field("country", string().min_length(1));

        let contact_schema = object()
            .field("email", string().email())
            .optional_field("phone", string())
            .field("address", address_schema);

        let schema = object()
            .field("name", string())
            .field("contact", contact_schema);

        let valid_data = json!({
            "name": "John",
            "contact": {
                "email": "john@example.com",
                "address": {
                    "street": "123 Main St",
                    "city": "New York",
                    "country": "USA"
                }
            }
        });

        let result = schema.validate(&valid_data);
        assert!(result.is_ok());
    }
}
