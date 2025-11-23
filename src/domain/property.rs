use anyhow::{anyhow, Result};
use std::fmt;

/// A task property with validated key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub key: PropertyKey,
    pub value: String,
}

impl Property {
    pub fn new(key: PropertyKey, value: String) -> Self {
        Property { key, value }
    }
}

/// A validated property key (capitalized, max 3 words)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PropertyKey(String);

impl PropertyKey {
    /// Create a new PropertyKey with validation
    pub fn new(key: &str) -> Result<Self> {
        // Must start with uppercase
        if !key.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            return Err(anyhow!(
                "Property key '{}' must start with uppercase letter",
                key
            ));
        }

        // Max 3 words
        let word_count = key.split_whitespace().count();
        if word_count > 3 {
            return Err(anyhow!(
                "Property key '{}' must have max 3 words (has {})",
                key,
                word_count
            ));
        }

        if word_count == 0 {
            return Err(anyhow!("Property key cannot be empty"));
        }

        Ok(PropertyKey(key.to_string()))
    }

    /// Try to parse a property key, returning None if invalid
    pub fn try_parse(key: &str) -> Option<Self> {
        Self::new(key).ok()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    // Predefined common properties
    pub fn status() -> Self {
        PropertyKey("Status".to_string())
    }

    pub fn assigned_to() -> Self {
        PropertyKey("Assigned To".to_string())
    }

    pub fn tags() -> Self {
        PropertyKey("Tags".to_string())
    }

    pub fn priority() -> Self {
        PropertyKey("Priority".to_string())
    }

    pub fn due_date() -> Self {
        PropertyKey("Due Date".to_string())
    }
}

impl fmt::Display for PropertyKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_single_word() {
        let key = PropertyKey::new("Status").unwrap();
        assert_eq!(key.as_str(), "Status");
    }

    #[test]
    fn test_valid_two_words() {
        let key = PropertyKey::new("Assigned To").unwrap();
        assert_eq!(key.as_str(), "Assigned To");
    }

    #[test]
    fn test_valid_three_words() {
        let key = PropertyKey::new("Some Other Property").unwrap();
        assert_eq!(key.as_str(), "Some Other Property");
    }

    #[test]
    fn test_invalid_lowercase() {
        assert!(PropertyKey::new("status").is_err());
    }

    #[test]
    fn test_invalid_too_many_words() {
        assert!(PropertyKey::new("This Has Too Many Words").is_err());
    }

    #[test]
    fn test_invalid_empty() {
        assert!(PropertyKey::new("").is_err());
    }

    #[test]
    fn test_predefined_keys() {
        assert_eq!(PropertyKey::status().as_str(), "Status");
        assert_eq!(PropertyKey::assigned_to().as_str(), "Assigned To");
        assert_eq!(PropertyKey::tags().as_str(), "Tags");
    }

    #[test]
    fn test_try_parse() {
        assert!(PropertyKey::try_parse("Status").is_some());
        assert!(PropertyKey::try_parse("status").is_none());
    }
}
