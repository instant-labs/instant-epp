use std::time::Duration;

#[cfg(feature = "__rustls")]
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tracing::{debug, error};

use crate::common::NoExtension;
pub use crate::connection::Connector;
use crate::connection::EppConnection;
use crate::error::Error;
use crate::hello::{Greeting, Hello};
use crate::registry::{ExtensionRegistry, ValidationMode};
use crate::request::{Command, CommandWrapper, Extension, ExtensionCollection, Transaction};
use crate::response::{
    Response, ResponseExtensionCollection, ResponseStatus, ResponseWithExtensions,
};
use crate::xml;

/// An `EppClient` provides an interface to sending EPP requests to a registry
///
/// Once initialized, the EppClient instance can serialize EPP requests to XML and send them
/// to the registry and deserialize the XML responses from the registry to local types.
///
/// # Examples
///
/// ```no_run
/// # use std::collections::HashMap;
/// # use std::net::ToSocketAddrs;
/// # use std::time::Duration;
/// #
/// use instant_epp::EppClient;
/// use instant_epp::domain::DomainCheck;
/// use instant_epp::common::NoExtension;
///
/// # #[cfg(feature = "rustls")]
/// # #[tokio::main]
/// # async fn main() {
/// // Create an instance of EppClient
/// let timeout = Duration::from_secs(5);
/// let mut client = match EppClient::connect("registry_name".to_string(), ("example.com".to_owned(), 7000), None, timeout).await {
///     Ok(client) => client,
///     Err(e) => panic!("Failed to create EppClient: {}",  e)
/// };
///
/// // Make a EPP Hello call to the registry
/// let greeting = client.hello().await.unwrap();
/// println!("{:?}", greeting);
///
/// // Execute an EPP Command against the registry with distinct request and response objects
/// let domain_check = DomainCheck { domains: &["eppdev.com", "eppdev.net"] };
/// let response = client.transact(&domain_check, "transaction-id").await.unwrap();
/// response
///     .res_data()
///     .unwrap()
///     .list
///     .iter()
///     .for_each(|chk| println!("Domain: {}, Available: {}", chk.inner.id, chk.inner.available));
/// # }
/// #
/// # #[cfg(not(feature = "rustls"))]
/// # fn main() {}
/// ```
///
/// The output would look like this:
///
/// ```text
/// Domain: eppdev.com, Available: 1
/// Domain: eppdev.net, Available: 1
/// ```
pub struct EppClient<C: Connector> {
    connection: EppConnection<C>,
    extension_registry: ExtensionRegistry,
}

#[cfg(feature = "__rustls")]
impl EppClient<RustlsConnector> {
    /// Connect to the specified `addr` and `hostname` over TLS
    ///
    /// The `registry` is used as a name in internal logging; `host` provides the host name
    /// and port to connect to), `hostname` is sent as the TLS server name indication and
    /// `identity` provides optional TLS client authentication (using) rustls as the TLS
    /// implementation. The `timeout` limits the time spent on any underlying network operations.
    ///
    /// Alternatively, use `EppClient::new()` with any established `AsyncRead + AsyncWrite + Unpin`
    /// implementation.
    pub async fn connect(
        registry: String,
        server: (String, u16),
        identity: Option<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>,
        timeout: Duration,
    ) -> Result<Self, Error> {
        let builder =
            RustlsConnector::builder(server).map_err(|err| Error::Other(Box::new(err)))?;
        let builder = match identity {
            Some((certs, key)) => builder.client_auth(certs, key),
            None => builder,
        };

        let connector = builder.build().map_err(|err| Error::Other(Box::new(err)))?;
        Self::new(connector, registry, timeout).await
    }
}

impl<C: Connector> EppClient<C> {
    /// Create an `EppClient` from an already established connection
    pub async fn new(connector: C, registry: String, timeout: Duration) -> Result<Self, Error> {
        let connection = EppConnection::new(connector, registry, timeout).await?;

        // Parse greeting to initialize extension registry
        let greeting = xml::deserialize::<Greeting>(&connection.greeting)?;
        let extension_registry = ExtensionRegistry::from_greeting(&greeting, ValidationMode::Warn);

        Ok(Self {
            connection,
            extension_registry,
        })
    }

