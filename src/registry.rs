//! Extension registry for session-level extension management
//!
//! This module provides types for managing EPP extensions at the session level.
//! Extensions must be negotiated during login and can be validated before use.
//!
//! # Examples
//!
//! ```no_run
//! use instant_epp::registry::{ExtensionRegistry, ValidationMode};
//!
//! // Create registry with strict validation
//! let mut registry = ExtensionRegistry::new(ValidationMode::Strict);
//!
//! // Enable extensions for this session
//! registry.enable("http://www.verisign-grs.com/epp/namestoreExt-1.1");
//! registry.enable("urn:ietf:params:xml:ns:rgp-1.0");
//!
//! // Check if extension is enabled
//! assert!(registry.is_enabled("http://www.verisign-grs.com/epp/namestoreExt-1.1"));
//!
//! // Validate extension usage
//! registry.validate("http://www.verisign-grs.com/epp/namestoreExt-1.1").unwrap();
//! ```

use std::collections::HashSet;

use crate::hello::Greeting;

/// Metadata about a known EPP extension
///
/// Provides information about standard EPP extensions including their namespace URI,
/// human-readable name, description, and relevant RFC or specification reference.
///
/// # Examples
///
/// ```
/// use instant_epp::registry::{ExtensionMetadata, KnownExtensions};
///
/// // Access metadata for a known extension
/// let namestore = KnownExtensions::NAMESTORE;
/// assert_eq!(namestore.xmlns, "http://www.verisign-grs.com/epp/namestoreExt-1.1");
/// assert_eq!(namestore.name, "Namestore");
///
/// // Find extension by namespace
/// if let Some(metadata) = KnownExtensions::by_xmlns("urn:ietf:params:xml:ns:rgp-1.0") {
///     println!("Found: {}", metadata.name);
///     println!("RFC: {}", metadata.rfc.unwrap_or("N/A"));
/// }
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtensionMetadata {
    /// The XML namespace URI that identifies this extension
    pub xmlns: &'static str,
    /// Human-readable name of the extension
    pub name: &'static str,
    /// Brief description of what the extension does
    pub description: &'static str,
    /// RFC number or specification reference, if applicable
    pub rfc: Option<&'static str>,
}

/// Registry of well-known EPP extensions
///
/// Provides constants and lookup methods for commonly-used EPP extensions.
/// This is useful for discovery, documentation, and validation.
///
/// # Examples
///
/// ```
/// use instant_epp::registry::KnownExtensions;
///
/// // Access extension metadata
/// println!("Namestore: {}", KnownExtensions::NAMESTORE.description);
/// println!("RGP: {}", KnownExtensions::RGP.description);
///
/// // Get all known extensions
/// for ext in KnownExtensions::all() {
///     println!("{}: {}", ext.name, ext.xmlns);
/// }
///
/// // Look up by namespace
/// if let Some(ext) = KnownExtensions::by_xmlns("urn:ietf:params:xml:ns:secDNS-1.1") {
///     println!("Found: {} - {}", ext.name, ext.description);
/// }
/// ```
pub struct KnownExtensions;

impl KnownExtensions {
    /// Verisign Namestore Extension
    ///
    /// Used for TLD selection in registries that manage multiple TLDs.
    pub const NAMESTORE: ExtensionMetadata = ExtensionMetadata {
        xmlns: "http://www.verisign-grs.com/epp/namestoreExt-1.1",
        name: "Namestore",
        description: "Verisign Namestore Extension for TLD selection",
        rfc: None,
    };

    /// Registry Grace Period (RGP) Extension
    ///
    /// Implements domain lifecycle grace periods for restoration and deletion.
    pub const RGP: ExtensionMetadata = ExtensionMetadata {
        xmlns: "urn:ietf:params:xml:ns:rgp-1.0",
        name: "Registry Grace Period",
        description: "Domain lifecycle grace periods (restore, redemption)",
        rfc: Some("RFC 3915"),
    };

