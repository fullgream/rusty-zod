# Rusty ZOD

A Rust library for schema declaration and validation, inspired by [Zod](https://github.com/colinhacks/zod). Rusty ZOD provides a fluent API for defining schemas and validating JSON data with strong type safety and comprehensive error messages.

## Features

- 🦾 Strong type inference
- 🔬 JSON validation with detailed error messages
- ⚡ Composable schema definitions
- 🔄 Data transformation support
- 🎯 Rich set of validations
- 🌟 Union and intersection types
- 💪 Zero dependencies (except for serde_json)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rusty-zod = "0.1.0"
```

## Quick Start

```rust
use rusty_zod::prelude::*;
use serde_json::json;

// Define a schema for a user
let user_schema = object!({
    "id" => number().integer().min(1.0),
    "name" => string()
        .min_length(2)
        .max_length(50)
        .error_message("string.too_short", "Name must be at least {min_length} characters"),
    "email" => string()
        .email()
        .error_message("string.email", "Invalid email address format"),
    "age" => number()
        .min(0.0)
        .max(150.0)
        .optional()
        .error_message("number.min", "Age must be positive"),
    "tags" => array(string().min_length(1))
        .min_items(1)
        .error_message("array.min_items", "At least one tag is required")
});

// Validate data
let valid_data = json!({
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "age": 30,
    "tags": ["user"]
});

match user_schema.validate(&valid_data) {
    Ok(validated) => println!("Valid data: {:?}", validated),
    Err(error) => println!("Validation error: {}", error),
}
```

## Schema Types

### String Schema

```rust
let schema = string()
    .min_length(5)
    .max_length(50)
    .pattern(r"^[A-Za-z]+$")
    .email()  // Predefined email pattern
    .url()    // Predefined URL pattern
    .uuid()   // Predefined UUID pattern
    .ip()     // Predefined IP address pattern
    .optional()
    .error_message("string.too_short", "Must be at least {min_length} characters");
```

### Number Schema

```rust
let schema = number()
    .min(0.0)
    .max(100.0)
    .integer()  // Must be an integer
    .coerce()   // Allow string to number coercion
    .optional()
    .error_message("number.min", "Must be positive");
```

### Boolean Schema

```rust
let schema = boolean()
    .optional()
    .error_message("boolean.invalid_type", "Must be true or false");
```

### Array Schema

```rust
let schema = array(string().min_length(1))  // Array of non-empty strings
    .min_items(1)
    .max_items(10)
    .error_message("array.min_items", "At least one item required");
```

### Object Schema

```rust
let schema = object()
    .field("name", string())
    .optional_field("age", number().integer())
    .strict()  // No additional properties allowed
    .error_message("object.unknown_key", "Unknown property: {key}");
```

## Union Types

### OneOf (First Match)

```rust
let id_schema = union!(
    string().uuid(),           // UUID string
    number().integer().min(1)  // Positive integer
);
```

### AllOf (Intersection)

```rust
let password_schema = all_of!(
    string().min_length(8),
    string().pattern(r"[A-Z]").error_message("string.pattern", "Must contain uppercase"),
    string().pattern(r"[a-z]").error_message("string.pattern", "Must contain lowercase"),
    string().pattern(r"\d").error_message("string.pattern", "Must contain digit")
);
```

### Best Match

```rust
let number_schema = best_of!(
    number().integer(),  // Prefer integers
    number(),           // Allow any number
    string().pattern(r"^\d+$").error_message("string.pattern", "Must be numeric");
    |e| match e.context.code.as_str() {
        "number.integer" => 1,    // integer validation failure (least severe)
        "number.invalid_type" => 2,  // not a number
        "string.pattern" => 3,    // not even a numeric string (most severe)
        _ => 4,
    }
);
```

## Data Transformation

Schemas can transform data during validation:

```rust
let schema = string()
    .trim()               // Remove whitespace
    .to_lowercase()       // Convert to lowercase
    .transform(|v| {      // Custom transformation
        if let Value::String(s) = v {
            Value::String(format!("prefix_{}", s))
        } else {
            v
        }
    });
```

## Error Handling

Validation errors provide detailed information:

```rust
match schema.validate(&data) {
    Ok(validated) => println!("Valid: {:?}", validated),
    Err(error) => {
        println!("Code: {}", error.context.code);
        println!("Message: {}", error.to_string());
        println!("Path: {}", error.context.path);
        // Additional context like min, max, pattern, etc.
        if let Some(min) = error.context.min {
            println!("Min value: {}", min);
        }
    }
}
```

## Custom Validation

Add custom validation rules:

```rust
let schema = string()
    .custom(|s| {
        if s.chars().all(|c| c.is_ascii_digit()) {
            Ok(())
        } else {
            Err("Must contain only digits".to_string())
        }
    });
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.