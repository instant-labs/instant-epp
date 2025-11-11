//! Types for EPP responses

use std::fmt::Debug;

use chrono::{DateTime, Utc};
use instant_xml::{FromXml, Kind};

use crate::common::EPP_XMLNS;
use crate::request::Extension as ExtensionTrait;
use crate::xml;

/// Type corresponding to the `<undef>` tag an EPP response XML
#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "undef", ns(EPP_XMLNS))]
pub struct Undef;

/// Type corresponding to the `<value>` tag under `<extValue>` in an EPP response XML
#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "value", ns(EPP_XMLNS))]
pub struct ResultValue {
    /// The `<undef>` element
    pub undef: Undef,
}

/// Type corresponding to the `<extValue>` tag in an EPP response XML
#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "extValue", ns(EPP_XMLNS))]
pub struct ExtValue {
    /// Data under the `<value>` tag
    pub value: ResultValue,
    /// Data under the `<reason>` tag
    pub reason: String,
}

/// Type corresponding to the `<result>` tag in an EPP response XML
#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "result", ns(EPP_XMLNS))]
pub struct EppResult {
    /// The result code
    #[xml(attribute)]
    pub code: ResultCode,
    /// The result message
    #[xml(rename = "msg")]
    pub message: String,
    /// Data under the `<extValue>` tag
    pub ext_value: Option<ExtValue>,
}

/// Response codes as enumerated in section 3 of RFC 5730
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResultCode {
    CommandCompletedSuccessfully = 1000,
    CommandCompletedSuccessfullyActionPending = 1001,
    CommandCompletedSuccessfullyNoMessages = 1300,
    CommandCompletedSuccessfullyAckToDequeue = 1301,
    CommandCompletedSuccessfullyEndingSession = 1500,
    UnknownCommand = 2000,
    CommandSyntaxError = 2001,
    CommandUseError = 2002,
    RequiredParameterMissing = 2003,
    ParameterValueRangeError = 2004,
    ParameterValueSyntaxError = 2005,
    UnimplementedProtocolVersion = 2100,
    UnimplementedCommand = 2101,
    UnimplementedOption = 2102,
    UnimplementedExtension = 2103,
    BillingFailure = 2104,
    ObjectIsNotEligibleForRenewal = 2105,
    ObjectIsNotEligibleForTransfer = 2106,
    AuthenticationError = 2200,
    AuthorizationError = 2201,
    InvalidAuthorizationInformation = 2202,
    ObjectPendingTransfer = 2300,
    ObjectNotPendingTransfer = 2301,
    ObjectExists = 2302,
    ObjectDoesNotExist = 2303,
    ObjectStatusProhibitsOperation = 2304,
    ObjectAssociationProhibitsOperation = 2305,
    ParameterValuePolicyError = 2306,
    UnimplementedObjectService = 2307,
    DataManagementPolicyViolation = 2308,
    CommandFailed = 2400,
    CommandFailedServerClosingConnection = 2500,
    AuthenticationErrorServerClosingConnection = 2501,
    SessionLimitExceededServerClosingConnection = 2502,
}

