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
Sanctum is a personal finance/crypto app focused on **security and privacy first**.
Default to safer behavior: minimize data exposure, avoid unnecessary network calls, and
preserve user trust. When in doubt, prioritize privacy and data integrity over convenience.

## Project Structure and Module Organization
This repo uses a **feature-sliced** layout with strict layer separation:
`UI → IPC (Tauri commands) → Controller → Features (service/repository) → DB`.

`src/` (Rust — business logic crate)
```
src/
├── core/            # Shared validation, errors, DB wrappers
├── features/        # finance/, crypto/, dashboard/, ingestion/ (service + repository + validation)
├── controller/      # Orchestration per domain + settings
├── db/              # Domain SQL + migrations (finance, crypto)
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
- Do NOT use npm, only pnpm (with `ignore-scripts=true` in `.npmrc`).
- Never use emojis — use SVG icons from `ui/assets/icons/` instead.
### IPC rules
- IPC types go in `src/ui/dto/` — one file per domain, `#[derive(Serialize, Deserialize)]`.
- Tauri commands go in `src-tauri/src/commands/` — `#[tauri::command]` functions.
- DTOs must map 1:1 with what the frontend needs — never expose internal models directly.
- Frontend uses `@tauri-apps/api` `invoke()` to call Rust commands.
- Tauri invoke parameter names must exactly match Rust function parameter names (snake_case).
- No external CDN or runtime network resources in the frontend.

## AGPL Header
Every source file (including tests) must start with this exact header:
```rust
// Sanctum — a privacy-first personal finance and crypto vault.
// Copyright (C) 2026  yfloress
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

## Definition of Done
All generated code must pass the following without exception before being considered complete:
- `nix develop -c cargo fmt` (formatting applied)
- `nix develop -c cargo clippy -- -D warnings` (zero warnings)
- `nix develop -c cargo test -j 2` (all tests pass)
- `nix develop -c cargo machete` (no new unused dependencies)

Do not mark a task as done if any of these fail. If clippy or tests
fail, fix them before continuing. Do not silence warnings with `#[allow]`
without explicitly justifying it.

## Commit Guidelines
- Do not create commits unless the user explicitly asks for a commit.
- Git-flow style: `feat:`, `fix:`, `docs:`, `chore:`, etc.
  Example: `feat(crypto): add tax report export`.

## Security Notes
- **Database at rest**: SQLCipher encrypts the SQLite database. The vault password derives the encryption key — never log it, never store it.
- **Secrets in memory**: use `secrecy::SecretString` / `Zeroize` for sensitive values (passwords, keys). Zero memory after use.
- **No sensitive data in logs**: financial details, passwords, keys, and PII must never reach log output. Review `security_log.rs` usage.
- **Supply chain**: `cargo audit` for Rust vulnerabilities, pnpm with `ignore-scripts=true` for JS deps.
- **Tauri CSP**: keep `tauri.conf.json` CSP restrictive. No inline scripts, no external CDNs.
- **Local-first**: all data stays on device. No network calls unless user-initiated (e.g., price fetch).
- **Input validation**: use `core/validation.rs` for shared rules, domain validators in `features/*/validation.rs`. Never trust raw input.
- **No secrets in git**: API keys, tokens, passwords → env vars or config files outside the repo. Semgrep/secrets rules in CI.
