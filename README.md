<div align="center">

<img src="./ui/assets/logo/sanctum_logo.svg" alt="Sanctum Logo" width="120" height="120" />

<h1>SANCTUM</h1>

<p align="center">
  <img src="https://readme-typing-svg.demolab.com?font=Fira+Code&weight=600&size=20&pause=1000&color=8B5CF6&center=true&vCenter=true&width=435&lines=Your+personal+fortress.;Private.+;Encrypted.;Local-first." alt="Typing SVG">
</p>

<div align="center">

[![Español](https://img.shields.io/badge/Idioma-Espa%C3%B1ol-8b5cf6?style=for-the-badge)](README_ES.md) [![Website](https://img.shields.io/badge/🌐_Website-Sanctum-blueviolet?style=for-the-badge)](https://kyronix.codeberg.page/Sanctum/)

</div>

<div align="center">
    <a href="">
      <img src="https://img.shields.io/badge/Core-Rust-orange?style=for-the-badge&logo=rust" alt="Rust" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/GUI-Slint-blue?style=for-the-badge" alt="Slint" />
    </a>
    <a href="">
      <img src="https://img.shields.io/badge/Render-Skia%20%2B%20OpenGL-informational?style=for-the-badge" alt="Skia" />
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

![Sanctum](assets/sanctum.jpg)

## About

> **NOT READY FOR USE, IN DEVELOPMENT**

**Sanctum** is a privacy-first desktop vault for money, crypto, and habits. It runs
locally on Linux with encrypted storage, no telemetry, and zero cloud dependency.
You keep the keys, the database, and the backups.

## Current Capabilities

- **Dashboard:** Net worth and trend analytics across finance + crypto.
- **Finances:** Accounts, categories, transfers, and full transaction ledger.
- **Crypto:** Wallets, trades, swaps, portfolio balances, privacy-preserving price sync via CoinGecko, and **comprehensive tax engine** (Chile/SII, USA/IRS, International).
- **Crypto Taxes:** Offline-first tax engine supporting multiple jurisdictions, cost-basis methods (FIFO, LIFO, HIFO, CPP), IPC adjustments for Chile, and automated tax report generation.
- **Habits:** Daily logs, heatmaps, streaks, and rewards/goals.
- **Import & Backup:** JSON/CSV/TXT ingestion with preview + dedup, **manual IPC data import** for tax adjustments, plus encrypted backups and restore/rollback.
- **Settings:** USD/CLP currency, EN/ES language, proxy support for crypto APIs.

## Import Formats

Sanctum accepts multiple offline formats, designed for travel or low-connectivity workflows:

- **JSON** (recommended): Full-fidelity format used by the Sanctum Generator.
- **CSV**: Spreadsheet exports (separate files for transactions, habit logs, crypto).
- **TXT**: Prefixed, line-based notes for quick capture.

All imports are **best-effort**, validated per row, and include duplicate detection.
No network calls are made during ingestion.

## Security & Privacy

- **SQLCipher encryption** with a user-held master password.
- **No telemetry** or analytics.
- **Local-first** storage with explicit import/export only.
- **Encrypted backups** with restore + rollback safety.

## Tech Stack

Sanctum prioritizes performance, type safety, and auditability.

| Component            | Technology             | Description                                                       |
| :------------------- | :--------------------- | :---------------------------------------------------------------- |
| **Core**             | **Rust**               | Business logic, validation, and calculations.                     |
| **GUI Framework**    | **Slint**              | Native Rust UI toolkit. Lightweight and type-safe.                |
| **Renderer**         | **Skia / OpenGL**       | High-performance 2D rendering via Winit.                          |
| **Database**         | **SQLite + SQLCipher**  | Locally encrypted relational storage.                             |
| **Environment**      | **Nix + Direnv**        | Reproducible dev environment.                                     |

### Installation & Development

This project uses **Nix Flakes** to guarantee a reproducible environment without
polluting your global system.

> **Note:** For manual installation on Linux, macOS, or Windows, please see the complete [Building Guide](docs/BUILDING.md).

### Quick Start (Nix)

1. **Clone the repository:**
   ```bash
   git clone https://codeberg.org/Kyronix/Sanctum.git
   cd Sanctum
   ```

2. **Activate the Environment:**
   ```bash
   direnv allow
   ```

3. **Verify the workspace:**
   ```bash
   # Or without --release
   nix develop -c cargo run --release
   ```

For detailed setup on other platforms, see [docs/BUILDING.md](docs/BUILDING.md).

## Development Transparency

This project embraces open collaboration without compromising auditability.

- **Human-led architecture:** Privacy and data integrity are the priority.
- **AI-assisted development:** Most of the code has been generated or
  refactored with LLMs under strict human auditing. The primary models used,
  in order, are **Claude Opus 4.5 (now 4.6)**, **Claude Sonnet 4.5**, **Codex 5.2 (now 5.3)**, and
  **Gemini 3 Pro**.
- **Auditability:** The code is open for inspection and verification.

## Disclaimer

**Sanctum is currently in ALPHA.** Although the encryption used is
industry-standard, the software is under active development. Always keep backups
of your recovery keys.

## License

This project is open-source and available under the **GNU General Public License
v3.0**. See the [LICENSE](LICENSE) file for more info.

-----

<div align="center">
<sub>Built with ❤️, 🦀 Rust and ❄️ Nix</sub>
</div>
