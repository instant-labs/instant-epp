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

/// Collection of extensions from responses
///
/// Stores raw XML elements and provides lazy deserialization on demand.
/// This allows handling multiple extensions without knowing all types at compile time.
#[derive(Debug, Default)]
pub struct ResponseExtensionCollection {
    /// Stores (namespace, xml_content) tuples
    elements: Vec<(String, String)>,
}

impl ResponseExtensionCollection {
    /// Create a new empty response extension collection
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Add a raw XML element to the collection
    pub(crate) fn add_raw(&mut self, namespace: String, xml: String) {
        self.elements.push((namespace, xml));
    }

    /// Get an extension by type, deserializing it on demand
    ///
    /// Returns `Ok(Some(extension))` if the extension is found and successfully deserialized,
    /// `Ok(None)` if the extension is not present, or `Err` if deserialization fails.
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
    pub fn contains<E: ExtensionTrait>(&self) -> bool {
        self.elements.iter().any(|(ns, _)| ns == E::XMLNS)
    }

    /// Get a list of all extension namespaces present in the response
    pub fn namespaces(&self) -> Vec<&str> {
        self.elements.iter().map(|(ns, _)| ns.as_str()).collect()
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Get the number of extensions in the collection
    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

/// Response with extension collection for multi-extension support
///
/// Similar to `Response` but uses `ResponseExtensionCollection` for multiple extensions.
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
    pub fn res_data(&self) -> Option<&D> {
        self.res_data.as_ref()
    }

    /// Get a specific extension by type
    pub fn extension<E: ExtensionTrait>(&self) -> Result<Option<E::Response>, crate::Error> {
        self.extensions.get::<E>()
    }

    /// Returns the data under the corresponding `<msgQ>` from the EPP XML
    pub fn message_queue(&self) -> Option<&MessageQueue> {
        self.message_queue.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::{ResponseStatus, ResultCode};
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
}
