//! Test support library for instant-epp integration tests
//!
//! This module provides utilities for testing EPP extensions with XML fixtures
//! that include source attribution comments.

use anyhow::Context;
use async_trait::async_trait;
use instant_epp::client::Connector;
use instant_epp::Error;
use regex::Regex;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::Duration;
use tokio_test::io::Builder;

pub use instant_epp::client::RequestData;
pub use instant_epp::request::{Command, Extension, Transaction};
pub use instant_epp::response::Response;
pub use instant_epp::test_helpers::{deserialize_response, serialize_request};

// Re-export similar_asserts for use in tests
pub use similar_asserts;

struct TestWriter;

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        print!(
            "{}",
            str::from_utf8(buf).expect("tried to log invalid UTF-8")
        );
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        io::stdout().flush()
    }
}

pub fn log_to_stdout() -> tracing::subscriber::DefaultGuard {
    let sub = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .with_writer(|| TestWriter)
        .finish();
    tracing::subscriber::set_default(sub)
}

pub fn len_bytes(bytes: &str) -> [u8; 4] {
    ((bytes.len() as u32) + 4).to_be_bytes()
}

/// Load an XML fixture file from the tests/resources directory and strip XML comments.
///
/// This function:
/// 1. Loads the XML file from `tests/resources/{path}`
/// 2. Strips all XML comments (`<!-- ... -->`)
/// 3. Normalizes whitespace (removes excessive whitespace)
/// 4. Formats with proper line endings (`\r\n` after XML declaration)
///
/// # Arguments
///
/// * `path` - Path relative to `tests/resources/` directory
///
/// # Returns
///
/// Processed XML string ready for serialization/deserialization tests
///
/// # Example
///
/// ```ignore
/// let xml = get_xml("request/extensions/namestore/namestore.xml").unwrap();
/// ```
pub fn xml(path: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    let ws_regex = Regex::new(r"[\s]{2,}")?;
    let end_regex = Regex::new(r"\?>")?;

    let mut f = File::open(Path::new("tests/resources").join(path))?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)?;

    if !buf.is_empty() {
        buf = strip_xml_comments(&buf);
        let mat = end_regex
            .find(buf.as_str())
            .context("couldn't find end of XML")?;
        let start = mat.end();
        buf = format!(
            "{}\r\n{}",
            &buf[..start],
            ws_regex.replace_all(&buf[start..], "")
        );
    }

    Ok(buf)
}

/// Strip XML comments from fixture content.
///
/// Removes all XML comments (`<!-- ... -->`) while preserving the rest of the XML.
/// This allows test fixtures to be self-documenting with source attribution,
/// descriptions, and notes without affecting serialization/deserialization tests.
///
/// The function handles:
/// - Single-line comments: `<!-- comment -->`
/// - Multi-line comments:
///   ```xml
///   <!-- This is a
///        multi-line comment -->
///   ```
/// - Multiple comments in the same file
///
/// # Arguments
///
/// * `xml` - XML string that may contain comments
///
/// # Returns
///
/// XML string with all comments removed
pub fn strip_xml_comments(xml: &str) -> String {
    // Regex to match XML comments including surrounding whitespace/newlines
    // The (?s) flag enables dot-all mode so . matches newlines
    let comment_regex = Regex::new(r"(?s)<!--.*?-->\s*").unwrap();
    comment_regex.replace_all(xml, "").trim_start().to_string()
}

/// Assert that a command with optional extension serializes to match the expected fixture.
///
/// This function:
/// 1. Loads the fixture from the given path
/// 2. Strips XML comments from the fixture
/// 3. Serializes the command and extension
/// 4. Compares the serialized output with the expected fixture
///
/// # Arguments
///
/// * `path` - Path to the fixture file relative to `tests/resources/`
/// * `req` - The request data (command and optional extension)
///
/// # Panics
///
/// Panics if the serialized output doesn't match the expected fixture.
///
/// # Example
///
/// ```ignore
/// let check = DomainCheck { domains: &["example.com"] };
/// let ext = NameStore { subproduct: Cow::Borrowed("com") };
/// assert_serialized("request/extensions/namestore/namestore.xml", (&check, &ext));
/// ```
pub fn assert_serialized<'c, 'e, Cmd, Ext>(
    path: impl AsRef<Path>,
    req: impl Into<RequestData<'c, 'e, Cmd, Ext>>,
) where
    Cmd: Transaction<Ext> + Command + 'c,
    Ext: Extension + 'e,
{
    let expected = xml(path).unwrap();
    let actual = serialize_request(req).unwrap();
    similar_asserts::assert_eq!(expected, actual);
}

