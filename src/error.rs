use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct ValidationContext {
    pub code: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
}

#[derive(Debug, Error)]
#[error("{}", self.format_message())]
pub struct ValidationError {
    pub context: ValidationContext,
    message: Option<String>,
}

impl ValidationError {
    fn format_message(&self) -> String {
        let mut msg = self.message.clone().unwrap_or_default();
        if let Some(min_length) = self.context.min_length {
            msg = msg.replace("{min_length}", &min_length.to_string());
        }
        if let Some(max_length) = self.context.max_length {
            msg = msg.replace("{max_length}", &max_length.to_string());
        }
        if let Some(min) = self.context.min {
            msg = msg.replace("{min}", &min.to_string());
        }
        if let Some(max) = self.context.max {
            msg = msg.replace("{max}", &max.to_string());
        }
        msg
    }

    pub fn new(code: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            context: ValidationContext {
                code: code.into(),
                path: path.into(),
                min: None,
                max: None,
                min_length: None,
                max_length: None,
                pattern: None,
                expected: None,
                actual: None,
            },
            message: None,
        }
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_min(mut self, min: i64) -> Self {
        self.context.min = Some(min);
        self
    }

    pub fn with_max(mut self, max: i64) -> Self {
        self.context.max = Some(max);
        self
    }

    pub fn with_min_length(mut self, min_length: usize) -> Self {
        self.context.min_length = Some(min_length);
        self
    }

    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.context.max_length = Some(max_length);
        self
    }

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.context.pattern = Some(pattern.into());
        self
    }

    pub fn with_type_info(mut self, expected: impl Into<String>, actual: impl Into<String>) -> Self {
        self.context.expected = Some(expected.into());
        self.context.actual = Some(actual.into());
        self
    }

    pub fn with_path_prefix(mut self, prefix: &str) -> Self {
        if self.context.path.is_empty() {
            self.context.path = prefix.to_string();
        } else {
            self.context.path = format!("{}.{}", prefix, self.context.path);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_formatting() {
        let error = ValidationError::new("test", "field")
            .with_message("Value must be between {min} and {max}")
            .with_min(5)
            .with_max(10);

        assert_eq!(error.format_message(), "Value must be between 5 and 10");
    }

    #[test]
    fn test_error_path_prefix() {
        let error = ValidationError::new("test", "field")
            .with_path_prefix("user")
            .with_path_prefix("data");

        assert_eq!(error.context.path, "data.user.field");
    }

    #[test]
    fn test_error_serialization() {
        let error = ValidationError::new("string.length", "name")
            .with_message("String length must be between {min_length} and {max_length}")
            .with_min_length(3)
            .with_max_length(10);

        let json = serde_json::to_value(&error.context).unwrap();
        assert_eq!(json["code"], "string.length");
        assert_eq!(json["path"], "name");
        assert_eq!(json["min_length"], 3);
        assert_eq!(json["max_length"], 10);
    }
}