    /// DNSSEC Extension
    ///
    /// Manages DNSSEC delegation signer (DS) records for domains.
    pub const SECDNS: ExtensionMetadata = ExtensionMetadata {
        xmlns: "urn:ietf:params:xml:ns:secDNS-1.1",
        name: "SecDNS",
        description: "DNSSEC delegation signer (DS) record management",
        rfc: Some("RFC 5910"),
    };

    /// Verisign ConsoliDate Extension
    ///
    /// Manages consolidation date for domain updates.
    pub const CONSOLIDATE: ExtensionMetadata = ExtensionMetadata {
        xmlns: "http://www.verisign.com/epp/sync-1.0",
        name: "ConsoliDate",
        description: "Verisign consolidation date mapping",
        rfc: None,
    };

    /// Low Balance Notification Extension
    ///
    /// Notifies when account balance falls below threshold.
    pub const LOW_BALANCE: ExtensionMetadata = ExtensionMetadata {
        xmlns: "http://www.verisign-grs.com/epp/lowbalance-poll-1.0",
        name: "Low Balance",
        description: "Account balance notification extension",
        rfc: None,
    };

    /// Change Poll Extension
    ///
    /// Provides notifications about domain changes.
    pub const CHANGE_POLL: ExtensionMetadata = ExtensionMetadata {
        xmlns: "http://www.verisign.com/epp/emailFwd-1.0",
        name: "Change Poll",
        description: "Domain change notification extension",
        rfc: None,
    };

    /// Get all known extensions as a slice
    ///
    /// Returns a reference to an array containing metadata for all known extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::KnownExtensions;
    ///
    /// let all_extensions = KnownExtensions::all();
    /// println!("instant-epp knows about {} extensions", all_extensions.len());
    ///
    /// for ext in all_extensions {
    ///     println!("  - {}: {}", ext.name, ext.xmlns);
    /// }
    /// ```
    pub fn all() -> &'static [ExtensionMetadata] {
        &[
            Self::NAMESTORE,
            Self::RGP,
            Self::SECDNS,
            Self::CONSOLIDATE,
            Self::LOW_BALANCE,
            Self::CHANGE_POLL,
        ]
    }

    /// Look up extension metadata by namespace URI
    ///
    /// Searches the registry of known extensions for one matching the given namespace.
    ///
    /// # Arguments
    ///
    /// * `xmlns` - The XML namespace URI to search for
    ///
    /// # Returns
    ///
    /// - `Some(metadata)` if a matching extension is found
    /// - `None` if no known extension has that namespace
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::KnownExtensions;
    ///
    /// // Look up by namespace
    /// if let Some(ext) = KnownExtensions::by_xmlns("urn:ietf:params:xml:ns:rgp-1.0") {
    ///     assert_eq!(ext.name, "Registry Grace Period");
    ///     assert_eq!(ext.rfc, Some("RFC 3915"));
    /// }
    ///
    /// // Unknown extension returns None
    /// assert!(KnownExtensions::by_xmlns("http://unknown.example/ext").is_none());
    /// ```
    pub fn by_xmlns(xmlns: &str) -> Option<&'static ExtensionMetadata> {
        Self::all().iter().find(|ext| ext.xmlns == xmlns)
    }

    /// Look up extension metadata by name (case-insensitive)
    ///
    /// Searches for an extension by its human-readable name.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::KnownExtensions;
    ///
    /// // Case-insensitive lookup
    /// assert!(KnownExtensions::by_name("namestore").is_some());
    /// assert!(KnownExtensions::by_name("Namestore").is_some());
    /// assert!(KnownExtensions::by_name("NAMESTORE").is_some());
    ///
    /// let ext = KnownExtensions::by_name("Registry Grace Period").unwrap();
    /// assert_eq!(ext.xmlns, "urn:ietf:params:xml:ns:rgp-1.0");
    /// ```
    pub fn by_name(name: &str) -> Option<&'static ExtensionMetadata> {
        let name_lower = name.to_lowercase();
        Self::all()
            .iter()
            .find(|ext| ext.name.to_lowercase() == name_lower)
    }
}

