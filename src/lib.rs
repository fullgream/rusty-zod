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
    // Empty object
    () => {
        $crate::object()
    };

    // Object with string keys: { "name" => schema }
    ({ $($key:expr => $value:expr),* $(,)? }) => {{
        let mut schema = $crate::object();
        $(
            schema = schema.field($key, $value);
        )*
        schema
    }};

    // Object with field names as identifiers: { name: schema }
    ({ $($field:ident $(: $schema:expr)? $(?)? $(,)?)* }) => {{
        let mut schema = $crate::object();
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use serde::{Deserialize, Serialize};

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
        postal_code: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Person {
        name: String,
        age: u32,
        email: Option<String>,
        address: Address,
        tags: Vec<String>,
        settings: Settings,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Settings {
        theme: String,
        notifications: Option<bool>,
    }

    #[test]
    fn test_object_macro_basic() {
        let schema = object! {
            name: string().min_length(2),
            age: number().min(0.0),
            email?: string().email()
        };

        // Test valid data with all fields
        let valid_data = json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com"
        });

        let result = schema.validate(&valid_data).unwrap();
        let user: User = serde_json::from_value(result).unwrap();
        assert_eq!(user.name, "John");
        assert_eq!(user.age, 30);
        assert_eq!(user.email, Some("john@example.com".to_string()));

        // Test valid data without optional field
        let valid_data = json!({
            "name": "John",
            "age": 30
        });

        let result = schema.validate(&valid_data).unwrap();
        let user: User = serde_json::from_value(result).unwrap();
        assert_eq!(user.name, "John");
        assert_eq!(user.age, 30);
        assert_eq!(user.email, None);

        // Test invalid data: name too short
        let invalid_data = json!({
            "name": "J",
            "age": 30
        });
        assert!(schema.validate(&invalid_data).is_err());

        // Test invalid data: negative age
        let invalid_data = json!({
            "name": "John",
            "age": -1
        });
        assert!(schema.validate(&invalid_data).is_err());

        // Test invalid data: invalid email
        let invalid_data = json!({
            "name": "John",
            "age": 30,
            "email": "not-an-email"
        });
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_object_macro_nested() {
        let schema = object! {
            name: string(),
            age: number(),
            email?: string().email(),
            address: object! {
                street: string(),
                city: string(),
                postal_code?: string()
            },
            tags: array(string()),
            settings: object! {
                theme: string(),
                notifications?: boolean()
            }
        };

        let valid_data = json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com",
            "address": {
                "street": "123 Main St",
                "city": "New York",
                "postal_code": "10001"
            },
            "tags": ["user", "admin"],
            "settings": {
                "theme": "dark",
                "notifications": true
            }
        });

        let result = schema.validate(&valid_data).unwrap();
        let person: Person = serde_json::from_value(result).unwrap();
        assert_eq!(person.name, "John");
        assert_eq!(person.age, 30);
        assert_eq!(person.email, Some("john@example.com".to_string()));
        assert_eq!(person.address.street, "123 Main St");
        assert_eq!(person.address.city, "New York");
        assert_eq!(person.address.postal_code, Some("10001".to_string()));
        assert_eq!(person.tags, vec!["user", "admin"]);
        assert_eq!(person.settings.theme, "dark");
        assert_eq!(person.settings.notifications, Some(true));

        // Test without optional fields
        let valid_data = json!({
            "name": "John",
            "age": 30,
            "address": {
                "street": "123 Main St",
                "city": "New York"
            },
            "tags": ["user"],
            "settings": {
                "theme": "dark"
            }
        });

        let result = schema.validate(&valid_data).unwrap();
        let person: Person = serde_json::from_value(result).unwrap();
        assert_eq!(person.name, "John");
        assert_eq!(person.age, 30);
        assert_eq!(person.email, None);
        assert_eq!(person.address.postal_code, None);
        assert_eq!(person.settings.notifications, None);
    }

    #[test]
    fn test_object_macro_validation_errors() {
        let schema = object! {
            name: string().min_length(2),
            age: number().min(0.0).max(150.0),
            email?: string().email(),
            address: object! {
                street: string().min_length(5),
                city: string(),
                postal_code?: string().pattern(r"^\d{5}$")
            }
        };

        // Test missing required field
        let invalid_data = json!({
            "name": "John"
            // missing age field
        });
        let err = schema.validate(&invalid_data).unwrap_err();
        assert!(err.to_string().contains("required"));

        // Test invalid nested field
        let invalid_data = json!({
            "name": "John",
            "age": 30,
            "address": {
                "street": "123", // too short
                "city": "New York"
            }
        });
        let err = schema.validate(&invalid_data).unwrap_err();
        assert!(err.to_string().contains("too short"));

        // Test invalid pattern
        let invalid_data = json!({
            "name": "John",
            "age": 30,
            "address": {
                "street": "123 Main St",
                "city": "New York",
                "postal_code": "1234" // invalid postal code format
            }
        });
        let err = schema.validate(&invalid_data).unwrap_err();
        assert!(err.to_string().contains("pattern"));
    }

    #[test]
    fn test_object_macro_type_conversion() {
        let schema = object! {
            name: string(),
            age: number(),
            active: boolean()
        };

        // Test type mismatch errors
        let invalid_data = json!({
            "name": 123, // should be string
            "age": "30", // should be number
            "active": 1  // should be boolean
        });

        let err = schema.validate(&invalid_data).unwrap_err();
        assert!(err.to_string().contains("Expected string"));

        // Test with correct types
        let valid_data = json!({
            "name": "John",
            "age": 30,
            "active": true
        });

        assert!(schema.validate(&valid_data).is_ok());
    }
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

    #[test]
    fn test_string_transformations() {
        let schema = object! {
            username: string().trim().to_lower_case(),
            password: string().trim(),
            email: string().trim().to_lower_case().email(),
            tags: array(string().trim().to_lower_case())
        };

        let input = json!({
            "username": "  JohnDoe  ",
            "password": "  secret123  ",
            "email": "  JOHN@EXAMPLE.COM  ",
            "tags": ["  TAG1  ", "  Tag2  ", "  TAG3  "]
        });

        let result = schema.validate(&input).unwrap();
        let output = result.as_object().unwrap();

        assert_eq!(output["username"], "johndoe");
        assert_eq!(output["password"], "secret123");
        assert_eq!(output["email"], "john@example.com");
        assert_eq!(
            output["tags"].as_array().unwrap(),
            vec!["tag1", "tag2", "tag3"]
        );
    }

    #[test]
    fn test_number_transformations() {
        let schema = object! {
            age: number().min(0.0).to_integer(),
            score: number().min(0.0).max(100.0),
            prices: array(number().min(0.0).to_integer())
        };

        let input = json!({
            "age": 30.6,
            "score": 85.5,
            "prices": [10.8, 20.2, 30.7]
        });

        let result = schema.validate(&input).unwrap();
        let output = result.as_object().unwrap();

        assert_eq!(output["age"], 30);
        assert_eq!(output["score"], 85.5);
        assert_eq!(
            output["prices"].as_array().unwrap(),
            vec![10, 20, 30]
        );
    }

    #[test]
    fn test_nested_transformations() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct UserProfile {
            name: String,
            email: String,
            settings: Settings,
            tags: Vec<String>,
        }

        let schema = object! {
            name: string().trim(),
            email: string().trim().to_lower_case().email(),
            settings: object! {
                theme: string().trim().to_lower_case(),
                notifications?: boolean()
            },
            tags: array(string().trim().to_lower_case())
        };

        let input = json!({
            "name": "  John Doe  ",
            "email": "  JOHN@EXAMPLE.COM  ",
            "settings": {
                "theme": "  DARK  ",
                "notifications": true
            },
            "tags": ["  USER  ", "  ADMIN  "]
        });

        let result = schema.validate(&input).unwrap();
        let profile: UserProfile = serde_json::from_value(result).unwrap();

        assert_eq!(profile.name, "John Doe");
        assert_eq!(profile.email, "john@example.com");
        assert_eq!(profile.settings.theme, "dark");
        assert_eq!(profile.settings.notifications, Some(true));
        assert_eq!(profile.tags, vec!["user", "admin"]);
    }

    #[test]
    fn test_custom_validators_with_transformations() {
        let schema = object! {
            username: string()
                .trim()
                .to_lower_case()
                .min_length(3)
                .custom(|s| {
                    if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                        Ok(())
                    } else {
                        Err("Username can only contain letters, numbers, and underscores".to_string())
                    }
                }),
            password: string()
                .trim()
                .min_length(8)
                .custom(|s| {
                    let has_upper = s.chars().any(|c| c.is_ascii_uppercase());
                    let has_lower = s.chars().any(|c| c.is_ascii_lowercase());
                    let has_digit = s.chars().any(|c| c.is_ascii_digit());
                    if has_upper && has_lower && has_digit {
                        Ok(())
                    } else {
                        Err("Password must contain uppercase, lowercase, and digits".to_string())
                    }
                })
        };

        // Test valid data
        let input = json!({
            "username": "  JohnDoe123  ",
            "password": "  Password123  "
        });
        let result = schema.validate(&input).unwrap();
        let output = result.as_object().unwrap();
        assert_eq!(output["username"], "johndoe123");
        assert_eq!(output["password"], "Password123");

        // Test invalid username
        let input = json!({
            "username": "  John@Doe  ",
            "password": "Password123"
        });
        let err = schema.validate(&input).unwrap_err();
        assert!(err.to_string().contains("can only contain"));

        // Test invalid password
        let input = json!({
            "username": "johndoe",
            "password": "password123"
        });
        let err = schema.validate(&input).unwrap_err();
        assert!(err.to_string().contains("must contain uppercase"));
    }

    #[test]
    fn test_recursive_schema() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Comment {
            id: i32,
            text: String,
            author: String,
            replies: Vec<Comment>,
        }

        fn comment_schema() -> impl Schema {
            object! {
                id: number().integer().min(1.0),
                text: string().trim().min_length(1),
                author: string().trim().min_length(2),
                replies: array(comment_schema())
            }
        }

        let input = json!({
            "id": 1,
            "text": "  Parent comment  ",
            "author": "  John  ",
            "replies": [
                {
                    "id": 2,
                    "text": "  First reply  ",
                    "author": "  Jane  ",
                    "replies": [
                        {
                            "id": 3,
                            "text": "  Nested reply  ",
                            "author": "  Bob  ",
                            "replies": []
                        }
                    ]
                }
            ]
        });

        let schema = comment_schema();
        let result = schema.validate(&input).unwrap();
        let comment: Comment = serde_json::from_value(result).unwrap();

        assert_eq!(comment.text, "Parent comment");
        assert_eq!(comment.author, "John");
        assert_eq!(comment.replies[0].text, "First reply");
        assert_eq!(comment.replies[0].replies[0].text, "Nested reply");
    }

    #[test]
    fn test_union_types_with_transformations() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[serde(untagged)]
        enum UserId {
            Numeric(i64),
            Username(String),
            Email(String),
        }

        let schema = union![
            number().integer().min(1.0),
            string().trim().min_length(3).max_length(20).pattern(r"^[a-zA-Z0-9_]+$"),
            string().trim().to_lower_case().email()
        ];

        // Test numeric ID
        let input = json!(42);
        let result = schema.validate(&input).unwrap();
        let id: UserId = serde_json::from_value(result).unwrap();
        assert_eq!(id, UserId::Numeric(42));

        // Test username
        let input = json!("  JohnDoe123  ");
        let result = schema.validate(&input).unwrap();
        let id: UserId = serde_json::from_value(result).unwrap();
        assert_eq!(id, UserId::Username("JohnDoe123".to_string()));

        // Test email
        let input = json!("  JOHN@EXAMPLE.COM  ");
        let result = schema.validate(&input).unwrap();
        let id: UserId = serde_json::from_value(result).unwrap();
        assert_eq!(id, UserId::Email("john@example.com".to_string()));

        // Test invalid cases
        assert!(schema.validate(&json!(0)).is_err()); // Invalid numeric ID
        assert!(schema.validate(&json!("ab")).is_err()); // Username too short
        assert!(schema.validate(&json!("not-an-email")).is_err()); // Invalid email
    }

    #[test]
    fn test_edge_cases() {
        let schema = object! {
            // Empty string after trim
            name: string().trim().min_length(1),
            // Integer that would overflow u8
            small_number: number().integer().min(0.0).max(255.0),
            // Array with exact length
            codes: array(string()).min_items(2).max_items(2),
            // Optional field with transformation
            email?: string().trim().to_lower_case().email(),
            // Nested object with all optional fields
            settings: object! {
                theme?: string().trim().to_lower_case(),
                notifications?: boolean(),
                volume?: number().min(0.0).max(100.0)
            }
        };

        // Test empty string after trim
        let input = json!({
            "name": "   ",
            "small_number": 100,
            "codes": ["A1", "B2"]
        });
        let err = schema.validate(&input).unwrap_err();
        assert!(err.to_string().contains("length"));

        // Test number overflow
        let input = json!({
            "name": "John",
            "small_number": 256,
            "codes": ["A1", "B2"]
        });
        let err = schema.validate(&input).unwrap_err();
        assert!(err.to_string().contains("too large"));

        // Test array length
        let input = json!({
            "name": "John",
            "small_number": 100,
            "codes": ["A1"]
        });
        let err = schema.validate(&input).unwrap_err();
        assert!(err.to_string().contains("items"));

        // Test empty object
        let input = json!({
            "name": "John",
            "small_number": 100,
            "codes": ["A1", "B2"],
            "settings": {}
        });
        assert!(schema.validate(&input).is_ok());

        // Test all optional fields present
        let input = json!({
            "name": "John",
            "small_number": 100,
            "codes": ["A1", "B2"],
            "email": "  JOHN@EXAMPLE.COM  ",
            "settings": {
                "theme": "  DARK  ",
                "notifications": true,
                "volume": 50
            }
        });
        let result = schema.validate(&input).unwrap();
        let output = result.as_object().unwrap();
        assert_eq!(output["email"], "john@example.com");
        assert_eq!(output["settings"]["theme"], "dark");
    }

    #[test]
    fn test_transformation_error_handling() {
        let schema = object! {
            email: string().trim().to_lower_case().email(),
            age: number().min(0.0).to_integer(),
            tags: array(string().trim().to_lower_case())
        };

        // Test invalid email after transformation
        let input = json!({
            "email": "  NOT-AN-EMAIL  ",
            "age": 30,
            "tags": ["TAG1"]
        });
        let err = schema.validate(&input).unwrap_err();
        assert!(err.to_string().contains("email"));

        // Test invalid number for integer conversion
        let input = json!({
            "email": "john@example.com",
            "age": -1.5,
            "tags": ["TAG1"]
        });
        let err = schema.validate(&input).unwrap_err();
        assert!(err.to_string().contains("too small"));

        // Test type mismatch
        let input = json!({
            "email": "john@example.com",
            "age": "30",
            "tags": ["TAG1"]
        });
        let err = schema.validate(&input).unwrap_err();
        assert!(err.to_string().contains("number"));
    }

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
        postal_code: Option<String>,
    }

    #[test]
    fn test_object_macro_new_syntax() {
        // Using new identifier syntax
        let schema = object! {
            name: string().min_length(2),
            age: number().min(0.0),
            email?: string().email(),  // Optional field
        };

        let valid_data = json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com"
        });

        let result = schema.validate(&valid_data).unwrap();
        let user: User = serde_json::from_value(result).unwrap();
        assert_eq!(user.name, "John");
        assert_eq!(user.age, 30);
        assert_eq!(user.email, Some("john@example.com".to_string()));

        // Test without optional field
        let valid_data = json!({
            "name": "John",
            "age": 30
        });

        let result = schema.validate(&valid_data).unwrap();
        let user: User = serde_json::from_value(result).unwrap();
        assert_eq!(user.name, "John");
        assert_eq!(user.age, 30);
        assert_eq!(user.email, None);
    }

    #[test]
    fn test_nested_object_macro() {
        let schema = object! {
            name: string(),
            address: object! {
                street: string(),
                city: string(),
                postal_code?: string(),  // Optional field
            }
        };

        let valid_data = json!({
            "name": "John",
            "address": {
                "street": "123 Main St",
                "city": "New York",
                "postal_code": "10001"
            }
        });

        assert!(schema.validate(&valid_data).is_ok());

        // Without optional field
        let valid_data = json!({
            "name": "John",
            "address": {
                "street": "123 Main St",
                "city": "New York"
            }
        });

        assert!(schema.validate(&valid_data).is_ok());
    }
}
