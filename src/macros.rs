#[macro_export]
macro_rules! object {
    // Empty object
    () => {
        $crate::schemas::ObjectSchema::default()
    };

    // Object with fields
    (
        $(
            $field:ident $(: $schema:expr)? $(?)? $(,)?
        )*
    ) => {{
        let mut schema = $crate::schemas::ObjectSchema::default();
        $(
            schema = if false $(|| true)?  { // Optional field check
                schema.optional_field(
                    stringify!($field),
                    $($schema)?.into_schema_type()
                )
            } else {
                schema.field(
                    stringify!($field),
                    $($schema)?.into_schema_type()
                )
            };
        )*
        schema
    }};
}

#[macro_export]
macro_rules! string {
    () => {
        $crate::schemas::StringSchemaImpl::default()
    };
}

#[macro_export]
macro_rules! number {
    () => {
        $crate::schemas::NumberSchema::default()
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{Schema, StringSchema, NumberSchema};
    use serde_json::json;

    #[test]
    fn test_empty_object() {
        let schema = object!();
        assert!(schema.validate(&json!({})).is_ok());
    }

    #[test]
    fn test_simple_object() {
        let schema = object! {
            name: string!(),
            age: number!()
        };

        assert!(schema.validate(&json!({
            "name": "John",
            "age": 30
        })).is_ok());

        assert!(schema.validate(&json!({
            "name": "John"
        })).is_err());
    }

    #[test]
    fn test_optional_fields() {
        let schema = object! {
            name: string!(),
            age: number!()?,
            email: string!()?
        };

        assert!(schema.validate(&json!({
            "name": "John"
        })).is_ok());

        assert!(schema.validate(&json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com"
        })).is_ok());

        assert!(schema.validate(&json!({
            "age": 30
        })).is_err());
    }

    #[test]
    fn test_nested_objects() {
        let schema = object! {
            name: string!(),
            address: object! {
                street: string!(),
                city: string!()
            }
        };

        assert!(schema.validate(&json!({
            "name": "John",
            "address": {
                "street": "123 Main St",
                "city": "New York"
            }
        })).is_ok());

        assert!(schema.validate(&json!({
            "name": "John",
            "address": {
                "street": "123 Main St"
            }
        })).is_err());
    }

    #[test]
    fn test_complex_schema() {
        let schema = object! {
            name: string!(),
            age: number!(),
            email: string!()?,
            address: object! {
                street: string!(),
                city: string!(),
                country: string!()?,
                postal_code: string!()?
            }
        };

        assert!(schema.validate(&json!({
            "name": "John",
            "age": 30,
            "address": {
                "street": "123 Main St",
                "city": "New York"
            }
        })).is_ok());

        assert!(schema.validate(&json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com",
            "address": {
                "street": "123 Main St",
                "city": "New York",
                "country": "USA",
                "postal_code": "10001"
            }
        })).is_ok());
    }
}