/// Known registry-specific quirks and workarounds
///
/// Some EPP registries have non-standard behaviors or quirks that require special handling.
/// This enum documents and enables automated handling of these known issues.
///
/// # Examples
///
/// ```
/// use instant_epp::registry::{ExtensionRegistry, RegistryQuirk, ValidationMode};
///
/// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
///
/// // Enable quirk handling for Verisign Namestore + RGP
/// registry.add_quirk(RegistryQuirk::VerisignNamestoreRgpConflict);
///
/// // Now the registry knows about this quirk and can handle it appropriately
/// assert!(registry.has_quirk(&RegistryQuirk::VerisignNamestoreRgpConflict));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum RegistryQuirk {
    /// Verisign: Namestore extension not returned when RGP is used
    ///
    /// **Registry**: Verisign (.com, .net, etc.)
    ///
    /// **Issue**: When a command includes both Namestore and RGP extensions, the response
    /// may only contain the RGP extension, even though the operation was successful.
    ///
    /// **Workaround**: Don't validate that Namestore is present in the response when
    /// RGP is also used. The operation's success indicates both were processed.
    ///
    /// **Example**:
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, RegistryQuirk, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.add_quirk(RegistryQuirk::VerisignNamestoreRgpConflict);
    ///
    /// // This quirk is now active and will be considered during validation
    /// ```
    VerisignNamestoreRgpConflict,

    /// Generic placeholder for custom/unknown quirks
    ///
    /// Use this for registry-specific behaviors that aren't yet documented.
    /// Consider submitting a PR to document new quirks!
    Other(&'static str),
}

impl RegistryQuirk {
    /// Get a human-readable description of the quirk
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::RegistryQuirk;
    ///
    /// let quirk = RegistryQuirk::VerisignNamestoreRgpConflict;
    /// assert!(quirk.description().contains("Namestore"));
    /// assert!(quirk.description().contains("RGP"));
    /// ```
    pub fn description(&self) -> &str {
        match self {
            Self::VerisignNamestoreRgpConflict => {
                "Verisign does not return Namestore extension in responses when RGP extension is also used"
            }
            Self::Other(desc) => desc,
        }
    }

    /// Check if this quirk affects the given extension namespace
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{RegistryQuirk, KnownExtensions};
    ///
    /// let quirk = RegistryQuirk::VerisignNamestoreRgpConflict;
    /// assert!(quirk.affects_extension(KnownExtensions::NAMESTORE.xmlns));
    /// assert!(quirk.affects_extension(KnownExtensions::RGP.xmlns));
    /// assert!(!quirk.affects_extension(KnownExtensions::SECDNS.xmlns));
    /// ```
    pub fn affects_extension(&self, xmlns: &str) -> bool {
        match self {
            Self::VerisignNamestoreRgpConflict => {
                xmlns == KnownExtensions::NAMESTORE.xmlns || xmlns == KnownExtensions::RGP.xmlns
            }
            Self::Other(_) => false, // Unknown quirks don't affect anything by default
        }
    }
}

/// Validation mode for extension usage
///
/// Controls how the extension registry validates extension usage during transactions.
/// Different modes allow flexibility between development (warn) and production (strict).
///
/// # Examples
///
/// ```
/// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
///
/// // Development: warn about issues but don't fail
/// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
///
/// // Production: strict validation, fail on misuse
/// let mut registry = ExtensionRegistry::new(ValidationMode::Strict);
///
/// // Testing: no validation
/// let mut registry = ExtensionRegistry::new(ValidationMode::None);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationMode {
    /// Strict: Return error if extension not enabled
    ///
    /// Use in production environments where extension misuse should be caught immediately.
    Strict,
    /// Warn: Log warning if extension not enabled
    ///
    /// Use in development environments to identify issues without stopping execution.
    /// This is the default mode.
    Warn,
    /// None: No validation
    ///
    /// Use in testing or when you want to bypass validation entirely.
    None,
}

impl Default for ValidationMode {
    fn default() -> Self {
        Self::Warn
    }
}

