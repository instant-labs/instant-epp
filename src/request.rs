//! Types for EPP requests

use std::fmt::Debug;

use instant_xml::{FromXmlOwned, ToXml};

use crate::common::EPP_XMLNS;

pub const EPP_VERSION: &str = "1.0";
pub const EPP_LANG: &str = "en";

/// Trait to set correct value for xml tags when tags are being generated from generic types
pub trait Transaction<Ext: Extension>: Command + Sized {}

pub trait Command: ToXml + Debug {
    type Response: FromXmlOwned + Debug;
    const COMMAND: &'static str;
}

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

#[derive(Debug, ToXml)]
#[xml(rename = "extension", ns(EPP_XMLNS))]
struct Ext<E> {
    inner: E,
}

/// Internal structure for storing serialized extensions
#[derive(Debug)]
pub(crate) struct SerializedExtension {
    pub(crate) xmlns: &'static str,
    pub(crate) name: Option<&'static str>,
    pub(crate) xml: String,
}

/// Collection of extensions for requests
///
/// Allows multiple extensions to be attached to a single EPP command.
/// Extensions are serialized when added and stored as XML strings.
///
/// # Examples
///
/// ```no_run
/// use instant_epp::request::ExtensionCollection;
/// // let ext = ExtensionCollection::new()
/// //     .add(NameStore::new("com"))
/// //     .add(RgpRequest::new());
/// ```
#[derive(Debug)]
pub struct ExtensionCollection {
    extensions: Vec<SerializedExtension>,
}

impl ExtensionCollection {
    /// Create a new empty extension collection
    pub fn new() -> Self {
        Self {
            extensions: Vec::new(),
        }
    }

    /// Add an extension to the collection (builder pattern)
    ///
    /// The extension is serialized when added. Returns an error if serialization fails.
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
    pub fn with<E: Extension>(ext: E) -> Result<Self, crate::Error> {
        Self::new().add(ext)
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }

    /// Get the number of extensions in the collection
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
