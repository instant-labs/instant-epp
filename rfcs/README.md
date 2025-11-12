# EPP RFCs and Specifications

This directory contains RFCs and specifications related to the Extensible Provisioning Protocol (EPP) and its extensions.

## Core EPP Protocol RFCs (STD 69)

✅ **Present in this directory:**

- **RFC 5730** (`epp-rfc5730.txt`) - Extensible Provisioning Protocol (EPP) - Core Protocol
- **RFC 5731** (`epp-domain-rfc5731.txt`) - EPP Domain Name Mapping
- **RFC 5732** (`epp-host-rfc5732.txt`) - EPP Host Mapping
- **RFC 5733** (`epp-contact-rfc5733.txt`) - EPP Contact Mapping
- **RFC 5734** (`epp-tcp-rfc5734.txt`) - EPP Transport over TCP

## EPP Extension RFCs

✅ **Present in this directory:**

### Grace Period & Lifecycle

- **RFC 3915** (`epp-rgp-rfc3915.txt`) - Domain Registry Grace Period Mapping
- **RFC 9154** (`secure-authinfo-transfer-rfc9154.txt`) - Secure Authorization Information for Transfers
- **RFC 9167** (`registry-maintenance-rfc9167.txt`) - Registry Maintenance Notification

### Security & DNSSEC

- **RFC 4310** (`epp-secdns-rfc4310.txt`) - Domain Name System (DNS) Security Extensions Mapping (obsoleted by RFC 5910)
- **RFC 5910** (`epp-secdns-rfc5910.txt`) - Domain Name System (DNS) Security Extensions Mapping
- **RFC 8807** (`epp-login-security-rfc8807.txt`) - Login Security Extension

### Domain Lifecycle & Changes

- **RFC 8334** (`epp-launchphase-rfc8334.txt`) - Launch Phase Mapping
- **RFC 8495** (`epp-allocation-token-rfc8495.txt`) - Allocation Token Extension
- **RFC 8590** (`epp-change-poll-rfc8590.txt`) - Change Poll Extension
- **RFC 9038** (`epp-unhandled-namespaces-rfc9038.txt`) - Unhandled Namespaces

### Fees & Organizations

- **RFC 8748** (`epp-fee-rfc8748.txt`) - Fee Extension
- **RFC 8543** (`epp-org-mapping-rfc8543.txt`) - Organization Extension Mapping
- **RFC 8544** (`epp-org-ext-rfc8544.txt`) - Organization Extension for Contact Objects

### Trademark & Brand Protection

- **RFC 7848** (`tmch-smd-rfc7848.txt`) - Mark and Signed Mark Objects Mapping (TMCH)

## Verisign-Specific Extensions

✅ **Present in this directory:**

- `verisign_epp-extension_namestoreext_v01.txt` - Namestore Extension (Multi-TLD management)
- `verisign_epp-extension_sync_v01.txt` - Sync/ConsoliDate Extension
- `verisign_epp-extension_low-balance_v01.txt` - Low Balance Notification
- `verisign_epp-extension_balance_v01.txt` - Balance Extension
- `verisign_epp-extension_idn-lang_v01.txt` - IDN Language Extension
- `verisign_epp-extension_registry_v01.txt` - Registry Extension
- `verisign_epp-extension_related-domain_v01.txt` - Related Domain Extension
- `verisign_epp-extension_rgp-poll_v01.txt` - RGP Poll Extension
- `verisign_epp-extension_whois-info_v01.txt` - WHOIS Info Extension
- `verisign_epp-extension_coa_v01.txt` - Change of Registrant (CoA) Extension
- `EPP-DefReg-Mapping.pdf` - Defensive Registration Mapping
- `EPP-EmailFwd-Mapping.pdf` - Email Forwarding Mapping
- `EPP-NameWatch-Mapping.pdf` - NameWatch Mapping
- `EPP-PersReg-Mapping.pdf` - Personal Registration Mapping

## Draft Specifications

✅ **Present in this directory:**

- `draft-gould-carney-regext-registry-04.txt` - Registry Mapping
- `draft-gould-idn-table.txt` - IDN Table Extension
- `draft-gould-regext-launch-policy-01.txt` - Launch Policy Extension
- `draft-gould-regext-login-security-policy-03.txt` - Login Security Policy
- `draft-ietf-eppext-idnmap.txt` - IDN Mapping Extension
- `draft-ietf-regext-epp-eai-22.txt` - Email Address Internationalization
- `draft-ietf-regext-epp-ttl-16.txt` - TTL Mapping
- `draft-ietf-regext-nv-mapping.txt` - Name and Version Extension
- `draft-ietf-regext-validate-04.txt` - Domain Name Validation Extension
- `draft-ietf-regext-verificationcode.txt` - Verification Code Extension
- `draft-loffredo-regext-epp-over-http-05.txt` - EPP over HTTP
- `draft-yao-regext-epp-quic-03.txt` - EPP over QUIC

## Referenced but Not EPP-Specific

The following RFCs are referenced in EPP specifications but are general Internet standards (not included in this directory as they're not EPP-specific):

### DNS Standards

- RFC 1034 - Domain Names - Concepts and Facilities
- RFC 1035 - Domain Names - Implementation and Specification
- RFC 4033 - DNS Security Introduction and Requirements
- RFC 4034 - Resource Records for DNS Security Extensions
- RFC 4035 - Protocol Modifications for DNS Security Extensions

### General Standards

- RFC 2119 - Key words for use in RFCs (BCP 14)
- RFC 8174 - Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words
- RFC 3688 - The IETF XML Registry
- RFC 3735 - Guidelines for Extending EPP
- RFC 7451 - Extension Registry for EPP

### IDN Standards

- RFC 5890 - Internationalized Domain Names for Applications (IDNA)

## Implementation Status in instant-epp

### Fully Implemented & Tested

- ✅ RFC 5730, 5731, 5732, 5733, 5734 - Core EPP
- ✅ RFC 3915 - RGP (Registry Grace Period)
- ✅ RFC 5910 - SecDNS
- ✅ RFC 8590 - ChangePoll
- ✅ Verisign Namestore Extension
- ✅ Verisign Sync/ConsoliDate Extension
- ✅ Verisign Low Balance Extension

### Partially Implemented

- 🟡 Some Verisign extensions (Balance, RGP Poll, etc.)

### Not Implemented (Future)

- ❌ RFC 8495 - Allocation Token
- ❌ RFC 8334 - Launch Phase
- ❌ RFC 8748 - Fee Extension
- ❌ Many draft specifications

## Sources

- RFCs downloaded from: <https://www.rfc-editor.org/>
- Verisign specs from: <https://www.verisign.com/resources/registrar-resources/epp-sdk/>
- Internet Drafts from: <https://datatracker.ietf.org/>

## Last Updated

Generated: 2024

## Notes

This directory contains specifications for reference during development and testing. The presence of an RFC in this directory does not necessarily mean the extension is implemented in instant-epp.