/// Registry of enabled extensions for a session
///
/// The `ExtensionRegistry` tracks which EPP extensions have been negotiated during login
/// and validates their usage in subsequent commands. It maintains two sets:
/// - **Enabled extensions**: Extensions successfully negotiated during login
/// - **Server-supported extensions**: Extensions advertised by the server in its greeting
///
/// # Validation
///
/// The registry can validate extension usage based on the configured [`ValidationMode`]:
/// - `Strict`: Returns an error if an extension is not enabled
/// - `Warn`: Logs a warning but allows the operation (default)
/// - `None`: No validation performed
///
/// # Examples
///
/// ```no_run
/// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
/// # use instant_epp::hello::Greeting;
///
/// # let greeting = todo!();
/// // Create from greeting
/// let mut registry = ExtensionRegistry::from_greeting(&greeting, ValidationMode::Strict);
///
/// // Enable extensions after login
/// registry.enable_all(vec![
///     "http://www.verisign-grs.com/epp/namestoreExt-1.1",
///     "urn:ietf:params:xml:ns:rgp-1.0",
/// ]);
///
/// // Query enabled extensions
/// let enabled = registry.enabled_extensions();
/// println!("Enabled: {:?}", enabled);
///
/// // Validate before use
/// if let Err(e) = registry.validate("http://example.com/ext-1.0") {
///     eprintln!("Extension not enabled: {}", e);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ExtensionRegistry {
    /// Extensions negotiated during login
    enabled: HashSet<String>,
    /// Extensions supported by server (from greeting)
    server_supported: HashSet<String>,
    /// Validation mode
    validation: ValidationMode,
    /// Known registry quirks that affect this session
    quirks: HashSet<RegistryQuirk>,
}

impl ExtensionRegistry {
    /// Create a new extension registry with the given validation mode
    ///
    /// Creates an empty registry with no enabled or supported extensions.
    /// Extensions can be added with [`enable`](Self::enable) or [`enable_all`](Self::enable_all).
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    ///
    /// let registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// assert!(registry.enabled_extensions().is_empty());
    /// ```
    pub fn new(validation: ValidationMode) -> Self {
        Self {
            enabled: HashSet::new(),
            server_supported: HashSet::new(),
            validation,
            quirks: HashSet::new(),
        }
    }

    /// Create an extension registry from a greeting
    ///
    /// Initializes the registry with server-supported extensions parsed from the greeting's
    /// `<svcExtension>` element. The enabled set is initially empty and should be populated
    /// after successful login.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    /// # use instant_epp::hello::Greeting;
    ///
    /// # let greeting: Greeting = todo!();
    /// let registry = ExtensionRegistry::from_greeting(&greeting, ValidationMode::Strict);
    /// let supported = registry.supported_extensions();
    /// println!("Server supports: {:?}", supported);
    /// ```
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
    ///
    /// Marks the extension as enabled, typically called after successful login negotiation.
    /// The extension URI should match the namespace used in extension definitions.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.enable("http://www.verisign-grs.com/epp/namestoreExt-1.1");
    /// assert!(registry.is_enabled("http://www.verisign-grs.com/epp/namestoreExt-1.1"));
    /// ```
    pub fn enable(&mut self, uri: &str) {
        self.enabled.insert(uri.to_string());
    }

    /// Enable multiple extensions for this session
    ///
    /// Convenience method to enable several extensions at once, typically called after
    /// successful login with multiple extension URIs.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.enable_all(vec![
    ///     "http://www.verisign-grs.com/epp/namestoreExt-1.1",
    ///     "urn:ietf:params:xml:ns:rgp-1.0",
    ///     "urn:ietf:params:xml:ns:secDNS-1.1",
    /// ]);
    /// assert_eq!(registry.enabled_extensions().len(), 3);
    /// ```
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
    ///
    /// Returns `true` if the extension was successfully negotiated during login.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.enable("urn:ietf:params:xml:ns:rgp-1.0");
    ///
    /// assert!(registry.is_enabled("urn:ietf:params:xml:ns:rgp-1.0"));
    /// assert!(!registry.is_enabled("http://unknown.com/ext-1.0"));
    /// ```
    pub fn is_enabled(&self, uri: &str) -> bool {
        self.enabled.contains(uri)
    }

