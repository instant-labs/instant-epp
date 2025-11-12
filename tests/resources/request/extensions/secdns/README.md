# SecDNS (DNSSEC) Extension Test Fixtures

This directory contains test fixtures for the **DNS Security Extensions (DNSSEC)** mapping for EPP.

## Specification

- **RFC**: RFC 5910
- **Name**: Domain Name System (DNS) Security Extensions Mapping for EPP
- **Spec File**: [`../../../../rfcs/epp-secdns-rfc5910.txt`](../../../../rfcs/epp-secdns-rfc5910.txt)
- **Namespace**: `urn:ietf:params:xml:ns:secDNS-1.1`

## Key Elements

The extension supports:
- **DS Data**: Delegation Signer records (keyTag, algorithm, digestType, digest)
- **Key Data**: Public key data (flags, protocol, algorithm, pubKey)
- **maxSigLife**: Maximum signature lifetime in seconds
