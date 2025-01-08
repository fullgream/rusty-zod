use rusty_zod::prelude::*;
use serde_json::json;
use std::sync::Arc;

fn main() {
    // Example 1: Union type (oneOf)
    let id_schema = union!(
        string().uuid(),                     // UUID string
        number().integer().min(1.0)          // Positive integer
    ).error_message("union.no_match", "ID must be either a UUID or a positive integer");

    println!("Testing union type (UUID or positive integer):");
    test_schema(&id_schema, &[
        json!("550e8400-e29b-41d4-a716-446655440000"),  // valid UUID
        json!(42),  // valid positive integer
        json!(-1),  // invalid negative integer
        json!("not-a-uuid"),  // invalid string
    ]);

    // Example 2: Intersection type (allOf)
    let password_schema = all_of!(
        string().min_length(8),  // at least 8 chars
        string().pattern(r"[A-Z]").error_message("string.pattern", "Must contain uppercase letter"),
        string().pattern(r"[a-z]").error_message("string.pattern", "Must contain lowercase letter"),
        string().pattern(r"\d").error_message("string.pattern", "Must contain digit")
    ).error_message("union.no_match", "Password must meet all requirements");

    println!("\nTesting intersection type (password requirements):");
    test_schema(&password_schema, &[
        json!("Password123"),  // valid
        json!("password"),     // missing uppercase and digit
        json!("SHORT1"),      // too short
        json!("NOLOWER123"),  // missing lowercase
    ]);

    // Example 3: Best match with custom scoring
    let number_schema = best_of!(
        number().integer(),                                                    // prefer integers
        number(),                                                             // fallback to any number
        string().pattern(r"^\d+$")
            .error_message("string.pattern", "Must be numeric string");       // numeric strings
        |e| match e.context.code.as_str() {
            "number.integer" => 1,    // integer validation failure (least severe)
            "number.invalid_type" => 2,  // not a number
            "string.pattern" => 3,    // not even a numeric string (most severe)
            _ => 4,
        }
    );

    println!("\nTesting best match (number preference):");
    test_schema(&number_schema, &[
        json!(42),      // perfect: integer
        json!(42.5),    // ok: number
        json!("123"),   // acceptable: numeric string
        json!("abc"),   // invalid: not numeric
    ]);
}

fn test_schema(schema: &impl Schema, values: &[serde_json::Value]) {
    for value in values {
        print!("Testing {:?}: ", value);
        match schema.validate(value) {
            Ok(_) => println!("✅ Valid"),
            Err(e) => println!("❌ {}", e),
        }
    }
}