    /// Check if an extension is supported by the server
    ///
    /// Returns `true` if the server advertised support for this extension in its greeting.
    /// Note that an extension can be supported but not enabled if it wasn't negotiated during login.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    /// # use instant_epp::hello::Greeting;
    ///
    /// # let greeting: Greeting = todo!();
    /// let registry = ExtensionRegistry::from_greeting(&greeting, ValidationMode::Warn);
    ///
    /// // Check if server supports an extension before trying to enable it
    /// if registry.is_supported("urn:ietf:params:xml:ns:rgp-1.0") {
    ///     println!("RGP extension is available");
    /// }
    /// ```
    pub fn is_supported(&self, uri: &str) -> bool {
        self.server_supported.contains(uri)
    }

    /// Get all enabled extension URIs
    ///
    /// Returns a vector of all extension URIs that have been enabled for this session.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.enable("http://www.verisign-grs.com/epp/namestoreExt-1.1");
    /// registry.enable("urn:ietf:params:xml:ns:rgp-1.0");
    ///
    /// let enabled = registry.enabled_extensions();
    /// assert_eq!(enabled.len(), 2);
    /// ```
    pub fn enabled_extensions(&self) -> Vec<&str> {
        self.enabled.iter().map(|s| s.as_str()).collect()
    }

    /// Get all server-supported extension URIs
    ///
    /// Returns a vector of all extension URIs advertised by the server in its greeting.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    /// # use instant_epp::hello::Greeting;
    ///
    /// # let greeting: Greeting = todo!();
    /// let registry = ExtensionRegistry::from_greeting(&greeting, ValidationMode::Warn);
    /// let supported = registry.supported_extensions();
    /// println!("Server supports {} extensions", supported.len());
    /// ```
    pub fn supported_extensions(&self) -> Vec<&str> {
        self.server_supported.iter().map(|s| s.as_str()).collect()
    }

    /// Validate that an extension is enabled
    ///
    /// Checks if the given extension URI is enabled for this session according to the
    /// configured validation mode:
    /// - `ValidationMode::None`: Always returns `Ok(())`
    /// - `ValidationMode::Warn`: Logs a warning if not enabled, returns `Ok(())`
    /// - `ValidationMode::Strict`: Returns `Err` if not enabled
    ///
    /// # Errors
    ///
    /// Returns an error if validation mode is `Strict` and the extension is not enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Strict);
    /// registry.enable("urn:ietf:params:xml:ns:rgp-1.0");
    ///
    /// // Enabled extension - validation passes
    /// assert!(registry.validate("urn:ietf:params:xml:ns:rgp-1.0").is_ok());
    ///
    /// // Not enabled - validation fails in strict mode
    /// assert!(registry.validate("http://unknown.com/ext-1.0").is_err());
    /// ```
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

    /// Get the current validation mode
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    ///
    /// let registry = ExtensionRegistry::new(ValidationMode::Strict);
    /// assert_eq!(registry.validation_mode(), ValidationMode::Strict);
    /// ```
    pub fn validation_mode(&self) -> ValidationMode {
        self.validation
    }

    /// Set the validation mode
    ///
    /// Changes the validation mode for this registry. Useful for switching between
    /// strict validation in production and warn mode during development.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.set_validation_mode(ValidationMode::Strict);
    /// assert_eq!(registry.validation_mode(), ValidationMode::Strict);
    /// ```
    pub fn set_validation_mode(&mut self, mode: ValidationMode) {
        self.validation = mode;
    }

    /// Add a known registry quirk
    ///
    /// Enables handling for a specific registry quirk. This affects how validation and
    /// response parsing behave for affected extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, RegistryQuirk, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    ///
    /// // Enable Verisign Namestore + RGP quirk handling
    /// registry.add_quirk(RegistryQuirk::VerisignNamestoreRgpConflict);
    ///
    /// assert!(registry.has_quirk(&RegistryQuirk::VerisignNamestoreRgpConflict));
    /// ```
    pub fn add_quirk(&mut self, quirk: RegistryQuirk) {
        self.quirks.insert(quirk);
    }

