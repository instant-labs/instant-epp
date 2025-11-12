# RGP Poll Extension Response Fixtures

This directory contains poll message fixtures for the **Verisign RGP Poll Extension** for EPP.

## Specification

- **Name**: Verisign RGP Poll Extension
- **Version**: 1.0
- **Spec File**: [`../../../../rfcs/verisign_epp-extension_rgp-poll_v01.txt`](../../../../rfcs/verisign_epp-extension_rgp-poll_v01.txt)
- **Namespace**: `http://www.verisign.com/epp/rgp-poll-1.0`

## Key Elements

Poll messages include:
- **name** - The domain name affected
- **rgpStatus** - Current RGP status (e.g., "pendingRestore")
- **reqDate** - Date the restore request was submitted
- **reportDueDate** - Deadline for submitting the restore report
