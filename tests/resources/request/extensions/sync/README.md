# Sync/ConsoliDate Extension Test Fixtures

This directory contains test fixtures for the **Verisign Sync/ConsoliDate Extension** for EPP.

## Specification

- **Name**: Sync Extension Mapping for EPP
- **Version**: 1.0
- **Spec File**: [`../../../../rfcs/verisign_epp-extension_sync_v01.txt`](../../../../rfcs/verisign_epp-extension_sync_v01.txt)
- **Namespace**: `http://www.verisign.com/epp/sync-1.0`

## Key Elements

The extension adds a `<sync:update>` element containing:
- `<expMonthDay>` - Target expiration date in `--MM-DD` format (e.g., `--05-31` for May 31)

The domain's expiration year is adjusted to align with the specified month and day while maintaining the appropriate registration period.
