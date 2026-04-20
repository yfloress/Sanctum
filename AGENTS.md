# Repository Guidelines

## Approach
- Think before acting. Read existing files before writing code.
- Be concise in output but thorough in reasoning.
- Prefer editing over rewriting whole files.
- Do not re-read files you have already read unless the file may have changed.
- Test your code before declaring done.
- No sycophantic openers or closing fluff.
- Keep solutions simple and direct.
- User instructions always override this file.

## Search Tooling
- **NEVER use `grep`.** Always use `ripgrep` (`rg`) via the Grep tool or direct shell invocation.
  Ripgrep respects `.gitignore`, is orders of magnitude faster, and produces cleaner output.
- Use `rg --type rust` / `rg --type svelte` / `rg -g '*.ts'` to scope by language.
- For file discovery, prefer `rg --files | rg <pattern>` or the Glob tool over `find`.

## Memory & Progress Tracking (basic-memory MCP)
> **This entire section only applies when the `basic-memory` MCP server is active in the current session.**
> If the `mcp__basic-memory__*` tools are not available, ignore everything in this section — do not attempt to invoke them, do not fabricate notes, do not mention them to the user.

The `basic-memory` MCP server is the **authoritative knowledge store** for this project.
Use it proactively to persist context across sessions — not just ad-hoc conversation memory.

**Read `config/SOUL.md` at session start.** It defines the persona, tone, and interaction style the agent must adopt (the "Víctor" mentor profile — frío, técnico, cero paja, método de ingeniería inversa). Those behavioral rules take precedence over generic assistant defaults. Re-read it whenever the user's domain shifts significantly so the response register stays aligned.

**When to write a note** (`mcp__basic-memory__write_note`):
- After completing a non-trivial task: save what was done, why, and what remains.
- When making an architectural or cross-cutting decision that future sessions need to respect.
- When discovering a non-obvious bug, its root cause, or a workaround.
- When finishing an audit, migration step, or feature — summarize findings and pending items.

**When to read** (`mcp__basic-memory__read_note` / `mcp__basic-memory__search_notes` / `mcp__basic-memory__list_directory`):
- At session start: read `config/SOUL.md` for behavioral calibration.
- At the start of a non-trivial task: check `sanctum/` for prior context on the area you're touching.
- Before making architectural decisions: check for prior decisions in existing notes.
- When the user references something from a past session ("the thing we did with X").

**Note conventions**:
- Directory: `sanctum/` for project-specific knowledge. Personal config under `config/`, pages under `pages/`.
- Title format: `Sanctum - <Topic>` (e.g. `Sanctum - Settings Audit`, `Sanctum - Work Log`).
- Include frontmatter `tags` for grouping: `sanctum`, plus domain tags (`settings`, `crypto`, `habits`, etc.).
- End notes with `## Observations` (structured `[type] fact #tag`) and `## Relations` (`relates_to [[Other Note]]`).
- Update `Sanctum - Work Log` after each meaningful session so the next one has continuity.

**Core notes to keep current**:
- `config/SOUL.md` — persona & behavioral contract (read, not rewritten unless user asks)
- `Sanctum - Overview` — project state, strategy, blockers
- `Sanctum - Arquitectura Técnica` — stack, conventions, key decisions
- `Sanctum - Frontend Migration Status` — what's done, what's pending in the Svelte migration
- `Sanctum - Work Log` — running log of sessions with date + what changed

## Project Intent & Mindset
Sanctum is a personal finance/crypto/habits app focused on **security and privacy first**.
Default to safer behavior: minimize data exposure, avoid unnecessary network calls, and
preserve user trust. When in doubt, prioritize privacy and data integrity over convenience.

## Project Structure and Module Organization
This repo uses a **feature-sliced** layout with strict layer separation:
`UI → IPC (Tauri commands) → Controller → Features (service/repository) → DB`.

`src/` (Rust — business logic crate)
```
src/
├── core/            # Shared validation, errors, DB wrappers
├── features/        # finance/, crypto/, habits/, dashboard/, ingestion/ (service + repository + validation)
├── controller/      # Orchestration per domain + settings
├── db/              # Domain SQL + migrations (finance, crypto, habits, rewards)
├── services/        # Cross-cutting services (charts, i18n)
├── ui/              # UI layer
│   ├── dto/         # Tauri IPC DTOs (Serialize/Deserialize structs per domain)
│   ├── data.rs      # Intermediate display data types
│   ├── helpers.rs   # Formatting, color utils
│   ├── currency.rs  # Currency formatting
│   └── mod.rs
├── lib.rs           # Crate exports
└── models.rs        # Domain models
```

`src-tauri/` (Tauri shell — separate crate, depends on `sanctum` lib)
```
src-tauri/
├── Cargo.toml          # Tauri crate with sanctum dependency
├── tauri.conf.json     # Tauri config (window, CSP, frontend path)
├── capabilities/       # Tauri permission definitions
├── src/
│   ├── main.rs         # Tauri entry point
│   └── lib.rs          # Tauri builder + command registration
└── build.rs
```

`ui-svelte/` (Svelte 5 frontend)
```
ui-svelte/
├── package.json        # pnpm, svelte 5, vite 6, @tauri-apps/api
├── vite.config.ts
├── svelte.config.js
├── tsconfig.json
├── index.html
└── src/
    ├── main.ts         # Svelte mount
    └── App.svelte      # Root component
```

`locales/` (i18n)
```
locales/
├── en.ftl       # English translations (Fluent format)
└── es.ftl       # Spanish translations
```

## Workflow Rules (Important)
- Controllers coordinate only; business logic must live in `features/*/service.rs` or `services/*`.
- Validation: shared rules in `src/core/validation.rs`, domain wrappers in `features/*/validation.rs`.
- Use `nix develop -c ...` for Rust commands (build/test).
- Never run cargo run or cargo build.
- Tax rule: for Chile jurisdiction, tax reports, tax history exports, and displayed tax totals must always use CLP.
### IPC rules
- IPC types go in `src/ui/dto/` — one file per domain, `#[derive(Serialize, Deserialize)]`.
- Tauri commands go in `src-tauri/src/commands/` — `#[tauri::command]` functions.
- DTOs must map 1:1 with what the frontend needs — never expose internal models directly.
- Frontend uses `@tauri-apps/api` `invoke()` to call Rust commands.
- Tauri invoke parameter names must exactly match Rust function parameter names (snake_case).
- No external CDN or runtime network resources in the frontend.
- pnpm with `ignore-scripts=true` (`.npmrc`) for supply chain security.
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
- `nix develop -c cargo check -j 2` (library crate)
- `nix develop -c cargo check -j 2 --manifest-path src-tauri/Cargo.toml` (Tauri crate)
- `nix develop -c cargo clippy -j 2`
- `nix develop -c cargo test -j 2`
- `cd ui-svelte && pnpm install` (frontend dependencies)
- `cd ui-svelte && pnpm check` (svelte-check, alias: `cd ui-svelte && npx svelte-check --tsconfig ./tsconfig.json`)

## Coding Style and Naming Conventions
- Rust: `rustfmt` defaults (4-space).
- Svelte/TypeScript: 2-space indent, single quotes, no semicolons in TS.
- Naming: snake_case (Rust + Tauri commands), PascalCase (types), kebab-case (files when adding new ones).

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
- DO NOT USE NPM ONLY PNPM
