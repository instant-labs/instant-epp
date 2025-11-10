# EPP Registry Quirks and Workarounds

This document describes known quirks and unexpected behaviors in various EPP registry implementations and how to work around them using instant-epp's flexible extension system.

## Table of Contents

- [Verisign (.com, .net, etc.)](#verisign)
- [General Best Practices](#general-best-practices)
- [Configuring for Specific Registries](#configuring-for-specific-registries)

---

## Verisign

### Namestore + RGP Interaction

**Issue**: When using both the Namestore extension and RGP (Registry Grace Period) extension together, Verisign may not return the Namestore extension in the response, even if it was included in the request and the operation was successful.

**Affected Operations**:
- Domain update with RGP restore + Namestore
- Domain info with RGP status + Namestore

**Example Scenario**:
```rust
// Request includes both extensions
let extensions = ExtensionCollection::new()
    .add(NameStore { subproduct: Cow::Borrowed("dotCOM") })?
    .add(RgpUpdate { data: RgpRestoreRequest::default() })?;

let response = client.transact_multi(&update, extensions, "txn-001").await?;

// Response may only contain RGP extension, Namestore is omitted
assert!(response.extensions.contains::<RgpUpdate<RgpRestoreRequest>>());
// This may be false even though request succeeded:
// assert!(response.extensions.contains::<NameStore>());
```

**Workarounds**:

**Option 1: Don't Require Namestore in Response**

```rust
// Send both extensions
let extensions = ExtensionCollection::new()
    .add(NameStore { subproduct: Cow::Borrowed("dotCOM") })?
    .add(RgpUpdate { data: RgpRestoreRequest::default() })?;

let response = client.transact_multi(&update, extensions, "txn-001").await?;

// Check RGP response (this will be present)
if let Some(rgp) = response.extension::<RgpUpdate<RgpRestoreRequest>>()? {
    println!("RGP status: {:?}", rgp);
}

// Don't check for Namestore in response - it's implied by the success
// The command was successful, so Namestore was processed even if not returned
```

**Option 2: Use Warn Validation Mode**

```rust
// In development, use Warn mode to see what's happening
client.extensions_mut().set_validation_mode(ValidationMode::Warn);

// This will log if extensions are missing but won't fail
let response = client.transact_multi(&update, extensions, "txn-001").await?;
```

**Option 3: Conditional Extension Building**

```rust
// If you need to check Namestore data, do it in a separate command
let info = DomainInfo::new("example.com");

// First: RGP operation
let rgp_ext = ExtensionCollection::with(
    RgpUpdate { data: RgpRestoreRequest::default() }
)?;
let rgp_response = client.transact_multi(&update, rgp_ext, "txn-001").await?;

// Then: Query domain info with Namestore if you need the data
let ns_ext = ExtensionCollection::with(
    NameStore { subproduct: Cow::Borrowed("dotCOM") }
)?;
let info_response = client.transact_multi(&info, ns_ext, "txn-002").await?;

if let Some(ns) = info_response.extension::<NameStore>()? {
    println!("Namestore data: {:?}", ns);
}
```

**Recommendation**: Unless you specifically need the Namestore response data, treat it as a request-only extension when combined with RGP. The operation's success indicates that Namestore was processed correctly.

---

## General Best Practices

### 1. Extension Response Validation

Different registries have different behaviors for extension responses. Some always return extension data, others only return it conditionally.

**Defensive Pattern**:
```rust
// Always check if extension is present before accessing
if response.extensions.contains::<MyExtension>() {
    let ext_data = response.extension::<MyExtension>()?.unwrap();
    // Use extension data
} else {
    // Handle missing extension gracefully
    log::warn!("Extension not returned by registry");
}
```

### 2. Testing Against Real Registries

Extension behavior can vary significantly between registries. Always test against your target registry's sandbox/OT&E environment.

```rust
// Use Warn mode during development
#[cfg(debug_assertions)]
client.extensions_mut().set_validation_mode(ValidationMode::Warn);

// Use Strict mode in production
#[cfg(not(debug_assertions))]
client.extensions_mut().set_validation_mode(ValidationMode::Strict);
```

### 3. Checking Server Capabilities

Always check what the server actually supports:

```rust
let supported = client.extensions().supported_extensions();
log::info!("Registry supports: {:?}", supported);

// Only use extensions that are supported
if client.extensions().is_supported(MyExtension::XMLNS) {
    // Safe to enable and use
    client.extensions_mut().enable(MyExtension::XMLNS);
}
```

### 4. Handling Unknown Extensions

When processing responses with extensions you don't recognize:

```rust
// Log all extension namespaces for debugging
let namespaces = response.extensions.namespaces();
for ns in namespaces {
    log::debug!("Response contains extension: {}", ns);
}

// Safely try to access known extensions
if let Some(data) = response.extension::<KnownExtension>()? {
    // Process known extension
}

// Unknown extensions are safely ignored - they're in the collection
// but won't cause errors unless you try to deserialize them
```

---

## Configuring for Specific Registries

### Verisign (.com, .net, .cc, .tv, .name, .jobs)

```rust
// After connecting and logging in
client.extensions_mut().enable_all(vec![
    "http://www.verisign-grs.com/epp/namestoreExt-1.1",
    "urn:ietf:params:xml:ns:rgp-1.0",
    "urn:ietf:params:xml:ns:secDNS-1.1",
]);

// Use Warn mode due to Namestore + RGP quirk
client.extensions_mut().set_validation_mode(ValidationMode::Warn);
```

### Generic Registry Configuration

```rust
// Discover supported extensions from greeting
let supported = client.extensions().supported_extensions();

// Enable what you need from the supported set
for ext_uri in supported {
    if required_extensions.contains(&ext_uri) {
        client.extensions_mut().enable(ext_uri);
    }
}

// Set validation based on your risk tolerance
client.extensions_mut().set_validation_mode(
    if cfg!(debug_assertions) {
        ValidationMode::Warn  // Development
    } else {
        ValidationMode::Strict  // Production
    }
);
```

---

## Troubleshooting

### Extension Not Returned in Response

**Symptoms**: You send an extension in the request, the command succeeds, but the extension is not in the response.

**Possible Causes**:
1. Registry doesn't return that extension for that operation
2. Registry has a quirk (like Verisign Namestore + RGP)
3. Extension was processed but has no data to return

**Solutions**:
- Check if the operation typically returns that extension
- Look for registry-specific documentation
- Use `ValidationMode::Warn` to log but not fail
- Consider if you actually need the response data

### Extension Validation Errors

**Symptoms**: `Error: Extension ... is not enabled for this session`

**Possible Causes**:
1. Extension wasn't included in login
2. Extension wasn't enabled after login
3. Using Strict validation mode

**Solutions**:
```rust
// Check if extension is enabled
if !client.extensions().is_enabled(MyExtension::XMLNS) {
    // Enable it
    client.extensions_mut().enable(MyExtension::XMLNS);
}

// Or use less strict validation
client.extensions_mut().set_validation_mode(ValidationMode::Warn);
```

### Deserialization Failures

**Symptoms**: `Error: failed to deserialize extension response`

**Possible Causes**:
1. Registry returns a different XML structure than expected
2. Extension version mismatch
3. Registry-specific variation

**Solutions**:
1. Check the raw XML response (enable debug logging)
2. Verify extension namespace and version match
3. Check if the registry has custom extension variants
4. Report the issue if it's a bug in instant-epp

---

## Reporting Quirks

If you discover a registry quirk not documented here:

1. **Verify the behavior**: Test multiple times, check with registry documentation
2. **Collect evidence**: Save example XML requests/responses
3. **Document**: Note the registry, operation, and unexpected behavior
4. **Share**: Open an issue in instant-epp repository or submit a PR to this document

### Quirk Report Template

```markdown
## Registry Name

### Extension Quirk Title

**Issue**: Brief description of unexpected behavior

**Affected Operations**:
- List of affected EPP operations

**Example**:
```rust
// Code example showing the quirk
```

**Workarounds**:
- Suggested solutions

**Confirmed With**: Registry name, version, test date
```

---

## Summary

- Registry implementations vary in their extension handling
- Some registries have quirks that require workarounds
- Use `ValidationMode` appropriately for your situation
- Always test against the actual target registry
- Don't assume extension responses mirror extension requests
- Handle missing extensions gracefully
- Document any new quirks you discover

For more information:
- See `MIGRATION_GUIDE.md` for API usage
- See `IMPLEMENTATION_SUMMARY.md` for technical details
- Check registry-specific EPP documentation

