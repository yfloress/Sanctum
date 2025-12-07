<div align="center">

<h1>SANCTUM</h1>

<p>
    <strong>Your personal financial fortress.</strong>
    <br />
    Private. Encrypted. Local-first.
  </p>

<div align="center">
    <a href="">
      <img src="https://img.shields.io/badge/Core-Rust-orange?style=for-the-badge&logo=rust" alt="Rust" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/Frontend-Slint-4B8BBE?style=for-the-badge&logo=rust" alt="Slint" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/Security-SQLCipher-green?style=for-the-badge&logo=sqlite" alt="SQLCipher" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/Env-Nix-blueviolet?style=for-the-badge&logo=nixos" alt="Nix" />
    </a>
  </div>

<br />
</div>

---

## About

**Sanctum** is a desktop application designed for those who refuse to compromise
on privacy. Unlike cloud-based solutions, Sanctum runs entirely offline on your
Linux machine.

Your financial data, crypto portfolio, and habits are stored in a local **SQLite
database encrypted with SQLCipher**. You hold the keys. No servers, no tracking,
no leaks.

## Key Features

- **Military-Grade Encryption:** Zero-knowledge architecture. The database is
  unreadable without your master password (AES-256).
- **Financial Ledger:** Advanced management of wallets, income, expenses, and
  transfers. Complete audit trail.
- **Crypto Portfolio:** Investment tracking, PnL (Profit/Loss) calculation, and
  multi-wallet asset aggregation.
- **Atomic Habits:** (In development) Integrated productivity tracker to align
  your finances with your lifestyle.
- **Linux Native:** Optimized for the Linux desktop ecosystem with a Rust-only
  Slint UI (Wayland-first, X11 fallback optional).

## Tech Stack

Sanctum is built prioritizing performance, type safety, and auditability.

| Component            | Technology             | Description                                               |
| :------------------- | :--------------------- | :-------------------------------------------------------- |
| **Core**             | **Rust**               | Business logic, financial calculations, and security.     |
| **GUI Framework**    | **Slint**              | Native Rust UI, no JavaScript runtime.                    |
| **Frontend Runtime** | **Rust**               | Single-toolchain build; no npm/Deno involved.             |
| **Database**         | **SQLite + SQLCipher** | Locally encrypted relational storage.                     |
| **Environment**      | **Nix + Direnv**       | Reproducible and hermetic development environment.        |

## Branch Strategy

- `main`: Slint-based Rust UI (current active line).
- `tauri-legacy`: frozen Tauri/React client for reference and hotfixes.
- `feature/slint-migration`: long-lived branch that landed the Slint rewrite.
- Short-lived work branches should target `main`; hotfixes can cherry-pick into `tauri-legacy` if needed.

## Workspace Layout

- `core/`: shared Rust library (SQLCipher DB, crypto client, models, session logic).
- `app-slint/`: Slint desktop client (Rust-only UI) consuming `core`.
- `app-tauri/` (legacy): previous Tauri/React client kept for reference on `tauri-legacy`.

## Installation & Development

This project uses **Nix Flakes** to guarantee a reproducible environment without
polluting your global system. You don't need to manually install Rust or system
libraries.

### Prerequisites

- [Nix Package Manager](https://nixos.org/download.html)
- [Direnv](https://direnv.net/) (Optional, but highly recommended)
- Git

### Quick Start (Slint client)

1. **Clone the repository:**
   ```bash
   git clone https://codeberg.org/Kyronix/Sanctum.git
   cd Sanctum
   ```

2. **Activate the Environment:**

   - **Option A (Recommended with `direnv`):**
     ```bash
     direnv allow
     ```

   - **Option B (Manual with Nix):**
     ```bash
     nix develop
     ```

3. **Run in Development Mode:**
   ```bash
   cargo run -p app-slint
   ```

4. **Build release binary:**
   ```bash
   cargo build -p app-slint --release
   ```

Legacy Tauri client lives on branch `tauri-legacy` and can be built with `cargo tauri dev` there if you need it.

## Development Transparency

This is a modern Open Source project that embraces the evolution of software
development.

- **Architecture and Vision:** Designed and directed by humans, prioritizing
  privacy and local security.
- **AI Collaboration:** Parts of the code have been generated and refactored
  with the assistance of advanced LLMs. Models such as **Gemini 3 Pro**,
  **Claude Opus 4.5**, **Claude Sonnet 4.5**, and **Codex 5.1** have been used
  under strict human supervision and auditing to ensure security and business
  logic.
- **Auditability:** The code is open so anyone can verify that there is no
  hidden telemetry or attack vectors.

## Disclaimer

**Sanctum is currently in ALPHA.** Although the encryption used is
industry-standard, the software is under active development. Always keep backups
of your recovery keys.

## License

This project is open-source and available under the **GNU General Public License
v3.0**. See the [LICENSE](LICENSE) file for more info.

---

<div align="center">
  <sub>Built with ❤️, 🦀 Rust and ❄️ Nix</sub>
</div>
