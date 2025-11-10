# Migration Guide: Flexible EPP Extensions

## Overview

This guide helps you migrate from the old single-extension API to the new flexible multi-extension system in instant-epp.

## Quick Start

### TL;DR for Existing Users

**Nothing breaks!** Your existing code works as-is. When you're ready for multiple extensions:

```rust
use instant_epp::request::ExtensionCollection;

let extensions = ExtensionCollection::new()
    .add(NameStore { subproduct: "dotCOM" })?
    .add(RgpRestoreRequest::new())?;

let response = client.transact_multi(&command, extensions, "txn-id").await?;

// Access extensions by type
if let Some(ns) = response.extension::<NameStore>()? {
    println!("Namestore: {:?}", ns);
}
```

**That's it!** Read on for details and migration strategies.

---

## What's Changed?

### New Features

- ✨ **Multiple extensions in one request** - No more combinator structs needed
- ✨ **Extension registry** - Track which extensions are enabled per session
- ✨ **Validation modes** - Configure how strictly to validate extension usage
- ✨ **Type-safe access** - Get extensions from responses with compile-time safety

### Backward Compatibility

- ✅ **No breaking changes** - All existing code continues to work
- ✅ **Old API preserved** - `transact()` with single extensions still works
- ✅ **Gradual migration** - Migrate at your own pace

## API Changes

### Old API (Still Works)

```rust
use instant_epp::{EppClient, domain::DomainCheck};
use instant_epp::extensions::namestore::NameStore;

// Single extension
let check = DomainCheck { domains: &["example.com"] };
let ext = NameStore::new("com");
let response = client.transact((&check, &ext), "txn-001").await?;

// Access extension response
if let Some(ext_response) = response.extension() {
    println!("Subproduct: {}", ext_response.subproduct);
}
```

### New API (Recommended)

```rust
use instant_epp::{EppClient, domain::DomainCheck};
use instant_epp::extensions::namestore::NameStore;
use instant_epp::request::ExtensionCollection;

// Multiple extensions
let check = DomainCheck { domains: &["example.com"] };
let extensions = ExtensionCollection::new()
    .add(NameStore::new("com"))?;  // Note: add() returns Result now

let response = client.transact_multi(&check, extensions, "txn-001").await?;

// Access extension from collection
if let Some(ext_response) = response.extension::<NameStore>()? {
    println!("Subproduct: {}", ext_response.subproduct);
}
```

## Common Migration Scenarios

### Scenario 1: Single Extension

**Before:**

```rust
let domain_check = DomainCheck { domains: &["example.com"] };
let namestore = NameStore::new("com");
let response = client.transact((&domain_check, &namestore), "txn-001").await?;
```

**After (Option A - Keep using old API):**

```rust
// No changes needed - old API still works!
let domain_check = DomainCheck { domains: &["example.com"] };
let namestore = NameStore::new("com");
let response = client.transact((&domain_check, &namestore), "txn-001").await?;
```

**After (Option B - Use new API):**

```rust
let domain_check = DomainCheck { domains: &["example.com"] };
let extensions = ExtensionCollection::new()
    .add(NameStore::new("com"))?;
let response = client.transact_multi(&domain_check, extensions, "txn-001").await?;
```

### Scenario 2: Multiple Extensions (Previously Required Combinators)

**Before (Required custom combinator struct):**

```rust
// Had to create a custom struct combining Namestore + RGP
// or use UpdateWithNameStore combinator
use instant_epp::extensions::consolidate::UpdateWithNameStore;

let exp = GMonthDay::new(5, 31, None)?;
let update = DomainUpdate::new("example.com");
let ext = UpdateWithNameStore::new(exp, "com");
let response = client.transact((&update, &ext), "txn-001").await?;
```

**After (Compose freely):**

```rust
use instant_epp::extensions::namestore::NameStore;
use instant_epp::extensions::consolidate::Update;

let update = DomainUpdate::new("example.com");
let extensions = ExtensionCollection::new()
    .add(NameStore::new("com"))?
    .add(Update::new(GMonthDay::new(5, 31, None)?))?;

let response = client.transact_multi(&update, extensions, "txn-001").await?;
```

### Scenario 3: Extension Registry Management

**New capability - Check extension support:**

