#[cfg(test)]
mod tests {
    use crate::{string, number, array, object, Schema};
    use serde_json::json;

    #[test]
    fn test_string_validation() {
        let schema = string()
            .min_length(3)
            .max_length(10)
            .pattern(r"^[a-zA-Z]+$")
            .error_message("string.too_short", "String is too short, minimum length is {min_length}")
            .error_message("string.pattern", "String must contain only letters");

        assert!(schema.validate(&json!("hello")).is_ok());
        assert!(schema.validate(&json!("hi")).is_err());
        assert!(schema.validate(&json!("toolongstring")).is_err());
        assert!(schema.validate(&json!("123")).is_err());
    }

    #[test]
    fn test_email_validation() {
        let schema = string()
            .email()
            .error_message("string.email", "Invalid email address format");

        assert!(schema.validate(&json!("test@example.com")).is_ok());
        assert!(schema.validate(&json!("invalid-email")).is_err());
    }

    #[test]
    fn test_optional_string() {
        let schema = string().min_length(3).optional();

        assert!(schema.validate(&json!("hello")).is_ok());
        assert!(schema.validate(&json!(null)).is_ok());
        assert!(schema.validate(&json!("hi")).is_err());
    }

    #[test]
    fn test_number_validation() {
        let schema = number()
            .min(0.0)
            .max(100.0)
            .integer();

        assert!(schema.validate(&json!(42)).is_ok());
        assert!(schema.validate(&json!(-1)).is_err());
        assert!(schema.validate(&json!(101)).is_err());
        assert!(schema.validate(&json!(42.5)).is_err());
    }

    #[test]
    fn test_array_validation() {
        let schema = array(string().min_length(2))
            .min_items(1)
            .max_items(3);

        assert!(schema.validate(&json!(["aa", "bbb"])).is_ok());
        assert!(schema.validate(&json!([])).is_err());
        assert!(schema.validate(&json!(["a"])).is_err());
        assert!(schema.validate(&json!(["aa", "bb", "cc", "dd"])).is_err());
    }

    #[test]
    fn test_object_validation() {
        let schema = object!({
            "name" => string().min_length(3),
            "age" => number().min(0.0).max(150.0),
            "email" => string().email(),
            "tags" => array(string())
        });

        let valid_data = json!({
            "name": "John",
            "age": 30,
            "email": "john@example.com",
            "tags": ["user", "admin"]
        });

        let invalid_data = json!({
            "name": "Jo",
            "age": -1,
            "email": "invalid-email",
            "tags": ["user", 123]
        });

        assert!(schema.validate(&valid_data).is_ok());
        assert!(schema.validate(&invalid_data).is_err());
    }

    #[test]
    fn test_custom_validation() {
        let schema = string().custom(|s| {
            if s.chars().all(|c| c.is_ascii_uppercase()) {
                Ok(())
            } else {
                Err("String must be uppercase".to_string())
            }
        });

        assert!(schema.validate(&json!("HELLO")).is_ok());
        assert!(schema.validate(&json!("Hello")).is_err());
    }

    #[test]
    fn test_string_patterns() {
        let schema = string().url();
        assert!(schema.validate(&json!("https://example.com")).is_ok());
        assert!(schema.validate(&json!("not a url")).is_err());

        let schema = string().uuid();
        assert!(schema.validate(&json!("550e8400-e29b-41d4-a716-446655440000")).is_ok());
        assert!(schema.validate(&json!("not a uuid")).is_err());

        let schema = string().ip();
        assert!(schema.validate(&json!("192.168.1.1")).is_ok());
        assert!(schema.validate(&json!("not an ip")).is_err());
    }

    #[test]
    fn test_error_context() {
        let schema = string()
            .min_length(5)
            .error_message("string.too_short", "String is too short, minimum length is {min_length}");

        let err = schema.validate(&json!("hi")).unwrap_err();
        assert_eq!(err.context.code, "string.too_short");
        assert_eq!(err.context.min_length, Some(5));
        assert!(err.to_string().contains("minimum length is 5"));
    }

    #[test]
    fn test_error_paths() {
        let schema = object!({
            "user" => object!({
                "address" => object!({
                    "zipcode" => string()
                        .pattern(r"^\d{5}$")
                        .error_message("string.pattern", "Invalid ZIP code format")
                })
            })
        });

        let invalid_data = json!({
            "user": {
                "address": {
                    "zipcode": "invalid"
                }
            }
        });

        let err = schema.validate(&invalid_data).unwrap_err();
        assert!(err.to_string().contains("Invalid ZIP code format"));
        assert_eq!(err.context.path, "user.address.zipcode");
    }
}