# Repository Guidelines

## Project Structure and Module Organization
- `src/`: Rust application code (controllers, models, DB, services, crypto).
- `ui/`: Slint UI (pages, components, modals, globals, widgets).
- `assets/` and `ui/assets/`: Images, icons, and static UI assets.
- `docs/` and `MAPA_PROYECTO.md`: Architecture and project map.
- `flake.nix` / `flake.lock`: Nix dev environment for consistent builds.

## Build, Test, and Development Commands
Use the Nix dev shell for builds:
- `nix develop -c cargo check`: fast compile check.
- `nix develop -c cargo clippy`: lint Rust code.
- `nix develop -c cargo test`: run unit tests (if available).
- `nix develop -c cargo run`: run the app locally.

## Coding Style and Naming Conventions
- Rust: follow `rustfmt` defaults (4-space indentation).
- Slint: keep layout readable; align properties and use consistent spacing.
- Naming: snake_case for Rust functions/vars, PascalCase for types, kebab-case for files when adding new ones.
- Prefer ASCII in edits unless the file already uses non-ASCII text.

## Testing Guidelines
- Primary tests are Rust unit tests via `cargo test`.
- If you add logic-heavy code, include a test or document why it is not tested.
- Keep tests deterministic and avoid network calls.

## Commit and Pull Request Guidelines
- Use git-flow style messages seen in history: `feat:`, `fix:`, `docs:`, `chore:`, etc.
  Example: `feat(habits): add radar analytics`.
- Before commit: run `nix develop -c cargo check` when possible.
- PRs (if used) should include a short summary, testing notes, and screenshots for UI changes.

## Security and Configuration Notes
- Database uses SQLCipher; avoid logging sensitive data.
- Crypto price fetching uses external APIs; respect privacy settings and avoid auto-fetch where not intended.

## Architecture Patterns (src/)

### Layer Separation
```
UI (callbacks/) → Controller → Features (service/repository) → DB
```
- **UI callbacks**: Only call controller methods, never import from `features/` directly.
- **Controller**: Orchestrates features, provides fallback methods when needed.
- **Features**: Domain logic with service.rs + repository.rs pattern.
- **Core**: Shared utilities (validation.rs, error types).

### File Organization
- Keep files under ~500 lines. Split large modules into subdirectories:
  ```
  feature.rs (>500 lines) → feature/mod.rs + service.rs + helpers.rs + callbacks.rs
  ```
- Validation: Domain-specific in `features/X/validation.rs`, shared in `core/validation.rs`.
- Domain errors: Each feature defines its own error type (e.g., `CryptoError`, `FinanceError`).

### Validation Pattern
- Core validation returns `Result<T, String>`.
- Feature validation wraps core and converts to domain error:
  ```rust
  pub fn validate_uuid(id: &str) -> Result<String, FeatureError> {
      crate::core::validation::validate_uuid(id).map_err(FeatureError::Validation)
  }
  ```

### Callbacks Structure (src/ui/callbacks/)
- One subdirectory per feature: `crypto/`, `finance/`, `habits/`, `dashboard/`.
- Each has: `mod.rs` (coordinator), `callbacks.rs` (on_* registrations), `helpers.rs`.

## Architecture Patterns (ui/)

### Structure
```
ui/
├── pages/           # One file per page (FinancesPage, CryptoPage, etc.)
├── modals/          # One file per modal dialog
├── components/      # Reusable UI components
│   ├── buttons/     # TabButton, etc.
│   ├── sections/    # SectionHeader, EmptyState
│   ├── filters/     # FilterInput
│   ├── forms/       # FormField, TypeSelector, WalletSelector, CoinSelector
│   └── *.slint      # Feature-specific components
├── globals.slint    # Adapters (data binding) and Palette (theme)
├── widgets.slint    # Base widgets (TextButton, ActionButton, etc.)
└── app.slint        # Main app layout
```

### Guidelines
- Keep files under ~500 lines. Extract repeated components to `components/`.
- Shared components go in subdirectories by type (buttons/, sections/, forms/).
- Use re-exports for backwards compatibility when splitting files.
- Adapters in `globals.slint` bridge Rust ↔ Slint (set from callbacks).
- kebab-case for properties and callbacks, PascalCase for components.
