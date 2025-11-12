# RGP (Registry Grace Period) Extension Response Fixtures

This directory contains response fixtures for the **Registry Grace Period (RGP) Extension** for EPP.

## Specification

- **RFC**: RFC 3915
- **Name**: Domain Registry Grace Period Mapping for EPP
- **Spec File**: [`../../../../rfcs/epp-rgp-rfc3915.txt`](../../../../rfcs/epp-rgp-rfc3915.txt)
- **Namespace**: `urn:ietf:params:xml:ns:rgp-1.0`

## RGP Status Values

Responses may include:
- **addPeriod** - Domain is in the add grace period
- **autoRenewPeriod** - Domain is in the auto-renew grace period
- **renewPeriod** - Domain is in the renew grace period
- **transferPeriod** - Domain is in the transfer grace period
- **redemptionPeriod** - Domain is in the redemption grace period (can be restored)
- **pendingRestore** - Restore request has been submitted
- **pendingDelete** - Domain is pending deletion (will be purged)