    /// Get a reference to the extension registry for this session
    ///
    /// The extension registry tracks which EPP extensions are enabled for this connection
    /// and which are supported by the server. Use this to query extension state or check
    /// capabilities before making requests.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::EppClient;
    /// # async fn example<C: instant_epp::connection::Connector>(client: &EppClient<C>) {
    /// let registry = client.extensions();
    ///
    /// // Check which extensions are enabled
    /// let enabled = registry.enabled_extensions();
    /// println!("Enabled extensions: {:?}", enabled);
    ///
    /// // Check if specific extension is supported
    /// if registry.is_supported("urn:ietf:params:xml:ns:rgp-1.0") {
    ///     println!("RGP extension is available");
    /// }
    /// # }
    /// ```
    pub fn extensions(&self) -> &ExtensionRegistry {
        &self.extension_registry
    }

    /// Get a mutable reference to the extension registry
    ///
    /// Use this to modify the extension registry, such as enabling extensions after login
    /// or changing the validation mode.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use instant_epp::EppClient;
    /// # use instant_epp::registry::ValidationMode;
    /// # async fn example<C>(client: &mut EppClient<C>)
    /// # where C: instant_epp::connection::Connector {
    /// // Enable extensions after login
    /// client.extensions_mut().enable_all(vec![
    ///     "http://www.verisign-grs.com/epp/namestoreExt-1.1",
    ///     "urn:ietf:params:xml:ns:rgp-1.0",
    /// ]);
    ///
    /// // Change validation mode
    /// client.extensions_mut().set_validation_mode(ValidationMode::Strict);
    /// # }
    /// ```
    pub fn extensions_mut(&mut self) -> &mut ExtensionRegistry {
        &mut self.extension_registry
    }

    /// Executes an EPP Hello call and returns the response as a `Greeting`
    pub async fn hello(&mut self) -> Result<Greeting, Error> {
        let xml = xml::serialize(Hello)?;

        debug!("{}: hello: {}", self.connection.registry, &xml);
        let response = self.connection.transact(&xml)?.await?;
        debug!("{}: greeting: {}", self.connection.registry, &response);

        xml::deserialize::<Greeting>(&response)
    }

    pub async fn transact<'c, 'e, Cmd, Ext>(
        &mut self,
        data: impl Into<RequestData<'c, 'e, Cmd, Ext>>,
        id: &str,
    ) -> Result<Response<Cmd::Response, Ext::Response>, Error>
    where
        Cmd: Transaction<Ext> + Command + 'c,
        Ext: Extension + 'e,
    {
        let data = data.into();
        let document = CommandWrapper::new(data.command, data.extension, id);
        let xml = xml::serialize(&document)?;

        debug!("{}: request: {}", self.connection.registry, &xml);
        let response = self.connection.transact(&xml)?.await?;
        debug!("{}: response: {}", self.connection.registry, &response);

        let rsp = match xml::deserialize::<Response<Cmd::Response, Ext::Response>>(&response) {
            Ok(rsp) => rsp,
            Err(e) => {
                error!(%response, "failed to deserialize response for transaction: {e}");
                return Err(e);
            }
        };

        if rsp.result.code.is_success() {
            return Ok(rsp);
        }

        let err = crate::error::Error::Command(Box::new(ResponseStatus {
            result: rsp.result,
            tr_ids: rsp.tr_ids,
        }));

        Err(err)
    }

