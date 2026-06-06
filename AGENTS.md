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

## Memory & Progress Tracking (engram MCP)
> **This entire section only applies when the `engram` MCP server is active in the current session.**
> If the `mcp__engram__*` tools are not available, ignore everything in this section — do not attempt to invoke them, do not fabricate memories, do not mention them to the user.

The `engram` MCP server is the **authoritative knowledge store** for this project. It provides
persistent memory that survives across sessions *and* context compactions. Use it proactively to
persist context — not just ad-hoc conversation memory.

Memories are stored as **observations**, not files. Each observation has a `title`, a `type`, and a
`content` body, and is automatically scoped to a **project** that engram detects from the git remote
(this repo resolves to project `sanctum`). There is no directory layout to maintain — you search and
recall by query.

**Core tools** (always available, call directly — no `ToolSearch` needed):
- `mcp__engram__mem_current_project` — detect the current project from the cwd. Good first call.
- `mcp__engram__mem_context` — recent sessions and observations. Call at session start or right after a compaction to reload prior context.
- `mcp__engram__mem_search` — find past decisions, bugs, patterns, or context by natural-language query.
- `mcp__engram__mem_get_observation` — fetch the full, untruncated body of a result found via search.
- `mcp__engram__mem_save` — persist an observation. Call this **proactively**, not only when asked.
- `mcp__engram__mem_save_prompt` — record a notable user request/intent.
- `mcp__engram__mem_session_summary` — write an end-of-session summary. Do this before declaring a session "done".

**Deferred tools** (load via `ToolSearch` with `select:<name>` before calling): `mem_update`,
`mem_delete`, `mem_judge`, `mem_compare`, `mem_timeline`, `mem_stats`, `mem_doctor`,
`mem_session_start`, `mem_session_end`, `mem_suggest_topic_key`, `mem_capture_passive`,
`mem_merge_projects`.

**When to save** (`mem_save`) — proactively, immediately after any of these:
- An architectural or cross-cutting decision future sessions must respect.
- A non-obvious bug: what was wrong, the root cause, and the fix.
- A new pattern, convention, or configuration/environment change.
- A discovery or gotcha that would save a future session time.
- Finishing an audit, migration step, or feature — what was done and what remains.

**When to read** (`mem_context` / `mem_search`):
- At session start or after a compaction: `mem_context` to reload what was happening.
- At the start of a non-trivial task: `mem_search` for prior context on the area you're touching.
- Before architectural decisions: `mem_search` for prior decisions so you don't contradict them.
- When the user references something from a past session ("the thing we did with X").

**Content conventions** for `mem_save`:
- Keep `title` short and searchable (e.g. `Settings audit`, `Kraken refid pairing`, `Fixed N+1 in tax engine`).
- Pick a `type`: `decision`, `architecture`, `bugfix`, `pattern`, `config`, `discovery`, or `learning`.
- Structure `content` with the `**What**` / `**Why**` / `**Where**` / `**Learned**` fields (omit `Learned` if there's nothing notable).
- For knowledge that evolves over time, pass a stable `topic_key` (e.g. `architecture/tax-engine`) so the entry is upserted instead of duplicated.

**Conflict surfacing**: after a `mem_save`, check the response for `judgment_required`. If true, iterate
`candidates[]` and call `mem_judge` once per candidate (using that candidate's `judgment_id`). Resolve
silently when confidence ≥ 0.7 and the relation is `related` / `compatible` / `scoped` / `not_conflict`;
ask the user first when confidence < 0.7, or when the relation is `supersedes` / `conflicts_with` on an
`architecture`/`decision` observation.

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