impl ResultCode {
    pub fn from_u16(code: u16) -> Option<Self> {
        match code {
            1000 => Some(Self::CommandCompletedSuccessfully),
            1001 => Some(Self::CommandCompletedSuccessfullyActionPending),
            1300 => Some(Self::CommandCompletedSuccessfullyNoMessages),
            1301 => Some(Self::CommandCompletedSuccessfullyAckToDequeue),
            1500 => Some(Self::CommandCompletedSuccessfullyEndingSession),
            2000 => Some(Self::UnknownCommand),
            2001 => Some(Self::CommandSyntaxError),
            2002 => Some(Self::CommandUseError),
            2003 => Some(Self::RequiredParameterMissing),
            2004 => Some(Self::ParameterValueRangeError),
            2005 => Some(Self::ParameterValueSyntaxError),
            2100 => Some(Self::UnimplementedProtocolVersion),
            2101 => Some(Self::UnimplementedCommand),
            2102 => Some(Self::UnimplementedOption),
            2103 => Some(Self::UnimplementedExtension),
            2104 => Some(Self::BillingFailure),
            2105 => Some(Self::ObjectIsNotEligibleForRenewal),
            2106 => Some(Self::ObjectIsNotEligibleForTransfer),
            2200 => Some(Self::AuthenticationError),
            2201 => Some(Self::AuthorizationError),
            2202 => Some(Self::InvalidAuthorizationInformation),
            2300 => Some(Self::ObjectPendingTransfer),
            2301 => Some(Self::ObjectNotPendingTransfer),
            2302 => Some(Self::ObjectExists),
            2303 => Some(Self::ObjectDoesNotExist),
            2304 => Some(Self::ObjectStatusProhibitsOperation),
            2305 => Some(Self::ObjectAssociationProhibitsOperation),
            2306 => Some(Self::ParameterValuePolicyError),
            2307 => Some(Self::UnimplementedObjectService),
            2308 => Some(Self::DataManagementPolicyViolation),
            2400 => Some(Self::CommandFailed),
            2500 => Some(Self::CommandFailedServerClosingConnection),
            2501 => Some(Self::AuthenticationErrorServerClosingConnection),
            2502 => Some(Self::SessionLimitExceededServerClosingConnection),
            _ => None,
        }
    }

    pub fn is_success(&self) -> bool {
        use ResultCode::*;
        matches!(
            self,
            CommandCompletedSuccessfully
                | CommandCompletedSuccessfullyActionPending
                | CommandCompletedSuccessfullyNoMessages
                | CommandCompletedSuccessfullyAckToDequeue
                | CommandCompletedSuccessfullyEndingSession
        )
    }

    /// Returns true if this error is likely to persist across similar requests inside the same
    /// connection or session.
    ///
    /// The same command with different arguments might succeed in some cases.
    pub fn is_persistent(&self) -> bool {
        use ResultCode::*;
        match self {
            // The same command with different arguments will result in the same error
            UnknownCommand
            | CommandSyntaxError
            | RequiredParameterMissing
            | ParameterValueRangeError
            | ParameterValueSyntaxError
            | UnimplementedProtocolVersion
            | UnimplementedCommand
            | UnimplementedOption
            | UnimplementedExtension => true,
            // The connection is in an unhealthy state
            CommandFailedServerClosingConnection
            | AuthenticationErrorServerClosingConnection
            | SessionLimitExceededServerClosingConnection => true,
            _ => false,
        }
    }
}

impl<'xml> FromXml<'xml> for ResultCode {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        let mut value = None;
        u16::deserialize(&mut value, field, deserializer)?;
        if let Some(value) = value {
            *into = match Self::from_u16(value) {
                Some(value) => Some(value),
                None => {
                    return Err(instant_xml::Error::UnexpectedValue(format!(
                        "unexpected result code '{value}'"
                    )))
                }
            };
        }

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = Kind::Scalar;
}

/// Type corresponding to the `<trID>` tag in an EPP response XML
#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "trID", ns(EPP_XMLNS))]
pub struct ResponseTRID {
    /// The client TRID
    #[xml(rename = "clTRID")]
    pub client_tr_id: Option<String>,
    /// The server TRID
    #[xml(rename = "svTRID")]
    pub server_tr_id: String,
}

/// Type corresponding to the `<msgQ>` tag in an EPP response XML
#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "msgQ", ns(EPP_XMLNS))]
pub struct MessageQueue {
    /// The message count
    #[xml(attribute)]
    pub count: u32,
    /// The message ID
    #[xml(attribute)]
    pub id: String,
    /// The message date
    #[xml(rename = "qDate")]
    pub date: Option<DateTime<Utc>>,
    /// The message text
    #[xml(rename = "msg")]
    pub message: Option<Message>,
}

#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "msg", ns(EPP_XMLNS))]
pub struct Message {
    #[xml(attribute)]
    pub lang: Option<String>,
    #[xml(direct)]
    pub text: String,
}