    /// Execute an EPP command with multiple extensions
    ///
    /// This method enables sending EPP commands with zero or more extensions in a single request.
    /// It's the recommended way to use extensions as it provides more flexibility than the legacy
    /// single-extension API.
    ///
    /// # Arguments
    ///
    /// - `command`: The EPP command to execute (check, create, update, etc.)
    /// - `extensions`: Collection of extensions to include in the request
    /// - `id`: Client transaction ID for tracking
    ///
    /// # Returns
    ///
    /// Returns [`ResponseWithExtensions`] containing:
    /// - Command response data
    /// - Extension response data (accessible via type-safe methods)
    /// - Result codes and transaction IDs
    ///
    /// # Errors
    ///
    /// - Returns error if any extension is not enabled (when validation is strict)
    /// - Returns error if command fails (non-1xxx result code)
    /// - Returns error on network or XML serialization/deserialization failures
    ///
    /// # Examples
    ///
    /// ## Single Extension
    /// ```no_run
    /// # use instant_epp::Error;
    /// # use instant_epp::domain::DomainCheck;
    /// # use instant_epp::request::ExtensionCollection;
    /// # use instant_epp::extensions::namestore::NameStore;
    /// # async fn example<C>(client: &mut instant_epp::client::EppClient<C>) -> Result<(), Error>
    /// # where C: instant_epp::connection::Connector {
    /// let check = DomainCheck { domains: &["example.com"] };
    /// let extensions = ExtensionCollection::with(
    ///     NameStore { subproduct: "dotCOM".into() }
    /// )?;
    ///
    /// let response = client.transact_multi(&check, extensions, "txn-001").await?;
    ///
    /// if let Some(data) = response.res_data() {
    ///     println!("Domain available: {}", data.list[0].inner.available);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Multiple Extensions
    /// ```no_run
    /// # use instant_epp::Error;
    /// # use instant_epp::domain::DomainUpdate;
    /// # use instant_epp::request::ExtensionCollection;
    /// # use instant_epp::extensions::namestore::NameStore;
    /// # async fn example<C>(client: &mut instant_epp::client::EppClient<C>) -> Result<(), Error>
    /// # where C: instant_epp::connection::Connector {
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
    /// let update = DomainUpdate::new("example.com");
    /// let extensions = ExtensionCollection::new()
    ///     .add(NameStore { subproduct: "dotCOM".into() })?
    ///     .add(RgpRestore)?;
    ///
    /// let response = client.transact_multi(&update, extensions, "txn-002").await?;
    ///
    /// // Access individual extensions
    /// if let Some(ns) = response.extension::<NameStore>()? {
    ///     println!("Namestore response: {:?}", ns);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn transact_multi<Cmd>(
        &mut self,
        command: &Cmd,
        extensions: ExtensionCollection,
        id: &str,
    ) -> Result<ResponseWithExtensions<Cmd::Response>, Error>
    where
        Cmd: Command,
    {
        // Validate each extension in the collection
        for ext in extensions.extensions() {
            self.extension_registry.validate(ext.xmlns)?;
        }

        let document = CommandWrapper::new_multi(command, Some(&extensions), id);
        let xml = xml::serialize(&document)?;

        debug!("{}: request: {}", self.connection.registry, &xml);
        let response = self.connection.transact(&xml)?.await?;
        debug!("{}: response: {}", self.connection.registry, &response);

        // Parse extensions from raw XML
        let extensions = ResponseExtensionCollection::from_response_xml(&response)?;

        // Deserialize the main response structure
        let rsp = match xml::deserialize::<Response<Cmd::Response, NoExtension>>(&response) {
            Ok(rsp) => rsp,
            Err(e) => {
                error!(%response, "failed to deserialize response for transaction: {e}");
                return Err(e);
            }
        };

        if !rsp.result.code.is_success() {
            return Err(crate::error::Error::Command(Box::new(ResponseStatus {
                result: rsp.result,
                tr_ids: rsp.tr_ids,
            })));
        }

        // Convert Response to ResponseWithExtensions
        Ok(ResponseWithExtensions {
            result: rsp.result,
            message_queue: rsp.message_queue,
            res_data: rsp.res_data.map(|d| d.into_inner()),
            extensions,
            tr_ids: rsp.tr_ids,
        })
    }

    /// Accepts raw EPP XML and returns the raw EPP XML response to it.
    /// Not recommended for direct use but sometimes can be useful for debugging
    pub async fn transact_xml(&mut self, xml: &str) -> Result<String, Error> {
        self.connection.transact(xml)?.await
    }

    /// Returns the greeting received on establishment of the connection in raw xml form
    pub fn xml_greeting(&self) -> String {
        String::from(&self.connection.greeting)
    }

    /// Returns the greeting received on establishment of the connection as an `Greeting`
    pub fn greeting(&self) -> Result<Greeting, Error> {
        xml::deserialize::<Greeting>(&self.connection.greeting)
    }

    pub async fn reconnect(&mut self) -> Result<(), Error> {
        self.connection.reconnect().await
    }

    pub async fn shutdown(mut self) -> Result<(), Error> {
        self.connection.shutdown().await
    }
}

#[derive(Debug)]
pub struct RequestData<'c, 'e, C, E> {
    pub(crate) command: &'c C,
    pub(crate) extension: Option<&'e E>,
}

