# Namestore Extension Response Fixtures

This directory contains response fixtures for the **Verisign Namestore Extension** for EPP.

## Specification

- **Name**: Namestore Extension Mapping for EPP
- **Version**: 1.1
- **Spec File**: [`../../../../rfcs/verisign_epp-extension_namestoreext_v01.txt`](../../../../rfcs/verisign_epp-extension_namestoreext_v01.txt)
- **Namespace**: `http://www.verisign-grs.com/epp/namestoreExt-1.1`

## Response Structure

Responses include the `<namestoreExt>` extension in the response `<extension>` block, echoing back the sub-product that was used for the operation.
