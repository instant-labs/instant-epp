//! Extension registry for session-level extension management

use std::collections::HashSet;

use crate::hello::Greeting;

/// Validation mode for extension usage
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationMode {
    /// Strict: Return error if extension not enabled
    Strict,
    /// Warn: Log warning if extension not enabled
    Warn,
    /// None: No validation
    None,
}

impl Default for ValidationMode {
    fn default() -> Self {
        Self::Warn
    }
}

/// Registry of enabled extensions for a session
///
/// Tracks which extensions have been negotiated during login and are available for use.
/// Can validate extension usage based on configured mode.
#[derive(Clone, Debug)]
pub struct ExtensionRegistry {
    /// Extensions negotiated during login
    enabled: HashSet<String>,
    /// Extensions supported by server (from greeting)
    server_supported: HashSet<String>,
    /// Validation mode
    validation: ValidationMode,
}

impl ExtensionRegistry {
    /// Create a new extension registry with the given validation mode
    pub fn new(validation: ValidationMode) -> Self {
        Self {
            enabled: HashSet::new(),
            server_supported: HashSet::new(),
            validation,
        }
    }

    /// Create an extension registry from a greeting
    pub fn from_greeting(greeting: &Greeting, validation: ValidationMode) -> Self {
        let mut registry = Self::new(validation);
        
        // Parse server-supported extensions from greeting
        if let Some(svc_ext) = &greeting.svc_menu.services.svc_ext {
            for uri in &svc_ext.ext_uris {
                registry.server_supported.insert(uri.to_string());
            }
        }
        
        registry
    }

    /// Enable an extension for this session
    pub fn enable(&mut self, uri: &str) {
        self.enabled.insert(uri.to_string());
    }

    /// Enable multiple extensions for this session
    pub fn enable_all<I>(&mut self, uris: I)
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        for uri in uris {
            self.enabled.insert(uri.as_ref().to_string());
        }
    }

    /// Check if an extension is enabled for this session
    pub fn is_enabled(&self, uri: &str) -> bool {
        self.enabled.contains(uri)
    }

    /// Check if an extension is supported by the server
    pub fn is_supported(&self, uri: &str) -> bool {
        self.server_supported.contains(uri)
    }

    /// Get all enabled extension URIs
    pub fn enabled_extensions(&self) -> Vec<&str> {
        self.enabled.iter().map(|s| s.as_str()).collect()
    }

    /// Get all server-supported extension URIs
    pub fn supported_extensions(&self) -> Vec<&str> {
        self.server_supported.iter().map(|s| s.as_str()).collect()
    }

    /// Validate that an extension is enabled
    ///
    /// Returns Ok(()) if validation passes or is disabled,
    /// Err if validation is strict and extension is not enabled.
    pub fn validate(&self, xmlns: &str) -> Result<(), crate::Error> {
        match self.validation {
            ValidationMode::None => Ok(()),
            ValidationMode::Warn => {
                if !self.is_enabled(xmlns) {
                    tracing::warn!(
                        "Extension {} is not enabled for this session. Enabled extensions: {:?}",
                        xmlns,
                        self.enabled_extensions()
                    );
                }
                Ok(())
            }
            ValidationMode::Strict => {
                if !self.is_enabled(xmlns) {
                    return Err(crate::Error::Other(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!(
                            "Extension {} is not enabled for this session. Enabled extensions: {:?}",
                            xmlns,
                            self.enabled_extensions()
                        ),
                    ))));
                }
                Ok(())
            }
        }
    }

    /// Get the validation mode
    pub fn validation_mode(&self) -> ValidationMode {
        self.validation
    }

    /// Set the validation mode
    pub fn set_validation_mode(&mut self, mode: ValidationMode) {
        self.validation = mode;
    }
}

impl Default for ExtensionRegistry {
    fn default() -> Self {
        Self::new(ValidationMode::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registry() {
        let registry = ExtensionRegistry::new(ValidationMode::Strict);
        assert_eq!(registry.validation_mode(), ValidationMode::Strict);
        assert!(registry.enabled_extensions().is_empty());
        assert!(registry.supported_extensions().is_empty());
    }

    #[test]
    fn test_enable_extension() {
        let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
        registry.enable("http://example.com/ext-1.0");
        
        assert!(registry.is_enabled("http://example.com/ext-1.0"));
        assert!(!registry.is_enabled("http://other.com/ext-1.0"));
    }

    #[test]
    fn test_enable_multiple() {
        let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
        registry.enable_all(vec![
            "http://example.com/ext-1.0",
            "http://example.com/ext-2.0",
        ]);
        
        assert!(registry.is_enabled("http://example.com/ext-1.0"));
        assert!(registry.is_enabled("http://example.com/ext-2.0"));
        assert_eq!(registry.enabled_extensions().len(), 2);
    }

    #[test]
    fn test_validation_none() {
        let registry = ExtensionRegistry::new(ValidationMode::None);
        assert!(registry.validate("http://example.com/ext-1.0").is_ok());
    }

    #[test]
    fn test_validation_warn() {
        let registry = ExtensionRegistry::new(ValidationMode::Warn);
        // Should not error, only warn
        assert!(registry.validate("http://example.com/ext-1.0").is_ok());
    }

    #[test]
    fn test_validation_strict() {
        let registry = ExtensionRegistry::new(ValidationMode::Strict);
        // Should error when extension not enabled
        assert!(registry.validate("http://example.com/ext-1.0").is_err());
        
        let mut registry = ExtensionRegistry::new(ValidationMode::Strict);
        registry.enable("http://example.com/ext-1.0");
        // Should succeed when enabled
        assert!(registry.validate("http://example.com/ext-1.0").is_ok());
    }
}