#[derive(Debug, FromXml, PartialEq)]
/// Type corresponding to the `<response>` tag in an EPP response XML
/// containing an `<extension>` tag
#[xml(rename = "response", ns(EPP_XMLNS))]
pub struct Response<D, E> {
    /// Data under the `<result>` tag
    pub result: EppResult,
    /// Data under the `<msgQ>` tag
    #[xml(rename = "msgQ")]
    pub message_queue: Option<MessageQueue>,
    /// Data under the `<resData>` tag
    pub res_data: Option<ResponseData<D>>,
    /// Data under the `<extension>` tag
    pub extension: Option<Extension<E>>,
    /// Data under the `<trID>` tag
    pub tr_ids: ResponseTRID,
}

#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "resData", ns(EPP_XMLNS))]
pub struct ResponseData<D> {
    data: D,
}

impl<D> ResponseData<D> {
    pub fn into_inner(self) -> D {
        self.data
    }
}

#[derive(Debug, FromXml, PartialEq)]
/// Type corresponding to the `<response>` tag in an EPP response XML
/// without `<msgQ>` or `<resData>` sections. Generally used for error handling
#[xml(rename = "response", ns(EPP_XMLNS))]
pub struct ResponseStatus {
    /// Data under the `<result>` tag
    pub result: EppResult,
    #[xml(rename = "trID")]
    /// Data under the `<trID>` tag
    pub tr_ids: ResponseTRID,
}

impl<T, E> Response<T, E> {
    /// Returns the data under the corresponding `<resData>` from the EPP XML
    pub fn res_data(&self) -> Option<&T> {
        match &self.res_data {
            Some(res_data) => Some(&res_data.data),
            None => None,
        }
    }

    pub fn extension(&self) -> Option<&E> {
        match &self.extension {
            Some(extension) => Some(&extension.data),
            None => None,
        }
    }

    /// Returns the data under the corresponding `<msgQ>` from the EPP XML
    pub fn message_queue(&self) -> Option<&MessageQueue> {
        match &self.message_queue {
            Some(queue) => Some(queue),
            None => None,
        }
    }
}

#[derive(Debug, Eq, FromXml, PartialEq)]
#[xml(rename = "extension", ns(EPP_XMLNS))]
pub struct Extension<E> {
    pub data: E,
}

/// Collection of extensions from EPP responses
///
/// `ResponseExtensionCollection` stores extension data from the `<extension>` element
/// in EPP responses. Extensions are stored as raw XML and deserialized on-demand when
/// accessed via [`get()`](Self::get), enabling support for multiple extensions without
/// compile-time knowledge of all types.
///
/// # Features
///
/// - **Lazy Deserialization**: Extensions are only parsed when requested
/// - **Type-Safe Access**: Use generic [`get()`](Self::get) method to retrieve specific extensions
/// - **Unknown Extension Support**: Gracefully handles extensions not known at compile time
/// - **Multiple Extensions**: Can store any number of extensions from the same response
///
/// # Usage
///
/// This type is returned by [`ResponseWithExtensions`] and typically accessed through
/// the [`extension()`](ResponseWithExtensions::extension) helper method.
///
/// # Examples
///
/// ```no_run
/// # use instant_epp::Error;
/// # use instant_epp::domain::DomainCheck;
/// # use instant_epp::request::ExtensionCollection;
/// # use instant_epp::extensions::namestore::NameStore;
/// # async fn example<C>(client: &mut instant_epp::client::EppClient<C>) -> Result<(), Error>
/// # where C: instant_epp::connection::Connector {
/// let check = DomainCheck { domains: &["example.com"] };
/// let extensions = ExtensionCollection::with(NameStore { subproduct: "dotCOM".into() })?;
///
/// let response = client.transact_multi(&check, extensions, "txn-001").await?;
///
/// // Type-safe access to specific extension
/// if let Some(namestore) = response.extension::<NameStore>()? {
///     println!("Namestore data: {:?}", namestore);
/// }
///
/// // Check if an extension is present
/// if response.extensions.contains::<NameStore>() {
///     println!("Namestore extension was returned");
/// }
///
/// // List all extension namespaces
/// for ns in response.extensions.namespaces() {
///     println!("Extension namespace: {}", ns);
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Default)]
pub struct ResponseExtensionCollection {
    /// Stores (namespace, xml_content) tuples
    elements: Vec<(String, String)>,
}

