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