/// Deserialize a response with extension from a fixture file.
///
/// This function:
/// 1. Loads the fixture from the given path
/// 2. Strips XML comments from the fixture
/// 3. Deserializes the XML to the expected response type
/// 4. Verifies the response indicates success
///
/// # Arguments
///
/// * `path` - Path to the fixture file relative to `tests/resources/`
///
/// # Returns
///
/// The deserialized response
///
/// # Panics
///
/// Panics if deserialization fails or the response doesn't indicate success.
///
/// # Example
///
/// ```ignore
/// let response = response_from_file::<DomainCheck, NameStore>(
///     "response/extensions/namestore/namestore.xml",
/// );
/// assert_eq!(response.result.code, ResultCode::CommandCompletedSuccessfully);
/// ```
pub fn response_from_file<Cmd, Ext>(
    path: impl AsRef<Path>,
) -> Response<<Cmd as Command>::Response, <Ext as Extension>::Response>
where
    Cmd: Transaction<instant_epp::common::NoExtension> + Command,
    Ext: Extension,
{
    let xml_content = xml(path).unwrap();
    let rsp = deserialize_response::<Response<Cmd::Response, Ext::Response>>(&xml_content).unwrap();
    assert!(rsp.result.code.is_success());
    rsp
}

pub struct MockConnector<'a> {
    sequence: Vec<&'a Path>,
}

impl<'a> MockConnector<'a> {
    pub fn new<P: AsRef<Path>>(sequence: &'a [P]) -> Self {
        Self {
            sequence: sequence.into_iter().map(|p| p.as_ref()).collect(),
        }
    }
}

#[async_trait]
impl<'a> Connector for MockConnector<'a> {
    type Connection = tokio_test::io::Mock;

    async fn connect(&self, _: Duration) -> Result<Self::Connection, Error> {
        Ok(build_stream(&self.sequence).build())
    }
}

fn build_stream(units: &[&Path]) -> Builder {
    let mut builder = Builder::new();
    for (i, path) in units.iter().enumerate() {
        let buf = xml(path).unwrap();
        match i % 2 {
            0 => builder.read(&len_bytes(&buf)).read(buf.as_bytes()),
            1 => builder.write(&len_bytes(&buf)).write(buf.as_bytes()),
            _ => unreachable!(),
        };
    }

    builder
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{strip_xml_comments, xml};

    #[test]
    fn test_strip_xml_comments_single_line() {
        let xml = r#"<!-- This is a comment -->
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <command>...</command>
</epp>"#;

        let result = strip_xml_comments(xml);
        assert!(!result.contains("<!--"));
        assert!(!result.contains("-->"));
        assert!(result.contains("<epp"));
        assert!(result.contains("<command>"));
    }

    #[test]
    fn test_strip_xml_comments_multi_line() {
        let xml = r#"<!--
Source: RFC 5910
Description: Test fixture
-->
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <!-- Another comment -->
  <command>...</command>
</epp>"#;

        let result = strip_xml_comments(xml);
        assert!(!result.contains("<!--"));
        assert!(!result.contains("-->"));
        assert!(!result.contains("RFC 5910"));
        assert!(result.contains("<epp"));
        assert!(result.contains("<command>"));
    }

    #[test]
    fn test_strip_xml_comments_multiple_comments() {
        let xml = r#"<!-- Comment 1 -->
<epp>
  <!-- Comment 2 -->
  <command>
    <!-- Comment 3 -->
    <data>test</data>
  </command>
</epp>
<!-- Comment 4 -->"#;

        let result = strip_xml_comments(xml);
        assert!(!result.contains("<!--"));
        assert!(!result.contains("-->"));
        assert!(!result.contains("Comment"));
        assert!(result.contains("<epp>"));
        assert!(result.contains("<data>test</data>"));
    }

    #[test]
    fn test_strip_xml_comments_no_comments() {
        let xml = r#"<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  <command>...</command>
</epp>"#;

        let result = strip_xml_comments(xml);
        assert_eq!(result, xml);
    }

    #[test]
    fn test_strip_xml_comments_empty_string() {
        let xml = "";
        let result = strip_xml_comments(xml);
        assert_eq!(result, "");
    }

    #[test]
    fn test_get_xml_loads_existing_fixture() {
        // Test with an existing fixture that doesn't have comments
        let result = xml(Path::new("request/hello.xml"));
        assert!(result.is_ok());
        let xml = result.unwrap();
        assert!(xml.contains("<hello"));
    }
}
