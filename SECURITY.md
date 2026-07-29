# Security Policy

## Status

**Sanctum is in ALPHA and has not undergone an external security audit.**
The cryptographic primitives it relies on are industry-standard, but their
integration in this project has been reviewed only by its maintainer. Do not
treat Sanctum as a hardened product yet.

## Supported Versions

Only the latest commit on `main` receives security fixes. There are no
maintained release branches while the project is in alpha.

## Reporting a Vulnerability

**Do not open a public issue for a security vulnerability.**

Report it privately through GitHub Security Advisories:

<https://github.com/yfloress/Sanctum/security/advisories/new>

Please include:

- A description of the issue and its impact.
- Steps to reproduce, or a proof of concept.
- The commit hash you tested against.
- Your environment (OS, desktop or Android).

**Never include real financial data, wallet addresses, seed phrases, or your
vault password in a report.** Reproduce with a throwaway vault instead.

### What to expect

- Acknowledgement within 7 days.
- An assessment and a plan, or an explanation of why it is out of scope,
  within 30 days.
- Credit in the advisory when the fix ships, unless you prefer to stay
  anonymous.

This is a single-maintainer project, so please allow reasonable time before
public disclosure.

## Scope

In scope:

- Weaknesses in the encryption at rest (SQLCipher) or in how the master
  password is derived, held in memory, or cleared.
- Vault unlock, session handling, and backup/restore integrity.
- Anything that causes data to leave the device unexpectedly, including
  unintended network calls or telemetry.
- Memory-safety issues, or parsing flaws in the CSV/JSON/TXT ingestion path
  reachable from a malicious import file.
- Privilege or path-traversal issues in the self-hosted server mode.

Out of scope:

- Attacks that require an already-compromised host, a root/administrator
  attacker, or physical access to an unlocked device.
- A weak master password chosen by the user.
- Denial of service caused by deliberately malformed input that is rejected
  during import validation.
- Vulnerabilities in third-party dependencies without a demonstrated impact on
  Sanctum. Report those upstream.

## Threat Model

Sanctum is designed to protect your data from anyone who obtains the database
file: a stolen laptop, a leaked backup, or a compromised sync target. It does
not attempt to protect against an attacker who already controls the running
operating system, nor does it hide the fact that you use Sanctum.
