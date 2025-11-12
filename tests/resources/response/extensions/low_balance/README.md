# Low Balance Poll Extension Response Fixtures

This directory contains poll message fixtures for the **Verisign Low Balance Poll Extension** for EPP.

## Specification

- **Name**: Verisign Low Balance Poll Extension
- **Version**: 1.0
- **Spec File**: [`../../../../rfcs/verisign_epp-extension_low-balance_v01.txt`](../../../../rfcs/verisign_epp-extension_low-balance_v01.txt)
- **Namespace**: `http://www.verisign.com/epp/lowbalance-poll-1.0`

## Key Elements

Poll messages include:
- **registrarName** - Name of the registrar account
- **creditLimit** - The credit limit configured for the account
- **creditThreshold** - The threshold that triggered the notification
  - `type` attribute: "FIXED" or "PERCENT"
- **availableCredit** - Current available credit balance
