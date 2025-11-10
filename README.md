![Cover logo](./cover.svg)

# EPP client library for async Rust

[![Documentation](https://docs.rs/instant-epp/badge.svg)](https://docs.rs/instant-epp)
[![Crates.io](https://img.shields.io/crates/v/instant-epp.svg)](https://crates.io/crates/instant-epp)
[![Build status](https://github.com/InstantDomain/instant-epp/workflows/CI/badge.svg)](https://github.com/InstantDomain/instant-epp/actions?query=workflow%3ACI)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE-MIT)

## Description

instant-epp is a client library written in Rust for Internet domain registration and management
for domain registrars. We have implemented support for the following standards:

- [RFC 5730](https://tools.ietf.org/html/rfc5730) - Extensible Provisioning Protocol (EPP)
- [RFC 5731](https://tools.ietf.org/html/rfc5731) - Extensible Provisioning Protocol (EPP) Domain Name Mapping
- [RFC 5732](https://tools.ietf.org/html/rfc5732) - Extensible Provisioning Protocol (EPP) Host Mapping
- [RFC 5733](https://tools.ietf.org/html/rfc5733) - Extensible Provisioning Protocol (EPP) Contact Mapping
- [RFC 5734](https://tools.ietf.org/html/rfc5734) - Extensible Provisioning Protocol (EPP) Transport over TCP
- [RFC 3915](https://tools.ietf.org/html/rfc3915) - Domain Registry Grace Period Mapping
- [ConsoliDate mapping](https://www.verisign.com/assets/consolidate-mapping.txt)
- [Namestore Extension Mapping](https://www.verisign.com/assets/epp-sdk/verisign_epp-extension_namestoreext_v01.html)
- [Low Balance Mapping](https://www.verisign.com/assets/epp-sdk/verisign_epp-extension_low-balance_v01.html)

This library is used in production at [Instant Domain Search](https://instantdomainsearch.com/).

## History

instant-epp was originally created by [@masalachai](https://github.com/masalachai) as
[epp-client](https://github.com/masalachai/epp-client) in the summer of 2021. By fall, Instant
Domains employees started contributing to the project. In February of 2023, after most of the
contributions to epp-client had come from Instant Domains for the intervening years, we decided
to fork the project, replacing its dependency on quick-xml with
[instant-xml](https://github.com/InstantDomain/instant-xml/) in the process. Many thanks to
@masalachai for starting epp-client!

## Getting started

You can create a mut variable of type `EppClient` with the domain registry config.

```rust
use std::collections::HashMap;
use std::net::ToSocketAddrs;
use std::time::Duration;

use instant_epp::EppClient;
use instant_epp::domain::DomainCheck;
use instant_epp::common::NoExtension;

#[tokio::main]
async fn main() {
    // Create an instance of EppClient
    let timeout = Duration::from_secs(5);
    let mut client = match EppClient::connect("registry_name".to_string(), ("example.com".to_owned(), 7000), None, timeout).await {
        Ok(client) => client,
        Err(e) => panic!("Failed to create EppClient: {}",  e)
    };

    // Make a EPP Hello call to the registry
    let greeting = client.hello().await.unwrap();
    println!("{:?}", greeting);

    // Execute an EPP Command against the registry with distinct request and response objects
    let domain_check = DomainCheck { domains: &["eppdev.com", "eppdev.net"] };
    let response = client.transact(&domain_check, "transaction-id").await.unwrap();
    response
        .res_data()
        .unwrap()
        .list
        .iter()
        .for_each(|chk| println!("Domain: {}, Available: {}", chk.inner.id, chk.inner.available));
}
```

The output would look like this:

```
Domain: eppdev.com, Available: 1
Domain: eppdev.net, Available: 1
```

## Features

### 🎯 Core EPP Operations
- **Domain operations**: Check, create, renew, transfer, update, delete
- **Host operations**: Check, create, update, delete
- **Contact operations**: Check, create, update, delete
- **Session management**: Hello, login, logout, poll

### ✨ Flexible Extension System
instant-epp now supports a powerful extension system for EPP protocol extensions:

- **Multiple Extensions**: Combine multiple extensions in a single request without combinator structs
- **Extension Registry**: Track which extensions are enabled and supported per session
- **Type-Safe Access**: Generic methods for compile-time safe extension retrieval from responses
- **Validation Modes**: Configure strict, warn, or no validation for extension usage
- **Backward Compatible**: Existing single-extension API continues to work unchanged

### 🔧 Production Ready
- **Async/Await**: Built on Tokio for efficient async I/O
- **Type Safety**: Strongly-typed request/response patterns with instant-xml
- **Connection Pooling**: Session-level extension tracking supports connection pooling
- **Error Handling**: Comprehensive error types with clear messages
- **Battle Tested**: Used in production at Instant Domains

## Working with Extensions

instant-epp supports EPP extensions through a flexible system that allows combining multiple extensions easily.

### Single Extension (Legacy API - Still Supported)

```rust
use instant_epp::extensions::namestore::NameStore;
use std::borrow::Cow;

// Create extension
let namestore = NameStore { subproduct: Cow::Borrowed("dotCOM") };

// Use with command
let response = client.transact((&domain_check, &namestore), "txn-id").await?;

// Access extension response
if let Some(ext_data) = response.extension() {
    println!("Extension data: {:?}", ext_data);
}
```

### Multiple Extensions (New API - Recommended)

```rust
use instant_epp::request::ExtensionCollection;
use instant_epp::extensions::namestore::NameStore;
use instant_epp::extensions::rgp::request::{RgpRestoreRequest, Update as RgpUpdate};
use std::borrow::Cow;

// Build extension collection
let extensions = ExtensionCollection::new()
    .add(NameStore { subproduct: Cow::Borrowed("dotCOM") })?
    .add(RgpUpdate { data: RgpRestoreRequest::default() })?;

// Use with command
let response = client.transact_multi(&domain_check, extensions, "txn-id").await?;

// Type-safe access to individual extensions
if let Some(namestore) = response.extension::<NameStore>()? {
    println!("Namestore: {:?}", namestore);
}

if let Some(rgp) = response.extension::<RgpUpdate<RgpRestoreRequest>>()? {
    println!("RGP: {:?}", rgp);
}
```

### Extension Registry

```rust
// Check server capabilities
let supported = client.extensions().supported_extensions();
println!("Server supports: {:?}", supported);

// Enable extensions for session
client.extensions_mut().enable_all(vec![
    "http://www.verisign-grs.com/epp/namestoreExt-1.1",
    "urn:ietf:params:xml:ns:rgp-1.0",
]);

// Check if enabled
if client.extensions().is_enabled("urn:ietf:params:xml:ns:rgp-1.0") {
    // Safe to use RGP extension
}

// Configure validation
use instant_epp::registry::ValidationMode;
client.extensions_mut().set_validation_mode(ValidationMode::Strict);
```

### Supported Extensions

- **Namestore** (`http://www.verisign-grs.com/epp/namestoreExt-1.1`) - Verisign TLD selection
- **RGP** (`urn:ietf:params:xml:ns:rgp-1.0`) - Registry Grace Period operations
- **SecDNS** (`urn:ietf:params:xml:ns:secDNS-1.1`) - DNSSEC management
- **ConsoliDate** - Verisign consolidation date mapping
- **Low Balance** - Balance notification
- **Change Poll** - Change notification polling
- **AFNIC Extensions** - French registry specific extensions

## Documentation

- **[API Documentation](https://docs.rs/instant-epp)** - Complete API reference
- **[Migration Guide](MIGRATION_GUIDE.md)** - Guide for upgrading to the new extension API
- **[Implementation Summary](IMPLEMENTATION_SUMMARY.md)** - Technical architecture details
- **[Registry Quirks](REGISTRY_QUIRKS.md)** - Known registry-specific behaviors and workarounds
- **[Examples](examples/)** - Runnable examples demonstrating key features

## Examples

See the [examples](examples/) directory for complete, runnable examples:

- **[single_extension.rs](examples/single_extension.rs)** - Using the legacy single-extension API
- **[multiple_extensions.rs](examples/multiple_extensions.rs)** - Combining multiple extensions
- **[extension_validation.rs](examples/extension_validation.rs)** - Validation modes and error handling
- **[extension_registry.rs](examples/extension_registry.rs)** - Registry management and capabilities

Run examples with:
```bash
cargo run --example single_extension
cargo run --example multiple_extensions
```
