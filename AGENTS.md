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
├── features/        # finance/, crypto/, habits/ (service + repository + validation)
├── controller/      # Orchestration per domain + settings
├── db/              # Domain SQL (finance, crypto, habits)
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
- Git-flow style: `feat:`, `fix:`, `docs:`, `chore:`, etc.
  Example: `feat(habits): add radar analytics`.

## Security Notes
- DB uses SQLCipher; avoid logging sensitive values.
- Crypto price fetching uses external APIs; respect privacy settings.

## Extra
- NEVER use emojis. Use SVG icons from `ui/assets/icons/` instead.
- If you cannot find the icon you need, there is an older commit where many SVGs were imported from the Lucide repository. Find it and bring the icon back.
