//! Types for EPP requests
//!
//! This module defines the core traits and types for constructing EPP command requests.
//! The main types are:
//! - [`Command`]: Trait for EPP commands (check, create, update, etc.)
//! - [`Extension`]: Trait for EPP extensions
//! - [`ExtensionCollection`]: Container for multiple extensions in a single request
//! - [`Transaction`]: Trait linking commands to extensions (deprecated)
//!
//! # Examples
//!
//! ## Single Extension (Legacy)
//! ```no_run
//! # use instant_epp::domain::DomainCheck;
//! # use instant_epp::extensions::namestore::NameStore;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # use instant_epp::client::{EppClient, RustlsConnector};
//! # let mut client: EppClient<RustlsConnector> = todo!();
//! let check = DomainCheck { domains: &["example.com"] };
//! let ext = NameStore { subproduct: "dotCOM".into() };
//! let response = client.transact((&check, &ext), "txn-001").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Multiple Extensions (New)
//! ```no_run
//! # use instant_epp::domain::DomainCheck;
//! # use instant_epp::request::ExtensionCollection;
//! # use instant_epp::extensions::namestore::NameStore;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! # use instant_epp::client::{EppClient, RustlsConnector};
//! # let mut client: EppClient<RustlsConnector> = todo!();
//! let check = DomainCheck { domains: &["example.com"] };
//! let extensions = ExtensionCollection::new()
//!     .add(NameStore { subproduct: "dotCOM".into() })?;
//! let response = client.transact_multi(&check, extensions, "txn-001").await?;
//! # Ok(())
//! # }
//! ```

use std::fmt::Debug;

use instant_xml::{FromXmlOwned, ToXml};

use crate::common::EPP_XMLNS;

pub const EPP_VERSION: &str = "1.0";
pub const EPP_LANG: &str = "en";

/// Trait to set correct value for xml tags when tags are being generated from generic types
///
/// **Note**: This trait is maintained for backward compatibility but is no longer required
/// for the new multi-extension API. Use [`ExtensionCollection`] with
/// [`EppClient::transact_multi`](crate::client::EppClient::transact_multi) instead.
pub trait Transaction<Ext: Extension>: Command + Sized {}

/// Trait for EPP commands
///
/// Implement this trait for types that represent EPP commands (check, create, update, etc.).
/// The associated `Response` type defines what data structure the command returns.
///
/// # Examples
///
/// ```
/// use instant_xml::{ToXml, FromXml};
/// use instant_epp::request::Command;
///
/// #[derive(Debug, ToXml)]
/// struct MyCommand;
///
/// #[derive(Debug, FromXml)]
/// struct MyResponse;
///
/// impl Command for MyCommand {
///     type Response = MyResponse;
///     const COMMAND: &'static str = "myCommand";
/// }
/// ```
pub trait Command: ToXml + Debug {
    type Response: FromXmlOwned + Debug;
    const COMMAND: &'static str;
}

/// Trait for EPP extensions
///
/// Implement this trait for types that represent EPP protocol extensions.
/// Extensions add additional data to commands and may return additional data in responses.
///
/// # Required Associated Items
///
/// - `Response`: The type of data returned in the response `<extension>` element
/// - `XMLNS`: The XML namespace URI that identifies this extension
/// - `NAME`: Optional human-readable name for debugging/logging
///
/// # Examples
///
/// ```
/// use instant_xml::{ToXml, FromXml};
/// use instant_epp::request::Extension;
///
/// #[derive(Debug, ToXml)]
/// struct MyExtension {
///     value: String,
/// }
///
/// #[derive(Debug, FromXml)]
/// struct MyExtensionResponse {
///     result: String,
/// }
///
/// impl Extension for MyExtension {
///     type Response = MyExtensionResponse;
///     const XMLNS: &'static str = "urn:example:xml:ns:myext-1.0";
///     const NAME: Option<&'static str> = Some("My Extension");
/// }
/// ```
pub trait Extension: ToXml + Debug {
    type Response: FromXmlOwned + Debug;
    /// The XML namespace URI for this extension
    const XMLNS: &'static str;
    /// Optional: Human-readable name for debugging
    const NAME: Option<&'static str> = None;
}

use crate::xml;

#[derive(Debug, PartialEq)]
/// Type corresponding to the `<command>` tag in an EPP XML request
/// with an `<extension>` tag
pub(crate) struct CommandWrapper<'a, D, E> {
    command: &'static str,
    /// The instance that will be used to populate the `<command>` tag
    data: &'a D,
    /// The client TRID
    extension: Option<&'a E>,
    client_tr_id: String,
}

