# RGP (Registry Grace Period) Extension Test Fixtures

This directory contains test fixtures for the **Registry Grace Period (RGP) Extension** for EPP.

## Specification

- **RFC**: RFC 3915
- **Name**: Domain Registry Grace Period Mapping for EPP
- **Spec File**: [`../../../../rfcs/epp-rgp-rfc3915.txt`](../../../../rfcs/epp-rgp-rfc3915.txt)
- **Namespace**: `urn:ietf:params:xml:ns:rgp-1.0`

## Key Grace Periods

- **Add Grace Period** - Period after domain creation
- **Renew Grace Period** - Period after domain renewal
- **Transfer Grace Period** - Period after domain transfer
- **Auto-Renew Grace Period** - Period after automatic renewal
- **Redemption Grace Period** - Period after domain deletion when restore is possible
- **Pending Delete** - Final period before domain is purged
