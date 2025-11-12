# FRNIC (AFNIC) Contact Extension Test Fixtures

This directory contains test fixtures for the **AFNIC (French Registry) Contact Extension** for EPP.

## Specification

- **Name**: AFNIC Contact Extension
- **Version**: 2.0
- **Registry**: AFNIC (.fr and other French TLDs)
- **Namespace**: `http://www.afnic.fr/xml/epp/frnic-2.0`
- **Documentation**: <http://www.afnic.fr/xml/epp/frnic-2.0>

## Key Elements

The extension adds:
- **firstName** - First name for natural persons
- **legalEntityInfos** - Legal entity information:
  - **legalStatus** - Legal status (company, association, individual, etc.)
  - **siren** - French business identification number (9 digits)
  - **asso** - Association details:
    - **decl** - Declaration date
    - **publ** - Publication information (date, page, announce number)
