# Repository Guidelines

## Project Intent & Mindset
Sanctum is a personal finance/crypto/habits app focused on **security and privacy first**.
Default to safer behavior: minimize data exposure, avoid unnecessary network calls, and
preserve user trust. When in doubt, prioritize privacy and data integrity over convenience.

## Project Structure and Module Organization
This repo uses a **feature-sliced** layout with strict layer separation:
`UI callbacks → Controller → Features (service/repository) → DB`.

`src/` (Rust)
```
src/
├── core/            # Shared validation, errors, DB wrappers
├── features/        # finance/, crypto/, habits/, dashboard/, ingestion/ (service + repository + validation)
├── controller/      # Orchestration per domain + settings
├── db/              # Domain SQL + migrations (finance, crypto, habits, rewards)
├── services/        # Cross-cutting services (charts)
├── ui/              # Rust-side UI callbacks + helpers + data
├── main.rs          # Slint bootstrap
├── lib.rs           # Crate exports
└── models.rs
```

`ui/` (Slint)
```
ui/
├── pages/       # One file per page
├── modals/      # Dialogs
├── components/  # Reusable UI blocks (+ subfolders)
├── globals.slint
├── widgets.slint
└── app.slint
```

`locales/` (i18n)
```
locales/
├── en.ftl       # English translations (Fluent format)
└── es.ftl       # Spanish translations
```

## Workflow Rules (Important)
- UI logic lives in `src/ui/callbacks/`; keep `src/main.rs` as bootstrap + wiring only.
- Controllers coordinate only; business logic must live in `features/*/service.rs` or `services/*`.
- All chart rendering (`plotters`) goes in `src/services/charts.rs` only.
- Validation: shared rules in `src/core/validation.rs`, domain wrappers in `features/*/validation.rs`.
- Use `nix develop -c ...` for Rust commands (build/test).
- Never run cargo run or cargo build.
- UI text must use i18n: add keys to `locales/*.ftl` and use `Translations.*` in Slint.
- Use `ui/globals.slint` (Palette) for colors/spacing; avoid hardcoded styling values.
- Crypto icons live in `ui/assets/crypto-icons`; the base path is defined in `src/ui/helpers.rs`.
- Tax rule: for Chile jurisdiction, tax reports, tax history exports, and displayed tax totals must always use CLP.
- Every source file (including tests) must start with this exact AGPL header:
```rust
// Sanctum — a privacy-first personal finance, crypto, and habits vault.
// Copyright (C) 2026  Kyronix
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//
```

## Build, Test, and Development Commands
Always ask the user if they want to run any of these commands:
- `nix develop -c cargo check -j 2`
- `nix develop -c cargo clippy -j 2`
- `nix develop -c cargo test -j 2`

## Coding Style and Naming Conventions
- Rust: `rustfmt` defaults (4-space).
- Slint: align properties, keep layout readable.
- Naming: snake_case (Rust), PascalCase (types), kebab-case (files when adding new ones).

## Testing Guidelines
- Only when needed use `cargo test` for unit tests; keep them deterministic.
- Avoid network calls in tests.

## Commit Guidelines
- Do not create commits unless the user explicitly asks for a commit.
- Git-flow style: `feat:`, `fix:`, `docs:`, `chore:`, etc.
  Example: `feat(habits): add radar analytics`.

## Security Notes
- DB uses SQLCipher; avoid logging sensitive values.
- Crypto price fetching uses external APIs; respect privacy settings.

## Extra
- NEVER use emojis. Use SVG icons from `ui/assets/icons/` instead.
- If you cannot find the icon you need, there is an older commit where many SVGs were imported from the Lucide repository. Find it and bring the icon back.