impl<'a, E: Extension, D: Transaction<E>> CommandWrapper<'a, D, E> {
    pub(crate) fn new(data: &'a D, extension: Option<&'a E>, client_tr_id: &'a str) -> Self {
        Self {
            command: D::COMMAND,
            data,
            extension,
            client_tr_id: client_tr_id.into(),
        }
    }
}

// Specialized constructor for ExtensionCollection
impl<'a, D: Command> CommandWrapper<'a, D, ExtensionCollection> {
    pub(crate) fn new_multi(
        data: &'a D,
        extensions: Option<&'a ExtensionCollection>,
        client_tr_id: &'a str,
    ) -> Self {
        Self {
            command: D::COMMAND,
            data,
            extension: extensions,
            client_tr_id: client_tr_id.into(),
        }
    }
}

impl<D: ToXml, E: ToXml> ToXml for CommandWrapper<'_, D, E> {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("command", EPP_XMLNS)?;
        serializer.end_start()?;
        self.data.serialize(None, serializer)?;
        if let Some(extension) = self.extension {
            Ext { inner: extension }.serialize(None, serializer)?;
        }

        let id_prefix = serializer.write_start("clTRID", EPP_XMLNS)?;
        serializer.end_start()?;
        serializer.write_str(&self.client_tr_id)?;
        serializer.write_close(id_prefix, "clTRID")?;

        serializer.write_close(prefix, "command")?;
        Ok(())
    }
}

// Specialized implementation for ExtensionCollection
impl<D: ToXml> ToXml for CommandWrapper<'_, D, ExtensionCollection> {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let prefix = serializer.write_start("command", EPP_XMLNS)?;
        serializer.end_start()?;
        self.data.serialize(None, serializer)?;

        // Serialize extension collection if not empty
        if let Some(extensions) = self.extension {
            if !extensions.is_empty() {
                let ext_prefix = serializer.write_start("extension", EPP_XMLNS)?;
                serializer.end_start()?;

                // Write each extension's XML directly
                for ext in extensions.extensions() {
                    serializer.write_str(&ext.xml)?;
                }

                serializer.write_close(ext_prefix, "extension")?;
            }
        }

        let id_prefix = serializer.write_start("clTRID", EPP_XMLNS)?;
        serializer.end_start()?;
        serializer.write_str(&self.client_tr_id)?;
        serializer.write_close(id_prefix, "clTRID")?;

        serializer.write_close(prefix, "command")?;
        Ok(())
    }
}

#[derive(Debug, ToXml)]
#[xml(rename = "extension", ns(EPP_XMLNS))]
struct Ext<E> {
    inner: E,
}

/// Internal structure for storing serialized extensions
#[derive(Debug)]
pub(crate) struct SerializedExtension {
    pub(crate) xmlns: &'static str,
    #[allow(dead_code)] // Used for debugging and future logging
    pub(crate) name: Option<&'static str>,
    pub(crate) xml: String,
}

/// Collection of extensions for requests
///
/// `ExtensionCollection` allows attaching multiple EPP extensions to a single command request.
/// This is useful when a registry operation requires multiple extensions (e.g., Verisign
/// Namestore + RGP).
///
/// Extensions are serialized to XML when added to the collection using the builder pattern.
/// The collection preserves the order in which extensions are added.
///
/// # Usage
///
/// Use the builder pattern to construct a collection:
/// 1. Create with [`new()`](Self::new) or [`with()`](Self::with)
/// 2. Add extensions with [`add()`](Self::add)
/// 3. Pass to [`EppClient::transact_multi()`](crate::client::EppClient::transact_multi)
///
/// # Examples
///
/// ## Single Extension
/// ```no_run
/// # use instant_epp::request::ExtensionCollection;
/// # use instant_epp::extensions::namestore::NameStore;
/// let extensions = ExtensionCollection::with(
///     NameStore { subproduct: "dotCOM".into() }
/// )?;
/// # Ok::<(), instant_epp::Error>(())
/// ```
///
/// ## Multiple Extensions
/// ```no_run
/// # use instant_epp::request::ExtensionCollection;
/// # use instant_epp::extensions::namestore::NameStore;
/// # #[derive(Debug)]
/// # struct RgpRestore;
/// # impl instant_xml::ToXml for RgpRestore {
/// #     fn serialize<W: std::fmt::Write + ?Sized>(
/// #         &self,
/// #         _: Option<instant_xml::Id<'_>>,
/// #         _: &mut instant_xml::Serializer<W>,
/// #     ) -> Result<(), instant_xml::Error> { Ok(()) }
/// # }
/// # impl instant_epp::request::Extension for RgpRestore {
/// #     type Response = ();
/// #     const XMLNS: &'static str = "urn:ietf:params:xml:ns:rgp-1.0";
/// # }
/// let extensions = ExtensionCollection::new()
///     .add(NameStore { subproduct: "dotCOM".into() })?
///     .add(RgpRestore)?;
///
/// assert_eq!(extensions.len(), 2);
/// # Ok::<(), instant_epp::Error>(())
/// ```
///
/// ## With Client
/// ```no_run
/// # use instant_epp::Error;
/// # use instant_epp::domain::DomainUpdate;
/// # use instant_epp::request::ExtensionCollection;
/// # use instant_epp::extensions::namestore::NameStore;
/// # async fn example<C>( client: &mut instant_epp::client::EppClient<C>) -> Result<(), Error>
/// # where C: instant_epp::connection::Connector {
/// let update = DomainUpdate::new("example.com");
/// let extensions = ExtensionCollection::new()
///     .add(NameStore { subproduct: "dotCOM".into() })?;
///
/// let response = client.transact_multi(&update, extensions, "txn-123").await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ExtensionCollection {
    extensions: Vec<SerializedExtension>,
}

