# Integration Test Architecture

This document describes the architecture of the integration test infrastructure for instant-epp, particularly for testing EPP extensions with self-documenting XML fixtures.

## Overview

The integration test system is built around a **support crate** that provides reusable utilities for:
- Loading XML fixtures with automatic comment stripping
- Serialization testing (Rust → XML)
- Deserialization testing (XML → Rust)
- Test assertions with detailed diffs

## Directory Structure

```
tests/
├── support/                    # Test support crate
│   ├── mod.rs                 # Main support module
│   └── README.md              # Support crate documentation
├── verisign_extensions.rs      # Verisign extension tests
├── basic.rs                    # Basic EPP tests
└── resources/                  # XML fixture files
    ├── request/
    │   └── extensions/        # Extension request fixtures
    └── response/
        └── extensions/        # Extension response fixtures
```

## Architecture Components

### 1. Support Crate (`tests/support/`)

The support crate is a test-only module that provides:

#### Core Functions

- **`get_xml(path)`** - Loads XML fixtures and strips comments
- **`strip_xml_comments(xml)`** - Removes XML comments from strings
- **`assert_serialized(path, request)`** - Serialization testing
- **`response_from_file<Cmd, Ext>(path)`** - Deserialization testing

#### Re-exports

The support module re-exports commonly needed types:
- `RequestData` - For building EPP requests
- `Command`, `Extension`, `Transaction` - EPP traits
- `Response` - EPP response type
- `serialize_request`, `deserialize_response` - From `instant_epp::test_helpers`
- `similar_asserts` - For better test output

### 2. Test Helpers in Main Library (`src/lib.rs`)

The main library exposes a `test_helpers` module for use by integration tests:

```rust
pub mod test_helpers {
    pub const CLTRID: &str = "cltrid:1626454866";
    pub fn serialize_request<'c, 'e, Cmd, Ext>(...) -> Result<String, Error>
    pub fn deserialize_response<T: FromXmlOwned>(...) -> Result<T, Error>
}
```

This module bridges the gap between the library's internal serialization
and the integration tests.

### 3. Self-Documenting XML Fixtures

XML fixtures include source attribution comments:

```xml
<!-- Source: Verisign EPP SDK - Namestore Extension Mapping v1.1 -->
<!-- URL: https://www.verisign.com/assets/epp-sdk/verisign_epp-extension_namestoreext_v01.html -->
<!-- Description: Domain check with namestore extension -->
<!-- Notes: The subProduct field specifies the TLD (e.g., "com", "net", "tv") -->
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<epp xmlns="urn:ietf:params:xml:ns:epp-1.0">
  ...
</epp>
```

Comments are automatically stripped during test execution, so they don't affect test results.

## Test Flow

### Serialization Test Flow

```
1. Test calls: assert_serialized("request/extensions/namestore.xml", (&cmd, &ext))
2. Support loads fixture and strips XML comments
3. Support calls test_helpers::serialize_request(cmd, ext)
4. test_helpers creates CommandWrapper and calls xml::serialize
5. Result is compared with expected fixture using similar_asserts
```

### Deserialization Test Flow

```
1. Test calls: response_from_file::<DomainCheck, NameStore>("response/...")
2. Support loads fixture and strips XML comments
3. Support calls test_helpers::deserialize_response(xml)
4. test_helpers calls xml::deserialize
5. Response is validated and returned to test
```

## Benefits of This Architecture

### 1. Separation of Concerns
- **Library code** (`src/`) - Core EPP functionality
- **Test helpers** (`src/lib.rs::test_helpers`) - Bridge to internal APIs
- **Support crate** (`tests/support/`) - Test utilities
- **Integration tests** (`tests/*.rs`) - Actual test cases

### 2. Reusability
- Support crate can be used by any integration test
- Helper functions work with any Command/Extension combination
- Same infrastructure for all EPP extensions

### 3. Self-Documenting Fixtures
- XML comments provide context without affecting tests
- Source attribution makes maintenance easier
- Description and notes help understand test cases

### 4. Maintainability
- Comment stripping is centralized
- Test utilities are in one place
- Adding new tests is straightforward

## Adding New Tests

### 1. Create XML Fixtures

Create request and response fixtures in `tests/resources/`:

```bash
tests/resources/request/extensions/my_extension.xml
tests/resources/response/extensions/my_extension.xml
```

Add source comments:

```xml
<!-- Source: RFC XXXX Section Y.Z -->
<!-- Description: ... -->
<!-- Notes: ... -->
```

### 2. Write Test

```rust
use support::{assert_serialized, response_from_file};

#[test]
fn my_extension_test() {
    let cmd = MyCommand { ... };
    let ext = MyExtension { ... };
    
    // Test serialization
    assert_serialized("request/extensions/my_extension.xml", (&cmd, &ext));
    
    // Test deserialization
    let response = response_from_file::<MyCommand, MyExtension>(
        "response/extensions/my_extension.xml"
    );
    
    // Assert response data
    assert_eq!(response.result.code, ResultCode::CommandCompletedSuccessfully);
    // ...
}
```

## Unit Tests vs Integration Tests

### Unit Tests (`src/tests/`)
- Test individual modules and functions
- Use internal test helper `src/tests/mod.rs::assert_serialized`
- Can access private module APIs
- Also support comment stripping as of this architecture update

### Integration Tests (`tests/`)
- Test public APIs and extension combinations
- Use `tests/support/` crate
- Only access public library APIs
- Test realistic usage scenarios

## Performance Considerations

- XML comments are stripped once per fixture load
- Fixtures are loaded fresh for each test (no caching)
- Regex compilation is done once (in function definitions)
- Tests run in parallel by default

## Future Enhancements

Possible improvements to this architecture:

1. **Fixture caching** - Cache loaded/processed XML to speed up tests
2. **Validation** - Validate fixtures against EPP XML schemas
3. **Generation** - Generate fixtures from Rust structs
4. **Coverage reporting** - Track which fixtures cover which code paths
5. **Fixture library** - Share common fixture components

## References

- `tests/support/README.md` - Detailed support crate documentation
- `tests/verisign_extensions.rs` - Example integration test usage
- `.kiro/specs/verisign-extension-tests/` - Test specification documents