impl<'c, C: Command> From<&'c C> for RequestData<'c, 'static, C, NoExtension> {
    fn from(command: &'c C) -> Self {
        Self {
            command,
            extension: None,
        }
    }
}

impl<'c, 'e, C: Command, E: Extension> From<(&'c C, &'e E)> for RequestData<'c, 'e, C, E> {
    fn from((command, extension): (&'c C, &'e E)) -> Self {
        Self {
            command,
            extension: Some(extension),
        }
    }
}

// Manual impl because this does not depend on whether `C` and `E` are `Clone`
impl<C, E> Clone for RequestData<'_, '_, C, E> {
    fn clone(&self) -> Self {
        *self
    }
}

// Manual impl because this does not depend on whether `C` and `E` are `Copy`
impl<C, E> Copy for RequestData<'_, '_, C, E> {}

#[cfg(feature = "__rustls")]
pub use rustls_connector::RustlsConnector;

#[cfg(feature = "__rustls")]
mod rustls_connector {
    use std::io;
    use std::sync::Arc;
    use std::time::Duration;

    use async_trait::async_trait;
    use rustls_platform_verifier::BuilderVerifierExt;
    use tokio::net::lookup_host;
    use tokio::net::TcpStream;
    use tokio_rustls::client::TlsStream;
    use tokio_rustls::rustls::pki_types::InvalidDnsNameError;
    use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
    use tokio_rustls::rustls::ClientConfig;
    use tokio_rustls::TlsConnector;
    use tracing::info;

    use crate::connection::{self, Connector};
    use crate::error::Error;

    pub struct RustlsConnector {
        inner: TlsConnector,
        server_name: ServerName<'static>,
        server: (String, u16),
    }

    impl RustlsConnector {
        /// Create a builder with the given `server` (consisting of a hostname and port)
        pub fn builder(
            server: (String, u16),
        ) -> Result<RustlsConnectorBuilder, InvalidDnsNameError> {
            Ok(RustlsConnectorBuilder {
                server_name: ServerName::try_from(server.0.as_str())?.to_owned(),
                server,
                identity: None,
            })
        }
    }

    #[async_trait]
    impl Connector for RustlsConnector {
        type Connection = TlsStream<TcpStream>;

        async fn connect(&self, timeout: Duration) -> Result<Self::Connection, Error> {
            info!("connecting to server: {}:{}", self.server.0, self.server.1);
            let addr = match lookup_host(&self.server).await?.next() {
                Some(addr) => addr,
                None => {
                    return Err(Error::Io(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("invalid host: {}", &self.server.0),
                    )))
                }
            };

            let stream = TcpStream::connect(addr).await?;
            let future = self.inner.connect(self.server_name.clone(), stream);
            connection::timeout(timeout, future).await
        }
    }

    pub struct RustlsConnectorBuilder {
        server: (String, u16),
        server_name: ServerName<'static>,
        identity: Option<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)>,
    }

    impl RustlsConnectorBuilder {
        /// Enable client authentication
        ///
        /// Only used when `build()` is called.
        pub fn client_auth(
            mut self,
            certs: Vec<CertificateDer<'static>>,
            key: PrivateKeyDer<'static>,
        ) -> Self {
            self.identity = Some((certs, key));
            self
        }

        /// Use the given `config` for the TLS connector
        ///
        /// Any client authentication set with `client_auth` will be ignored.
        pub fn build_with_config(self, config: Arc<ClientConfig>) -> RustlsConnector {
            let Self {
                server,
                server_name,
                identity: _identity,
            } = self;

            RustlsConnector {
                inner: TlsConnector::from(config),
                server_name,
                server,
            }
        }

        /// Build a new [`ClientConfig`]
        pub fn build(self) -> Result<RustlsConnector, tokio_rustls::rustls::Error> {
            let Self {
                server,
                server_name,
                identity,
            } = self;

            let builder = ClientConfig::builder().with_platform_verifier();
            let config = match identity {
                Some((certs, key)) => builder.with_client_auth_cert(certs, key)?,
                None => builder.with_no_client_auth(),
            };

            Ok(RustlsConnector {
                inner: TlsConnector::from(Arc::new(config)),
                server_name,
                server,
            })
        }
    }
}