```rust
// After connecting, check what the server supports
let supported = client.extensions().supported_extensions();
println!("Server supports: {:?}", supported);

// Check if a specific extension is supported
if client.extensions().is_supported("http://www.verisign-grs.com/epp/namestoreExt-1.1") {
    println!("Namestore extension is available");
}
```

**New capability - Enable extensions for session:**

```rust
// Enable an extension (typically done after login)
client.extensions_mut().enable("http://www.verisign-grs.com/epp/namestoreExt-1.1");

// Or enable multiple at once
client.extensions_mut().enable_all(vec![
    "http://www.verisign-grs.com/epp/namestoreExt-1.1",
    "urn:ietf:params:xml:ns:rgp-1.0",
]);
```

### Scenario 4: Validation Mode Configuration

**Default behavior (Warn mode):**

```rust
// By default, using an extension that's not enabled will log a warning
// but won't fail the request
let extensions = ExtensionCollection::new()
    .add(NameStore::new("com"))?;
let response = client.transact_multi(&check, extensions, "txn-001").await?;
// Works, but logs warning if Namestore wasn't enabled
```

**Strict mode (Recommended for production):**

```rust
use instant_epp::registry::ValidationMode;

// Set strict mode - error if extension not enabled
client.extensions_mut().set_validation_mode(ValidationMode::Strict);

// Now this will return an error if Namestore wasn't enabled
let extensions = ExtensionCollection::new()
    .add(NameStore::new("com"))?;
let response = client.transact_multi(&check, extensions, "txn-001").await?;
// Returns Err if extension not enabled
```

**No validation (For testing/debugging):**

```rust
// Disable validation entirely
client.extensions_mut().set_validation_mode(ValidationMode::None);

// No validation performed
let extensions = ExtensionCollection::new()
    .add(NameStore::new("com"))?;
let response = client.transact_multi(&check, extensions, "txn-001").await?;
// Always succeeds (unless server rejects it)
```

## Error Handling Changes

### Old API

```rust
match client.transact((&check, &ext), "txn-001").await {
    Ok(response) => { /* handle success */ },
    Err(Error::Command(status)) => { /* EPP protocol error */ },
    Err(e) => { /* other error */ },
}
```

### New API

```rust
use instant_epp::request::ExtensionCollection;

// add() returns Result now
let extensions = match ExtensionCollection::new().add(NameStore::new("com")) {
    Ok(ext) => ext,
    Err(e) => return Err(e), // Serialization error
};

// Validation errors possible
match client.transact_multi(&check, extensions, "txn-001").await {
    Ok(response) => { /* handle success */ },
    Err(Error::Command(status)) => { /* EPP protocol error */ },
    Err(Error::Other(msg)) if msg.contains("not enabled") => {
        /* Extension validation error */
    },
    Err(e) => { /* other error */ },
}
```

## Step-by-Step Migration Path

### Phase 1: No Changes Required

- ✅ Your existing code continues to work
- ✅ No immediate action needed
- ✅ Test that everything still compiles and runs

### Phase 2: Add Extension Registry Awareness (Optional)

```rust
// After connecting, log what's supported
let supported = client.extensions().supported_extensions();
tracing::info!("EPP server supports extensions: {:?}", supported);

// After login, enable the extensions you negotiated
client.extensions_mut().enable_all(vec![
    "http://www.verisign-grs.com/epp/namestoreExt-1.1",
    // ... other extensions from login
]);
```

### Phase 3: Migrate to Multi-Extension API (Optional)

- Replace `transact()` calls with `transact_multi()`
- Build `ExtensionCollection` instead of using tuples
- Handle the `Result` from `add()` method
- Update error handling if using strict validation

### Phase 4: Enable Strict Validation (Recommended for Production)

```rust
// In production, use strict mode
if cfg!(not(debug_assertions)) {
    client.extensions_mut().set_validation_mode(ValidationMode::Strict);
}
```

## Best Practices

### 1. Use Extension Registry After Login

```rust
// After successful login, update the registry
let login_response = client.transact(&login_request, "login-001").await?;

// Enable extensions that were negotiated
if let Some(extensions) = login_request.extensions_list() {
    client.extensions_mut().enable_all(extensions);
}
```

### 2. Check Support Before Using Extensions

