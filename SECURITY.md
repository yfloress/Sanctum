# Security Policy

Sanctum holds people's financial records, so security reports are treated as the
highest priority work on the project. This document says how to report a problem,
what happens next, what Sanctum is designed to protect against, and which controls
are in place today.

The disclosure process follows **ISO/IEC 29147** (vulnerability disclosure) and
**ISO/IEC 30111** (vulnerability handling). Severity is rated with **CVSS v4.0** and
weaknesses are classified with **CWE**. The controls below are described against
**OWASP ASVS**, the **OWASP Cheat Sheet Series** and **NIST** guidance. "Follows" means
the project uses these documents as its reference; Sanctum has **not** been certified
or audited against any of them.

## Status

**Sanctum is alpha software and has not had an external security audit.** The
cryptography comes from well-reviewed libraries, but how it is put together has only
been reviewed by the maintainer. Do not rely on Sanctum as a hardened product yet.

## Supported versions

| Version | Security fixes |
| :-- | :-- |
| Latest commit on `main` | Yes |
| Anything older | No |

There are no maintained release branches while the project is in alpha.

## Reporting a vulnerability

**Do not open a public issue, discussion or pull request for a vulnerability.**

Report it privately through GitHub Security Advisories:

<https://github.com/yfloress/Sanctum/security/advisories/new>

Please include:

- What the problem is and what an attacker gains from it.
- Steps to reproduce or a proof of concept.
- The commit you tested and your platform (Linux, macOS, Windows or Android).
- If you can, a suggested CVSS v4.0 vector and CWE ID. This is optional; it will be
  agreed with you during triage.

> [!WARNING]
> **Never send real financial data, wallet addresses, seed phrases, API keys or a vault
> password.** Reproduce the issue with a throwaway vault and invented data.

### What happens next

| Step | Target |
| :-- | :-- |
| Acknowledgement of your report | 7 days |
| Triage: confirmed or not, severity agreed, or why it is out of scope | 30 days |
| Fix for a critical or high severity issue | 30 days after triage |
| Fix for a medium or low severity issue | 90 days after triage |
| Public advisory | When the fix is on `main`, or 90 days after the report, whichever comes first |

This is a single-maintainer project, so the targets are honest goals rather than
guarantees. If a deadline will be missed you will be told why, and the disclosure date
can be agreed with you. Published advisories credit the reporter unless you ask to stay
anonymous, and a CVE is requested for every confirmed vulnerability.

### Safe harbour

Research done in good faith under this policy is welcome. The project will not pursue
or support legal action against you for it, provided that you:

- test only against your own installation and your own data;
- do not access, change or delete anyone else's data;
- give the project reasonable time to fix the issue before disclosing it;
- do not degrade a service or system that other people depend on.

## Scope

**In scope**

- Encryption at rest: SQLCipher configuration, key derivation, and anything that
  leaves vault contents readable without the password.
- Handling of the master password and other secrets: in memory, in logs, in the
  clipboard, in crash output or on disk.
- Unlocking, the failed-attempt lockout, automatic locking, and backup, restore and
  rollback integrity.
- Any network request Sanctum makes that the user did not ask for, or any way a request
  reveals more than it should (holdings, identity, a bypassed proxy).
- Files a user can be tricked into importing (CSV, JSON, TXT, PDF bank statements):
  memory-safety bugs, crashes that lose data, path traversal, or code execution.
- The interface: script injection, escaping the Tauri content security policy, or
  reaching backend commands the interface should not reach.
- Exchange connections: storage of API keys and anything that would let a key do more
  than read.

**Out of scope**

- Attacks that need an already compromised operating system, an administrator or root
  attacker, malware on the device, or physical access to an unlocked session.
- A weak master password the user chose after being warned.
- Crashes from deliberately malformed input that is rejected without losing data.
- Vulnerabilities in third-party dependencies with no demonstrated effect on Sanctum.
  Report those to the upstream project.
- The self-hosted server and web app, which do not exist yet. This policy will cover
  them when they ship.

## Threat model

**What is protected:** the vault database (accounts, transactions, crypto holdings,
tax data), the master password, exchange API keys, and backups.

**Against whom:**

| Adversary | Example | Protection |
| :-- | :-- | :-- |
| Someone who gets the database file | Stolen or lost laptop, leaked backup, compromised sync folder | The whole file is encrypted; the password is needed to read any of it |
| Someone guessing the password on the device | Shared or unattended computer | Expensive key derivation, lockout after repeated failures |
| Someone at an unlocked but abandoned session | User walks away | Automatic lock after inactivity |
| A price or exchange service, or the network | Provider logging requests, a network observer | Offline by default; identical price requests for every user; optional proxy or Tor |
| A malicious import file | A crafted CSV or PDF statement | Size limits, per-row validation, parsers isolated from panics, nothing in a file is executed |

**Outside the model:** an attacker who already controls the running operating system
(keyloggers, memory readers, screen capture), hardware attacks, and hiding the fact
that Sanctum is installed or in use.

## Security controls