    /// Add multiple registry quirks at once
    ///
    /// Convenience method to enable several quirks simultaneously.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, RegistryQuirk, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    ///
    /// registry.add_quirks(vec![
    ///     RegistryQuirk::VerisignNamestoreRgpConflict,
    ///     RegistryQuirk::Other("Custom quirk"),
    /// ]);
    ///
    /// assert_eq!(registry.active_quirks().len(), 2);
    /// ```
    pub fn add_quirks<I>(&mut self, quirks: I)
    where
        I: IntoIterator<Item = RegistryQuirk>,
    {
        for quirk in quirks {
            self.quirks.insert(quirk);
        }
    }

    /// Check if a specific quirk is enabled
    ///
    /// Returns `true` if the given quirk has been added to this registry.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, RegistryQuirk, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.add_quirk(RegistryQuirk::VerisignNamestoreRgpConflict);
    ///
    /// assert!(registry.has_quirk(&RegistryQuirk::VerisignNamestoreRgpConflict));
    /// assert!(!registry.has_quirk(&RegistryQuirk::Other("unknown")));
    /// ```
    pub fn has_quirk(&self, quirk: &RegistryQuirk) -> bool {
        self.quirks.contains(quirk)
    }

    /// Get all active quirks for this registry
    ///
    /// Returns a vector of all quirks that have been enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, RegistryQuirk, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.add_quirk(RegistryQuirk::VerisignNamestoreRgpConflict);
    ///
    /// let quirks = registry.active_quirks();
    /// assert_eq!(quirks.len(), 1);
    /// assert_eq!(quirks[0], RegistryQuirk::VerisignNamestoreRgpConflict);
    /// ```
    pub fn active_quirks(&self) -> Vec<RegistryQuirk> {
        self.quirks.iter().copied().collect()
    }