```rust
// Check before using
if !client.extensions().is_supported(NameStore::XMLNS) {
    return Err("Namestore extension not supported by server");
}

let extensions = ExtensionCollection::new()
    .add(NameStore::new("com"))?;
```

### 3. Handle Serialization Errors

```rust
// add() can fail if serialization fails
let extensions = ExtensionCollection::new()
    .add(ext1)?  // Handle error
    .add(ext2)?  // Handle error
    .add(ext3)?; // Handle error
```

### 4. Use Appropriate Validation Mode

```rust
// Development: Warn mode (default)
// - Logs warnings but doesn't fail
// - Good for discovering issues

// Production: Strict mode
// - Fails if extension not enabled
// - Prevents protocol violations

// Testing: None mode
// - No validation
// - For testing edge cases
```

## Troubleshooting

### Q: My code doesn't compile after updating

**A**: The new code is fully backward compatible. If it doesn't compile, you may have a dependency issue. Try `cargo clean && cargo build`.

### Q: I get "Extension not enabled" errors

**A**: You're in Strict validation mode. Either:

1. Enable the extension: `client.extensions_mut().enable(XMLNS)`
2. Use Warn mode: `client.extensions_mut().set_validation_mode(ValidationMode::Warn)`
3. Use the old `transact()` API which doesn't validate

### Q: How do I know which extensions the server supports?

**A**: Check the extension registry:

```rust
let supported = client.extensions().supported_extensions();
println!("Supported: {:?}", supported);
```

### Q: Can I mix old and new APIs?

**A**: Yes! Use `transact()` for single extensions and `transact_multi()` for multiple extensions.

### Q: Do I need to migrate immediately?

**A**: No. The old API is not deprecated. Migrate when it makes sense for your use case.

## Summary

| Feature | Old API | New API |
|---------|---------|---------|
| Single extension | ✅ `transact((&cmd, &ext), id)` | ✅ `transact_multi(cmd, ext_col, id)` |
| Multiple extensions | ❌ Requires combinators | ✅ `ExtensionCollection` |
| Extension tracking | ❌ Not available | ✅ `ExtensionRegistry` |
| Validation | ❌ Not available | ✅ Configurable modes |
| Type safety | ✅ Compile-time | ✅ Compile-time |
| Breaking changes | N/A | ❌ None |

## Documentation Resources

### API Documentation

All new types and methods have comprehensive inline documentation:

```rust
// Use `cargo doc --open` to view full API docs
// Or check your IDE's hover tooltips

use instant_epp::registry::ExtensionRegistry;  // Hover for docs
use instant_epp::request::ExtensionCollection;  // Hover for docs
use instant_epp::response::ResponseExtensionCollection;  // Hover for docs
```

### Additional Resources

- **`IMPLEMENTATION_SUMMARY.md`** - Technical architecture and implementation details
- **`STATUS.md`** - Project status and what's been completed
- **`COMPLETION_REPORT.md`** - Full completion report with metrics
- **API docs** - Run `cargo doc --open` for complete API reference
- **This guide** - Migration examples and best practices

### Finding Information

1. **Type documentation**: Hover over types in your IDE or check `cargo doc`
2. **Method documentation**: All public methods have usage examples
3. **Extension examples**: See module-level docs in `src/registry.rs`, `src/request.rs`, `src/response.rs`
4. **Integration examples**: Check the examples in this guide

## Need Help?

### Common Questions

**Q: Where do I start?**  
A: No changes needed immediately. When you need multiple extensions, use `ExtensionCollection` and `transact_multi()`.

**Q: How do I discover available extensions?**  
A: Check `client.extensions().supported_extensions()` after connecting.

**Q: What if my extension isn't in the registry?**  
A: Set validation mode to `None` or `Warn`, or manually enable it with `client.extensions_mut().enable(xmlns)`.

**Q: Can I still use the old API?**  
A: Absolutely! It's not deprecated and will continue to work.

### Getting Support

- Review inline API documentation (`cargo doc --open`)
- Check examples in this migration guide
- Look at `IMPLEMENTATION_SUMMARY.md` for technical details
- See existing extension implementations in `src/extensions/`

The new API is designed to be intuitive and safe while maintaining full backward compatibility. Happy coding!
