use rusty_zod::{prelude::*, StringSchema};
use serde_json::json;

fn main() {
    let user_schema = object!({
        "id" => number()
            .min(1.0)
            .error_message("number.min", "ID must be a positive number"),
        "username" => string()
            .min_length(3)
            .max_length(20)
            .pattern(r"^[a-zA-Z0-9_]+$")
            .error_message("string.pattern", "Username must contain only letters, numbers, and underscores")
            .error_message("string.too_short", "Username must be at least {min_length} characters long")
            .error_message("string.too_long", "Username cannot be longer than {max_length} characters"),
        "email" => string()
            .email()
            .error_message("string.email", "Please enter a valid email address"),
        "website" => string()
            .url()
            .optional()
            .error_message("string.url", "Please enter a valid website URL"),
        "bio" => string()
            .min_length(10)
            .max_length(500)
            .optional()
            .error_message("string.too_short", "Bio must be at least {min_length} characters")
            .error_message("string.too_long", "Bio cannot exceed {max_length} characters"),
        "age" => number()
            .min(13.0)
            .max(150.0)
            .coerce()
            .optional()
            .error_message("number.min", "User must be at least {min} years old")
            .error_message("number.max", "Invalid age value"),
        "is_active" => boolean()
    });

    let valid_user = json!({
        "id": 10,
        "username": "rustacean_2024",
        "email": "rust@example.com",
        "website": "https://rust-lang.org",
        "bio": "A passionate Rust developer with over 5 years of experience.",
        "age": "25",
        "is_active": true
    });

    let invalid_user = json!({
        "id": 0,
        "username": "a@b",
        "email": "invalid-email",
        "website": "not-a-url",
        "bio": "too short",
        "age": "not a number",
        "is_active": true
    });

    println!("Validating valid user data:");
    match user_schema.validate(&valid_user) {
        Ok(_) => println!("✅ Valid user data: {}", valid_user),
        Err(e) => eprintln!("❌ Validation error: {}", e),
    }

    println!("\nValidating invalid user data:");
    match user_schema.validate(&invalid_user) {
        Ok(_) => println!("✅ Valid user data: {}", invalid_user),
        Err(e) => {
            eprintln!("❌ Validation error: {}", e);
            eprintln!("Error context: {}", serde_json::to_string_pretty(&e.context).unwrap());
        }
    }
}