### Data at rest (ASVS V6 Stored Cryptography; OWASP Cryptographic Storage Cheat Sheet)

- The entire database is encrypted with **SQLCipher**: AES-256 with a per-page
  HMAC-SHA512, so tampering is detected as well as reading prevented.
- The key is derived from the master password with **PBKDF2-HMAC-SHA512 at 600,000
  iterations**, above the 210,000 that the OWASP Password Storage Cheat Sheet
  recommends for this algorithm (NIST SP 800-132).
- SQLCipher's memory security is enabled, so its buffers are wiped when released.
- On Unix the vault and every backup are created with `0600` permissions and the data
  directory with `0700`.
- A backup is a byte-for-byte copy of the encrypted file and is never decrypted on the
  way. Restoring keeps the previous vault so the restore can be rolled back.

### Authentication and sessions (ASVS V2 Authentication, V3 Session Management)

- There are no accounts and no server: the master password is the only credential, and
  it never leaves the device.
- Five failed attempts within a minute lock unlocking for five minutes. The counter is
  kept in a separate owner-only file next to the vault, so it survives a restart.
- The vault locks itself after 15 minutes of inactivity by default; the user can change
  this.
- A weak password produces a warning when the vault is created.

### Secrets in memory (ASVS V8 Data Protection)

- The master password and exchange API keys are held in `secrecy` / `zeroize` types,
  which are wiped from memory when dropped and are never printed by debug output.
- Exchange API keys are stored only inside the encrypted vault, and the user is told to
  create keys that can only read the ledger.

### Network (ASVS V9 Communication, V14 Configuration)

- **Offline by default.** Sanctum makes no network request unless the user asks for one
  or turns automatic price updates on. There is no telemetry, crash reporting or update
  check.
- Requests use HTTPS only, follow no redirects, have time limits and send a generic
  user agent.
- **Price privacy:** prices are fetched for the same fixed set of the 250 largest coins
  for every user, so a request does not reveal what the user holds. A coin outside that
  set is looked up only after explicit consent.
- **Proxy and Tor:** requests can go through a SOCKS5 or HTTP proxy. A `socks5://`
  address is upgraded to `socks5h://` so host names are resolved by the proxy and not
  leaked to the local DNS. If the proxy fails the request fails; there is no fallback to
  a direct connection.

### Input handling (ASVS V5 Validation, Sanitization and Encoding; V12 Files)

- Imported files are capped at 10 MB, parsed in memory, validated row by row, and
  never executed.
- The PDF reader runs behind a panic guard, so a malformed statement produces an error
  instead of crashing the app.
- Password-protected statements are opened with a password that is used for that call
  only and never stored.
- Export paths are normalised and confined to the app's data directory, so a crafted
  path cannot write elsewhere.

### Interface (ASVS V14.4 HTTP Security Headers; OWASP Desktop App Security)

- The interface is a Tauri webview with a strict content security policy: only the
  app's own scripts, no remote content, no frames, and network access limited to the
  local IPC channel.
- Backend commands are an explicit list, and every input crossing the IPC boundary is
  validated in Rust before it is used.
- Only the Tauri core, dialog and clipboard capabilities are granted.

### Logging (ASVS V7 Error Handling and Logging)

- Security events (vault opened, failed unlock, lockout, password changed) are logged
  for the user's own diagnosis, without amounts, descriptions, addresses or secrets.

### Supply chain (OWASP Top 10 A06 Vulnerable and Outdated Components)

- Every change runs `cargo fmt`, `cargo clippy -D warnings`, the full test suite and
  `cargo machete` in CI.
- `cargo deny` checks Rust dependencies against the RustSec advisory database, and
  enforces allowed licences and sources.
- JavaScript dependencies install from a frozen lockfile, with lifecycle scripts
  disabled and a three-day quarantine on newly published versions.
- Renovate and Dependabot propose updates; security updates skip the waiting period.
- The development environment is a pinned Nix flake, so builds use known toolchains.

## Known limitations

- No external security audit has been done.
- The content security policy allows inline styles (`style-src 'unsafe-inline'`), which
  the interface framework needs. Inline scripts are not allowed.
- The password strength check relies on character-variety rules and only warns. NIST
  SP 800-63B now recommends judging length and checking against breached-password
  lists instead.
- There is no signed release process yet: no published binaries are signed. Build from
  source and verify the commit.
- The PDF reader is isolated from panics, not sandboxed in a separate process.

## References

- ISO/IEC 29147:2018, Vulnerability disclosure
- ISO/IEC 30111:2019, Vulnerability handling processes
- FIRST, Common Vulnerability Scoring System v4.0
- MITRE, Common Weakness Enumeration (CWE)
- OWASP Application Security Verification Standard (ASVS)
- OWASP Cheat Sheet Series: Password Storage, Cryptographic Storage
- OWASP Mobile Application Security Verification Standard (MASVS), for the Android port
- NIST SP 800-63B, Digital Identity Guidelines: Authentication
- NIST SP 800-132, Recommendation for Password-Based Key Derivation