impl ResponseExtensionCollection {
    /// Create a new empty response extension collection
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::response::ResponseExtensionCollection;
    ///
    /// let collection = ResponseExtensionCollection::new();
    /// assert!(collection.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Create a ResponseExtensionCollection from raw XML response string
    ///
    /// Extracts all child elements from the <extension> tag and stores them
    /// as (namespace, xml_content) tuples for lazy deserialization.
    pub(crate) fn from_response_xml(xml: &str) -> Result<Self, crate::Error> {
        let mut collection = Self::new();

        // Find the <extension> element
        if let Some(ext_start) = xml.find("<extension>") {
            if let Some(ext_end) = xml.find("</extension>") {
                let extension_content = &xml[ext_start + 11..ext_end];

                // Parse child elements within <extension>
                collection.parse_extension_children(extension_content)?;
            }
        }

        Ok(collection)
    }

    /// Parse child elements from extension content
    fn parse_extension_children(&mut self, content: &str) -> Result<(), crate::Error> {
        let content = content.trim();
        let mut pos = 0;

        while pos < content.len() {
            // Skip whitespace
            while pos < content.len() && content[pos..].chars().next().unwrap().is_whitespace() {
                pos += 1;
            }

            if pos >= content.len() {
                break;
            }

            // Find next element start
            if let Some(start_pos) = content[pos..].find('<') {
                let absolute_start = pos + start_pos;

                // Skip comments and processing instructions
                if content[absolute_start..].starts_with("<!--") {
                    if let Some(comment_end) = content[absolute_start..].find("-->") {
                        pos = absolute_start + comment_end + 3;
                        continue;
                    }
                }

                if content[absolute_start..].starts_with("<?") {
                    if let Some(pi_end) = content[absolute_start..].find("?>") {
                        pos = absolute_start + pi_end + 2;
                        continue;
                    }
                }

                // Extract element name and namespace
                let after_bracket = absolute_start + 1;
                let element_def_end = content[after_bracket..]
                    .find('>')
                    .map(|p| after_bracket + p)
                    .unwrap_or(content.len());

                let element_def = &content[after_bracket..element_def_end];

                // Skip closing tags
                if element_def.starts_with('/') {
                    pos = element_def_end + 1;
                    continue;
                }

                // Extract tag name
                let tag_name = element_def
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .trim_end_matches('/');

                // Extract namespace from xmlns attribute
                let namespace = Self::extract_namespace(element_def);

                // Find matching closing tag
                let closing_tag = format!("</{}>", tag_name);
                if let Some(close_pos) = content[element_def_end..].find(&closing_tag) {
                    let absolute_close = element_def_end + close_pos + closing_tag.len();
                    let element_xml = &content[absolute_start..absolute_close];

                    if let Some(ns) = namespace {
                        self.elements.push((ns, element_xml.to_string()));
                    }

                    pos = absolute_close;
                } else {
                    // Self-closing tag
                    if element_def.ends_with('/') {
                        let element_xml = &content[absolute_start..=element_def_end];
                        if let Some(ns) = namespace {
                            self.elements.push((ns, element_xml.to_string()));
                        }
                        pos = element_def_end + 1;
                    } else {
                        pos = element_def_end + 1;
                    }
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Extract namespace from element definition
    fn extract_namespace(element_def: &str) -> Option<String> {
        // Look for xmlns attribute
        if let Some(xmlns_start) = element_def.find("xmlns=\"") {
            let value_start = xmlns_start + 7;
            if let Some(value_end) = element_def[value_start..].find('"') {
                return Some(element_def[value_start..value_start + value_end].to_string());
            }
        }

        // Look for xmlns:prefix attribute (take the first one we find)
        if let Some(xmlns_start) = element_def.find("xmlns:") {
            let after_xmlns = xmlns_start + 6;
            // Skip prefix name
            if let Some(eq_pos) = element_def[after_xmlns..].find('=') {
                let value_start = after_xmlns + eq_pos + 1;
                // Skip opening quote
                let value_start = if element_def[value_start..].starts_with('"') {
                    value_start + 1
                } else {
                    value_start
                };

                if let Some(value_end) = element_def[value_start..].find('"') {
                    return Some(element_def[value_start..value_start + value_end].to_string());
                }
            }
        }

        None
    }

    /// Add a raw XML element to the collection
    #[allow(dead_code)] // Reserved for future testing utilities
    pub(crate) fn add_raw(&mut self, namespace: String, xml: String) {
        self.elements.push((namespace, xml));
    }

    /// Get an extension by type, deserializing it on demand
    ///
    /// Searches for an extension matching the namespace of `E` and deserializes it if found.
    /// Extensions are identified by their `XMLNS` constant.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(extension))`: Extension found and successfully deserialized
    /// - `Ok(None)`: Extension not present in the response
    /// - `Err`: Deserialization failed (XML parsing error)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::response::ResponseExtensionCollection;
    /// # use instant_epp::extensions::namestore::NameStore;
    /// # fn example(collection: &ResponseExtensionCollection) -> Result<(), instant_epp::Error> {
    /// // Get a specific extension type
    /// match collection.get::<NameStore>()? {
    ///     Some(namestore) => println!("Got namestore: {:?}", namestore),
    ///     None => println!("Namestore not in response"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get<E: ExtensionTrait>(&self) -> Result<Option<E::Response>, crate::Error> {
        for (ns, xml_content) in &self.elements {
            if ns == E::XMLNS {
                let response = xml::deserialize::<E::Response>(xml_content)?;
                return Ok(Some(response));
            }
        }
        Ok(None)
    }

    /// Check if an extension of the given type is present
    ///
    /// Returns `true` if the collection contains an extension with the given type's namespace.
    /// This is cheaper than [`get()`](Self::get) as it doesn't deserialize the extension.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::response::ResponseExtensionCollection;
    /// # use instant_epp::extensions::namestore::NameStore;
    /// # fn example(collection: &ResponseExtensionCollection) {
    /// if collection.contains::<NameStore>() {
    ///     println!("Namestore extension is present");
    /// }
    /// # }
    /// ```
    pub fn contains<E: ExtensionTrait>(&self) -> bool {
        self.elements.iter().any(|(ns, _)| ns == E::XMLNS)
    }

    /// Get a list of all extension namespaces present in the response
    ///
    /// Returns a vector of namespace URIs for all extensions in this collection.
    /// Useful for debugging or discovering which extensions were returned.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::response::ResponseExtensionCollection;
    /// # fn example(collection: &ResponseExtensionCollection) {
    /// let namespaces = collection.namespaces();
    /// println!("Response contains {} extension(s)", namespaces.len());
    /// for ns in namespaces {
    ///     println!("  - {}", ns);
    /// }
    /// # }
    /// ```
    pub fn namespaces(&self) -> Vec<&str> {
        self.elements.iter().map(|(ns, _)| ns.as_str()).collect()
    }

    /// Check if the collection is empty
    ///
    /// Returns `true` if no extensions were present in the response.
    ///
    /// # Examples
    ///
    /// ```
    /// use instant_epp::response::ResponseExtensionCollection;
    ///
    /// let collection = ResponseExtensionCollection::new();
    /// assert!(collection.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get the number of extensions in the collection
    ///
    /// Returns the count of extension elements in this collection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::response::ResponseExtensionCollection;
    /// # fn example(collection: &ResponseExtensionCollection) {
    /// match collection.len() {
    ///     0 => println!("No extensions"),
    ///     1 => println!("One extension"),
    ///     n => println!("{} extensions", n),
    /// }
    /// # }
    /// ```
    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

/// Response with extension collection for multi-extension support
///
/// `ResponseWithExtensions` is returned by [`EppClient::transact_multi()`](crate::client::EppClient::transact_multi)
/// and contains the standard EPP response data plus a [`ResponseExtensionCollection`] that can
/// hold multiple extensions.
///
/// This is analogous to the regular [`Response`] type but designed for the multi-extension API.
///
/// # Fields
///
/// - `result`: The result code and message from the EPP server
/// - `message_queue`: Optional message queue information
/// - `res_data`: The command-specific response data (e.g., domain check results)
/// - `extensions`: Collection of all extension data from the response
/// - `tr_ids`: Transaction IDs (client and server)
///
/// # Examples
///
/// ```no_run
/// # use instant_epp::Error;
/// # use instant_epp::domain::DomainCheck;
/// # use instant_epp::request::ExtensionCollection;
/// # use instant_epp::extensions::namestore::NameStore;
/// # async fn example<C>(client: &mut instant_epp::client::EppClient<C>) -> Result<(), Error>
/// # where C: instant_epp::connection::Connector {
/// let check = DomainCheck { domains: &["example.com"] };
/// let extensions = ExtensionCollection::with(NameStore { subproduct: "dotCOM".into() })?;
///
/// let response = client.transact_multi(&check, extensions, "txn-001").await?;
///
/// // Access main response data
/// if let Some(data) = response.res_data() {
///     println!("Check results: {:?}", data);
/// }
///
/// // Access specific extension
/// if let Some(namestore) = response.extension::<NameStore>()? {
///     println!("Namestore: {:?}", namestore);
/// }
///
/// // Check result code
/// println!("Result: {:?}", response.result.code);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct ResponseWithExtensions<D> {
    /// Data under the `<result>` tag
    pub result: EppResult,
    /// Data under the `<msgQ>` tag
    pub message_queue: Option<MessageQueue>,
    /// Data under the `<resData>` tag
    pub res_data: Option<D>,
    /// Extension collection
    pub extensions: ResponseExtensionCollection,
    /// Data under the `<trID>` tag
    pub tr_ids: ResponseTRID,
}

impl<D> ResponseWithExtensions<D> {
    /// Returns the data under the corresponding `<resData>` from the EPP XML
    ///
    /// This is the command-specific response data (e.g., for a domain check, this would
    /// contain the availability results).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::response::ResponseWithExtensions;
    /// # fn example<D: std::fmt::Debug>(response: ResponseWithExtensions<D>) {
    /// if let Some(data) = response.res_data() {
    ///     println!("Command response data: {:?}", data);
    /// }
    /// # }
    /// ```
    pub fn res_data(&self) -> Option<&D> {
        self.res_data.as_ref()
    }

    /// Get a specific extension by type
    ///
    /// Convenience method that delegates to [`ResponseExtensionCollection::get()`].
    /// Returns the extension data if present and successfully deserialized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::response::ResponseWithExtensions;
    /// # use instant_epp::extensions::namestore::NameStore;
    /// # fn example<D>(response: &ResponseWithExtensions<D>) -> Result<(), instant_epp::Error> {
    /// match response.extension::<NameStore>()? {
    ///     Some(ns) => println!("Namestore: {:?}", ns),
    ///     None => println!("No namestore in response"),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn extension<E: ExtensionTrait>(&self) -> Result<Option<E::Response>, crate::Error> {
        self.extensions.get::<E>()
    }

    /// Returns the data under the corresponding `<msgQ>` from the EPP XML
    ///
    /// Message queue information is present when the server has messages for the client.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::response::ResponseWithExtensions;
    /// # fn example<D>(response: &ResponseWithExtensions<D>) {
    /// if let Some(msg_queue) = response.message_queue() {
    ///     println!("Messages in queue: {}", msg_queue.count);
    /// }
    /// # }
    /// ```
    pub fn message_queue(&self) -> Option<&MessageQueue> {
        self.message_queue.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use instant_xml::ToXml;

    use super::{ResponseExtensionCollection, ResponseStatus, ResultCode};
    use crate::tests::{get_xml, CLTRID, SVTRID};
    use crate::xml;

    #[test]
    fn error() {
        let xml = get_xml("response/error.xml").unwrap();
        let object = xml::deserialize::<ResponseStatus>(xml.as_str()).unwrap();

        assert_eq!(object.result.code, ResultCode::ObjectDoesNotExist);
        assert_eq!(object.result.message, "Object does not exist");
        assert_eq!(
            object.result.ext_value.unwrap().reason,
            "545 Object not found"
        );
        assert_eq!(object.tr_ids.client_tr_id.unwrap(), CLTRID);
        assert_eq!(object.tr_ids.server_tr_id, SVTRID);
    }

    #[test]
    fn test_response_extension_collection_empty() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <response>
    <result code="1000">
      <msg>Command completed successfully</msg>
    </result>
  </response>
</epp>"#;

        let collection = ResponseExtensionCollection::from_response_xml(xml).unwrap();
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
        assert_eq!(collection.namespaces().len(), 0);
    }

    #[test]
    fn test_response_extension_collection_single() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <response>
    <result code="1000">
      <msg>Command completed successfully</msg>
    </result>
    <extension>
      <namestoreExt:namestoreExt xmlns:namestoreExt="http://www.verisign-grs.com/epp/namestoreExt-1.1">
        <namestoreExt:subProduct>COM</namestoreExt:subProduct>
      </namestoreExt:namestoreExt>
    </extension>
  </response>
</epp>"#;

        let collection = ResponseExtensionCollection::from_response_xml(xml).unwrap();
        assert!(!collection.is_empty());
        assert_eq!(collection.len(), 1);
        assert_eq!(collection.namespaces().len(), 1);
        assert_eq!(
            collection.namespaces()[0],
            "http://www.verisign-grs.com/epp/namestoreExt-1.1"
        );
    }

    #[test]
    fn test_response_extension_collection_multiple() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <response>
    <result code="1000">
      <msg>Command completed successfully</msg>
    </result>
    <extension>
      <namestoreExt:namestoreExt xmlns:namestoreExt="http://www.verisign-grs.com/epp/namestoreExt-1.1">
        <namestoreExt:subProduct>COM</namestoreExt:subProduct>
      </namestoreExt:namestoreExt>
      <rgp:upData xmlns:rgp="urn:ietf:params:xml:ns:rgp-1.0">
        <rgp:rgpStatus s="pendingRestore"/>
      </rgp:upData>
    </extension>
  </response>
</epp>"#;

        let collection = ResponseExtensionCollection::from_response_xml(xml).unwrap();
        assert!(!collection.is_empty());
        assert_eq!(collection.len(), 2);
        assert_eq!(collection.namespaces().len(), 2);

        let namespaces = collection.namespaces();
        assert!(namespaces.contains(&"http://www.verisign-grs.com/epp/namestoreExt-1.1"));
        assert!(namespaces.contains(&"urn:ietf:params:xml:ns:rgp-1.0"));
    }

    #[test]
    fn test_response_extension_collection_contains() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <response>
    <extension>
      <namestoreExt:namestoreExt xmlns:namestoreExt="http://www.verisign-grs.com/epp/namestoreExt-1.1">
        <namestoreExt:subProduct>COM</namestoreExt:subProduct>
      </namestoreExt:namestoreExt>
    </extension>
  </response>
</epp>"#;

        let collection = ResponseExtensionCollection::from_response_xml(xml).unwrap();

        // Mock extension type for testing
        #[derive(Debug, ToXml)]
        struct TestExt;
        impl crate::request::Extension for TestExt {
            type Response = String;
            const XMLNS: &'static str = "http://www.verisign-grs.com/epp/namestoreExt-1.1";
        }

        assert!(collection.contains::<TestExt>());
    }
}
