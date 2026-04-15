# Translation Status — Svelte Frontend

> Tracks the dual work of: (1) wiring `i18n.t()` calls into Svelte components,
> and (2) updating locale files (`en.ftl` / `es.ftl`) to match the new frontend text.
>
> The locale files were written for the Slint frontend. ~80% of keys should carry over,
> but the Svelte frontend has different component structure, labels, and phrasing.

---

## How i18n Works

1. **Backend**: `get_translations` command reads `locales/{lang}.ftl` (Fluent format) and returns all key-value pairs.
2. **Store**: `i18n.svelte.ts` stores them in a reactive `$state` map, exposes `i18n.t(key, fallback)`.
3. **Language switch**: `SettingsPage` calls `set_preferred_language` → backend switches Fluent bundle → frontend calls `i18n.load()` to refresh strings.
4. **Problem**: Step 3 works, but no component ever calls `i18n.t()`. All strings are hardcoded.

---

## Per-Page Status

### Legend
- **Wire** = Replace hardcoded strings with `i18n.t()` calls
- **Audit** = Compare locale keys against actual text, update/add/remove keys

| Page / Component | Wire | Audit | Hardcoded strings (approx) | Notes |
|------------------|------|-------|---------------------------|-------|
| `Sidebar.svelte` | [ ] | [ ] | ~8 | `Dashboard`, `Finances`, `Crypto`, `Habits`, `Settings`, `Collapse`, `Lock`, `SANCTUM` |
| `LoginPage.svelte` | [ ] | [ ] | ~15 | Already imports `i18n`, just needs `.t()` calls. Has `login-*` keys in locale. |
| `SettingsPage.svelte` | [ ] | [ ] | ~40 | Section headers, labels, descriptions, buttons, import UI text. Has `settings-*` keys. |
| `DashboardPage.svelte` | [ ] | [ ] | ~25 | Chart titles, stat labels, recent activity text. Has `dashboard-*` keys. |
| `FinancesPage.svelte` | [ ] | [ ] | ~80 | Largest page (60KB). Tabs, modals, form labels, account types, categories. Has `finances-*` keys. |
| `HabitsPage.svelte` | [ ] | [ ] | ~60 | Tabs, modals, heatmap labels, analytics, rewards, goals. Has `habits-*` + `rewards-*` keys. |
| `CryptoPage.svelte` | [ ] | [ ] | ~120 | Largest by string count (78KB). Tax UI, modals, ticker config, portfolio. Has extensive `crypto-*` keys. |
| `Toast.svelte` | [ ] | [ ] | ~2 | Minimal text. |

---

## Locale File Audit Checklist

### Keys likely needing updates (Slint → Svelte differences)

- [ ] **Section headers**: Svelte uses different casing/wording (e.g. `h3` titles vs Slint panel titles)
- [ ] **Button labels**: Svelte may use different text (e.g. `+ ADD TRANSACTION` vs `action-add`)
- [ ] **Modal titles**: Svelte modals may have different titles than Slint modals
- [ ] **Empty states**: Svelte has different empty state messaging
- [ ] **Toast messages**: Error/success messages may differ from Slint
- [ ] **Chart labels**: ECharts labels are new (Slint didn't have ECharts)
- [ ] **Tab names**: Svelte tab structure differs from Slint

### Keys to add (new in Svelte, not in locale files)

- [ ] Cash flow chart labels (Dashboard)
- [ ] Overview/Activity/Settings tab labels (Finances)
- [ ] ECharts axis labels and tooltips
- [ ] Import flow: exchange detection, preview, confirm steps
- [ ] Any new modal/form text

### Keys to remove (Slint-only, not in Svelte)

- [ ] Slint-specific widget labels
- [ ] Slint panel/window titles
- [ ] Any keys referencing Slint components

---

## Workflow

1. Pick a page from the table above
2. Read the page source, extract all hardcoded English strings
3. Match each string to an existing locale key (or note that a new key is needed)
4. Replace the hardcoded string with `i18n.t('key', 'Fallback')`
5. Update `en.ftl` and `es.ftl` if keys need to be added, renamed, or removed
6. Mark the page as done in this file
7. Test by switching language in Settings