    /// Check if any active quirks affect the given extension
    ///
    /// Returns `true` if any enabled quirk affects the specified extension namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::registry::{ExtensionRegistry, RegistryQuirk, KnownExtensions, ValidationMode};
    ///
    /// let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
    /// registry.add_quirk(RegistryQuirk::VerisignNamestoreRgpConflict);
    ///
    /// // This quirk affects Namestore and RGP
    /// assert!(registry.extension_has_quirks(KnownExtensions::NAMESTORE.xmlns));
    /// assert!(registry.extension_has_quirks(KnownExtensions::RGP.xmlns));
    ///
    /// // But not SecDNS
    /// assert!(!registry.extension_has_quirks(KnownExtensions::SECDNS.xmlns));
    /// ```
    pub fn extension_has_quirks(&self, xmlns: &str) -> bool {
        self.quirks
            .iter()
            .any(|quirk| quirk.affects_extension(xmlns))
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

    #[test]
    fn test_known_extensions_constants() {
        // Test that all constants are properly defined
        assert_eq!(
            KnownExtensions::NAMESTORE.xmlns,
            "http://www.verisign-grs.com/epp/namestoreExt-1.1"
        );
        assert_eq!(KnownExtensions::NAMESTORE.name, "Namestore");

        assert_eq!(KnownExtensions::RGP.xmlns, "urn:ietf:params:xml:ns:rgp-1.0");
        assert_eq!(KnownExtensions::RGP.rfc, Some("RFC 3915"));

        assert_eq!(
            KnownExtensions::SECDNS.xmlns,
            "urn:ietf:params:xml:ns:secDNS-1.1"
        );
        assert_eq!(KnownExtensions::SECDNS.rfc, Some("RFC 5910"));
    }

    #[test]
    fn test_known_extensions_all() {
        let all = KnownExtensions::all();
        assert_eq!(all.len(), 6);

        // Check that Namestore is in the list
        assert!(all.iter().any(|ext| ext.name == "Namestore"));

        // Check that all have valid xmlns
        for ext in all {
            assert!(!ext.xmlns.is_empty());
            assert!(!ext.name.is_empty());
            assert!(!ext.description.is_empty());
        }
    }

    #[test]
    fn test_known_extensions_by_xmlns() {
        // Found extension
        let rgp = KnownExtensions::by_xmlns("urn:ietf:params:xml:ns:rgp-1.0");
        assert!(rgp.is_some());
        assert_eq!(rgp.unwrap().name, "Registry Grace Period");

        // Not found
        let unknown = KnownExtensions::by_xmlns("http://unknown.example/ext");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_known_extensions_by_name() {
        // Case-insensitive matching
        assert!(KnownExtensions::by_name("namestore").is_some());
        assert!(KnownExtensions::by_name("Namestore").is_some());
        assert!(KnownExtensions::by_name("NAMESTORE").is_some());

        // Get the extension
        let ns = KnownExtensions::by_name("namestore").unwrap();
        assert_eq!(ns.xmlns, "http://www.verisign-grs.com/epp/namestoreExt-1.1");

        // Not found
        assert!(KnownExtensions::by_name("unknown").is_none());
    }

    #[test]
    fn test_extension_metadata_equality() {
        let meta1 = KnownExtensions::NAMESTORE;
        let meta2 = KnownExtensions::NAMESTORE;
        assert_eq!(meta1, meta2);

        let meta3 = KnownExtensions::RGP;
        assert_ne!(meta1, meta3);
    }

    #[test]
    fn test_registry_quirk_description() {
        let quirk = RegistryQuirk::VerisignNamestoreRgpConflict;
        let desc = quirk.description();
        assert!(desc.contains("Namestore"));
        assert!(desc.contains("RGP"));

        let custom = RegistryQuirk::Other("Test quirk");
        assert_eq!(custom.description(), "Test quirk");
    }

    #[test]
    fn test_registry_quirk_affects_extension() {
        let quirk = RegistryQuirk::VerisignNamestoreRgpConflict;

        // This quirk affects Namestore and RGP
        assert!(quirk.affects_extension(KnownExtensions::NAMESTORE.xmlns));
        assert!(quirk.affects_extension(KnownExtensions::RGP.xmlns));

        // But not other extensions
        assert!(!quirk.affects_extension(KnownExtensions::SECDNS.xmlns));
        assert!(!quirk.affects_extension(KnownExtensions::CONSOLIDATE.xmlns));
    }

    #[test]
    fn test_add_quirk() {
        let mut registry = ExtensionRegistry::new(ValidationMode::Warn);

        // No quirks initially
        assert_eq!(registry.active_quirks().len(), 0);

        // Add a quirk
        registry.add_quirk(RegistryQuirk::VerisignNamestoreRgpConflict);
        assert_eq!(registry.active_quirks().len(), 1);
        assert!(registry.has_quirk(&RegistryQuirk::VerisignNamestoreRgpConflict));
    }

    #[test]
    fn test_add_multiple_quirks() {
        let mut registry = ExtensionRegistry::new(ValidationMode::Warn);

        registry.add_quirks(vec![
            RegistryQuirk::VerisignNamestoreRgpConflict,
            RegistryQuirk::Other("Test"),
        ]);

        assert_eq!(registry.active_quirks().len(), 2);
        assert!(registry.has_quirk(&RegistryQuirk::VerisignNamestoreRgpConflict));
        assert!(registry.has_quirk(&RegistryQuirk::Other("Test")));
    }

    #[test]
    fn test_extension_has_quirks() {
        let mut registry = ExtensionRegistry::new(ValidationMode::Warn);
        registry.add_quirk(RegistryQuirk::VerisignNamestoreRgpConflict);

        // Extensions affected by the quirk
        assert!(registry.extension_has_quirks(KnownExtensions::NAMESTORE.xmlns));
        assert!(registry.extension_has_quirks(KnownExtensions::RGP.xmlns));

        // Extensions not affected
        assert!(!registry.extension_has_quirks(KnownExtensions::SECDNS.xmlns));
    }

    #[test]
    fn test_quirk_equality() {
        let quirk1 = RegistryQuirk::VerisignNamestoreRgpConflict;
        let quirk2 = RegistryQuirk::VerisignNamestoreRgpConflict;
        assert_eq!(quirk1, quirk2);

        let quirk3 = RegistryQuirk::Other("test");
        assert_ne!(quirk1, quirk3);

        // Different Other quirks with same string are equal
        let quirk4 = RegistryQuirk::Other("test");
        let quirk5 = RegistryQuirk::Other("test");
        assert_eq!(quirk4, quirk5);
    }
}
