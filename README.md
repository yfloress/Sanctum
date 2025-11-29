<div align="center">

  <h1>SANCTUM</h1>
  
  <p>
    <strong>A fortress for your personal finances.</strong>
    <br />
    Private. Encrypted. Local-first.
  </p>

  <a href="">
    <img src="https://img.shields.io/badge/Built_with-Rust-orange?style=for-the-badge&logo=rust" alt="Rust" />
  </a>
  <a href="">
    <img src="https://img.shields.io/badge/Frontend-Tauri_v2-blue?style=for-the-badge&logo=tauri" alt="Tauri" />
  </a>
  <a href="">
    <img src="https://img.shields.io/badge/Security-SQLCipher-green?style=for-the-badge&logo=sqlite" alt="SQLCipher" />
  </a>
  <a href="">
    <img src="https://img.shields.io/badge/License-GPLv3-red?style=for-the-badge" alt="License" />
  </a>

  <br />
  <br />
</div>

---

## About

**Sanctum** is a desktop application designed for those who refuse to compromise on privacy. Unlike cloud-based solutions, Sanctum runs entirely offline on your Linux machine.

Your financial data, crypto portfolio, and habits are stored in a local **SQLite database encrypted with SQLCipher**. You hold the keys. No servers, no tracking, no leaks.

## Features (Planned)

* **Military-Grade Encryption:** Zero-knowledge architecture. Database is encrypted at rest.
* **Finance Tracker:** Income, expenses, and transfer management with visual analytics.
* **Crypto Portfolio:** Real-time prices and P&L monitoring (API integration).
* **Atomic Habits:** Integrated habit tracker to align your finances with your lifestyle.
* **Linux Native:** Optimized for the Linux desktop ecosystem.

## The Stack

Sanctum is built with a focus on performance, type safety, and auditability.

| Component | Technology | Description |
| :--- | :--- | :--- |
| **Core** | **Rust** | Business logic, calculations, and security. |
| **GUI Framework** | **Tauri v2** | Native system bindings and window management. |
| **Runtime** | **Deno** | Secure TypeScript runtime for the frontend build. |
| **Database** | **SQLite + SQLCipher** | Local, encrypted relational storage. |
| **Environment** | **Nix** | Reproducible development environment. |

## Development Setup

This project uses **Nix** to guarantee a reproducible environment without polluting your system. You don't need to install Rust or Deno globally.

### Prerequisites

* [Nix Package Manager](https://nixos.org/download.html)
* Git

### Quick Start

1.  **Clone the repository:**
    ```bash
    git clone https://codeberg.org/Kyronix/Sanctum
    cd sanctum
    ```

2.  **Activate the environment:**
    This will download Rust, Deno, and all system libraries required for Tauri.
    ```bash
    nix develop
    ```

3.  **Install frontend dependencies (via Deno):**
    ```bash
    cd src-tauri
    # Assuming standard npm compatibility layer
    deno install
    ```

4.  **Run in Development Mode:**
    ```bash
    cargo tauri dev
    ```

## Disclaimer

**Sanctum is currently in ALPHA.** While the encryption used is industry-standard, the software is under active development. Always keep backups of your recovery keys.

## License

This project is open-source and available under the **GNU General Public License v3.0**. 
See the [LICENSE](LICENSE) file for more info.

---
<div align="center">
  <sub>Built with ❤️ and 🦀</sub>
</div>