impl ExtensionCollection {
    /// Create a new empty extension collection
    ///
    /// Use this to start building a collection, then add extensions with [`add()`](Self::add).
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::request::ExtensionCollection;
    ///
    /// let collection = ExtensionCollection::new();
    /// assert!(collection.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    /// Add an extension to the collection (builder pattern)
    ///
    /// The extension is serialized to XML immediately. If serialization fails, returns an error.
    /// Extensions are stored in the order they are added and will be serialized in that order
    /// in the request XML.
    ///
    /// # Errors
    ///
    /// Returns an error if the extension cannot be serialized to XML.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::request::ExtensionCollection;
    /// # use instant_epp::extensions::namestore::NameStore;
    /// let collection = ExtensionCollection::new()
    ///     .add(NameStore { subproduct: "dotCOM".into() })?;
    ///
    /// assert_eq!(collection.len(), 1);
    /// # Ok::<(), instant_epp::Error>(())
    /// ```
    #[allow(clippy::should_implement_trait)] // Builder pattern, not std::ops::Add
    pub fn add<E: Extension>(mut self, ext: E) -> Result<Self, crate::Error> {
        let xml = xml::serialize(&ext)?;
        self.extensions.push(SerializedExtension {
            xmlns: E::XMLNS,
            name: E::NAME,
            xml,
        });
        Ok(self)
    }

    /// Create a new collection with a single extension
    ///
    /// Convenience method equivalent to `ExtensionCollection::new().add(ext)`.
    ///
    /// # Errors
    ///
    /// Returns an error if the extension cannot be serialized to XML.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::request::ExtensionCollection;
    /// # use instant_epp::extensions::namestore::NameStore;
    /// let collection = ExtensionCollection::with(
    ///     NameStore { subproduct: "dotCOM".into() }
    /// )?;
    /// assert_eq!(collection.len(), 1);
    /// # Ok::<(), instant_epp::Error>(())
    /// ```
    pub fn with<E: Extension>(ext: E) -> Result<Self, crate::Error> {
        Self::new().add(ext)
    }

    /// Check if the collection is empty
    ///
    /// Returns `true` if no extensions have been added to the collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::request::ExtensionCollection;
    ///
    /// let collection = ExtensionCollection::new();
    /// assert!(collection.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    /// Get the number of extensions in the collection
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::request::ExtensionCollection;
    /// # use instant_epp::common::NoExtension;
    /// let collection = ExtensionCollection::new()
    ///     .add(NoExtension)?
    ///     .add(NoExtension)?;
    /// assert_eq!(collection.len(), 2);
    /// # Ok::<(), instant_epp::Error>(())
    /// ```
    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    /// Get a reference to the internal extensions
    pub(crate) fn extensions(&self) -> &[SerializedExtension] {
        &self.extensions
    }
}

impl Default for ExtensionCollection {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::NoExtension;

    #[test]
    fn test_extension_collection_new() {
        let collection = ExtensionCollection::new();
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
    }

    #[test]
    fn test_extension_collection_add() {
        let collection = ExtensionCollection::new().add(NoExtension).unwrap();

        assert!(!collection.is_empty());
        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn test_extension_collection_with() {
        let collection = ExtensionCollection::with(NoExtension).unwrap();
        assert_eq!(collection.len(), 1);
    }

    #[test]
    fn test_extension_collection_builder_pattern() {
        let collection = ExtensionCollection::new()
            .add(NoExtension)
            .unwrap()
            .add(NoExtension)
            .unwrap();

        assert_eq!(collection.len(), 2);
    }

    #[test]
    fn test_extension_collection_extensions() {
        let collection = ExtensionCollection::new().add(NoExtension).unwrap();

        let extensions = collection.extensions();
        assert_eq!(extensions.len(), 1);
        assert_eq!(extensions[0].xmlns, "");
    }
}
