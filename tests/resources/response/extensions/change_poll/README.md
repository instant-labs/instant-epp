# Change Poll Extension Response Fixtures

This directory contains poll message fixtures for the **Change Poll Extension** for EPP.

## Specification

- **RFC**: RFC 8590
- **Name**: Change Poll Extension for EPP
- **Spec File**: [`../../../../rfcs/epp-change-poll-rfc8590.txt`](../../../../rfcs/epp-change-poll-rfc8590.txt)
- **Namespace**: `urn:ietf:params:xml:ns:changePoll-1.0`

## Key Elements

Change poll messages include:

- **operation** - Type of operation (update, delete, custom, autoPurge, etc.)
  - Optional `op` attribute for variations (e.g., "purge", "sync")
- **date** - When the change occurred
- **svTRID** - Server transaction ID
- **who** - Who initiated the change
- **reason** - Reason for the change (optional)
- **caseId** - Case identifier for specific scenarios (optional)
- **state** attribute - "before" or "after" (default: "after")

## States

- **before**: Shows object state before the change
- **after**: Shows object state after the change (default)

The `<resData>` block contains the full object info reflecting the specified state.
